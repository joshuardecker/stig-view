use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Benchmark as InternalBenchmark, BenchmarkError, Rule as InternalRule, Severity};

/// `<Benchmark>` – the root element of an XCCDF 1.1 STIG.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Benchmark {
    #[serde(rename = "@id")]
    pub id: Option<String>,

    #[serde(rename = "@lang", alias = "@xml:lang")]
    pub lang: Option<String>,

    #[serde(rename = "@style", alias = "@style-href")]
    pub style: Option<String>,

    #[serde(default, rename = "status")]
    pub statuses: Vec<Status>,

    #[serde(default, rename = "title")]
    pub titles: Vec<TextWithLang>,

    #[serde(default, rename = "description")]
    pub descriptions: Vec<TextWithLang>,

    #[serde(default, rename = "notice")]
    pub notices: Vec<Notice>,

    #[serde(default, rename = "front-matter")]
    pub front_matters: Vec<TextWithLang>,

    #[serde(default, rename = "rear-matter")]
    pub rear_matters: Vec<TextWithLang>,

    #[serde(default, rename = "reference")]
    pub references: Vec<Reference>,

    #[serde(default, rename = "plain-text")]
    pub plain_texts: Vec<PlainText>,

    #[serde(default, rename = "version")]
    pub versions: Vec<Version>,

    #[serde(default, rename = "Profile")]
    pub profiles: Vec<Profile>,

    #[serde(default, rename = "Group")]
    pub groups: Vec<Group>,
}

/// A text element that may carry an `xml:lang` attribute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextWithLang {
    #[serde(rename = "@lang", alias = "@xml:lang")]
    pub lang: Option<String>,

    #[serde(default, rename = "$text")]
    pub text: Option<String>,
}

/// `<status>` with an optional `date` attribute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Status {
    #[serde(rename = "@date")]
    pub date: Option<String>,

    #[serde(default, rename = "$text")]
    pub text: Option<String>,
}

/// `<notice>` with an `id` attribute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Notice {
    #[serde(rename = "@id")]
    pub id: Option<String>,

    #[serde(rename = "@lang", alias = "@xml:lang")]
    pub lang: Option<String>,

    #[serde(default, rename = "$text")]
    pub text: Option<String>,
}

/// `<plain-text>` with an `id` attribute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlainText {
    #[serde(rename = "@id")]
    pub id: Option<String>,

    #[serde(default, rename = "$text")]
    pub text: Option<String>,
}

/// `<version>` – simple text, no time/update attributes in XCCDF 1.1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Version {
    #[serde(default, rename = "$text")]
    pub text: Option<String>,
}

/// `<reference>` that may carry an `href` attribute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reference {
    #[serde(rename = "@href")]
    pub href: Option<String>,

    #[serde(default, rename = "title")]
    pub titles: Vec<TextWithLang>,

    #[serde(default, rename = "publisher")]
    pub publishers: Vec<String>,

    #[serde(default, rename = "source")]
    pub sources: Vec<String>,

    #[serde(rename = "$text")]
    pub text: Option<String>,
}

/// `<Profile>` inside a benchmark.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    #[serde(rename = "@id")]
    pub id: Option<String>,

    #[serde(default, rename = "title")]
    pub titles: Vec<TextWithLang>,

    #[serde(default, rename = "description")]
    pub descriptions: Vec<TextWithLang>,

    #[serde(default, rename = "select")]
    pub selects: Vec<Select>,
}

/// `<select>` inside a profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Select {
    #[serde(rename = "@idref")]
    pub idref: Option<String>,

    #[serde(rename = "@selected")]
    pub selected: Option<String>,
}

/// `<Group>` – may contain nested groups and rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Group {
    #[serde(rename = "@id")]
    pub id: Option<String>,

    #[serde(rename = "@class")]
    pub class: Option<String>,

    #[serde(rename = "@selected")]
    pub selected: Option<String>,

    #[serde(default, rename = "title")]
    pub titles: Vec<TextWithLang>,

    #[serde(default, rename = "description")]
    pub descriptions: Vec<TextWithLang>,

    #[serde(default, rename = "Rule")]
    pub rules: Vec<Rule>,

    #[serde(default, rename = "Group")]
    pub groups: Vec<Group>,
}

