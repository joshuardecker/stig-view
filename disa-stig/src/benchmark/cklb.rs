//! A module that handles parsing json based CKLB files into a native, easy to work with type.
//!
//! CKLB files can contain more than one benchmark.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Benchmark, BenchmarkError, CKLStatus, Rule, Severity};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CKLB {
    pub cklb_version: String,

    pub title: String,
    pub id: String,

    pub mode: i64,
    pub active: bool,

    pub has_path: bool,

    #[serde(default)]
    pub target_data: Option<CKLBTargetData>,

    #[serde(default)]
    pub stigs: Vec<CKLBBenchmark>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CKLBTargetData {
    pub target_type: String,

    pub host_name: String,

    pub ip_address: String,

    pub mac_address: String,

    pub fqdn: String,

    pub comments: String,
    pub role: String,

    pub is_web_database: bool,

    pub technology_area: String,

    pub web_db_site: String,

    pub web_db_instance: String,

    pub classification: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CKLBBenchmark {
    pub stig_name: String,

    pub display_name: String,

    pub stig_id: String,

    #[serde(default)]
    pub release_info: Option<String>,

    pub version: String,
    pub uuid: String,
    pub size: i64,

    #[serde(default)]
    pub rules: Vec<CKLBRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CKLBRule {
    pub uuid: String,
    pub stig_uuid: String,

    pub group_id: String,

    pub group_id_src: String,

    pub rule_id: String,

    pub rule_id_src: String,

    #[serde(default)]
    pub target_key: Option<String>,

    #[serde(default)]
    pub stig_ref: Option<String>,

    pub weight: String,
    pub classification: String,
    pub severity: String,

    #[serde(default)]
    pub rule_version: Option<String>,

    pub rule_title: String,
    pub fix_text: String,

    #[serde(default)]
    pub reference_identifier: Option<String>,

    pub group_title: String,

    #[serde(default)]
    pub false_positives: Option<String>,

    #[serde(default)]
    pub false_negatives: Option<String>,

    pub discussion: String,
    pub check_content: String,

    #[serde(default)]
    pub documentable: Option<String>,

    #[serde(default)]
    pub mitigations: Option<String>,

    #[serde(default)]
    pub potential_impacts: Option<String>,

    #[serde(default)]
    pub third_party_tools: Option<String>,

    #[serde(default)]
    pub mitigation_control: Option<String>,

    #[serde(default)]
    pub responsibility: Option<String>,

    #[serde(default)]
    pub security_override_guidance: Option<String>,

    #[serde(default)]
    pub ia_controls: Option<String>,

    #[serde(default)]
    pub check_content_ref: Option<String>,

    #[serde(default)]
    pub legacy_ids: Option<Vec<String>>,

    #[serde(default)]
    pub ccis: Option<Vec<String>>,

    #[serde(default)]
    pub group_tree: Option<Vec<CKLBGroupTreeEntry>>,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,

    pub status: CKLStatus,

    #[serde(default)]
    pub overrides: Option<serde_json::Value>,

    #[serde(default)]
    pub comments: Option<String>,

    #[serde(default)]
    pub finding_details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CKLBGroupTreeEntry {
    pub id: String,
    pub title: String,
    pub description: String,
}

/// Convert a CKLB file into a list of native Benchmarks.
///
/// Each STIG inside the CKLB becomes a separate Benchmark. Rules with missing
/// required fields are silently dropped.
pub fn convert_cklb(cklb: CKLB) -> Result<Vec<Benchmark>, BenchmarkError> {
    let mut benchmarks = Vec::new();

    for cklb_benchmark in cklb.stigs {
        let Some(benchmark) = convert_cklb_benchmark(cklb_benchmark) else {
            continue;
        };

        benchmarks.push(benchmark);
    }

    if benchmarks.is_empty() {
        return Err(BenchmarkError::CorruptFile);
    }

    Ok(benchmarks)
}

/// Convert a single CKLBBenchmark item into a benchmark.
fn convert_cklb_benchmark(cklb_benchmark: CKLBBenchmark) -> Option<Benchmark> {
    if cklb_benchmark.stig_id.is_empty() || cklb_benchmark.stig_name.is_empty() {
        return None;
    }

    let mut rules = BTreeMap::new();

    for cklb_rule in cklb_benchmark.rules {
        let Some(rule) = convert_cklb_rule(cklb_rule) else {
            continue;
        };

        rules.insert(rule.group_id.clone(), rule);
    }

    if rules.is_empty() {
        return None;
    }

    Some(Benchmark {
        id: cklb_benchmark.stig_id,
        title: cklb_benchmark.stig_name,
        version: Some(cklb_benchmark.version),
        release_info: cklb_benchmark.release_info,
        profile: None,
        rules,
    })
}

/// Convert a single rule from a CKLB to native type.s
fn convert_cklb_rule(cklb_rule: CKLBRule) -> Option<Rule> {
    let group_id = (!cklb_rule.group_id.is_empty()).then_some(cklb_rule.group_id)?;

    let rule_id = (!cklb_rule.rule_id.is_empty()).then_some(cklb_rule.rule_id)?;

    let title = (!cklb_rule.rule_title.is_empty()).then_some(cklb_rule.rule_title)?;

    let vuln_discussion = (!cklb_rule.discussion.is_empty()).then_some(cklb_rule.discussion)?;

    let check_text = (!cklb_rule.check_content.is_empty()).then_some(cklb_rule.check_content)?;

    let fix_text = (!cklb_rule.fix_text.is_empty()).then_some(cklb_rule.fix_text)?;

    Some(Rule {
        group_id,
        rule_id,
        stig_id: cklb_rule.rule_version.filter(|value| !value.is_empty()),
        severity: parse_severity(&cklb_rule.severity),
        title,
        vuln_discussion,
        check_text,
        fix_text,
        weight: Some(cklb_rule.weight),
        cci_refs: cklb_rule.ccis.filter(|values| !values.is_empty()),
        false_positives: cklb_rule.false_positives.filter(|value| !value.is_empty()),
        false_negatives: cklb_rule.false_negatives.filter(|value| !value.is_empty()),
        documentable: cklb_rule.documentable.map(|value| value.trim() == "true"),
        ckl_status: Some(cklb_rule.status),
    })
}

/// A helper function that converts a string of severity into an enum.
fn parse_severity(severity_str: &str) -> Severity {
    match severity_str {
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        _ => Severity::Unknown,
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::*;

    /// Read the `check.cklb` asset and verify it deserializes into a CKLB.
    #[test]
    fn parse_cklb() {
        let json =
            fs::read_to_string("../test_assets/check.cklb").expect("cklb asset should exist");

        let cklb: CKLB = serde_json::from_str(&json).expect("JSON should deserialize into CKLB");

        assert!(
            !cklb.stigs.is_empty(),
            "cklb should contain at least one stig"
        );

        let benchmark = &cklb.stigs[0];
        assert!(
            !benchmark.stig_id.is_empty(),
            "benchmark should have a stig_id"
        );

        assert!(
            !benchmark.rules.is_empty(),
            "benchmark should contain at least one rule"
        );

        let rule = &benchmark.rules[0];
        assert!(!rule.group_id.is_empty(), "rule should have a group_id");

        assert!(!rule.rule_id.is_empty(), "rule should have a rule_id");

        assert!(!rule.severity.is_empty(), "rule should have a severity");
    }

    /// Read the `check.cklb` asset and convert it into native Benchmarks.
    #[test]
    fn parse_cklb_convert() {
        let json =
            fs::read_to_string("../test_assets/check.cklb").expect("cklb asset should exist");

        let cklb: CKLB = serde_json::from_str(&json).expect("JSON should deserialize into CKLB");

        let benchmarks = convert_cklb(cklb).expect("CKLB should convert into Benchmarks");

        assert!(
            !benchmarks.is_empty(),
            "should contain at least one benchmark"
        );

        let benchmark = &benchmarks[0];
        assert!(!benchmark.id.is_empty(), "benchmark should have an id");

        assert!(!benchmark.title.is_empty(), "benchmark should have a title");

        assert!(
            !benchmark.rules.is_empty(),
            "benchmark should contain at least one rule"
        );

        let rule = benchmark.rules.values().next().unwrap();
        assert!(!rule.group_id.is_empty(), "rule should have a group_id");

        assert!(!rule.rule_id.is_empty(), "rule should have a rule_id");

        assert!(!rule.title.is_empty(), "rule should have a title");
    }
}
