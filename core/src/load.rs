use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::BTreeMap;

use crate::{Benchmark, BenchmarkErr, Format, Rule, Severity};
use crate::detect_stig_format;

/// Parse an XCCDF v1.1 benchmark from a raw XML string.
///
/// The string is typically obtained from [`detect_stig_format`], which reads
/// (and possibly unzips) the file once and stores the XML in
/// [`Format::XccdfV1_1`]. Accepting a `&str` here means no second read or
/// unzip is needed.
pub fn load_xccdf_v1_1(xml: &str) -> Result<Benchmark, BenchmarkErr> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    // Benchmark-level fields
    let mut bench_id = String::new();
    let mut bench_title = String::new();
    let mut bench_version: Option<String> = None;
    let mut bench_release: Option<String> = None;
    let mut bench_description: Option<String> = None;
    let mut bench_status: Option<String> = None;
    let mut bench_status_date: Option<String> = None;

    // Parsing state.
    // Each entry is a &'static str label for the context we pushed.
    let mut context: Vec<&'static str> = Vec::new();
    let mut current_group_id = String::new();
    let mut plain_text_id = String::new();
    let mut ident_system = String::new();
    let mut text_buf = String::new();
    // Byte offset in `xml` immediately after the `>` of a <description> tag
    // inside a Rule.  Used to slice the raw description content directly from
    // the source string, avoiding reliance on Text events for that element.
    let mut desc_content_start: usize = 0;

    // Current rule fields — reset on each <Rule>.
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

    let mut rules = BTreeMap::new();

    loop {
        match reader
            .read_event_into(&mut buf)
            .map_err(|_| BenchmarkErr::LoadErr("XCCDF XML parse error."))?
        {
            Event::Start(e) => {
                let local = e.local_name();
                let name = std::str::from_utf8(local.as_ref()).unwrap_or("");

                // Match on (element local-name, current context stack) to push
                // the right label.  Leaf collection contexts also clear the
                // text buffer so stale text from sibling elements cannot bleed
                // in.  Unknown/inline elements push "_" and do not clear so
                // that markup inside text fields (e.g. <code> inside <fixtext>)
                // is transparent — the text nodes inside still accumulate.
                match (name, context.as_slice()) {
                    ("Benchmark", _) => {
                        bench_id = get_attr(&e, b"id").unwrap_or_default();
                        context.push("Benchmark");
                    }
                    ("title", ["Benchmark"]) => {
                        text_buf.clear();
                        context.push("BenchTitle");
                    }
                    ("description", ["Benchmark"]) => {
                        text_buf.clear();
                        context.push("BenchDescription");
                    }
                    ("version", ["Benchmark"]) => {
                        text_buf.clear();
                        context.push("BenchVersion");
                    }
                    ("status", ["Benchmark"]) => {
                        bench_status_date = get_attr(&e, b"date");
                        text_buf.clear();
                        context.push("BenchStatus");
                    }
                    // plain-text id="release-info" carries the release string.
                    ("plain-text", ["Benchmark"]) => {
                        plain_text_id = get_attr(&e, b"id").unwrap_or_default();
                        text_buf.clear();
                        context.push("BenchPlainText");
                    }
                    ("Group", ["Benchmark"]) => {
                        current_group_id = get_attr(&e, b"id").unwrap_or_default();
                        context.push("Group");
                    }
                    ("Rule", ["Benchmark", "Group"]) => {
                        rule_id = get_attr(&e, b"id").unwrap_or_default();
                        rule_severity = get_attr(&e, b"severity").unwrap_or_default();
                        rule_stig_id = None;
                        rule_title = String::new();
                        rule_vuln_discussion = String::new();
                        rule_false_positives = None;
                        rule_false_negatives = None;
                        rule_documentable = None;
                        rule_fix_text = String::new();
                        rule_check_content = String::new();
                        rule_cci_refs = Vec::new();
                        context.push("Rule");
                    }
                    // <version> inside a Rule is the STIG ID (e.g. RHEL-08-010000).
                    ("version", [.., "Rule"]) => {
                        text_buf.clear();
                        context.push("RuleVersion");
                    }
                    ("title", [.., "Rule"]) => {
                        text_buf.clear();
                        context.push("RuleTitle");
                    }
                    // <description> inside a Rule.  Record where the content
                    // starts in the source string so we can slice it out
                    // directly when the element closes — no Text event needed.
                    ("description", [.., "Rule"]) => {
                        desc_content_start = reader.buffer_position() as usize;
                        context.push("RuleDesc");
                    }
                    ("fixtext", [.., "Rule"]) => {
                        text_buf.clear();
                        context.push("RuleFixText");
                    }
                    ("check", [.., "Rule"]) => {
                        context.push("RuleCheck");
                    }
                    ("check-content", [.., "RuleCheck"]) => {
                        text_buf.clear();
                        context.push("RuleCheckContent");
                    }
                    // CCI references live in <ident system="..."> elements.
                    ("ident", [.., "Rule"]) => {
                        ident_system = get_attr(&e, b"system").unwrap_or_default();
                        text_buf.clear();
                        context.push("RuleIdent");
                    }
                    _ => {
                        // Unknown or unneeded element.  Push "_" without
                        // clearing text_buf so any text inside is transparent
                        // to the enclosing collection context.
                        context.push("_");
                    }
                }
            }

            Event::End(_) => {
                let ctx = context.last().copied().unwrap_or("");

                match ctx {
                    "BenchTitle" => bench_title = text_buf.clone(),
                    "BenchDescription" => bench_description = Some(text_buf.clone()),
                    "BenchVersion" => bench_version = Some(text_buf.clone()),
                    "BenchStatus" => bench_status = Some(text_buf.clone()),
                    "BenchPlainText" => {
                        if plain_text_id == "release-info" {
                            bench_release = Some(text_buf.clone());
                        }
                    }
                    "RuleVersion" => rule_stig_id = Some(text_buf.clone()),
                    "RuleTitle" => rule_title = text_buf.clone(),
                    // Slice the raw description content directly from the
                    // source XML string.  buffer_position() is now just past
                    // the '>' of </description> (14 bytes).  Works for both
                    // real XML child elements and escaped-XML text nodes.
                    "RuleDesc" => {
                        let end_pos = reader.buffer_position() as usize;
                        let content_end = end_pos.saturating_sub("</description>".len());
                        if desc_content_start < content_end {
                            let raw = xml.as_bytes()
                                .get(desc_content_start..content_end)
                                .and_then(|b| std::str::from_utf8(b).ok())
                                .unwrap_or("");
                            let decoded = xml_unescape(raw);
                            rule_vuln_discussion =
                                extract_tag(&decoded, "VulnDiscussion").unwrap_or_default();
                            rule_false_positives =
                                extract_tag(&decoded, "FalsePositives").filter(|s| !s.is_empty());
                            rule_false_negatives =
                                extract_tag(&decoded, "FalseNegatives").filter(|s| !s.is_empty());
                            rule_documentable = extract_tag(&decoded, "Documentable")
                                .map(|s| s.trim() == "true");
                        }
                    }
                    "RuleFixText" => rule_fix_text = text_buf.clone(),
                    "RuleCheckContent" => rule_check_content = text_buf.clone(),
                    "RuleIdent" => {
                        if ident_system.contains("cyber.mil/cci") {
                            let cci = text_buf.trim().to_owned();
                            if !cci.is_empty() {
                                rule_cci_refs.push(cci);
                            }
                        }
                    }
                    "Rule" => {
                        let rule = Rule {
                            group_id: current_group_id.clone(),
                            rule_id: rule_id.trim_end_matches("_rule").to_owned(),
                            stig_id: rule_stig_id.clone(),
                            severity: parse_severity(&rule_severity),
                            title: rule_title.clone(),
                            vuln_discussion: rule_vuln_discussion.clone(),
                            check_text: rule_check_content.clone(),
                            fix_text: rule_fix_text.clone(),
                            cci_refs: if rule_cci_refs.is_empty() {
                                None
                            } else {
                                Some(rule_cci_refs.clone())
                            },
                            false_positives: rule_false_positives.clone(),
                            false_negatives: rule_false_negatives.clone(),
                            documentable: rule_documentable,
                        };
                        rules.insert(current_group_id.clone(), rule);
                    }
                    _ => {}
                }

                context.pop();

                // Clear the text buffer after known collection and structural
                // contexts.  For "_" (unknown/inline) contexts we leave it
                // alone so text from inline markup accumulates into the parent.
                if ctx != "_" {
                    text_buf.clear();
                }
            }

            Event::Text(e) => {
                if let Ok(text) = std::str::from_utf8(e.as_ref()) {
                    text_buf.push_str(text);
                }
            }

            Event::CData(e) => {
                // CData sections in descriptions / check-content are rare but valid.
                let bytes: &[u8] = &e;
                if let Ok(text) = std::str::from_utf8(bytes) {
                    text_buf.push_str(text);
                }
            }

            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }

    Ok(Benchmark {
        id: bench_id,
        title: bench_title,
        version: bench_version,
        release: bench_release,
        description: bench_description,
        status: bench_status,
        status_date: bench_status_date,
        rules,
    })
}