/// `<Rule>` – the individual STIG requirement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rule {
    #[serde(rename = "@id")]
    pub id: Option<String>,

    #[serde(rename = "@severity")]
    pub severity: Option<String>,

    #[serde(rename = "@weight")]
    pub weight: Option<String>,

    #[serde(rename = "@role")]
    pub role: Option<String>,

    #[serde(rename = "@selected")]
    pub selected: Option<String>,

    #[serde(default, rename = "version")]
    pub versions: Vec<Version>,

    #[serde(default, rename = "title")]
    pub titles: Vec<TextWithLang>,

    #[serde(default, rename = "description")]
    pub descriptions: Vec<String>,

    #[serde(default, rename = "reference")]
    pub references: Vec<Reference>,

    #[serde(default, rename = "ident")]
    pub idents: Vec<Ident>,

    #[serde(default, rename = "fixtext")]
    pub fixtexts: Vec<Fixtext>,

    #[serde(default, rename = "fix")]
    pub fixes: Vec<Fix>,

    #[serde(default, rename = "check")]
    pub checks: Vec<Check>,

    #[serde(default, rename = "rationale")]
    pub rationales: Vec<TextWithLang>,
}

/// `<ident>` – e.g. CCI references.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ident {
    #[serde(rename = "@system")]
    pub system: Option<String>,

    #[serde(default, rename = "$text")]
    pub text: Option<String>,
}

/// `<fixtext>` – human-readable remediation text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fixtext {
    #[serde(rename = "@fixref")]
    pub fixref: Option<String>,

    #[serde(rename = "@lang", alias = "@xml:lang")]
    pub lang: Option<String>,

    #[serde(default, rename = "$text")]
    pub text: Option<String>,
}

/// `<fix>` – automated remediation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fix {
    #[serde(rename = "@id")]
    pub id: Option<String>,

    #[serde(rename = "@strategy")]
    pub strategy: Option<String>,

    #[serde(rename = "@disruption")]
    pub disruption: Option<String>,

    #[serde(rename = "@system")]
    pub system: Option<String>,

    #[serde(default, rename = "$text")]
    pub text: Option<String>,
}

/// `<check>` – contains human-readable `<check-content>` in XCCDF 1.1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Check {
    #[serde(rename = "@system")]
    pub system: Option<String>,

    #[serde(default, rename = "check-content")]
    pub check_contents: Vec<String>,
}

pub fn convert_xccdf_v1_1(xccdf_benchmark: Benchmark) -> Result<InternalBenchmark, BenchmarkError> {
    let id = xccdf_benchmark
        .id
        .filter(|value| !value.is_empty())
        .ok_or(BenchmarkError::CorruptFile)?;

    let title = xccdf_benchmark
        .titles
        .into_iter()
        .next()
        .and_then(|text_with_lang| text_with_lang.text)
        .filter(|value| !value.is_empty())
        .ok_or(BenchmarkError::CorruptFile)?;

    let version = xccdf_benchmark
        .versions
        .into_iter()
        .next()
        .and_then(|version| version.text);

    let release_info = xccdf_benchmark
        .plain_texts
        .into_iter()
        .find(|plain_text| plain_text.id.as_deref() == Some("release-info"))
        .and_then(|plain_text| plain_text.text);

    let profile = xccdf_benchmark
        .profiles
        .into_iter()
        .next()
        .and_then(|profile| profile.id);

    let mut rules = BTreeMap::new();

    for group in &xccdf_benchmark.groups {
        convert_group(group, &mut rules)?;
    }

    if rules.is_empty() {
        return Err(BenchmarkError::CorruptFile);
    }

    Ok(InternalBenchmark {
        id,
        title,
        version,
        release_info,
        profile,
        rules,
    })
}

