use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock as AsyncRwLock;

use crate::stig::{Benchmark, Rule};

/// A memory based data base for storing stigs.
#[derive(Debug, Clone)]
pub struct DB {
    benchmark: Arc<AsyncRwLock<Benchmark>>,
    pins: Arc<AsyncRwLock<BTreeMap<String, Pinned>>>,
}

impl DB {
    /// Create a new memory database.
    pub fn new(benchmark: Benchmark) -> Self {
        Self {
            benchmark: Arc::new(AsyncRwLock::new(benchmark)),
            pins: Arc::new(AsyncRwLock::new(BTreeMap::new())),
        }
    }

    pub async fn insert(&self, group_id: String, rule: Arc<Rule>) {
        let mut benchmark = self.benchmark.write().await;
        benchmark.rules.insert(group_id, rule);
    }

    pub async fn get(&self, group_id: &str) -> Option<Arc<Rule>> {
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

    /// Set the pinned value.
    pub async fn set_pin(&mut self, group_id: &str, pin: Pinned) {
        self.pins.write().await.insert(group_id.to_owned(), pin);
    }

    /// Get the pinned value.
    pub async fn get_pin(&self, group_id: &str) -> Option<Pinned> {
        if let Some(pin) = self.pins.read().await.get(group_id) {
            Some(pin.to_owned())
        } else {
            None
        }
    }
}

/// Whether the stig has been pinned in the list for any reason.
#[derive(Debug, Clone)]
pub enum Pinned {
    Not,
    ByUser,
    ByFilter,
    ByFilterAndUser,
}
