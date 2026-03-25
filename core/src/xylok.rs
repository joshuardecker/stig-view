use crate::*;

/// Xylok toml's can be deserialized into this struct.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct XylokToml {
    pub versions: Vec<XylokVersion>,
    pub benchmark: XylokBenchmark,
    pub checks: Vec<XylokRule>,
}

/// Date and uuids.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct XylokVersion {
    date: String,
    check_pks: Vec<String>,
}

/// The information I care about from [benchmark].
/// Fail without these fields, they are required.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct XylokBenchmark {
    benchmark_id: String,
    title: String,
}

/// The information I care about from [[checks]].
/// Wrapping is kept very loose on purpose. Allow fields to be empty,
/// that way old and new versions (old versions will lack fields) can be read into the program.
/// Handle parsing after deserialization.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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

impl XylokToml {
    pub fn convert(mut self) -> Option<Benchmark> {
        self.versions.sort_by(|a, b| a.date.cmp(&b.date));

        let version = self.versions.last()?;

        let mut rules = BTreeMap::new();

        self.checks.into_iter().for_each(|xylok_rule| {
            // If there is no uuid, skip.
            if let None = xylok_rule.pk {
                return;
            }

            // Safe .expect call.
            // If the uuid is not contained in the most recent version, skip.
            if !version
                .check_pks
                .contains(&xylok_rule.pk.clone().expect("This should have a pk value."))
            {
                return;
            }

            // Convert the rule and insert it.
            // Known to be from the most recent version, so it is relevant.
            if let Some(rule) = xylok_rule.convert() {
                rules.insert(rule.group_id.clone(), rule);
            }
        });

        Some(Benchmark {
            id: self.benchmark.benchmark_id,
            title: self.benchmark.title,

            rules,
        })
    }
}

impl XylokRule {
    pub fn convert(self) -> Option<Rule> {
        let ccis: Vec<String> = self
            .ccis
            .unwrap_or_default()
            .iter()
            .map(|cci| cci.to_string())
            .collect();

        Some(Rule {
            group_id: self.vulnerability_id?,
            // Trim _rule from the end of the string when converting.
            // Verbose as always.
            rule_id: self
                .rule_id
                .clone()?
                .strip_suffix("_rule")
                .unwrap_or(&self.rule_id?)
                .to_owned(),
            stig_id: self.human_id,
            severity: self.nist_impact?,
            title: self.title?,
            vuln_discussion: self.discussion?,
            check_text: self.content?,
            fix_text: self.fix?,
            cci_refs: match ccis.len() {
                0 => None,
                _ => Some(ccis),
            },
            // Values never saved in Xylok format.
            false_positives: None,
            false_negatives: None,
            documentable: None,
        })
    }
}
