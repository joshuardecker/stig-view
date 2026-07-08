use std::{
    collections::HashMap,
    fs::{File, create_dir_all, read_to_string},
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

use disa_stig::RuleID;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A helper that remembers when a benchmark was last opened.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastOpened {
    benchmarks: HashMap<RuleID, u64>,
}

#[derive(Debug, Error)]
pub enum LastOpenedError {
    #[error("Could not access save directory")]
    DirError,
    #[error("Error accessing or creating the benchmark time file")]
    FileError(#[from] std::io::Error),
    #[error("Error serializing the benchmark time file")]
    SerializationError(#[from] toml::ser::Error),
    #[error("Error deserializing the benchmark time file")]
    DeserializationError(#[from] toml::de::Error),
}

impl LastOpened {
    pub fn new() -> Self {
        Self {
            benchmarks: HashMap::new(),
        }
    }

    pub fn load() -> Result<Self, LastOpenedError> {
        let mut save_dir = dirs::data_local_dir().ok_or(LastOpenedError::DirError)?;
        save_dir.push("xylok-view");
        save_dir.push("saved_when.toml");

        let saved_when_str = read_to_string(save_dir)?;

        let saved_when: LastOpened = toml::from_str(&saved_when_str)?;

        Ok(saved_when)
    }

    pub fn get_time_used(&self, benchmark_id: &str) -> u64 {
        match self.benchmarks.get(benchmark_id) {
            Some(time) => *time,
            None => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn insert(&mut self, benchmark_id: String) -> Result<(), LastOpenedError> {
        self.benchmarks.insert(
            benchmark_id,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );

        self.save()
    }

    fn save(&self) -> Result<(), LastOpenedError> {
        let Some(mut save_dir) = dirs::data_local_dir() else {
            return Err(LastOpenedError::DirError);
        };

        // Create the dir if it does not exist.
        save_dir.push("xylok-view");
        create_dir_all(&save_dir)?;

        save_dir.push("saved_when.toml");

        let string_to_save = toml::to_string(self)?;
        let mut file = File::create(save_dir)?;

        file.write_all(string_to_save.as_bytes())?;

        Ok(())
    }
}
