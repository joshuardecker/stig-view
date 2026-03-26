use std::collections::BTreeMap;

use crate::detect_stig_format;
use crate::{Benchmark, Format, Rule, Severity};

/// Load a benchmark given the string of an XCCDF v1.1 xml data.
pub fn load_v1_1(xml: &str) -> Option<Benchmark> {
    let xml_tree = roxmltree::Document::parse(xml).ok()?;
    let mut benchmark = Benchmark::empty();

    let benchmark_node = xml_tree
        .descendants()
        .find(|node| node.tag_name().name() == "Benchmark")?;

    benchmark.id = benchmark_node.attribute("id")?.to_owned();
    benchmark.title = benchmark_node
        .children()
        .find(|node| node.tag_name().name() == "title")
        .and_then(|node| node.text())?
        .to_owned();

    for group in benchmark_node
        .children()
        .filter(|node| node.tag_name().name() == "Group")
    {
        let group_id = match group.attribute("id") {
            Some(id) => id.to_owned(),
            None => continue,
        };

        let rule_node = match group.children().find(|n| n.tag_name().name() == "Rule") {
            Some(node) => node,
            None => continue,
        };

        let rule_id = match rule_node
            .attribute("id")
            .map(|id| id.trim_end_matches("_rule").to_owned())
        {
            Some(id) => id,
            None => continue,
        };

        let severity = parse_severity(rule_node.attribute("severity").unwrap_or(""));

        let stig_id = rule_node
            .children()
            .find(|node| node.tag_name().name() == "version")
            .and_then(|node| node.text())
            .map(|str| str.to_owned());

        let title = match rule_node
            .children()
            .find(|node| node.tag_name().name() == "title")
            .and_then(|node| node.text())
            .map(|str| str.to_owned())
        {
            Some(string) => string,
            None => continue,
        };

        let description = match rule_node
            .children()
            .find(|node| node.tag_name().name() == "description")
            .and_then(|node| node.text())
            .map(|str| str.to_owned())
        {
            Some(string) => string,
            None => continue,
        };

        let vuln_discussion = match data_in_tag(&description, "VulnDiscussion") {
            Some(string) => string,
            None => continue,
        };

        let false_positives =
            data_in_tag(&description, "FalsePositives").filter(|string| !string.is_empty());

        let false_negatives =
            data_in_tag(&description, "FalseNegatives").filter(|string| !string.is_empty());

        let documentable =
            data_in_tag(&description, "Documentable").map(|string| string.trim() == "true");

        let fix_text = match rule_node
            .children()
            .find(|node| node.tag_name().name() == "fixtext")
            .and_then(|node| node.text())
            .map(|str| str.to_owned())
        {
            Some(string) => string,
            None => continue,
        };

        let check_text = match rule_node
            .descendants()
            .find(|node| node.tag_name().name() == "check-content")
            .and_then(|node| node.text())
            .map(|str| str.to_owned())
        {
            Some(string) => string,
            None => continue,
        };

        let cci_refs: Vec<String> = rule_node
            .children()
            .filter(|node| node.tag_name().name() == "ident")
            .filter(|node| {
                node.attribute("system")
                    .map(|str| str.contains("cyber.mil/cci"))
                    .unwrap_or(false)
            })
            .filter_map(|node| node.text())
            .map(|str| str.trim().to_owned())
            .filter(|string| !string.is_empty())
            .collect();

        let rule = Rule {
            group_id: group_id.clone(),
            rule_id,
            stig_id,
            severity,
            title,
            vuln_discussion,
            check_text,
            fix_text,
            cci_refs: (!cci_refs.is_empty()).then_some(cci_refs),
            false_positives,
            false_negatives,
            documentable,
        };

        benchmark.rules.insert(group_id, rule);
    }

    // Can be a valid benchmark, but dont really care if it has no rules.
    if benchmark.rules.len() == 0 {
        return None;
    }

    Some(benchmark)
}

