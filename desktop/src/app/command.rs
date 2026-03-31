use std::collections::HashMap;

use regex::Regex;
use stig_view_core::Benchmark;

use crate::app::*;

pub fn parse_command(input: &str) -> Option<Command> {
    let cmd_regex: Regex =
        Regex::new(r"(\w+)\s*(.*)").expect("Error in creating command parse regex.");

    let captures = cmd_regex.captures(input)?;

    match captures[1].to_string().as_str() {
        "name" => Some(Command::NameSearch(captures[2].to_string())),
        "title" => Some(Command::NameSearch(captures[2].to_string())),

        "find" => Some(Command::KeywordSearch(captures[2].to_string())),
        "search" => Some(Command::KeywordSearch(captures[2].to_string())),

        "reset" => Some(Command::Reset),

        _ => None,
    }
}

pub fn run_search_cmd(
    cmd: Command,
    benchmark: &Benchmark,
    mut pins: HashMap<String, Pinned>,
) -> Option<HashMap<String, Pinned>> {
    match cmd {
        Command::KeywordSearch(keyword) => {
            let re = Regex::new(&keyword).ok()?;

            for (name, rule) in benchmark.rules.iter() {
                let mut is_match = false;

                // Search through all string fields for a match.map_err(|_| CommandErr::RegexErr)
                is_match |= re.is_match(&rule.group_id);
                is_match |= re.is_match(&rule.rule_id);

                if let Some(stig_id) = &rule.stig_id {
                    is_match |= re.is_match(stig_id);
                }

                is_match |= re.is_match(&rule.title);
                is_match |= re.is_match(&rule.vuln_discussion);
                is_match |= re.is_match(&rule.check_text);
                is_match |= re.is_match(&rule.fix_text);
                is_match |= re.is_match(&rule.cci_refs.as_deref().unwrap_or(&[]).join(" "));

                if let Some(false_postive) = &rule.false_positives {
                    is_match |= re.is_match(false_postive);
                }

                if let Some(false_negative) = &rule.false_negatives {
                    is_match |= re.is_match(false_negative);
                }

                if is_match {
                    match pins.get(name).unwrap_or(&Pinned::Not) {
                        Pinned::Not => {
                            let _ = pins.insert(name.to_owned(), Pinned::ByFilter);
                        }
                        Pinned::ByUser => {
                            let _ = pins.insert(name.to_owned(), Pinned::ByFilterAndUser);
                        }
                        Pinned::ByFilter => (),
                        Pinned::ByFilterAndUser => (),
                    }

                    continue;
                } else {
                    match pins.get(name).unwrap_or(&Pinned::Not) {
                        Pinned::Not => (),
                        Pinned::ByUser => (),
                        Pinned::ByFilter => {
                            let _ = pins.insert(name.to_owned(), Pinned::Not);
                        }
                        Pinned::ByFilterAndUser => {
                            let _ = pins.insert(name.to_owned(), Pinned::ByUser);
                        }
                    }
                }
            }
        }
        Command::NameSearch(name) => {
            let re = Regex::new(&name).ok()?;

            for (name, rule) in benchmark.rules.iter() {
                let mut is_match = false;

                is_match |= re.is_match(&rule.group_id);
                is_match |= re.is_match(&rule.rule_id);

                if let Some(stig_id) = &rule.stig_id {
                    is_match |= re.is_match(stig_id);
                }

                if is_match {
                    match pins.get(name).unwrap_or(&Pinned::Not) {
                        Pinned::Not => {
                            let _ = pins.insert(name.to_owned(), Pinned::ByFilter);
                        }
                        Pinned::ByUser => {
                            let _ = pins.insert(name.to_owned(), Pinned::ByFilterAndUser);
                        }
                        Pinned::ByFilter => (),
                        Pinned::ByFilterAndUser => (),
                    }

                    continue;
                } else {
                    match pins.get(name).unwrap_or(&Pinned::Not) {
                        Pinned::Not => (),
                        Pinned::ByUser => (),
                        Pinned::ByFilter => {
                            let _ = pins.insert(name.to_owned(), Pinned::Not);
                        }
                        Pinned::ByFilterAndUser => {
                            let _ = pins.insert(name.to_owned(), Pinned::ByUser);
                        }
                    }
                }
            }
        }
        Command::Reset => {
            pins.iter_mut()
                .for_each(|(_name, value)| *value = Pinned::Not);
        }
    }

    Some(pins)
}
