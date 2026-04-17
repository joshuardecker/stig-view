use regex::Regex;
use std::collections::HashMap;
use stig_view_core::Benchmark;

use crate::app::*;

/// Parse the given str into a command that can be run on a benchmark.
pub fn parse_command(input: &str) -> Option<Command> {
    let phrase = input.trim().to_string();

    if phrase.is_empty() {
        None
    } else if &phrase == "reset" {
        Some(Command::Reset)
    } else {
        Some(Command::Phrase(phrase))
    }
}

/// Run the given command on a given benchmark, updating what STIGs are pinned.
pub fn run_search_cmd(
    cmd: Command,
    benchmark: &Benchmark,
    mut pins: HashMap<String, Pinned>,
) -> Option<HashMap<String, Pinned>> {
    match cmd {
        Command::Phrase(phrase) => {
            let re = Regex::new(&phrase).ok()?;

            for (name, rule) in benchmark.rules.iter() {
                let mut is_match = false;

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

        Command::Reset => {
            pins.iter_mut()
                .for_each(|(_name, value)| *value = Pinned::Not);
        }
    }

    Some(pins)
}
