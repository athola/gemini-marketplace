//! Placeholder catalog service.

use anyhow::Result;

pub struct CatalogService;

impl CatalogService {
    pub fn new() -> Self {
        Self
    }

    pub fn list_extensions(&self) -> Result<()> {
        anyhow::bail!("list_extensions not yet implemented");
    }
}
