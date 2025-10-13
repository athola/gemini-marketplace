//! Search command handler leveraging the catalog list execution.

use anyhow::Result;

use crate::marketplace::commands::list::{self, ListOptions};

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
