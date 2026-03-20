use regex::Regex;
use stig_view_core::{DB, Pinned};

use crate::app::Command;

#[derive(Debug, Clone)]
pub enum CommandErr {
    NotCommand,
    RegexErr,
}

pub fn parse_command(input: &str) -> Result<Command, CommandErr> {
    let cmd_regex = Regex::new(r"(\w+)\s+(.*)").map_err(|_| CommandErr::RegexErr)?;
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

pub async fn run_search_cmd(cmd: Command, db: DB) -> Result<(), CommandErr> {
    match cmd {
        Command::KeywordSearch(keyword) => {
            let re = Regex::new(&keyword).map_err(|_| CommandErr::RegexErr)?;

            let snapshot = db.snapshot().await;

            for (name, rule) in snapshot.rules.iter() {
                let mut is_match = false;

                // Search through all string fields for a match.
                is_match |= re.is_match(&rule.group_id);
                is_match |= re.is_match(&rule.rule_id);
                // Cant make the default an empty string, because it would be very easy to unintentionally sort
                // all names that dont have a stig id by inputting an empty filter query.
                is_match = re.is_match(
                    &rule
                        .stig_id
                        .unwrap_or("oopsie this is a long string".to_string()),
                );
                is_match |= re.is_match(&rule.title);
                is_match |= re.is_match(&rule.vuln_discussion);
                is_match |= re.is_match(&rule.check_text);
                is_match |= re.is_match(&rule.fix_text);
                is_match |= re.is_match(
                    &rule
                        .cci_refs
                        .unwrap_or(vec![String::new()])
                        .iter()
                        .flatten()
                        .collect(),
                );
                // Cant make the default an empty string, because it would be very easy to unintentionally sort
                // all names that dont have a stig id by inputting an empty filter query.
                is_match |= re.is_match(
                    &rule
                        .false_positives
                        .unwrap_or("oopsie this is a long string".to_string()),
                );
                // Cant make the default an empty string, because it would be very easy to unintentionally sort
                // all names that dont have a stig id by inputting an empty filter query.
                is_match |= re.is_match(
                    &rule
                        .false_negatives
                        .unwrap_or("oopsie this is a long string".to_string()),
                );

                if is_match {
                    match db.get_pin(name).await {
                        Pinned::Not => {
                            db.set_pin(name.to_owned(), Pinned::ByFilter).await;
                        }
                        Pinned::ByUser => {
                            db.set_pin(name.to_owned(), Pinned::ByFilterAndUser).await;
                        }
                        Pinned::ByFilter => (),
                        Pinned::ByFilterAndUser => (),
                    }

                    continue;
                } else {
                    match db.get_pin(name).await {
                        Pinned::Not => (),
                        Pinned::ByUser => (),
                        Pinned::ByFilter => {
                            db.set_pin(name.to_owned(), Pinned::Not).await;
                        }
                        Pinned::ByFilterAndUser => {
                            db.set_pin(name.to_owned(), Pinned::ByUser).await;
                        }
                    }
                }
            }
        }
        Command::NameSearch(name) => {
            let re = Regex::new(&name).map_err(|_| CommandErr::RegexErr)?;

            let snapshot = db.snapshot().await;

            for (name, rule) in snapshot.rules.iter() {
                let mut is_match = false;

                is_match = re.is_match(&rule.group_id);
                is_match = re.is_match(&rule.rule_id);
                // Cant make the default an empty string, because it would be very easy to unintentionally sort
                // all names that dont have a stig id by inputting an empty filter query.
                is_match = re.is_match(
                    &rule
                        .stig_id
                        .unwrap_or("oopsie this is a long string".to_string()),
                );

                if is_match {
                    match db.get_pin(name).await {
                        Pinned::Not => {
                            db.set_pin(name.to_owned(), Pinned::ByFilter).await;
                        }
                        Pinned::ByUser => {
                            db.set_pin(name.to_owned(), Pinned::ByFilterAndUser).await;
                        }
                        Pinned::ByFilter => (),
                        Pinned::ByFilterAndUser => (),
                    }

                    continue;
                } else {
                    match db.get_pin(name).await {
                        Pinned::Not => (),
                        Pinned::ByUser => (),
                        Pinned::ByFilter => {
                            db.set_pin(name.to_owned(), Pinned::Not).await;
                        }
                        Pinned::ByFilterAndUser => {
                            db.set_pin(name.to_owned(), Pinned::ByUser).await;
                        }
                    }
                }
            }
        }
        Command::Reset => {
            db.unpin_all().await;
        }
    }

    Ok(())
}
