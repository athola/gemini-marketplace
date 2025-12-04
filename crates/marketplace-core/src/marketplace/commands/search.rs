//! The search command handler, which leverages the catalog list execution.

use anyhow::Result;

use crate::marketplace::commands::list::{self, ListOptions};

/// Options for searching marketplace extensions.
#[derive(Debug, Default)]
pub struct SearchOptions {
    pub keyword: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    pub installed_only: bool,
    pub json: bool,
}

pub async fn execute(options: SearchOptions) -> Result<()> {
    list::execute(ListOptions {
        search: options.keyword,
        category: options.category,
        source: options.source,
        installed_only: options.installed_only,
        json: options.json,
    })
    .await
}
