//! A module that handles parsing xml based CKL files into a native, easy to work with type.
//! The root of the xml file is called a 'Checklist'. Here is how to parse a checklist into
//! a nice to work with native type.
//!
//! CKL files can contain more than one benchmark.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Benchmark, BenchmarkError, CKLStatus, Rule, Severity};

/// Root element of a CKL file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Checklist {
    #[serde(default, rename = "ASSET")]
    pub assets: Vec<Asset>,

    #[serde(default, rename = "STIGS")]
    pub stigs: Vec<Stigs>,
}

/// Asset metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Asset {
    #[serde(rename = "ROLE")]
    pub role: Option<String>,

    #[serde(rename = "ASSET_TYPE")]
    pub asset_type: Option<String>,

    #[serde(rename = "HOST_NAME")]
    pub host_name: Option<String>,

    #[serde(rename = "HOST_IP")]
    pub host_ip: Option<String>,

    #[serde(rename = "HOST_MAC")]
    pub host_mac: Option<String>,

    #[serde(rename = "HOST_FQDN")]
    pub host_fqdn: Option<String>,

    #[serde(rename = "TECH_AREA")]
    pub tech_area: Option<String>,

    #[serde(rename = "TARGET_KEY")]
    pub target_key: Option<String>,

    #[serde(rename = "WEB_OR_DATABASE")]
    pub web_or_database: Option<String>,

    #[serde(rename = "WEB_DB_SITE")]
    pub web_db_site: Option<String>,

    #[serde(rename = "WEB_DB_INSTANCE")]
    pub web_db_instance: Option<String>,
}

/// Wrapper for the `<iSTIG>` elements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stigs {
    #[serde(default, rename = "iSTIG")]
    pub istigs: Vec<IStig>,
}

/// A single STIG instance inside a checklist.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IStig {
    #[serde(default, rename = "STIG_INFO")]
    pub stig_infos: Vec<StigInfo>,

    #[serde(default, rename = "VULN")]
    pub vulns: Vec<Vuln>,
}

/// STIG metadata as key-value pairs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StigInfo {
    #[serde(default, rename = "SI_DATA")]
    pub si_data: Vec<SiData>,
}

/// A single `<SI_DATA>` entry: `<SID_NAME>` / `<SID_DATA>`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SiData {
    #[serde(rename = "SID_NAME")]
    pub sid_name: String,

    #[serde(rename = "SID_DATA")]
    pub sid_data: Option<String>,
}

/// A single vulnerability finding (one per rule).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Vuln {
    /// Key-value rule metadata stored as `<STIG_DATA>` elements.
    #[serde(default, rename = "STIG_DATA")]
    pub stig_data: Vec<StigData>,

    /// Review status: e.g. "Open", "NotAFinding", "Not_Reviewed",
    /// "Not_Applicable".
    #[serde(rename = "STATUS")]
    pub status: Option<String>,

    /// Scanner output or manual finding details.
    #[serde(rename = "FINDING_DETAILS")]
    pub finding_details: Option<String>,

    /// Analyst comments.
    #[serde(rename = "COMMENTS")]
    pub comments: Option<String>,

    /// Optional severity override.
    #[serde(rename = "SEVERITY_OVERRIDE")]
    pub severity_override: Option<String>,

    /// Justification for the severity override.
    #[serde(rename = "SEVERITY_JUSTIFICATION")]
    pub severity_justification: Option<String>,
}

/// A single `<STIG_DATA>` entry inside a `<VULN>`.
///
/// DISA stores rule metadata as a flat list of key-value pairs where the key
/// is `<VULN_ATTRIBUTE>` and the value is `<ATTRIBUTE_DATA>`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StigData {
    #[serde(rename = "VULN_ATTRIBUTE")]
    pub vuln_attribute: String,

    #[serde(rename = "ATTRIBUTE_DATA")]
    pub attribute_data: Option<String>,
}

