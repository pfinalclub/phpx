pub mod cache;
pub mod cli;
pub mod config;
pub mod download;
pub mod error;
pub mod executor;
pub mod resolver;
pub mod runner;
pub mod security;

use std::path::PathBuf;

pub use error::{Error, Result};

#[derive(Debug, Clone)]
pub struct ToolOptions {
    pub clear_cache: bool,
    pub no_cache: bool,
    pub skip_verify: bool,
    pub php: Option<PathBuf>,
    pub no_local: bool,
}

impl Default for ToolOptions {
    fn default() -> Self {
        Self {
            clear_cache: false,
            no_cache: false,
            skip_verify: false,
            php: None,
            no_local: false,
        }
    }
}
