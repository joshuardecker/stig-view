use crate::{Benchmark, Rule, Severity};

/// Load a benchmark given the string of a CKL xml data.
pub fn load_ckl(xml: &str) -> Option<Benchmark> {
    let xml_tree = roxmltree::Document::parse(xml).ok()?;
    let mut benchmark = Benchmark::empty();

    let istig_node = xml_tree
        .descendants()
        .find(|node| node.tag_name().name() == "iSTIG")?;

    // Benchmark id and title are stored as SI_DATA key-value pairs under STIG_INFO.
    let stig_info_node = istig_node
        .children()
        .find(|node| node.tag_name().name() == "STIG_INFO")?;

    for si_data in stig_info_node
        .children()
        .filter(|node| node.tag_name().name() == "SI_DATA")
    {
        let name = si_data
            .children()
            .find(|node| node.tag_name().name() == "SID_NAME")
            .and_then(|node| node.text())
            .unwrap_or("");

        let data = si_data
            .children()
            .find(|node| node.tag_name().name() == "SID_DATA")
            .and_then(|node| node.text())
            .unwrap_or("");

        match name {
            "stigid" => benchmark.id = data.to_owned(),
            "title" => benchmark.title = data.to_owned(),
            _ => {}
        }
    }

    if benchmark.id.is_empty() || benchmark.title.is_empty() {
        return None;
    }

    for vuln in istig_node
        .children()
        .filter(|node| node.tag_name().name() == "VULN")
    {
        let mut group_id = String::new();
        let mut rule_id = String::new();
        let mut stig_id: Option<String> = None;
        let mut severity_str = String::new();
        let mut title = String::new();
        let mut vuln_discussion = String::new();
        let mut check_text = String::new();
        let mut fix_text = String::new();
        let mut cci_refs: Vec<String> = Vec::new();
        let mut false_positives: Option<String> = None;
        let mut false_negatives: Option<String> = None;
        let mut documentable: Option<bool> = None;

        for stig_data in vuln
            .children()
            .filter(|node| node.tag_name().name() == "STIG_DATA")
        {
            let attr = stig_data
                .children()
                .find(|node| node.tag_name().name() == "VULN_ATTRIBUTE")
                .and_then(|node| node.text())
                .unwrap_or("");

            let data = stig_data
                .children()
                .find(|node| node.tag_name().name() == "ATTRIBUTE_DATA")
                .and_then(|node| node.text())
                .unwrap_or("");

            match attr {
                "Vuln_Num" => group_id = data.to_owned(),
                "Rule_ID" => rule_id = data.trim_end_matches("_rule").to_owned(),
                "Rule_Ver" => stig_id = (!data.is_empty()).then(|| data.to_owned()),
                "Severity" => severity_str = data.to_owned(),
                "Rule_Title" => title = data.to_owned(),
                "Vuln_Discuss" => vuln_discussion = data.to_owned(),
                "Check_Content" => check_text = data.to_owned(),
                "Fix_Text" => fix_text = data.to_owned(),
                "CCI_REF" => {
                    if !data.is_empty() {
                        cci_refs.push(data.to_owned());
                    }
                }
                "False_Positives" => {
                    false_positives = (!data.is_empty()).then(|| data.to_owned())
                }
                "False_Negatives" => {
                    false_negatives = (!data.is_empty()).then(|| data.to_owned())
                }
                "Documentable" => documentable = Some(data.trim() == "true"),
                _ => {}
            }
        }

        if group_id.is_empty()
            || rule_id.is_empty()
            || title.is_empty()
            || vuln_discussion.is_empty()
            || check_text.is_empty()
            || fix_text.is_empty()
        {
            continue;
        }

        let rule = Rule {
            group_id: group_id.clone(),
            rule_id,
            stig_id,
            severity: parse_severity(&severity_str),
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

    if benchmark.rules.is_empty() {
        return None;
    }

    Some(benchmark)
}

fn parse_severity(str: &str) -> Severity {
    match str {
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        _ => Severity::Unknown,
    }
}