/// Load a benchmark given the string of an XCCDF v1.2 xml data.
pub fn load_v1_2(xml: &str) -> Option<Benchmark> {
    let xml_tree = roxmltree::Document::parse(xml).ok()?;
    let mut benchmark = Benchmark::empty();

    let benchmark_node = xml_tree
        .descendants()
        .find(|node| node.tag_name().name() == "Benchmark")?;

    benchmark.id = benchmark_node.attribute("id")?.to_owned();
    benchmark.title = benchmark_node
        .children()
        .find(|node| node.tag_name().name() == "title")
        .and_then(|node| node.text())?
        .to_owned();

    for group in benchmark_node
        .children()
        .filter(|node| node.tag_name().name() == "Group")
    {
        // v1.2 group IDs are prefixed: "xccdf_mil.disa.stig_group_V-220697"
        // Strip everything up to and including "_group_" to get "V-220697".
        let group_id = match group.attribute("id") {
            Some(id) => strip_scap_prefix(id, "group").to_owned(),
            None => continue,
        };

        let rule_node = match group.children().find(|n| n.tag_name().name() == "Rule") {
            Some(node) => node,
            None => continue,
        };

        // v1.2 rule IDs are prefixed: "xccdf_mil.disa.stig_rule_SV-220697r991589_rule"
        // Strip the prefix up to "_rule_", then the "_rule" suffix.
        let rule_id = match rule_node.attribute("id").map(|id| {
            strip_scap_prefix(id, "rule")
                .trim_end_matches("_rule")
                .to_owned()
        }) {
            Some(id) => id,
            None => continue,
        };

        let severity = parse_severity(rule_node.attribute("severity").unwrap_or(""));

        let stig_id = rule_node
            .children()
            .find(|node| node.tag_name().name() == "version")
            .and_then(|node| node.text())
            .map(|str| str.to_owned());

        let title = match rule_node
            .children()
            .find(|node| node.tag_name().name() == "title")
            .and_then(|node| node.text())
            .map(|str| str.to_owned())
        {
            Some(string) => string,
            None => continue,
        };

        let description = match rule_node
            .children()
            .find(|node| node.tag_name().name() == "description")
            .and_then(|node| node.text())
            .map(|str| str.to_owned())
        {
            Some(string) => string,
            None => continue,
        };

        let vuln_discussion = match data_in_tag(&description, "VulnDiscussion") {
            Some(string) => string,
            None => continue,
        };

        let false_positives =
            data_in_tag(&description, "FalsePositives").filter(|string| !string.is_empty());

        let false_negatives =
            data_in_tag(&description, "FalseNegatives").filter(|string| !string.is_empty());

        let documentable =
            data_in_tag(&description, "Documentable").map(|string| string.trim() == "true");

        let fix_text = match rule_node
            .children()
            .find(|node| node.tag_name().name() == "fixtext")
            .and_then(|node| node.text())
            .map(|str| str.to_owned())
        {
            Some(string) => string,
            None => continue,
        };

        let check_text = match rule_node
            .descendants()
            .find(|node| node.tag_name().name() == "check-content")
            .and_then(|node| node.text())
            .map(|str| str.to_owned())
        {
            Some(string) => string,
            None => continue,
        };

        let cci_refs: Vec<String> = rule_node
            .children()
            .filter(|node| node.tag_name().name() == "ident")
            .filter(|node| {
                node.attribute("system")
                    .map(|str| str.contains("cyber.mil/cci"))
                    .unwrap_or(false)
            })
            .filter_map(|node| node.text())
            .map(|str| str.trim().to_owned())
            .filter(|string| !string.is_empty())
            .collect();

        let rule = Rule {
            group_id: group_id.clone(),
            rule_id,
            stig_id,
            severity,
            title,
            vuln_discussion,
            check_text,
            fix_text,
            cci_refs: (!cci_refs.is_empty()).then_some(cci_refs),
            false_positives,
            false_negatives,
            documentable,
        };

        benchmark.rules.insert(group_id, rule);
    }

    // Can be a valid benchmark, but dont really care if it has no rules.
    if benchmark.rules.len() == 0 {
        return None;
    }

    Some(benchmark)
}

/// Strips the SCAP naming convention prefix from an ID.
/// e.g. strip_scap_prefix("xccdf_mil.disa.stig_group_V-220697", "group") -> "V-220697"
fn strip_scap_prefix<'a>(id: &'a str, type_tag: &str) -> &'a str {
    let marker = format!("_{type_tag}_");
    id.find(&marker)
        .map(|i| &id[i + marker.len()..])
        .unwrap_or(id)
}

/// Return the data in the given tag.
/// Ex: <Hello> ... </Hello>.
/// All contents between Hello's is returned.
/// None if the tag does not exist.
fn data_in_tag(str: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");

    let start = str.find(&open)? + open.len();
    let end = str.find(&close)?;

    Some(str[start..end].trim().to_owned())
}

/// Convert a str to a Severity.
fn parse_severity(str: &str) -> Severity {
    match str {
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        _ => Severity::Unknown,
    }
}
