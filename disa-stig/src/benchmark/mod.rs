//! A module that supports loading different DISA STIG formats into a easy to work with native Benchmark type.

mod ckl;
mod cklb;
mod version_detection;
mod xccdf_v1_1;
mod xccdf_v1_2;
mod xylok;

use std::{collections::BTreeMap, hash::Hash, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::benchmark::version_detection::{FormatWithData, detect_format_with_path};

pub type BenchmarkID = String;
pub type RuleID = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Benchmark {
    /// Unique identifier for this benchmark (e.g. the STIG ID).
    pub id: String,
    /// Human-readable title of the benchmark.
    pub title: String,
    /// Version string, if provided by the source format.
    pub version: Option<String>,
    /// Release information, such as a date or revision number.
    pub release_info: Option<String>,
    /// Selected XCCDF profile, if applicable.
    pub profile: Option<String>,
    /// Map of rule IDs to their parsed rule definitions.
    pub rules: BTreeMap<RuleID, Rule>,
}

#[derive(Debug, Error)]
pub enum BenchmarkError {
    #[error("An io error occured trying to read a benchmark: {0}")]
    IOError(#[from] std::io::Error),
    #[error("A serialization error occured with this benchmark: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Provided file has type which is not supported or recognised")]
    UnsupportedFileType,
    #[error("File is supported but corrupted")]
    CorruptFile,
    #[error("Error occured unzipping file {0}")]
    ZipError(#[from] zip::result::ZipError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Rule {
    /// Group identifier for this rule (often the Vuln_Num in DISA formats).
    pub group_id: String,
    /// Globally unique rule identifier.
    pub rule_id: String,
    /// STIG-specific rule version identifier, if present.
    pub stig_id: Option<String>,
    /// Severity level assigned to this rule.
    pub severity: Severity,
    /// Human-readable title summarizing the rule.
    pub title: String,
    /// Description of the vulnerability or risk this rule addresses.
    pub vuln_discussion: String,
    /// Manual or automated check instructions for compliance verification.
    pub check_text: String,
    /// Remediation steps to bring the system into compliance.
    pub fix_text: String,
    /// XCCDF weight value, if provided by the source.
    pub weight: Option<String>,
    /// List of CCI (Control Correlation Identifier) references.
    pub cci_refs: Option<Vec<String>>,
    /// Known false positive guidance, if any.
    pub false_positives: Option<String>,
    /// Known false negative guidance, if any.
    pub false_negatives: Option<String>,
    /// Whether this finding is considered documentable.
    pub documentable: Option<bool>,

    /// Review status from a CKL checklist, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ckl_status: Option<CKLStatus>,
}

/// The severity of each rule in a benchmark.
/// Severity is serialized as an integer to better reflect how the other formats
/// save this to disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(try_from = "u64", into = "u64")]
pub enum Severity {
    Unknown,
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// What format does the DISA STIG take / have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Format {
    /// An xml based format that only contains checks, not any findings.
    XccdfV1_1,
    /// An xml based format built to store SCAP automation, but some useful human readable data is stored.
    XccdfV1_2,
    /// The internal format used by Xylok.
    Xylok,
    /// An xml based format that stores each checks compliance verdict.
    CKL,
    /// A json based format that stores each checks compliance verdict.
    CKLB,
    /// An in house format that avoids nesting, and focuses on smaller file sizes.
    InHouse,
}

impl From<Severity> for u64 {
    fn from(value: Severity) -> u64 {
        match value {
            Severity::Unknown => 0,
            Severity::VeryLow => 1,
            Severity::Low => 2,
            Severity::Medium => 3,
            Severity::High => 4,
            Severity::VeryHigh => 5,
        }
    }
}

impl TryFrom<u64> for Severity {
    type Error = String;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Severity::Unknown),
            1 => Ok(Severity::VeryLow),
            2 => Ok(Severity::Low),
            3 => Ok(Severity::Medium),
            4 => Ok(Severity::High),
            5 => Ok(Severity::VeryHigh),
            _ => Err(format!("unknown nist_impact value: {value}")),
        }
    }
}

/// The compliance status finding of a single check in a CKL or CKLB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CKLStatus {
    NotAFinding,
    Open,
    NotApplicable,
    NotReviewed,
}

impl Benchmark {
    /// Create a new empty benchmark.
    pub fn new() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            version: None,
            release_info: None,
            profile: None,
            rules: BTreeMap::new(),
        }
    }

    /// Load a DISA STIG Benchmark from the provided file path.
    /// Supports paths for CKL, CKLB, xccdf version 1.1 and 1.2, as well as this crates custom format.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Vec<Benchmark>, BenchmarkError> {
        let formats = detect_format_with_path(path)?;

        let mut benchmarks = Vec::new();

        for format in formats {
            match format {
                FormatWithData::CKL(ckl) => benchmarks.append(&mut ckl::convert_ckl(ckl)?),
                FormatWithData::CKLB(cklb) => benchmarks.append(&mut cklb::convert_cklb(cklb)?),
                FormatWithData::InHouse(benchmark) => benchmarks.push(benchmark),
                FormatWithData::XccdfV1_1(xccdfv1_1) => {
                    benchmarks.push(xccdf_v1_1::convert_xccdf_v1_1(xccdfv1_1)?)
                }
                FormatWithData::XccdfV1_2(xccdfv1_2) => {
                    benchmarks.push(xccdf_v1_2::convert_xccdf_v1_2(xccdfv1_2)?)
                }
                FormatWithData::Xylok(xylok_benchmark) => {
                    benchmarks.push(xylok::convert_xylok(xylok_benchmark)?)
                }
            }
        }

        Ok(benchmarks)
    }

    /// Serialize this benchmark into the requested format.
    ///
    /// Be mindful choosing the correct format. Some formats can be lossy or have unexpected behavior.
    /// Ex: CKL -> XCCDF will loose any verdicts saved in the file.
    /// Ex: XCCDF -> CKL will have verdicts auto filled.
    ///
    /// Use the InHouse format if you intend to return to this file with this library.
    ///
    /// File extensions to note when serializing into files:
    ///
    /// XCCDF and CKL -> .xml
    ///
    /// CKLB -> .json
    ///
    /// Xylok -> .toml
    ///
    /// In House -> .json.zstd
    pub fn serialize(&self, format: Format) -> Result<Vec<u8>, BenchmarkError> {
        match format {
            Format::XccdfV1_1 => unimplemented!(),
            Format::XccdfV1_2 => unimplemented!(),
            Format::Xylok => unimplemented!(),
            Format::CKL => unimplemented!(),
            Format::CKLB => unimplemented!(),
            Format::InHouse => {
                let benchmark_bytes = serde_json::to_vec(self)?;

                // Compress it to shrink file size using zstd.
                let compressed = zstd::encode_all(&*benchmark_bytes, 3)?;

                Ok(compressed)
            }
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Unknown => formatter.write_str("Unknown"),
            Severity::VeryLow => formatter.write_str("Very Low"),
            Severity::Low => formatter.write_str("Low"),
            Severity::Medium => formatter.write_str("Medium"),
            Severity::High => formatter.write_str("High"),
            Severity::VeryHigh => formatter.write_str("Very High"),
        }
    }
}

impl std::fmt::Display for CKLStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CKLStatus::NotAFinding => formatter.write_str("Not a Finding"),
            CKLStatus::Open => formatter.write_str("Open"),
            CKLStatus::NotApplicable => formatter.write_str("Not Applicable"),
            CKLStatus::NotReviewed => formatter.write_str("Not Reviewed"),
        }
    }
}
