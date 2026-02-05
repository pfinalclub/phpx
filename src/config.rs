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

/// 配置文件磁盘格式：路径为字符串，便于 TOML 中使用 ~
#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    pub cache_dir: Option<String>,
    pub cache_ttl: Option<u64>,
    pub max_cache_size: Option<u64>,
    pub skip_verify: Option<bool>,
    pub default_php_path: Option<String>,
    pub download_mirrors: Option<Vec<String>>,
}

/// 将 "~" 或 "~/path" 展开为家目录路径
fn expand_tilde(path: &str) -> PathBuf {
    let path = path.trim();
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    if let Some(rest) = path.strip_prefix("~\\") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

impl Default for Config {
    fn default() -> Self {
        // 默认缓存目录 ~/.cache/phpx（与需求一致）
        let cache_dir = dirs::home_dir()
            .map(|h| h.join(".cache").join("phpx"))
            .unwrap_or_else(|| PathBuf::from(".cache").join("phpx"));

        Self {
            cache_dir,
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
    /// 默认配置文件路径：~/.config/phpx/config.toml（与 README 约定一致）
    pub fn default_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".config").join("phpx").join("config.toml"))
    }

    /// 从指定路径或默认路径加载配置；文件不存在时返回默认配置
    pub fn load(override_path: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = override_path.or_else(Self::default_config_path);
        let path = match path {
            Some(p) if p.exists() => p,
            _ => return Ok(Self::default()),
        };

        let content = std::fs::read_to_string(&path)?;
        let file: ConfigFile = toml::from_str(&content)?;

        let default = Self::default();
        let cache_dir = file
            .cache_dir
            .as_deref()
            .map(expand_tilde)
            .unwrap_or(default.cache_dir);
        let cache_ttl = file.cache_ttl.unwrap_or(default.cache_ttl);
        let max_cache_size = file.max_cache_size.unwrap_or(default.max_cache_size);
        let skip_verify = file.skip_verify.unwrap_or(default.skip_verify);
        let default_php_path = file
            .default_php_path
            .as_deref()
            .map(expand_tilde)
            .or(default.default_php_path);
        let download_mirrors = file
            .download_mirrors
            .unwrap_or(default.download_mirrors);

        Ok(Self {
            cache_dir,
            cache_ttl,
            max_cache_size,
            skip_verify,
            default_php_path,
            download_mirrors,
        })
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 保存到默认路径；路径字段序列化为字符串
        let path = Self::default_config_path()
            .ok_or("Cannot determine config directory")?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let cache_dir_str = self.cache_dir.to_string_lossy();
        let default_php_str = self
            .default_php_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        let file = ConfigFile {
            cache_dir: Some(cache_dir_str.to_string()),
            cache_ttl: Some(self.cache_ttl),
            max_cache_size: Some(self.max_cache_size),
            skip_verify: Some(self.skip_verify),
            default_php_path: default_php_str,
            download_mirrors: Some(self.download_mirrors.clone()),
        };
        let content = toml::to_string_pretty(&file)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
