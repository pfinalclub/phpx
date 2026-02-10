//! 无 phar 时在隔离目录用 Composer 安装并返回 vendor/bin 路径。
//! 优先使用 phpx 缓存的 composer.phar，不污染本机 Composer。
//! 另支持「override」安装：仅安装库包（无 bin）到 override 目录，用于前置 autoload 切版本。

use crate::cache::CacheManager;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::resolver::ComposerPackage;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 在 cache_dir/override/<package-slug>-<version> 下安装指定版本库包（不要求 bin），
/// 返回安装目录路径。用于「无缝切版本」：项目通过前置该目录的 vendor/autoload.php 加载指定版本。
pub fn ensure_override_installed(
    package: &str,
    version: &str,
    cache_dir: &Path,
    cache_manager: &mut CacheManager,
    config: &Config,
    php_path: Option<&PathBuf>,
) -> Result<PathBuf> {
    let slug = package.replace('/', "-");
    let install_dir = cache_dir
        .join("override")
        .join(format!("{}-{}", slug, version));

    let autoload = install_dir.join("vendor").join("autoload.php");
    if install_dir.exists() && autoload.exists() {
        return Ok(install_dir);
    }

    let composer_binary = resolve_composer_binary(cache_manager, config)?;
    let php_binary = find_php_for_composer(php_path)?;

    std::fs::create_dir_all(&install_dir)?;

    let composer_json = format!(r#"{{"require":{{"{}":"{}"}}}}"#, package, version);
    std::fs::write(install_dir.join("composer.json"), &composer_json)?;

    let composer_home = cache_dir.join("composer_home");
    let composer_cache = cache_dir.join("composer_cache");
    std::fs::create_dir_all(&composer_home).ok();
    std::fs::create_dir_all(&composer_cache).ok();

    let mut cmd = if composer_binary.extension().map_or(false, |e| e == "phar") {
        let mut c = Command::new(&php_binary);
        c.arg(&composer_binary);
        c
    } else {
        Command::new(&composer_binary)
    };

    cmd.arg("install")
        .arg("--no-interaction")
        .arg("--no-dev")
        .current_dir(&install_dir)
        .env("COMPOSER_HOME", &composer_home)
        .env("COMPOSER_CACHE_DIR", &composer_cache)
        .env_remove("COMPOSER");

    let output = cmd
        .output()
        .map_err(|e| Error::ComposerInstallFailed(format!("Failed to run composer: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(Error::ComposerInstallFailed(format!(
            "composer install failed. stderr: {} stdout: {}",
            stderr, stdout
        )));
    }

    if !autoload.exists() {
        return Err(Error::ComposerInstallFailed(
            "vendor/autoload.php not found after install".to_string(),
        ));
    }

    Ok(install_dir)
}

/// 在缓存目录下为 Composer 包创建隔离项目、执行 composer install，返回安装目录和 vendor/bin 下的可执行路径。
pub fn ensure_composer_installed(
    pkg: &ComposerPackage,
    cache_dir: &Path,
    cache_manager: &mut CacheManager,
    config: &Config,
    php_path: Option<&PathBuf>,
) -> Result<(PathBuf, PathBuf)> {
    let slug = pkg.package.replace('/', "-");
    let install_dir = cache_dir
        .join("composer")
        .join(format!("{}-{}", slug, pkg.version));

    let bin_name = pkg
        .bin_names
        .first()
        .cloned()
        .unwrap_or_else(|| pkg.package.split('/').last().unwrap_or("tool").to_string());

    let vendor_bin = install_dir.join("vendor").join("bin").join(&bin_name);
    if install_dir.exists() && vendor_bin.exists() {
        if let Some(entry) = cache_manager.get_entry(&pkg.package, &pkg.version) {
            if entry.is_composer && entry.file_path == install_dir {
                return Ok((install_dir, vendor_bin));
            }
        }
    }

    // 需要安装
    let composer_binary = resolve_composer_binary(cache_manager, config)?;
    let php_binary = find_php_for_composer(php_path)?;

    std::fs::create_dir_all(&install_dir)?;

    let composer_json = format!(r#"{{"require":{{"{}":"{}"}}}}"#, pkg.package, pkg.version);
    std::fs::write(install_dir.join("composer.json"), &composer_json)?;

    let composer_home = cache_dir.join("composer_home");
    let composer_cache = cache_dir.join("composer_cache");
    std::fs::create_dir_all(&composer_home).ok();
    std::fs::create_dir_all(&composer_cache).ok();

    let mut cmd = if composer_binary.extension().map_or(false, |e| e == "phar") {
        let mut c = Command::new(&php_binary);
        c.arg(&composer_binary);
        c
    } else {
        Command::new(&composer_binary)
    };

    cmd.arg("install")
        .arg("--no-interaction")
        .arg("--no-dev")
        .current_dir(&install_dir)
        .env("COMPOSER_HOME", &composer_home)
        .env("COMPOSER_CACHE_DIR", &composer_cache)
        .env_remove("COMPOSER"); // 避免使用项目根目录的 composer.json

    let output = cmd
        .output()
        .map_err(|e| Error::ComposerInstallFailed(format!("Failed to run composer: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(Error::ComposerInstallFailed(format!(
            "composer install failed. stderr: {} stdout: {}",
            stderr, stdout
        )));
    }

    if !vendor_bin.exists() {
        return Err(Error::ComposerInstallFailed(format!(
            "vendor/bin/{} not found after install",
            bin_name
        )));
    }

    cache_manager.add_composer_entry(
        pkg.package.clone(),
        pkg.version.clone(),
        install_dir.clone(),
        bin_name,
    )?;

    Ok((install_dir, vendor_bin))
}

/// 解析 Composer 可执行路径：优先 phpx 缓存的 composer.phar，再 config.composer_path，再 PATH。
fn resolve_composer_binary(cache_manager: &mut CacheManager, config: &Config) -> Result<PathBuf> {
    if let Some(ref path) = config.composer_path {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    if let Some(entry) = cache_manager.get_entry("composer", "latest") {
        if entry.file_path.exists() && !entry.is_composer {
            return Ok(entry.file_path.clone());
        }
    }
    if let Some(entry) = cache_manager.get_entry("composer", "stable") {
        if entry.file_path.exists() && !entry.is_composer {
            return Ok(entry.file_path.clone());
        }
    }

    let which = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };
    if let Ok(out) = Command::new(which).arg("composer").output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            let first = s.lines().next().map(str::trim);
            if let Some(p) = first.filter(|p| !p.is_empty()) {
                return Ok(PathBuf::from(p));
            }
        }
    }

    if let Ok(out) = Command::new(which).arg("composer.phar").output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            let first = s.lines().next().map(str::trim);
            if let Some(p) = first.filter(|p| !p.is_empty()) {
                return Ok(PathBuf::from(p));
            }
        }
    }

    Err(Error::ComposerNotFound)
}

fn find_php_for_composer(php_path: Option<&PathBuf>) -> Result<PathBuf> {
    if let Some(p) = php_path {
        if p.exists() {
            return Ok(p.clone());
        }
        return Err(Error::Execution(format!(
            "PHP path does not exist: {}",
            p.display()
        )));
    }
    let possible = ["php", "/usr/bin/php", "/usr/local/bin/php"];
    for name in &possible {
        let path = PathBuf::from(name);
        if Command::new(&path).arg("--version").output().is_ok() {
            return Ok(path);
        }
    }
    Err(Error::Execution(
        "PHP not found. Install PHP or use --php".to_string(),
    ))
}
