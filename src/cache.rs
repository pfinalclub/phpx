use crate::error::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// 将 PathBuf 序列化为字符串，确保 cache.json 可跨平台正确读写
mod path_serde {
    use super::*;

    pub fn serialize<S: Serializer>(path: &Path, s: S) -> std::result::Result<S::Ok, S::Error> {
        path.to_string_lossy().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<PathBuf, D::Error> {
        let s = String::deserialize(d)?;
        Ok(PathBuf::from(s))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub tool_name: String,
    pub version: String,
    #[serde(with = "path_serde")]
    pub file_path: PathBuf,
    pub download_url: String,
    pub file_hash: Option<String>,
    pub created_at: u64,
    pub last_accessed: u64,
    pub size: u64,
    /// Composer 安装目录时对应的 bin 名（如 rector）；phar 条目为 None
    #[serde(default)]
    pub bin_name: Option<String>,
    /// 是否为 Composer 安装目录（删除时需 remove_dir_all）
    #[serde(default)]
    pub is_composer: bool,
}

pub struct CacheManager {
    cache_dir: PathBuf,
    entries: HashMap<String, CacheEntry>,
}

impl CacheManager {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        let mut manager = Self {
            cache_dir,
            entries: HashMap::new(),
        };

        manager.load_cache()?;
        Ok(manager)
    }

    pub fn get_entry(&mut self, tool_name: &str, version: &str) -> Option<&CacheEntry> {
        let key = Self::build_key(tool_name, version);
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.last_accessed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Some(entry)
        } else {
            None
        }
    }

    pub fn add_entry(
        &mut self,
        tool_name: String,
        version: String,
        file_path: PathBuf,
        download_url: String,
        file_hash: Option<String>,
        size: u64,
    ) -> Result<()> {
        self.add_entry_inner(
            tool_name,
            version,
            file_path,
            download_url,
            file_hash,
            size,
            None,
            false,
        )
    }

    /// 添加 Composer 安装目录缓存条目
    pub fn add_composer_entry(
        &mut self,
        tool_name: String,
        version: String,
        dir_path: PathBuf,
        bin_name: String,
    ) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let entry = CacheEntry {
            tool_name,
            version,
            file_path: dir_path,
            download_url: String::new(),
            file_hash: None,
            created_at: now,
            last_accessed: now,
            size: 0,
            bin_name: Some(bin_name),
            is_composer: true,
        };
        let key = Self::build_key(&entry.tool_name, &entry.version);
        self.entries.insert(key, entry);
        self.save_cache()?;
        Ok(())
    }

    fn add_entry_inner(
        &mut self,
        tool_name: String,
        version: String,
        file_path: PathBuf,
        download_url: String,
        file_hash: Option<String>,
        size: u64,
        bin_name: Option<String>,
        is_composer: bool,
    ) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = CacheEntry {
            tool_name,
            version,
            file_path,
            download_url,
            file_hash,
            created_at: now,
            last_accessed: now,
            size,
            bin_name,
            is_composer,
        };

        let key = Self::build_key(&entry.tool_name, &entry.version);
        self.entries.insert(key, entry);
        self.save_cache()?;

        Ok(())
    }

    pub fn remove_entry(&mut self, tool_name: &str, version: Option<&str>) -> Result<()> {
        match version {
            Some(ver) => {
                let key = Self::build_key(tool_name, ver);
                if let Some(entry) = self.entries.remove(&key) {
                    if entry.file_path.exists() {
                        if entry.is_composer {
                            std::fs::remove_dir_all(&entry.file_path)?;
                        } else {
                            std::fs::remove_file(&entry.file_path)?;
                        }
                    }
                }
            }
            None => {
                let keys_to_remove: Vec<String> = self
                    .entries
                    .keys()
                    .filter(|k| k.starts_with(&format!("{}:", tool_name)))
                    .cloned()
                    .collect();

                for key in keys_to_remove {
                    if let Some(entry) = self.entries.remove(&key) {
                        if entry.file_path.exists() {
                            if entry.is_composer {
                                std::fs::remove_dir_all(&entry.file_path)?;
                            } else {
                                std::fs::remove_file(&entry.file_path)?;
                            }
                        }
                    }
                }
            }
        }

        self.save_cache()?;
        Ok(())
    }

    pub fn list_entries(&self) -> Vec<&CacheEntry> {
        self.entries.values().collect()
    }

    pub fn cleanup_old_entries(&mut self, ttl: u64) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let keys_to_remove: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| now - entry.last_accessed > ttl)
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_remove {
            if let Some(entry) = self.entries.remove(&key) {
                if entry.file_path.exists() {
                    if entry.is_composer {
                        let _ = std::fs::remove_dir_all(&entry.file_path);
                    } else {
                        let _ = std::fs::remove_file(&entry.file_path);
                    }
                }
            }
        }

        self.save_cache()?;
        Ok(())
    }

    fn build_key(tool_name: &str, version: &str) -> String {
        format!("{}:{}", tool_name, version)
    }

    fn load_cache(&mut self) -> Result<()> {
        let cache_file = self.cache_dir.join("cache.json");
        if cache_file.exists() {
            let content = std::fs::read_to_string(cache_file)?;
            self.entries = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    fn save_cache(&self) -> Result<()> {
        if !self.cache_dir.exists() {
            std::fs::create_dir_all(&self.cache_dir)?;
        }

        let cache_file = self.cache_dir.join("cache.json");
        let content = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(cache_file, content)?;

        Ok(())
    }
}
