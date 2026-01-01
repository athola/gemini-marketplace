//! A shim that exposes the cache management handlers through `cli::cache`.

pub use crate::marketplace::commands::cache::{
    execute_refresh, execute_ttl_set, CacheRefreshOptions, CacheTtlSetOptions,
};
