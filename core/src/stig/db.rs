use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as AsyncRwLock;

use crate::stig::{Benchmark, Rule};

/// A memory based data base for storing stigs.
#[derive(Debug, Clone)]
pub struct DB {
    benchmark: Arc<AsyncRwLock<Option<Benchmark>>>,

    data: Arc<AsyncRwLock<BTreeMap<String, Data>>>,
    cache: Arc<RwLock<BTreeMap<String, Data>>>,
}

#[derive(Debug, Clone)]
pub enum DBErr {
    CacheErr(&'static str),
    NoFirstEntry(&'static str),
}

impl DB {
    /// Create a new memory database.
    pub fn new() -> Self {
        Self {
            benchmark: Arc::new(AsyncRwLock::new(None)),

            data: Arc::new(AsyncRwLock::new(BTreeMap::new())),
            cache: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Insert a stig into the database.
    /// Name is equivalent to stig version.
    pub async fn insert(&self, group_id: String, data: Data) -> Result<(), DBErr> {
        let mut btree = self.data.write().await;
        btree.insert(group_id, data);
        *self.cache.write().map_err(|_| {
            DBErr::CacheErr("DB cache error. If this error persists, restart the application.")
        })? = btree.clone();

        Ok(())
    }

    /// Get an element from the database.
    pub async fn get(&self, group_id: &str) -> Option<Data> {
        let btree = self.data.read().await;

        let data = btree.get(group_id)?;
        Some(data.to_owned())
    }

    /// Get a snapshot of the database.
    /// Performance of calling this isnt too bad, most of the data
    /// is stored in read only smart pointers, so most data is copied by copying
    /// pointers, not the data.
    pub fn snapshot(&self) -> Result<BTreeMap<String, Data>, DBErr> {
        Ok(self
            .cache
            .read()
            .map_err(|_| {
                DBErr::CacheErr("DB cache error. If this error persists, restart the application.")
            })?
            .clone())
    }

    /// Get the first item from the data base cache.
    /// This will result in either the first filtered STIG,
    /// or just the first STIG sorted by version.
    pub fn first_snapshot(&self) -> Result<Data, DBErr> {
        if let Some((_key, value)) = self
            .cache
            .read()
            .map_err(|_| {
                DBErr::CacheErr("DB cache error. If this error persists, restart the application.")
            })?
            .first_key_value()
        {
            Ok(value.to_owned())
        } else {
            Err(DBErr::NoFirstEntry("No first entry found in the cache."))
        }
    }

    /// Completely clean out the database of all entries.
    pub async fn clean(&self) -> Result<(), DBErr> {
        let mut btree = self.data.write().await;
        *btree = BTreeMap::new();
        *self.cache.write().map_err(|_| {
            DBErr::CacheErr("DB cache error. If this error persists, restart the application.")
        })? = BTreeMap::new();

        Ok(())
    }
}

/// Data stored in the database is an enum that can change, and a read only pointer.
#[derive(Debug, Clone)]
pub struct Data {
    pub pinned: Pinned,

    rule: Arc<Rule>,
}

impl Data {
    /// Create data given a stig.
    pub fn new(rule: Arc<Rule>) -> Self {
        Self {
            pinned: Pinned::Not,
            rule,
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
    pub fn get_stig(&self) -> Arc<Rule> {
        self.rule.clone()
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
