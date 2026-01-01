//! A public CLI facade that exposes thin wrappers around the marketplace command implementations.
//!
//! This module keeps the binary lean while providing a stable surface (`gemini_marketplace::cli::*`) for other crates or tests.

pub mod cache;
pub mod list;
pub mod search;
pub mod show;
pub mod sources;
pub mod status;
