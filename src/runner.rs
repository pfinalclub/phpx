use crate::cache::CacheManager;
use crate::composer;
use crate::config::Config;
use crate::download::Downloader;
use crate::error::{Error, Result};
use crate::executor::Executor;
use crate::resolver::{ResolvedTool, ToolIdentifier, ToolResolver};
use crate::security::SecurityManager;
use std::path::PathBuf;

pub struct Runner {
    config: Config,
    cache_manager: CacheManager,
    downloader: Downloader,
    resolver: ToolResolver,
    security_manager: SecurityManager,
    executor: Executor,
}

impl Runner {
    /// 使用可选配置文件路径创建 Runner；无则使用默认路径，加载失败则回退默认配置
    pub fn new(config_path: Option<PathBuf>) -> Result<Self> {
        let config =
            Config::load(config_path).map_err(|e| crate::error::Error::Config(e.to_string()))?;
        let skip_verify = config.skip_verify;
        let mut cache_manager = CacheManager::new(config.cache_dir.clone())?;
        // 按配置 TTL 清理过期缓存（每次创建 Runner 时执行一次）
        cache_manager.cleanup_old_entries(config.cache_ttl)?;

        Ok(Self {
            config,
            cache_manager,
            downloader: Downloader::new(),
            resolver: ToolResolver::new(),
            security_manager: SecurityManager::new(skip_verify),
            executor: Executor::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn run_tool(
        &mut self,
        tool_identifier: &str,
        args: &[String],
        clear_cache: bool,
        no_cache: bool,
        skip_verify: bool,
        php_path: Option<&PathBuf>,
        no_local: bool,
        no_interaction: bool,
    ) -> Result<()> {
        tracing::info!("Running tool: {}", tool_identifier);

        // 需要向子工具追加 --no-interaction 时，在参数末尾加上
        let effective_args: Vec<String> = if no_interaction {
            let mut a = args.to_vec();
            a.push("--no-interaction".to_string());
            a
        } else {
            args.to_vec()
        };
        let effective_args: &[String] = &effective_args;

        // 命令行 --php 优先，否则使用配置中的 default_php_path（克隆避免长期借用 self）
        let effective_php = php_path
            .cloned()
            .or_else(|| self.config.default_php_path.clone());

        // 解析工具标识符
        let identifier = self.resolver.parse_identifier(tool_identifier)?;

        // 检查本地项目是否有该工具
        if !no_local {
            if let Some(local_path) = self.find_local_tool(&identifier.name) {
                tracing::info!("Found local tool at: {:?}", local_path);
                return self
                    .executor
                    .execute_phar(&local_path, effective_args, effective_php.as_ref());
            }
        }

        // 清理缓存（如果需要）
        if clear_cache {
            self.cache_manager.remove_entry(&identifier.name, None)?;
        }

        // 查找缓存中的工具
        if !no_cache {
            if let Some(version) = self.get_tool_version(&identifier).await? {
                let entry_owned = self
                    .cache_manager
                    .get_entry(&identifier.name, &version)
                    .cloned();
                if let Some(cache_entry) = entry_owned {
                    // 用户指定了具体版本或约束时，不得使用 version 为 "latest" 的缓存，否则会跑错版本
                    let user_wants_specific_version = identifier.version_constraint.is_some()
                        || identifier
                            .version
                            .as_deref()
                            .map_or(false, |v| v != "latest");
                    if user_wants_specific_version && cache_entry.version == "latest" {
                        // 视为缓存未命中，继续走解析与下载
                    } else if self.verify_cached_tool(&cache_entry, skip_verify).is_ok() {
                        tracing::info!("Using cached tool: {}@{}", identifier.name, version);
                        if cache_entry.is_composer {
                            let bin_path = cache_entry
                                .file_path
                                .join("vendor")
                                .join("bin")
                                .join(cache_entry.bin_name.as_deref().unwrap_or("tool"));
                            return self.executor.execute_script(
                                &bin_path,
                                effective_args,
                                effective_php.as_ref(),
                            );
                        } else {
                            return self.executor.execute_phar(
                                &cache_entry.file_path,
                                effective_args,
                                effective_php.as_ref(),
                            );
                        }
                    }
                }
            }
        }

        // 解析并执行：Phar 下载后执行，Composer 在隔离目录安装后执行 vendor/bin
        let resolved = self.resolver.resolve_tool(&identifier).await?;
        match resolved {
            ResolvedTool::Phar(tool_info) => {
                let downloaded_path = self
                    .download_and_cache_tool(&tool_info, skip_verify)
                    .await?;
                self.executor
                    .execute_phar(&downloaded_path, effective_args, effective_php.as_ref())
            }
            ResolvedTool::Composer(composer_pkg) => {
                let (_dir, bin_path) = composer::ensure_composer_installed(
                    &composer_pkg,
                    &self.config.cache_dir,
                    &mut self.cache_manager,
                    &self.config,
                    effective_php.as_ref(),
                )?;
                self.executor
                    .execute_script(&bin_path, effective_args, effective_php.as_ref())
            }
        }
    }

    fn find_local_tool(&self, tool_name: &str) -> Option<PathBuf> {
        // 检查项目 vendor/bin 目录
        let vendor_path = PathBuf::from("vendor").join("bin").join(tool_name);
        if vendor_path.exists() {
            return Some(vendor_path);
        }

        // 检查全局 Composer 目录
        if let Some(home_dir) = dirs::home_dir() {
            let global_path = home_dir
                .join(".composer")
                .join("vendor")
                .join("bin")
                .join(tool_name);
            if global_path.exists() {
                return Some(global_path);
            }
        }

        None
    }

    async fn get_tool_version(&self, identifier: &ToolIdentifier) -> Result<Option<String>> {
        if let Some(version) = &identifier.version {
            return Ok(Some(version.clone()));
        }

        // 如果没有指定版本，尝试解析得到版本号（Phar 或 Composer 均可）
        let resolved = self.resolver.resolve_tool(identifier).await.ok();
        match resolved {
            Some(ResolvedTool::Phar(t)) => Ok(Some(t.version)),
            Some(ResolvedTool::Composer(c)) => Ok(Some(c.version)),
            None => Ok(None),
        }
    }

    fn verify_cached_tool(
        &self,
        cache_entry: &crate::cache::CacheEntry,
        skip_verify: bool,
    ) -> Result<()> {
        if skip_verify || self.security_manager.skip_verification() {
            return Ok(());
        }

        if !cache_entry.file_path.exists() {
            return Err(Error::Cache(
                "Cached file or directory not found".to_string(),
            ));
        }

        if cache_entry.is_composer {
            let bin_name = cache_entry.bin_name.as_deref().unwrap_or("tool");
            let vendor_bin = cache_entry
                .file_path
                .join("vendor")
                .join("bin")
                .join(bin_name);
            if !vendor_bin.exists() {
                return Err(Error::Cache(format!(
                    "Cached composer tool vendor/bin/{} not found",
                    bin_name
                )));
            }
            return Ok(());
        }

        let metadata = std::fs::metadata(&cache_entry.file_path)?;
        if metadata.len() != cache_entry.size {
            return Err(Error::Cache("Cached file size mismatch".to_string()));
        }

        if let Some(expected_hash) = &cache_entry.file_hash {
            if !expected_hash.is_empty() {
                self.security_manager
                    .verify_hash(&cache_entry.file_path, expected_hash)?;
            }
        }

        Ok(())
    }

    async fn download_and_cache_tool(
        &mut self,
        tool_info: &crate::resolver::ToolInfo,
        skip_verify: bool,
    ) -> Result<PathBuf> {
        let file_name = format!("{}-{}.phar", tool_info.name, tool_info.version);
        let cache_path = self.config.cache_dir.join(&file_name);

        // 下载文件
        self.downloader
            .download_file(&tool_info.download_url, &cache_path)
            .await?;

        // 安全验证
        if !skip_verify && !self.security_manager.skip_verification() {
            if let Some(signature_url) = &tool_info.signature_url {
                self.security_manager
                    .verify_signature(&cache_path, Some(signature_url))?;
            }

            if let Some(expected_hash) = &tool_info.hash {
                self.security_manager
                    .verify_hash(&cache_path, expected_hash)?;
            }
        } else {
            // 即使跳过验证，也要计算哈希值用于缓存记录
            let _hash = self.calculate_file_hash(&cache_path).ok();
        }

        // 添加到缓存
        let metadata = std::fs::metadata(&cache_path)?;
        let file_hash = if skip_verify {
            None
        } else {
            Some(self.calculate_file_hash(&cache_path)?)
        };

        self.cache_manager.add_entry(
            tool_info.name.clone(),
            tool_info.version.clone(),
            cache_path.clone(),
            tool_info.download_url.clone(),
            Some(file_hash.unwrap_or_default()),
            metadata.len(),
        )?;

        Ok(cache_path)
    }

    fn calculate_file_hash(&self, file_path: &PathBuf) -> Result<String> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        Ok(format!("{:x}", md5::compute(&buffer)))
    }

    pub fn clean_cache(&mut self, tool_name: Option<String>) -> Result<()> {
        match tool_name {
            Some(name) => self.cache_manager.remove_entry(&name, None),
            None => {
                // 清理所有缓存
                let entries: Vec<_> = self
                    .cache_manager
                    .list_entries()
                    .into_iter()
                    .map(|e| (e.tool_name.clone(), e.version.clone()))
                    .collect();

                for (tool_name, version) in entries {
                    self.cache_manager
                        .remove_entry(&tool_name, Some(&version))?;
                }
                Ok(())
            }
        }
    }

    pub fn list_cache(&self) -> Result<()> {
        let entries = self.cache_manager.list_entries();

        if entries.is_empty() {
            println!("No cached tools found.");
            return Ok(());
        }

        println!(
            "{:<20} {:<15} {:<10} {:<12}",
            "Tool", "Version", "Size", "Last Accessed"
        );
        println!("{:-<60}", "");

        for entry in entries {
            let size_mb = entry.size as f64 / 1024.0 / 1024.0;
            let last_accessed = chrono::DateTime::from_timestamp(entry.last_accessed as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            println!(
                "{:<20} {:<15} {:<8.1}MB {:<12}",
                entry.tool_name, entry.version, size_mb, last_accessed
            );
        }

        Ok(())
    }

    pub fn cache_info(&self, tool_name: &str) -> Result<()> {
        let entries = self.cache_manager.list_entries();
        let tool_entries: Vec<_> = entries
            .into_iter()
            .filter(|e| e.tool_name == tool_name)
            .collect();

        if tool_entries.is_empty() {
            println!("No cache entries found for tool: {}", tool_name);
            return Ok(());
        }

        println!("Cache information for tool: {}", tool_name);
        println!("{:-<60}", "");

        for entry in tool_entries {
            println!("Version: {}", entry.version);
            println!("File: {}", entry.file_path.display());
            println!("Size: {:.1}MB", entry.size as f64 / 1024.0 / 1024.0);
            println!("Download URL: {}", entry.download_url);
            println!(
                "Created: {}",
                chrono::DateTime::from_timestamp(entry.created_at as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            );
            println!(
                "Last Accessed: {}",
                chrono::DateTime::from_timestamp(entry.last_accessed as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            );
            println!();
        }

        Ok(())
    }

    pub async fn run_tool_with_options(
        &mut self,
        tool_identifier: &str,
        args: &[String],
        options: &crate::ToolOptions,
    ) -> Result<()> {
        self.run_tool(
            tool_identifier,
            args,
            options.clear_cache,
            options.no_cache,
            options.skip_verify,
            options.php.as_ref(),
            options.no_local,
            options.no_interaction,
        )
        .await
    }

    /// 为「无缝切版本」在 override 目录安装指定库包（仅 Packagist zip 包），返回安装目录。
    /// 若解析结果为 Phar 则返回错误，提示用 phpx &lt;tool&gt; 运行。
    pub async fn install_override_package(
        &mut self,
        package_spec: &str,
        php_path: Option<&PathBuf>,
    ) -> Result<PathBuf> {
        let identifier = self.resolver.parse_identifier(package_spec)?;
        let resolved = self.resolver.resolve_tool(&identifier).await?;
        match resolved {
            ResolvedTool::Composer(pkg) => composer::ensure_override_installed(
                &pkg.package,
                &pkg.version,
                &self.config.cache_dir,
                &mut self.cache_manager,
                &self.config,
                php_path,
            ),
            ResolvedTool::Phar(_) => Err(Error::Execution(
                "phpx add only supports library packages (Packagist zip). \
                 For phar-based tools use: phpx <tool>"
                    .to_string(),
            )),
        }
    }

    /// 列出 override 目录下已安装的库包，返回 (package, version, path)。
    pub fn list_override_packages(&self) -> Result<Vec<(String, String, PathBuf)>> {
        let override_dir = self.config.cache_dir.join("override");
        if !override_dir.exists() {
            return Ok(vec![]);
        }
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&override_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            // 目录名格式: vendor-package-version，如 guzzlehttp-guzzle-7.10.0
            let parts: Vec<&str> = name.split('-').collect();
            if parts.len() < 2 {
                out.push((name.clone(), String::new(), path));
                continue;
            }
            let (package, version) = if parts.last().map_or(false, |s| {
                s.chars().next().map_or(false, |c| c.is_ascii_digit())
            }) {
                let version = parts.last().unwrap().to_string();
                let slug = parts[..parts.len() - 1].join("-");
                let package = slug.replacen('-', "/", 1);
                (package, version)
            } else {
                (name.replacen('-', "/", 1), String::new())
            };
            out.push((package, version, path));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        Ok(out)
    }

    /// 删除 override 目录下已安装的库包。package 如 guzzlehttp/guzzle；version 可选，不指定则删除该包所有版本。
    pub fn remove_override_package(
        &self,
        package: &str,
        version: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        let slug = package.replace('/', "-");
        let override_dir = self.config.cache_dir.join("override");
        if !override_dir.exists() {
            return Ok(vec![]);
        }
        let mut removed = Vec::new();
        let entries = std::fs::read_dir(&override_dir)?;
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            let prefix = format!("{}-", slug);
            if !name_str.starts_with(&prefix) {
                continue;
            }
            let rest = name_str.strip_prefix(&prefix).unwrap_or("");
            if let Some(ver) = version {
                if rest != ver {
                    continue;
                }
            }
            let path = entry.path();
            if path.is_dir() {
                std::fs::remove_dir_all(&path)?;
                removed.push(path);
            }
        }
        Ok(removed)
    }

    /// 在指定路径生成 override_autoload.php：先加载 override 目录的 autoload，再加载项目 vendor。
    pub fn write_override_bootstrap(
        override_install_dir: &PathBuf,
        bootstrap_path: &PathBuf,
    ) -> Result<()> {
        let override_autoload = override_install_dir
            .canonicalize()
            .unwrap_or_else(|_| override_install_dir.clone())
            .join("vendor")
            .join("autoload.php");
        let path_str = override_autoload.display().to_string();
        let escaped = path_str.replace('\\', "\\\\").replace('\'', "\\'");
        let content = format!(
            r#"<?php
// Generated by phpx add --bootstrap. Load override vendor first, then project vendor.
require '{}';
require __DIR__ . '/vendor/autoload.php';
"#,
            escaped
        );
        if let Some(parent) = bootstrap_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(bootstrap_path, content)?;
        Ok(())
    }
}
