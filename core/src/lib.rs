// Depricated files, but keeping them around until new stuff will replace them.
pub mod db_dep;
pub mod stig_dep;

mod db;
mod detection;
mod load;

// Re exports.
pub use crate::db::*;
pub use crate::detection::detect_stig_version;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// The overarching benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmark {
    pub id: String,
    pub title: String,
    pub version: Option<String>,
    pub release: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub source: Option<String>,
    pub status: String,
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
    pub documentable: bool,
}

#[derive(Debug, Deserialize)]
pub struct XylokChecks {
    checks: Vec<XylokRule>,
}

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

    // Fields found in the Xylok rule that I will ignore.
    #[serde(skip_serializing_if = "Option::is_none")]
    group_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_scc_check: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verified_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expert: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    similar_checks: Option<Vec<String>>,
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

/// Different formats a benchmark and rules can be stored in.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Version {
    XccdfV1_1,
    XccdfV1_2,
    Xylok,
}

#[test]
fn test_xylok_stig_loading() {
    use std::fs::read_to_string;

    let toml_str = read_to_string("../test_assets/packed.toml").unwrap();

    let xylok_toml: XylokChecks = toml::from_str(&toml_str).unwrap();
}
