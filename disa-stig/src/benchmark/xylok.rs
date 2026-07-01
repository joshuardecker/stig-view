use std::collections::{BTreeMap, HashSet};

use serde::Deserialize;

use crate::{Benchmark, BenchmarkError, Rule, Severity};

/// Xylok toml's can be deserialized into this struct.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct XylokToml {
    pub versions: Vec<XylokVersion>,

    pub benchmark: XylokBenchmark,

    #[serde(default)]
    pub checks: Vec<XylokRule>,
}

/// Date and uuids.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct XylokVersion {
    date: String,

    #[serde(default)]
    check_pks: HashSet<String>,
}

/// The information I care about from [benchmark].
/// Fail without these fields, they are required.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct XylokBenchmark {
    benchmark_id: String,
    title: String,
}

/// The information I care about from [[checks]].
/// Wrapping is kept very loose on purpose. Allow fields to be empty,
/// that way old and new versions (old versions will lack fields) can be read into the program.
/// Handle parsing after deserialization.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct XylokRule {
    // A uuid.
    #[serde(default)]
    pk: Option<String>,

    #[serde(default)]
    vulnerability_id: Option<String>,

    #[serde(default)]
    rule_id: Option<String>,

    #[serde(default)]
    human_id: Option<String>,

    #[serde(default)]
    nist_impact: Option<Severity>,

    #[serde(default)]
    title: Option<String>,

    #[serde(default)]
    discussion: Option<String>,

    #[serde(default)]
    content: Option<String>,

    #[serde(default)]
    fix: Option<String>,

    #[serde(default)]
    ccis: Option<Vec<u64>>,
}

/// Convert a Xylok TOML into a native Benchmark.
///
/// Uses the newest version entry to determine which checks are active.
/// Rules with missing required fields are silently dropped.
pub fn convert_xylok(xylok: XylokToml) -> Result<Benchmark, BenchmarkError> {
    let mut versions = xylok.versions;

    if versions.is_empty() {
        return Err(BenchmarkError::CorruptFile);
    }

    versions.sort_by(|left, right| left.date.cmp(&right.date));

    let Some(newest_version) = versions.into_iter().last() else {
        return Err(BenchmarkError::CorruptFile);
    };

    let mut rules = BTreeMap::new();

    for xylok_rule in xylok.checks {
        let rule_pk = match xylok_rule.pk.as_ref() {
            Some(pk) => pk,
            None => continue,
        };

        if !newest_version.check_pks.contains(rule_pk) {
            continue;
        }

        let Some(group_id) = xylok_rule.vulnerability_id else {
            continue;
        };

        let Some(raw_rule_id) = xylok_rule.rule_id else {
            continue;
        };

        let rule_id = raw_rule_id
            .strip_suffix("_rule")
            .unwrap_or(&raw_rule_id)
            .to_owned();

        let Some(severity) = xylok_rule.nist_impact else {
            continue;
        };

        let Some(title) = xylok_rule.title else {
            continue;
        };

        let Some(vuln_discussion) = xylok_rule.discussion else {
            continue;
        };

        let Some(check_text) = xylok_rule.content else {
            continue;
        };

        let Some(fix_text) = xylok_rule.fix else {
            continue;
        };

        let cci_refs: Vec<String> = xylok_rule
            .ccis
            .unwrap_or_default()
            .iter()
            .map(|cci_value| cci_value.to_string())
            .collect();

        let rule = Rule {
            group_id,
            rule_id,
            stig_id: xylok_rule.human_id,
            severity,
            title,
            vuln_discussion,
            check_text,
            fix_text,
            weight: None,
            cci_refs: match cci_refs.len() {
                0 => None,
                _ => Some(cci_refs),
            },
            false_positives: None,
            false_negatives: None,
            documentable: None,
            ckl_status: None,
        };

        rules.insert(rule.group_id.clone(), rule);
    }

    if rules.is_empty() {
        return Err(BenchmarkError::CorruptFile);
    }

    Ok(Benchmark {
        id: xylok.benchmark.benchmark_id,
        title: xylok.benchmark.title,
        version: None,
        release_info: None,
        profile: None,
        rules,
    })
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::*;

    /// Read the `packed.toml` asset and verify it deserializes into a
    /// XylokToml and converts into a Benchmark.
    #[test]
    fn parse_xylok_toml() {
        let toml_str = fs::read_to_string("../test_assets/packed.toml")
            .expect("xylok toml asset should exist");

        let xylok: XylokToml =
            toml::from_str(&toml_str).expect("TOML should deserialize into XylokToml");

        assert!(
            !xylok.versions.is_empty(),
            "xylok toml should contain at least one version"
        );

        assert!(
            !xylok.checks.is_empty(),
            "xylok toml should contain at least one check"
        );

        let benchmark = convert_xylok(xylok).expect("xylok toml should convert into a Benchmark");

        assert!(!benchmark.id.is_empty(), "benchmark should have an id");

        assert!(!benchmark.title.is_empty(), "benchmark should have a title");

        assert!(
            !benchmark.rules.is_empty(),
            "benchmark should contain at least one rule"
        );
    }
}
