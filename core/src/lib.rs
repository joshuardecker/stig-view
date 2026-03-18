// Depricated files, but keeping them around until new stuff will replace them.
pub mod db_dep;
pub mod stig_dep;

mod db;
mod detection;
mod load;

// Re exports.
pub use crate::db::*;
pub use crate::detection::detect_stig_format;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The overarching benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmark {
    pub id: String,
    pub title: String,
    pub version: Option<String>,
    pub release: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub status_date: Option<String>,

    pub rules: BTreeMap<String, Rule>,
}

/// Each check / rule of a benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub group_id: String,
    pub rule_id: String,
    pub stig_id: Option<String>,
    pub severity: Severity,
    pub title: String,
    pub vuln_discussion: String,
    pub check_text: String,
    pub fix_text: String,
    pub cci_refs: Option<Vec<String>>,
    pub false_positives: Option<String>,
    pub false_negatives: Option<String>,
    pub documentable: Option<bool>,
}

/// Xylok toml's can be deserialized into this struct.
#[derive(Debug, Deserialize)]
pub struct XylokChecks {
    pub benchmark: XylokBenchmark,
    pub checks: Vec<XylokRule>,
}

/// The information I care about from [benchmark].
/// Fail without these fields, they are required.
#[derive(Debug, Clone, Deserialize)]
pub struct XylokBenchmark {
    benchmark_id: String,
    title: String,
}

/// The information I care about from [[checks]].
/// Wrapping is kept very loose on purpose. Allow fields to be empty,
/// that way old and new versions (old versions will lack fields) can be read into the program.
/// Handle parsing after deserialization.
#[derive(Debug, Clone, Deserialize)]
pub struct XylokRule {
    // A uuid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pk: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    vulnerability_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    human_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nist_impact: Option<Severity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    discussion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ccis: Option<Vec<u64>>,
    //
    // What is not included:
    // - false positives
    // - false negatives
    // - documentable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "u64")]
pub enum Severity {
    Unknown,
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl TryFrom<u64> for Severity {
    type Error = String;
    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Severity::Unknown),
            1 => Ok(Severity::VeryLow),
            2 => Ok(Severity::Low),
            3 => Ok(Severity::Medium),
            4 => Ok(Severity::High),
            5 => Ok(Severity::VeryHigh),
            _ => Err(format!("unknown nist_impact value: {v}")),
        }
    }
}

/// Different formats a benchmark can be loaded from.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Format {
    XccdfV1_1,
    XccdfV1_2,
    Xylok,
}

impl XylokChecks {
    pub fn convert(&self) -> Option<Benchmark> {
        todo!()
    }
}

impl XylokRule {
    pub fn convert(&self) -> Option<Rule> {
        todo!()
    }
}
