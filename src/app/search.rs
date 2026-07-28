use disa_stig::{Benchmark, RuleID};
use regex::Regex;
use std::collections::HashMap;

use crate::app::app::Pinned;

/// Look for the given keyword in every field in every rule in the provided benchmark.
/// Pins rules that have those keywords.
pub fn run_search_cmd(keyword: &str, benchmark: &Benchmark, pins: &mut HashMap<RuleID, Pinned>) {
    let Ok(re) = Regex::new(&format!("(?i){}", keyword)) else {
        return;
    };

    for (name, rule) in benchmark.rules.iter() {
        let is_match = re.is_match(&rule.group_id)
            || re.is_match(&rule.rule_id)
            || rule.stig_id.as_deref().is_some_and(|id| re.is_match(id))
            || re.is_match(&rule.title)
            || re.is_match(&rule.vuln_discussion)
            || re.is_match(&rule.check_text)
            || re.is_match(&rule.fix_text)
            || rule
                .cci_refs
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .any(|cci| re.is_match(cci))
            || rule
                .false_positives
                .as_deref()
                .is_some_and(|false_p| re.is_match(false_p))
            || rule
                .false_negatives
                .as_deref()
                .is_some_and(|false_n| re.is_match(false_n));

        if is_match {
            match pins.get(name).unwrap_or(&Pinned::Not) {
                Pinned::Not => {
                    let _ = pins.insert(name.to_owned(), Pinned::ByFilter);
                }
                Pinned::ByUser => {
                    let _ = pins.insert(name.to_owned(), Pinned::ByFilterAndUser);
                }
                // If already pinned, do nothing.
                _ => (),
            }

            continue;
        } else {
            match pins.get(name).unwrap_or(&Pinned::Not) {
                Pinned::ByFilter => {
                    let _ = pins.insert(name.to_owned(), Pinned::Not);
                }
                Pinned::ByFilterAndUser => {
                    let _ = pins.insert(name.to_owned(), Pinned::ByUser);
                }
                // If its not pinned and shouldnt be pinned, do nothing.
                _ => (),
            }
        }
    }
}

/// Reset the provided pins so that no rule is pinned for any reason.
pub fn reset_search_cmd(pins: &mut HashMap<RuleID, Pinned>) {
    pins.iter_mut()
        .for_each(|(_name, value)| *value = Pinned::Not);
}