/// Decode the five predefined XML entities.  Rule descriptions are stored as
/// a single escaped blob (&lt;VulnDiscussion&gt;…) so one pass is enough.
fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Return the text content between the first `<tag>…</tag>` pair, or `None`
/// if the tag is absent.
fn extract_tag(s: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = s.find(&open)? + open.len();
    let end = s[start..].find(&close)?;
    Some(s[start..start + end].trim().to_owned())
}

/// Extract a named attribute from a start element, returning it as an owned
/// String.  Compares against the local (un-namespaced) attribute name.
fn get_attr(e: &quick_xml::events::BytesStart, name: &[u8]) -> Option<String> {
    e.attributes().flatten().find_map(|attr| {
        if attr.key.local_name().as_ref() == name {
            std::str::from_utf8(attr.value.as_ref()).ok().map(|s| s.to_owned())
        } else {
            None
        }
    })
}

fn parse_severity(s: &str) -> Severity {
    match s {
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
        let benchmark = load_xccdf_v1_1(&xml).expect("Could not load XCCDF v1.1 benchmark.");
        assert!(!benchmark.title.is_empty(), "benchmark title should not be empty");
        assert!(!benchmark.rules.is_empty(), "benchmark should have rules");
        // Spot-check: every rule must have a non-empty title and check text.
        for (group_id, rule) in &benchmark.rules {
            assert!(!rule.title.is_empty(), "rule {group_id} has an empty title");
            assert!(!rule.check_text.is_empty(), "rule {group_id} has empty check text");
        }
    } else {
        panic!("Expected XccdfV1_1 format.");
    }
}
