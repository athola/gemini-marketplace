//! Placeholder refresh scheduler service.

use anyhow::Result;

pub struct RefreshService;

impl RefreshService {
    pub fn new() -> Self {
        Self
    }

    pub fn queue_refresh(&self) -> Result<()> {
        anyhow::bail!("queue_refresh not yet implemented");
    }
}

impl Default for RefreshService {
    fn default() -> Self {
        Self::new()
    }
}
