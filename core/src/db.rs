use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock as AsyncRwLock;

use crate::{Benchmark, Rule};

/// A memory based data base for storing stigs.
#[derive(Debug, Clone)]
pub struct DB {
    benchmark: Arc<AsyncRwLock<Benchmark>>,
    pins: Arc<AsyncRwLock<BTreeMap<String, Pinned>>>,
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
    /// Converts a benchmark into a memory db.
    pub fn new(benchmark: Benchmark) -> Self {
        Self {
            benchmark: Arc::new(AsyncRwLock::new(benchmark)),
            pins: Arc::new(AsyncRwLock::new(BTreeMap::new())),
        }
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
    pub async fn snapshot(&self) -> Benchmark {
        self.benchmark.read().await.clone()
    }

    /// Set a pinned value.
    /// Trusts id isnt bullshit.
    /// If it is, nothing bad happens, just inserts random data into the BTreeMap.
    pub async fn set_pin(&mut self, id: String, pin: Pinned) {
        self.pins.write().await.insert(id, pin);
    }

    /// Get a pinned value.
    pub async fn get_pin(&self, id: &str) -> Pinned {
        if let Some(pin) = self.pins.read().await.get(id) {
            pin.to_owned()
        } else {
            // If the pinned status is not stored for a given id,
            // assume its not pinned, and return that.
            Pinned::Not
        }
    }

    pub async fn unpin_all(&mut self) {
        let mut pins = self.pins.write().await;

        pins.iter_mut()
            .for_each(|(_name, value)| *value = Pinned::Not);
    }
}
