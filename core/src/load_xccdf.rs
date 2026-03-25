use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::collections::BTreeMap;

use crate::detect_stig_format;
use crate::{Benchmark, Format, Rule, Severity};

#[derive(Debug, Clone, PartialEq, Eq)]
enum StackItem {
    Benchmark,
    BenchmarkTitle,

    Group,

    Rule,
    RuleGroupId,
    RuleTitle,
    RuleVulnDiscussion,
    RuleCheck, // The parent of RuleCheckText
    RuleCheckText,
    RuleFixText,
    Indent,

    Unknown,
    None,
}

pub fn load_v1_1(xml: &str) -> Option<Benchmark> {
    let mut xml_reader = Reader::from_str(xml);
    xml_reader.config_mut().trim_text(true);

    // Buffer that xml will be read into.
    let mut buf = Vec::new();

    let mut benchmark = Benchmark::empty();

    // Inital fields of the rule. Will be reused every rule found in the xml.
    let mut rule_id = String::new();
    let mut rule_severity = String::new();
    let mut rule_stig_id: Option<String> = None;
    let mut rule_title = String::new();
    let mut rule_vuln_discussion = String::new();
    let mut rule_false_positives: Option<String> = None;
    let mut rule_false_negatives: Option<String> = None;
    let mut rule_documentable: Option<bool> = None;
    let mut rule_fix_text = String::new();
    let mut rule_check_content = String::new();
    let mut rule_cci_refs: Vec<String> = Vec::new();

    let mut stack = vec![];

    let mut current_group_id = String::new();
    let mut plain_text_id = String::new();
    let mut ident_system = String::new();
    // A buffer that contains the text in a xml element.
    let mut text_buf = String::new();

    // Byte offset in `xml` immediately after the `>` of a <description> tag
    // inside a Rule.  Used to slice the raw description content directly from
    // the source string, avoiding reliance on Text events for that element.
    let mut desc_content_start = 0;

    loop {
        match xml_reader.read_event_into(&mut buf).ok()? {
            Event::Eof => break,

            Event::Start(start) => {
                let name = start.local_name();
                // Convert name to utf8.
                let name = std::str::from_utf8(name.as_ref()).unwrap_or("");

                match (name, stack.last().unwrap_or(&StackItem::None)) {
                    ("Benchmark", _) => {
                        // Return none if no benchmark id, this is a required field.
                        benchmark.id = get_attribute(&start, b"id")?;

                        stack.push(StackItem::Benchmark);
                    }
                    ("title", &StackItem::Benchmark) => {
                        text_buf.clear();
                        stack.push(StackItem::BenchmarkTitle);
                    }
                    ("Group", &StackItem::Benchmark) => {
                        current_group_id = get_attribute(&start, b"id").unwrap_or_default();

                        stack.push(StackItem::Group);
                    }
                    ("Rule", &StackItem::Group) => {
                        rule_id = get_attribute(&start, b"id").unwrap_or("".to_string());
                        rule_severity = get_attribute(&start, b"severity").unwrap_or_default();
                        rule_stig_id = None;
                        rule_title = String::new();
                        rule_vuln_discussion = String::new();
                        rule_false_positives = None;
                        rule_false_negatives = None;
                        rule_documentable = None;
                        rule_fix_text = String::new();
                        rule_check_content = String::new();
                        rule_cci_refs = Vec::new();

                        stack.push(StackItem::Rule);
                    }

                    ("version", &StackItem::Rule) => {
                        text_buf.clear();
                        stack.push(StackItem::RuleGroupId);
                    }
                    ("title", &StackItem::Rule) => {
                        text_buf.clear();
                        stack.push(StackItem::RuleTitle);
                    }
                    // <description> inside a Rule.  Record where the content
                    // starts in the source string so we can slice it out
                    // directly when the element closes — no Text event needed.
                    ("description", &StackItem::Rule) => {
                        desc_content_start = xml_reader.buffer_position() as usize;
                        stack.push(StackItem::RuleVulnDiscussion);
                    }
                    ("fixtext", &StackItem::Rule) => {
                        text_buf.clear();
                        stack.push(StackItem::RuleFixText);
                    }
                    ("check", &StackItem::Rule) => {
                        stack.push(StackItem::RuleCheck);
                    }
                    ("check-content", &StackItem::RuleCheck) => {
                        text_buf.clear();
                        stack.push(StackItem::RuleCheckText);
                    }
                    // CCI references live in <ident system="..."> elements.
                    ("ident", &StackItem::Rule) => {
                        ident_system = get_attribute(&start, b"system").unwrap_or_default();
                        stack.push(StackItem::Indent);
                        text_buf.clear();
                    }
                    _ => {
                        stack.push(StackItem::Unknown);
                    }
                }
            }

            Event::End(_) => {
                let ctx = stack.last().unwrap_or(&StackItem::None).to_owned();

                match &ctx {
                    &StackItem::Benchmark => benchmark.title = std::mem::take(&mut text_buf),

                    &StackItem::RuleGroupId => rule_stig_id = Some(std::mem::take(&mut text_buf)),
                    &StackItem::RuleTitle => rule_title = std::mem::take(&mut text_buf),
                    // Slice the raw description content directly from the
                    // source XML string.  buffer_position() is now just past
                    // the '>' of </description> (14 bytes).  Works for both
                    // real XML child elements and escaped-XML text nodes.
                    &StackItem::RuleVulnDiscussion => {
                        let end_pos = xml_reader.buffer_position() as usize;
                        // Returns None if overflow occurs from the subtraction.
                        let content_end = end_pos.checked_sub("</description>".len())?;
                        if desc_content_start < content_end {
                            let raw = xml
                                .as_bytes()
                                .get(desc_content_start..content_end)
                                .and_then(|slice| std::str::from_utf8(slice).ok())
                                .unwrap_or("");
                            let decoded = decode_symbols(raw);
                            rule_vuln_discussion =
                                extract_tag(&decoded, "VulnDiscussion").unwrap_or_default();
                            rule_false_positives =
                                extract_tag(&decoded, "FalsePositives").filter(|s| !s.is_empty());
                            rule_false_negatives =
                                extract_tag(&decoded, "FalseNegatives").filter(|s| !s.is_empty());
                            rule_documentable =
                                extract_tag(&decoded, "Documentable").map(|s| s.trim() == "true");
                        }
                    }
                    &StackItem::RuleFixText => rule_fix_text = std::mem::take(&mut text_buf),
                    &StackItem::RuleCheckText => rule_check_content = std::mem::take(&mut text_buf),
                    &StackItem::Indent => {
                        if ident_system.contains("cyber.mil/cci") {
                            let cci = text_buf.trim().to_owned();
                            if !cci.is_empty() {
                                rule_cci_refs.push(cci);
                            }
                        }
                    }
                    &StackItem::Rule => {
                        let rule = Rule {
                            group_id: current_group_id.clone(),
                            rule_id: rule_id.trim_end_matches("_rule").to_owned(),
                            stig_id: rule_stig_id.take(),
                            severity: parse_severity(&rule_severity),
                            title: std::mem::take(&mut rule_title),
                            vuln_discussion: std::mem::take(&mut rule_vuln_discussion),
                            check_text: std::mem::take(&mut rule_check_content),
                            fix_text: std::mem::take(&mut rule_fix_text),
                            cci_refs: (!rule_cci_refs.is_empty())
                                .then(|| std::mem::take(&mut rule_cci_refs)),
                            false_positives: rule_false_positives.take(),
                            false_negatives: rule_false_negatives.take(),
                            documentable: rule_documentable,
                        };
                        benchmark.rules.insert(current_group_id.clone(), rule);
                    }
                    _ => (),
                }

                let _ = stack.pop();

                if ctx != StackItem::Unknown {
                    text_buf.clear();
                }
            }

            Event::Text(char) => {
                if let Ok(char) = std::str::from_utf8(char.as_ref()) {
                    text_buf.push_str(char);
                }
            }

            // Usually special characters are embeded as &lt instead of < ,
            // which is handled by the decode_symbols() function, but it isnt a guarantee.
            // They can also be put into comments, so that has to be handled here.
            Event::CData(char) => {
                // CData sections in descriptions / check-content are rare but valid.
                if let Ok(char) = std::str::from_utf8(char.as_ref()) {
                    text_buf.push_str(char);
                }
            }

            _ => continue,
        }

        buf.clear();
    }

    Some(benchmark)
}