fn convert_group(
    group: &Group,
    rules: &mut BTreeMap<String, InternalRule>,
) -> Result<(), BenchmarkError> {
    let group_id = group.id.as_deref().unwrap_or("");

    for rule in &group.rules {
        let rule_id = match rule.id.as_deref() {
            Some(id) => id.strip_suffix("_rule").unwrap_or(id).to_owned(),
            None => continue,
        };

        if rule_id.is_empty() {
            continue;
        }

        let severity = parse_severity(rule.severity.as_deref().unwrap_or(""));

        let title = match rule.titles.first().and_then(|title| title.text.clone()) {
            Some(value) if !value.is_empty() => value,
            _ => continue,
        };

        let vuln_discussion = rule.descriptions.first().cloned().unwrap_or_default();

        let check_text = rule
            .checks
            .first()
            .and_then(|check| check.check_contents.first().cloned())
            .unwrap_or_default();

        let fix_text = rule
            .fixtexts
            .first()
            .and_then(|fixtext| fixtext.text.clone())
            .unwrap_or_default();

        let weight = rule.weight.clone();

        let cci_refs: Vec<String> = rule
            .idents
            .iter()
            .filter_map(|ident| ident.text.clone())
            .collect();

        let cci_refs = match cci_refs.len() {
            0 => None,
            _ => Some(cci_refs),
        };

        let stig_id = rule
            .versions
            .first()
            .and_then(|version| version.text.clone());

        let rule = InternalRule {
            group_id: group_id.to_owned(),
            rule_id,
            stig_id,
            severity,
            title,
            vuln_discussion,
            check_text,
            fix_text,
            weight,
            cci_refs,
            false_positives: None,
            false_negatives: None,
            documentable: None,
            ckl_status: None,
        };

        rules.insert(group_id.to_owned(), rule);
    }

    for nested_group in &group.groups {
        convert_group(nested_group, rules)?;
    }

    Ok(())
}

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
    use std::fs::File;
    use std::io::Read;

    use zip::ZipArchive;

    use super::*;

    /// Read the RHEL 8 V2R6 STIG zip asset and fully parse the embedded
    /// XCCDF 1.1 benchmark through the quick-xml serde structs.
    #[test]
    fn parse_xccdf_v1_1() {
        let file = File::open("../test_assets/U_RHEL_8_V2R6_STIG.zip")
            .expect("benchmark zip asset should exist");

        let mut archive = ZipArchive::new(file).expect("zip should be valid");

        let mut xml = String::new();
        let mut found = false;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).expect("zip entry should be valid");
            if entry.name().ends_with(".xml") {
                entry
                    .read_to_string(&mut xml)
                    .expect("entry should be valid UTF-8 XML");
                found = true;
                break;
            }
        }
        assert!(found, "zip should contain at least one .xml entry");

        let benchmark: Benchmark =
            quick_xml::de::from_str(&xml).expect("XML should deserialize into Benchmark");

        assert!(
            benchmark.id.is_some(),
            "benchmark should have an id attribute"
        );

        assert!(
            !benchmark.groups.is_empty(),
            "benchmark should contain at least one Group"
        );

        let group_with_rules = benchmark
            .groups
            .iter()
            .find(|group| !group.rules.is_empty())
            .expect("at least one group should contain a Rule");

        assert!(
            !group_with_rules.rules.is_empty(),
            "group should contain at least one Rule"
        );

        let rule = &group_with_rules.rules[0];

        assert!(rule.id.is_some(), "rule should have an id attribute");
        assert!(
            rule.severity.is_some(),
            "rule should have a severity attribute"
        );

        assert!(
            !rule.titles.is_empty(),
            "rule should have at least one title"
        );

        assert!(
            !rule.versions.is_empty(),
            "rule should have at least one version (STIG ID)"
        );

        assert!(
            !rule.descriptions.is_empty(),
            "rule should have at least one description"
        );

        let description = &rule.descriptions[0];
        assert!(
            description.contains("VulnDiscussion"),
            "first description should contain a VulnDiscussion"
        );

        // XCCDF 1.1 benchmarks contain human-readable check-content text.
        assert!(
            !rule.checks.is_empty(),
            "rule should have at least one check element"
        );

        let check = &rule.checks[0];
        assert!(
            !check.check_contents.is_empty(),
            "check should contain at least one check-content (human text)"
        );

        assert!(
            !rule.fixtexts.is_empty(),
            "rule should have at least one fixtext"
        );

        assert!(
            !rule.idents.is_empty(),
            "rule should contain at least one ident (CCI reference)"
        );
    }

    /// Read the RHEL 8 V2R6 STIG zip asset and convert it into a native
    /// Benchmark.
    #[test]
    fn parse_xccdf_v1_1_convert() {
        let file = File::open("../test_assets/U_RHEL_8_V2R6_STIG.zip")
            .expect("benchmark zip asset should exist");

        let mut archive = ZipArchive::new(file).expect("zip should be valid");

        let mut xml = String::new();
        let mut found = false;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).expect("zip entry should be valid");
            if entry.name().ends_with(".xml") {
                entry
                    .read_to_string(&mut xml)
                    .expect("entry should be valid UTF-8 XML");
                found = true;
                break;
            }
        }
        assert!(found, "zip should contain at least one .xml entry");

        let benchmark: Benchmark =
            quick_xml::de::from_str(&xml).expect("XML should deserialize into Benchmark");

        let benchmark =
            convert_xccdf_v1_1(benchmark).expect("XCCDF 1.1 should convert into a Benchmark");

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
