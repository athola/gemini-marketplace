//! Placeholder cache command handlers.

use anyhow::Result;

#[derive(Debug, Default)]
pub struct CacheRefreshOptions {
    pub force: bool,
    pub source: Option<String>,
    pub json: bool,
}

#[derive(Debug)]
pub struct CacheTtlSetOptions {
    pub hours: u16,
}

pub async fn execute_refresh(_options: CacheRefreshOptions) -> Result<()> {
    // TODO: invoke cache refresh service (later story).
    Ok(())
}

pub async fn execute_ttl_set(_options: CacheTtlSetOptions) -> Result<()> {
    // TODO: persist TTL value (later story).
    Ok(())
}
