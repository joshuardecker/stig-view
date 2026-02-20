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
    pub fn new() -> Self {
        unimplemented!()
    }

    pub async fn insert(&self, name: String, data: Data) {
        let mut btree = self.data.write().await;

        let _ = btree.insert(name, data);
    }

    pub async fn get(&self, name: &str) -> Option<Data> {
        let btree = self.data.read().await;

        let data = btree.get(name)?;
        Some(data.to_owned())
    }

    pub async fn snapshot(&self) -> BTreeMap<String, Data> {
        self.data.read().await.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    pub pinned: Pinned,

    stig: Arc<Stig>,
}

impl Data {
    pub fn new(stig: Arc<Stig>) -> Self {
        Self {
            pinned: Pinned::Not,
            stig,
        }
    }

    pub fn set_pin(&mut self, pin: Pinned) {
        self.pinned = pin;
    }

    pub fn get_pin(&self) -> Pinned {
        self.pinned.clone()
    }

    pub fn get_stig(&self) -> Arc<Stig> {
        self.stig.clone()
    }
}

#[derive(Debug, Clone)]
pub enum Pinned {
    Not,
    ByUser,
    ByFilter,
}