/// Convert a CKL checklist into a list of native Benchmarks.
///
/// Each `<iSTIG>` becomes a separate Benchmark. Rules with missing required
/// fields are silently dropped.
pub fn convert_ckl(checklist: Checklist) -> Result<Vec<Benchmark>, BenchmarkError> {
    let mut benchmarks = Vec::new();

    for stigs in checklist.stigs {
        for istig in stigs.istigs {
            let Some(benchmark) = convert_istig(istig) else {
                continue;
            };

            benchmarks.push(benchmark);
        }
    }

    if benchmarks.is_empty() {
        return Err(BenchmarkError::CorruptFile);
    }

    Ok(benchmarks)
}

/// Converts a single <iSTIG>.
fn convert_istig(istig: IStig) -> Option<Benchmark> {
    let stig_info = istig.stig_infos.first()?;

    let id = find_si_data(&stig_info.si_data, "stigid")?;
    let title = find_si_data(&stig_info.si_data, "title")?;
    let version = find_si_data(&stig_info.si_data, "version");
    let release_info = find_si_data(&stig_info.si_data, "releaseinfo");

    let mut rules = BTreeMap::new();

    for vuln in istig.vulns {
        let Some(rule) = convert_vuln(vuln) else {
            continue;
        };

        rules.insert(rule.group_id.clone(), rule);
    }

    if rules.is_empty() {
        return None;
    }

    Some(Benchmark {
        id,
        title,
        version,
        release_info,
        profile: None,
        rules,
    })
}

/// A helper function that looks for specific data in <SI_DATA>.
fn find_si_data(si_data: &[SiData], name: &str) -> Option<String> {
    si_data
        .iter()
        .find(|data| data.sid_name == name)
        .and_then(|data| data.sid_data.clone())
        .filter(|value| !value.is_empty())
}

/// A helper than converts a single <VULN> into a native rule type.
fn convert_vuln(vuln: Vuln) -> Option<Rule> {
    let group_id = find_stig_data(&vuln.stig_data, "Vuln_Num")?;
    let rule_id = find_stig_data(&vuln.stig_data, "Rule_ID")?;
    let title = find_stig_data(&vuln.stig_data, "Rule_Title")?;
    let vuln_discussion = find_stig_data(&vuln.stig_data, "Vuln_Discuss")?;
    let check_text = find_stig_data(&vuln.stig_data, "Check_Content")?;
    let fix_text = find_stig_data(&vuln.stig_data, "Fix_Text")?;

    let severity = parse_severity(
        find_stig_data(&vuln.stig_data, "Severity")
            .as_deref()
            .unwrap_or(""),
    );

    let stig_id = find_stig_data(&vuln.stig_data, "Rule_Ver");

    let cci_refs: Vec<String> = vuln
        .stig_data
        .iter()
        .filter(|data| data.vuln_attribute == "CCI_REF")
        .filter_map(|data| data.attribute_data.clone())
        .filter(|value| !value.is_empty())
        .collect();

    let cci_refs = match cci_refs.len() {
        0 => None,
        _ => Some(cci_refs),
    };

    let ckl_status = vuln.status.and_then(|status| parse_ckl_status(&status));

    Some(Rule {
        group_id,
        rule_id,
        stig_id,
        severity,
        title,
        vuln_discussion,
        check_text,
        fix_text,
        weight: None,
        cci_refs,
        false_positives: None,
        false_negatives: None,
        documentable: None,
        ckl_status,
    })
}

/// A helper function that looks for a specific attribute in <STIG_DATA>.
fn find_stig_data(stig_data: &[StigData], attribute: &str) -> Option<String> {
    stig_data
        .iter()
        .find(|data| data.vuln_attribute == attribute)
        .and_then(|data| data.attribute_data.clone())
        .filter(|value| !value.is_empty())
}

