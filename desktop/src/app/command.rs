use std::collections::HashMap;

use regex::Regex;
use stig_view_core::Benchmark;

use crate::app::*;

#[derive(Debug, Clone)]
pub enum CommandErr {
    NotCommand,
    RegexErr,
}

pub fn parse_command(input: &str) -> Result<Command, CommandErr> {
    let cmd_regex = Regex::new(r"(\w+)\s*(.*)").map_err(|_| CommandErr::RegexErr)?;
    let captures = cmd_regex.captures(input).ok_or(CommandErr::RegexErr)?;

    match captures[1].to_string().as_str() {
        "name" => Ok(Command::NameSearch(captures[2].to_string())),
        "title" => Ok(Command::NameSearch(captures[2].to_string())),

        "find" => Ok(Command::KeywordSearch(captures[2].to_string())),
        "search" => Ok(Command::KeywordSearch(captures[2].to_string())),

        "reset" => Ok(Command::Reset),

        _ => Err(CommandErr::NotCommand),
    }
}

pub fn run_search_cmd(
    cmd: Command,
    benchmark: Benchmark,
    mut pins: HashMap<String, Pinned>,
) -> Result<HashMap<String, Pinned>, CommandErr> {
    match cmd {
        Command::KeywordSearch(keyword) => {
            let re = Regex::new(&keyword).map_err(|_| CommandErr::RegexErr)?;

            for (name, rule) in benchmark.rules.iter() {
                let mut is_match = false;

                // Search through all string fields for a match.
                is_match |= re.is_match(&rule.group_id);
                is_match |= re.is_match(&rule.rule_id);
                // Cant make the default an empty string, because it would be very easy to unintentionally sort
                // all names that dont have a stig id by inputting an empty filter query.
                is_match |= re.is_match(
                    &rule
                        .stig_id
                        .clone()
                        .unwrap_or("oopsie this is a long string".to_string()),
                );
                is_match |= re.is_match(&rule.title);
                is_match |= re.is_match(&rule.vuln_discussion);
                is_match |= re.is_match(&rule.check_text);
                is_match |= re.is_match(&rule.fix_text);
                is_match |= re.is_match(&rule.cci_refs.as_deref().unwrap_or(&[]).join(" "));
                // Cant make the default an empty string, because it would be very easy to unintentionally sort
                // all names that dont have a stig id by inputting an empty filter query.
                is_match |= re.is_match(
                    &rule
                        .false_positives
                        .clone()
                        .unwrap_or("oopsie this is a long string".to_string()),
                );
                // Cant make the default an empty string, because it would be very easy to unintentionally sort
                // all names that dont have a stig id by inputting an empty filter query.
                is_match |= re.is_match(
                    &rule
                        .false_negatives
                        .clone()
                        .unwrap_or("oopsie this is a long string".to_string()),
                );

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
            let re = Regex::new(&name).map_err(|_| CommandErr::RegexErr)?;

            for (name, rule) in benchmark.rules.iter() {
                let mut is_match = false;

                is_match |= re.is_match(&rule.group_id);
                is_match |= re.is_match(&rule.rule_id);
                // Cant make the default an empty string, because it would be very easy to unintentionally sort
                // all names that dont have a stig id by inputting an empty filter query.
                is_match |= re.is_match(
                    &rule
                        .stig_id
                        .clone()
                        .unwrap_or("oopsie this is a long string".to_string()),
                );

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

    Ok(pins)
}
