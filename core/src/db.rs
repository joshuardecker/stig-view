use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as AsyncRwLock;

use crate::{Benchmark, Rule};

/// User can choose to display STIGs by their group id or Stig id.
/// Need to store both in the pins cache.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuleIdType {
    group_id: String,
    stig_id: Option<String>,
}

/// A memory based data base for storing stigs.
#[derive(Debug, Clone)]
pub struct DB {
    benchmark: Arc<AsyncRwLock<Benchmark>>,
    // A sync cache that can be accessed in the ui draw calls of the application.
    // Contains the RuleId displayed to the user, and that rules pinned status.
    pins: Arc<RwLock<BTreeMap<RuleIdType, Pinned>>>,
}

#[derive(Debug, Clone)]
pub enum DBErr {
    PinCache(&'static str),
}

/// Whether the stig has been pinned in the list for any reason.
#[derive(Debug, Clone)]
pub enum Pinned {
    Not,
    ByUser,
    ByFilter,
    ByFilterAndUser,
}

impl DB {
    /// Create a new memory database.
    pub fn new(benchmark: Benchmark) -> Self {
        Self {
            benchmark: Arc::new(AsyncRwLock::new(benchmark)),
            pins: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub async fn insert(&self, group_id: String, rule: Rule) {
        let mut benchmark = self.benchmark.write().await;
        benchmark.rules.insert(group_id, rule);
    }

    pub async fn get(&self, group_id: &str) -> Option<Rule> {
        let benchmark = self.benchmark.read().await;

        let rule = benchmark.rules.get(group_id);

        if let Some(rule) = rule {
            Some(rule.to_owned())
        } else {
            None
        }
    }

    /// Get a snapshot of the database.
    /// Performance of calling this isnt too bad, most of the data
    /// is stored in smart pointers, so most data is copied by copying
    /// pointers, not the data.
    pub async fn snapshot(&self) -> Benchmark {
        self.benchmark.read().await.clone()
    }

    /// Set a pinned value.
    pub fn set_pin(&mut self, id: RuleIdType, pin: Pinned) -> Result<(), DBErr> {
        self.pins
            .write()
            .map_err(|_| DBErr::PinCache("Error writing to the data base pin cache."))?
            .insert(id, pin);

        Ok(())
    }

    /// Get a pinned value.
    pub fn get_pin(&self, id: RuleIdType) -> Result<Option<Pinned>, DBErr> {
        if let Some(pin) = self
            .pins
            .read()
            .map_err(|_| DBErr::PinCache("Error reading from the data base pin cache."))?
            .get(&id)
        {
            Ok(Some(pin.to_owned()))
        } else {
            Ok(None)
        }
    }

    /// Get all of the pinned values.
    pub fn pin_cache_snapshot(&self) -> Result<BTreeMap<RuleIdType, Pinned>, DBErr> {
        Ok(self
            .pins
            .read()
            .map_err(|_| DBErr::PinCache("Error reading from the data base pin cache."))?
            .clone())
    }
}

impl RuleIdType {
    pub fn new(group_id: String, stig_id: Option<String>) -> Self {
        Self { group_id, stig_id }
    }
}
