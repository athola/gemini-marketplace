//! Placeholder source registry service.

use anyhow::Result;

pub struct SourcesService;

impl SourcesService {
    pub fn new() -> Self {
        Self
    }

    pub fn add_source(&self) -> Result<()> {
        anyhow::bail!("add_source not yet implemented");
    }
}

impl Default for SourcesService {
    fn default() -> Self {
        Self::new()
    }
}
