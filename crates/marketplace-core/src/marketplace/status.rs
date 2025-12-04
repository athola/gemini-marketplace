//! Persisted status information shared across commands.

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::marketplace::config::Config;
use crate::marketplace::error::{MarketplaceError, Result};

const STATUS_FILE: &str = "status.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggedError {
    pub source_slug: String,
    pub message: String,
    #[serde(with = "humantime_serde")]
    pub occurred_at: SystemTime,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusSnapshot {
    pub last_errors: Vec<LoggedError>,
}

#[derive(Debug, Clone)]
pub struct StatusStore {
    path: PathBuf,
}

impl StatusStore {
    pub fn new(config: &Config) -> Self {
        let path = config.config_dir().join(STATUS_FILE);
        Self { path }
    }

    pub fn load(&self) -> Result<StatusSnapshot> {
        if !self.path.exists() {
            return Ok(StatusSnapshot::default());
        }
        let data = fs::read_to_string(&self.path)
            .map_err(|err| MarketplaceError::io(self.path.clone(), err))?;
        let snapshot = serde_json::from_str(&data).map_err(|err| {
            MarketplaceError::Configuration(format!("Invalid status snapshot: {err}"))
        })?;
        Ok(snapshot)
    }

    pub fn record_error(
        &self,
        source_slug: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<()> {
        let mut snapshot = self.load().unwrap_or_else(|_| StatusSnapshot::default());
        let source_slug = source_slug.into();
        snapshot
            .last_errors
            .retain(|entry| entry.source_slug != source_slug);
        snapshot.last_errors.push(LoggedError {
            source_slug,
            message: message.into(),
            occurred_at: SystemTime::now(),
        });
        self.write_snapshot(&snapshot)
    }

    pub fn clear_error(&self, source_slug: &str) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }
        let mut snapshot = self.load().unwrap_or_else(|_| StatusSnapshot::default());
        let before = snapshot.last_errors.len();
        snapshot
            .last_errors
            .retain(|entry| entry.source_slug != source_slug);
        if snapshot.last_errors.len() == before {
            return Ok(());
        }
        self.write_snapshot(&snapshot)
    }

    fn write_snapshot(&self, snapshot: &StatusSnapshot) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| MarketplaceError::io(parent.to_path_buf(), err))?;
        }
        let data = serde_json::to_string_pretty(snapshot).map_err(|err| {
            MarketplaceError::Configuration(format!("Unable to serialize status snapshot: {err}"))
        })?;
        fs::write(&self.path, data).map_err(|err| MarketplaceError::io(self.path.clone(), err))
    }
}
