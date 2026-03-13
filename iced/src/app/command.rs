use regex::Regex;
use stig_view_core::db::{DB, Data, Pinned};

use crate::app::Command;

#[derive(Debug, Clone)]
pub enum CommandErr {
    NotCommand,
    RegexErr,
    DBCacheErr,
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

pub async fn run_search_cmd(cmd: Command, db: DB) -> Result<(), CommandErr> {
    match cmd {
        Command::KeywordSearch(keyword) => {
            let re = Regex::new(&keyword).map_err(|_| CommandErr::RegexErr)?;

            let snapshot = db.snapshot().map_err(|_| CommandErr::DBCacheErr)?;

            for (name, data) in snapshot.iter() {
                let mut is_match = false;

                is_match |= re.is_match(&data.get_stig().version);
                is_match |= re.is_match(&data.get_stig().intro);
                is_match |= re.is_match(&data.get_stig().desc);
                is_match |= re.is_match(&data.get_stig().check_text);
                is_match |= re.is_match(&data.get_stig().fix_text);
                is_match |= re.is_match(&data.get_stig().similar_checks);

                if is_match {
                    match data.get_pin() {
                        Pinned::Not => {
                            let mut data = data.to_owned();
                            data.set_pin(Pinned::ByFilter);

                            db.insert(name.to_owned(), data).await;
                        }
                        Pinned::ByUser => {
                            let mut data = data.to_owned();
                            data.set_pin(Pinned::ByFilterAndUser);

                            db.insert(name.to_owned(), data).await;
                        }
                        Pinned::ByFilter => (),
                        Pinned::ByFilterAndUser => (),
                    }

                    continue;
                } else {
                    match data.get_pin() {
                        Pinned::Not => (),
                        Pinned::ByUser => (),
                        Pinned::ByFilter => {
                            let mut data = data.to_owned();
                            data.set_pin(Pinned::Not);

                            db.insert(name.to_owned(), data).await;
                        }
                        Pinned::ByFilterAndUser => {
                            let mut data = data.to_owned();
                            data.set_pin(Pinned::ByUser);

                            db.insert(name.to_owned(), data).await;
                        }
                    }
                }
            }
        }
        Command::NameSearch(name) => {
            let re = Regex::new(&name).map_err(|_| CommandErr::RegexErr)?;

            let snapshot = db.snapshot().map_err(|_| CommandErr::DBCacheErr)?;

            for (name, data) in snapshot.iter() {
                let is_match = re.is_match(&data.get_stig().version);

                if is_match {
                    match data.get_pin() {
                        Pinned::Not => {
                            let mut data = data.to_owned();
                            data.set_pin(Pinned::ByFilter);

                            db.insert(name.to_owned(), data).await;
                        }
                        Pinned::ByUser => {
                            let mut data = data.to_owned();
                            data.set_pin(Pinned::ByFilterAndUser);

                            db.insert(name.to_owned(), data).await;
                        }
                        Pinned::ByFilter => (),
                        Pinned::ByFilterAndUser => (),
                    }

                    continue;
                } else {
                    match data.get_pin() {
                        Pinned::Not => (),
                        Pinned::ByUser => (),
                        Pinned::ByFilter => {
                            let mut data = data.to_owned();
                            data.set_pin(Pinned::Not);

                            db.insert(name.to_owned(), data).await;
                        }
                        Pinned::ByFilterAndUser => {
                            let mut data = data.to_owned();
                            data.set_pin(Pinned::ByUser);

                            db.insert(name.to_owned(), data).await;
                        }
                    }
                }
            }
        }
        Command::Reset => {
            let snapshot = db.snapshot().map_err(|_| CommandErr::DBCacheErr)?;

            for (name, data) in snapshot.iter() {
                match data.get_pin() {
                    Pinned::Not => (),
                    Pinned::ByUser => (),
                    Pinned::ByFilter => {
                        let mut data = data.to_owned();
                        data.set_pin(Pinned::Not);

                        db.insert(name.to_owned(), data).await;
                    }
                    Pinned::ByFilterAndUser => {
                        let mut data = data.to_owned();
                        data.set_pin(Pinned::ByUser);

                        db.insert(name.to_owned(), data).await;
                    }
                }
            }
        }
    }

    Ok(())
}
