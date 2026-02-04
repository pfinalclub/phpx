use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub cache_dir: PathBuf,
    pub cache_ttl: u64,
    pub max_cache_size: u64,
    pub skip_verify: bool,
    pub default_php_path: Option<PathBuf>,
    pub download_mirrors: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(".cache"))
                .join("phpx"),
            cache_ttl: 7 * 24 * 60 * 60,        // 7 days
            max_cache_size: 1024 * 1024 * 1024, // 1GB
            skip_verify: false,
            default_php_path: None,
            download_mirrors: vec![
                "https://packagist.org".to_string(),
                "https://github.com".to_string(),
            ],
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: 实现配置加载逻辑
        Ok(Self::default())
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: 实现配置保存逻辑
        Ok(())
    }
}