/// These characters need to be decoded to properly parse.
fn decode_symbols(str: &str) -> String {
    str.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Return the data in the given tag.
/// Ex: <Hello> ... </Hello>.
/// All contents between Hello's is returned.
/// None if the tag does not exist.
fn extract_tag(s: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = s.find(&open)? + open.len();
    let end = s[start..].find(&close)?;
    Some(s[start..start + end].trim().to_owned())
}

/// Looks for the value of of an attribute with the given name, in the given
/// xml element. Returns None if there is no key with that name.
fn get_attribute(start: &BytesStart, name: &[u8]) -> Option<String> {
    start.attributes().flatten().find_map(|attribute| {
        if attribute.key.local_name().as_ref() == name {
            Some(
                std::str::from_utf8(attribute.value.as_ref())
                    .ok()?
                    .to_owned(),
            )
        } else {
            None
        }
    })
}

fn parse_severity(str: &str) -> Severity {
    match str {
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        _ => Severity::Unknown,
    }
}

#[test]
fn test_saving_benchmark() {
    let format =
        detect_stig_format("../test_assets/packed.toml").expect("Could not load Xylok benchmark.");

    if let Format::Xylok(xylok_benchmark) = format {
        let benchmark = xylok_benchmark
            .convert()
            .expect("Could not convert benchmark.");
        benchmark.save().expect("Could not save benchmark.");
    } else {
        panic!("Incorrect format loaded.")
    }
}

#[test]
fn test_load_xccdf_v1_1() {
    let format = detect_stig_format("../test_assets/U_RHEL_8_V2R6_STIG.zip")
        .expect("Could not detect format.");

    if let Format::XccdfV1_1(xml) = format {
        let benchmark = load_v1_1(&xml).expect("Could not load XCCDF v1.1 benchmark.");
        assert!(
            !benchmark.title.is_empty(),
            "benchmark title should not be empty"
        );
        assert!(!benchmark.rules.is_empty(), "benchmark should have rules");
        // Spot-check: every rule must have a non-empty title and check text.
        for (group_id, rule) in &benchmark.rules {
            assert!(!rule.title.is_empty(), "rule {group_id} has an empty title");
            assert!(
                !rule.check_text.is_empty(),
                "rule {group_id} has empty check text"
            );
        }
    } else {
        panic!("Expected XccdfV1_1 format.");
    }
}
