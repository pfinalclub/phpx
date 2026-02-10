use crate::error::{Error, Result};
use semver::VersionReq;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// composer.json 中与 PHP 版本相关的字段（仅解析所需部分）
#[derive(Deserialize)]
struct ComposerJson {
    #[serde(default)]
    require: ComposerRequire,
    #[serde(default)]
    config: ComposerConfig,
}

#[derive(Deserialize, Default)]
struct ComposerRequire {
    #[serde(rename = "php")]
    php_constraint: Option<String>,
}

#[derive(Deserialize, Default)]
struct ComposerConfig {
    #[serde(default)]
    platform: ComposerPlatform,
}

#[derive(Deserialize, Default)]
struct ComposerPlatform {
    #[serde(rename = "php")]
    php_version: Option<String>,
}

pub struct Executor;

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute_phar(
        &self,
        phar_path: &PathBuf,
        args: &[String],
        php_path: Option<&PathBuf>,
    ) -> Result<()> {
        let php_binary = self.find_php_binary(php_path)?;

        // 若项目有 composer.json 的 PHP 约束且未指定 --php，校验当前 PHP 是否满足并打日志
        if php_path.is_none() {
            if let Some(constraint) = self.detect_project_php_version() {
                if let Some(actual) = Self::get_php_version(&php_binary) {
                    if !Self::php_version_matches_constraint(&actual, &constraint) {
                        tracing::warn!(
                            "Project composer.json requires PHP {}, but current PHP is {}",
                            constraint,
                            actual
                        );
                    }
                }
            }
        }

        tracing::info!(
            "Executing {} with PHP: {:?}",
            phar_path.display(),
            php_binary
        );

        let mut command = Command::new(&php_binary);
        command.arg(phar_path);
        command.args(args);

        // 继承当前环境变量
        command.envs(std::env::vars());

        // 设置标准输入/输出
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());

        let status = command.status()?;

        if status.success() {
            Ok(())
        } else {
            let code = status.code().unwrap_or(1);
            Err(Error::ExecutionFailed(code))
        }
    }

    /// 执行 PHP 脚本（如 vendor/bin/rector），与 execute_phar 共用 PHP 选择与环境
    pub fn execute_script(
        &self,
        script_path: &Path,
        args: &[String],
        php_path: Option<&PathBuf>,
    ) -> Result<()> {
        let php_binary = self.find_php_binary(php_path)?;

        if php_path.is_none() {
            if let Some(constraint) = self.detect_project_php_version() {
                if let Some(actual) = Self::get_php_version(&php_binary) {
                    if !Self::php_version_matches_constraint(&actual, &constraint) {
                        tracing::warn!(
                            "Project composer.json requires PHP {}, but current PHP is {}",
                            constraint,
                            actual
                        );
                    }
                }
            }
        }

        tracing::info!(
            "Executing {} with PHP: {:?}",
            script_path.display(),
            php_binary
        );

        let mut command = Command::new(&php_binary);
        command.arg(script_path);
        command.args(args);

        command.envs(std::env::vars());
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());

        let status = command.status()?;

        if status.success() {
            Ok(())
        } else {
            let code = status.code().unwrap_or(1);
            Err(Error::ExecutionFailed(code))
        }
    }

    fn find_php_binary(&self, custom_path: Option<&PathBuf>) -> Result<PathBuf> {
        if let Some(path) = custom_path {
            if path.exists() {
                return Ok(path.clone());
            } else {
                return Err(Error::Execution(format!(
                    "Custom PHP path does not exist: {}",
                    path.display()
                )));
            }
        }

        // 查找系统 PHP
        let possible_paths = [
            PathBuf::from("php"),
            PathBuf::from("/usr/bin/php"),
            PathBuf::from("/usr/local/bin/php"),
        ];

        for path in possible_paths {
            if Command::new(&path).arg("--version").output().is_ok() {
                return Ok(path);
            }
        }

        Err(Error::Execution(
            "PHP executable not found. Please install PHP or specify path with --php".to_string(),
        ))
    }

    /// 从当前目录向上查找 composer.json，解析 require.php 或 config.platform.php，返回 PHP 版本约束字符串
    pub fn detect_project_php_version(&self) -> Option<String> {
        let composer_path = Self::find_composer_json()?;
        let content = std::fs::read_to_string(&composer_path).ok()?;
        let composer: ComposerJson = serde_json::from_str(&content).ok()?;
        composer
            .require
            .php_constraint
            .filter(|s| !s.is_empty())
            .or(composer.config.platform.php_version)
            .filter(|s| !s.is_empty())
    }

    /// 获取指定 PHP 可执行文件的版本号（如 "8.2.1"）；若有后缀如 -ubuntu 则只取主版本段
    pub fn get_php_version(php_binary: &Path) -> Option<String> {
        let out = Command::new(php_binary)
            .arg("-r")
            .arg("echo PHP_VERSION;")
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let v = String::from_utf8_lossy(&out.stdout);
        let v = v.trim();
        if v.is_empty() {
            return None;
        }
        // 只保留主版本号段（如 8.2.1），去掉 -ubuntu、-dev 等后缀以便 semver 解析
        let core: String = v
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if core.is_empty() {
            return None;
        }
        Some(core)
    }

    /// 检查 PHP 版本是否满足 composer 约束（require.php 或 config.platform.php）
    pub fn php_version_matches_constraint(version: &str, constraint: &str) -> bool {
        let constraint = constraint.trim();
        if constraint.is_empty() {
            return true;
        }
        // 尝试解析为版本约束（^8.2.0, >=7.4 等）
        if let Ok(req) = VersionReq::parse(constraint) {
            if let Ok(v) = semver::Version::parse(version) {
                return req.matches(&v);
            }
        }
        // 可能是纯版本号如 8.2.0，当作最低版本
        if let Ok(min_ver) = semver::Version::parse(constraint) {
            if let Ok(actual) = semver::Version::parse(version) {
                return actual >= min_ver;
            }
        }
        false
    }

    /// 从当前目录向上查找直到找到 composer.json 或到达根目录
    fn find_composer_json() -> Option<PathBuf> {
        let mut dir = std::env::current_dir().ok()?;
        loop {
            let candidate = dir.join("composer.json");
            if candidate.exists() {
                return Some(candidate);
            }
            dir = dir.parent()?.to_path_buf();
        }
    }
}
