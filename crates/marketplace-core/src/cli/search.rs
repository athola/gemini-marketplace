//! A shim that exposes the search command via the public `cli` module.

pub use crate::marketplace::commands::search::{execute, SearchOptions};
