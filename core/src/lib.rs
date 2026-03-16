// Depricated files, but keeping them around until new stuff will replace them.
pub mod db_dep;
pub mod stig_dep;

mod db;

pub use crate::db::*;

use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
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

    pub rules: BTreeMap<String, Arc<Rule>>,
}

#[derive(Debug)]
pub struct Rule {
    pub group_id: String,
    pub rule_id: String,
    pub stig_id: Option<String>,
    pub severity: Severity,
    pub weight: f32,
    pub title: String,
    pub vuln_discussion: String,
    pub check_text: String,
    pub fix_text: String,
    pub cci_refs: Option<Vec<String>>,
    pub false_positives: Option<String>,
    pub false_negatives: Option<String>,
    pub documentable: bool,
}

#[derive(Debug, Clone)]
pub enum Severity {
    High,
    Medium,
    Low,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Version {
    XccdfV1_1,
    XccdfV1_2,
    Xylok,
}
