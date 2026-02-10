pub mod cache;
pub mod cli;
pub mod composer;
pub mod config;
pub mod download;
pub mod error;
pub mod executor;
pub mod resolver;
pub mod runner;
pub mod security;

use std::path::PathBuf;

pub use error::{Error, Result};

#[derive(Debug, Clone, Default)]
pub struct ToolOptions {
    pub clear_cache: bool,
    pub no_cache: bool,
    pub skip_verify: bool,
    pub php: Option<PathBuf>,
    pub no_local: bool,
    /// 向子工具追加 --no-interaction，避免交互式提示（如 rector 询问是否生成配置）
    pub no_interaction: bool,
}
