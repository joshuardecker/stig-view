use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::stig::Stig;

/// A memory based data base for storing stigs.
#[derive(Debug, Clone)]
pub struct DB {
    data: Arc<RwLock<BTreeMap<String, Data>>>,
}

impl DB {
    /// Create a new memory database.
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Insert a stig into the database.
    /// Name is equivalent to stig version.
    pub async fn insert(&self, name: String, data: Data) {
        let mut btree = self.data.write().await;

        let _ = btree.insert(name, data);
    }

    /// Get an element from the database.
    pub async fn get(&self, name: &str) -> Option<Data> {
        let btree = self.data.read().await;

        let data = btree.get(name)?;
        Some(data.to_owned())
    }

    /// Take a snapshot of the database.
    /// Performance of calling this isnt too bad, most of the data
    /// is stored in read only smart pointers, so most data is copied by copying
    /// pointers, not the data.
    pub async fn snapshot(&self) -> BTreeMap<String, Data> {
        self.data.read().await.clone()
    }

    /// Completely clean out the database of all entries.
    pub async fn clean(&mut self) {
        let mut btree = self.data.write().await;

        *btree = BTreeMap::new();
    }
}

/// Data stored in the database is an enum that can change, and a read only pointer.
#[derive(Debug, Clone)]
pub struct Data {
    pub pinned: Pinned,

    stig: Arc<Stig>,
}

impl Data {
    /// Create data given a stig.
    pub fn new(stig: Arc<Stig>) -> Self {
        Self {
            pinned: Pinned::Not,
            stig,
        }
    }

    /// Set the pinned value.
    pub fn set_pin(&mut self, pin: Pinned) {
        self.pinned = pin;
    }

    /// Get the pinned value.
    pub fn get_pin(&self) -> Pinned {
        self.pinned.clone()
    }

    /// Get a pointer to the stig.
    pub fn get_stig(&self) -> Arc<Stig> {
        self.stig.clone()
    }
}

/// Whether the stig has been pinned in the list for any reason.
#[derive(Debug, Clone)]
pub enum Pinned {
    Not,
    ByUser,
    ByFilter,
}