/// A helper function that converts a ckl status string into an enum.
fn parse_ckl_status(status_str: &str) -> Option<CKLStatus> {
    match status_str {
        "Open" => Some(CKLStatus::Open),
        "NotAFinding" => Some(CKLStatus::NotAFinding),
        "Not_Applicable" => Some(CKLStatus::NotApplicable),
        "Not_Reviewed" => Some(CKLStatus::NotReviewed),
        _ => None,
    }
}

/// a helper function that converts severity as a string into an enum.
fn parse_severity(severity_str: &str) -> Severity {
    match severity_str.to_lowercase().as_str() {
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

    /// Read the `check.ckl` asset and fully parse the embedded STIG
    /// checklist through the quick-xml serde structs.
    #[test]
    fn parse_ckl() {
        let xml =
            fs::read_to_string("../test_assets/check.ckl").expect("checklist asset should exist");

        let checklist: Checklist =
            quick_xml::de::from_str(&xml).expect("XML should deserialize into Checklist");

        assert!(
            !checklist.assets.is_empty(),
            "checklist should contain at least one asset"
        );

        let asset = &checklist.assets[0];
        assert!(asset.host_name.is_some(), "asset should have a host_name");

        assert!(
            !checklist.stigs.is_empty(),
            "checklist should contain at least one STIGS block"
        );

        let stigs = &checklist.stigs[0];
        assert!(
            !stigs.istigs.is_empty(),
            "stigs should contain at least one iSTIG"
        );

        let istig = &stigs.istigs[0];
        assert!(
            !istig.stig_infos.is_empty(),
            "iSTIG should contain at least one STIG_INFO"
        );

        let stig_info = &istig.stig_infos[0];
        assert!(
            !stig_info.si_data.is_empty(),
            "STIG_INFO should contain at least one SI_DATA entry"
        );

        let title_data = stig_info
            .si_data
            .iter()
            .find(|data| data.sid_name == "title")
            .expect("STIG_INFO should contain a 'title' entry");
        assert!(
            title_data.sid_data.is_some(),
            "title entry should have data"
        );

        let stig_id_data = stig_info
            .si_data
            .iter()
            .find(|data| data.sid_name == "stigid")
            .expect("STIG_INFO should contain a 'stigid' entry");
        assert!(
            stig_id_data.sid_data.is_some(),
            "stigid entry should have data"
        );

        assert!(
            !istig.vulns.is_empty(),
            "iSTIG should contain at least one VULN"
        );

        let first_vuln = &istig.vulns[0];
        assert!(
            !first_vuln.stig_data.is_empty(),
            "vuln should contain at least one STIG_DATA entry"
        );

        let severity_data = first_vuln
            .stig_data
            .iter()
            .find(|data| data.vuln_attribute == "Severity")
            .expect("first vuln should have a Severity attribute");
        assert!(
            severity_data.attribute_data.is_some(),
            "severity should have a value"
        );

        let vuln_num = first_vuln
            .stig_data
            .iter()
            .find(|data| data.vuln_attribute == "Vuln_Num")
            .expect("first vuln should have a Vuln_Num attribute");
        assert!(
            vuln_num.attribute_data.is_some(),
            "vuln num should have a value"
        );

        let rule_id = first_vuln
            .stig_data
            .iter()
            .find(|data| data.vuln_attribute == "Rule_ID")
            .expect("first vuln should have a Rule_ID attribute");
        assert!(
            rule_id.attribute_data.is_some(),
            "rule id should have a value"
        );

        assert!(
            first_vuln.status.is_some(),
            "first vuln should have a status"
        );
    }

    /// Read the `check.ckl` asset and verify it converts into native Benchmarks.
    #[test]
    fn parse_ckl_convert() {
        let xml =
            fs::read_to_string("../test_assets/check.ckl").expect("checklist asset should exist");

        let checklist: Checklist =
            quick_xml::de::from_str(&xml).expect("XML should deserialize into Checklist");

        let benchmarks = convert_ckl(checklist).expect("CKL should convert into Benchmarks");

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
