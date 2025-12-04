//! A shim that exposes the `sources` subcommands so that the binary can stay lightweight.

pub use crate::marketplace::commands::sources::{add, list, remove};
