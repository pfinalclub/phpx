use crate::error::{Error, Result};
use semver::{Version, VersionReq};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ToolIdentifier {
    pub name: String,
    pub version_constraint: Option<VersionReq>,
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub download_url: String,
    pub signature_url: Option<String>,
    pub hash: Option<String>,
}

// Packagist 相关类型
#[derive(Deserialize)]
struct PackagistVersionInfo {
    dist: PackagistDist,
}

#[derive(Deserialize)]
struct PackagistDist {
    url: String,
    #[serde(rename = "type")]
    dist_type: String,
}

// GitHub 相关类型
#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub struct ToolResolver;

impl Default for ToolResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_identifier(&self, identifier: &str) -> Result<ToolIdentifier> {
        let parts: Vec<&str> = identifier.split('@').collect();

        match parts.len() {
            1 => Ok(ToolIdentifier {
                name: parts[0].to_string(),
                version_constraint: None,
                version: None,
            }),
            2 => {
                let name = parts[0].to_string();
                let version_str = parts[1];

                if version_str == "latest" {
                    Ok(ToolIdentifier {
                        name,
                        version_constraint: None,
                        version: Some("latest".to_string()),
                    })
                } else {
                    match VersionReq::parse(version_str) {
                        Ok(constraint) => Ok(ToolIdentifier {
                            name,
                            version_constraint: Some(constraint),
                            version: None,
                        }),
                        Err(_) => Ok(ToolIdentifier {
                            name,
                            version_constraint: None,
                            version: Some(version_str.to_string()),
                        }),
                    }
                }
            }
            _ => Err(Error::InvalidToolIdentifier(
                "Invalid tool identifier format".to_string(),
            )),
        }
    }

    pub async fn resolve_tool(&self, identifier: &ToolIdentifier) -> Result<ToolInfo> {
        // 首先尝试从 Packagist 解析
        if let Ok(tool_info) = self.resolve_from_packagist(identifier).await {
            return Ok(tool_info);
        }

        // 然后尝试从 GitHub Releases 解析
        if let Ok(tool_info) = self.resolve_from_github(identifier).await {
            return Ok(tool_info);
        }

        // 最后尝试直接 URL 解析
        if let Ok(tool_info) = self.resolve_from_direct_url(identifier).await {
            return Ok(tool_info);
        }

        Err(Error::ToolNotFound(identifier.name.clone()))
    }

    async fn resolve_from_packagist(&self, identifier: &ToolIdentifier) -> Result<ToolInfo> {
        let url = format!("https://packagist.org/packages/{}.json", identifier.name);

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(Error::ToolNotFound(identifier.name.clone()));
        }

        #[derive(Deserialize)]
        struct PackagistResponse {
            package: Package,
        }

        #[derive(Deserialize)]
        struct Package {
            versions: HashMap<String, PackagistVersionInfo>,
        }

        let packagist_response: PackagistResponse = response.json().await?;

        // 找到合适的版本
        let version =
            self.find_matching_version(&packagist_response.package.versions, identifier)?;

        let version_info = &packagist_response.package.versions[&version];
        let dist = &version_info.dist;

        // 支持 path 类型（单文件，多为 .phar）；zip 为源码包，无法直接作为 phar 运行，回退到 GitHub 等源
        let download_url = match dist.dist_type.as_str() {
            "path" => dist.url.clone(),
            "zip" => {
                return Err(Error::ToolNotFound(format!(
                    "Packagist 仅提供 zip 源码包，无法直接运行；将尝试 GitHub 等源解析 {}",
                    identifier.name
                )));
            }
            other => {
                return Err(Error::ToolNotFound(format!(
                    "Packagist 分发类型 \"{}\" 暂不支持，将尝试 GitHub 等源",
                    other
                )));
            }
        };

        Ok(ToolInfo {
            name: identifier.name.clone(),
            version,
            download_url,
            signature_url: None,
            hash: None,
        })
    }

    /// 将工具名解析为 GitHub (owner, repo)。支持 vendor/package 如 laravel/pint -> (laravel, pint)
    fn github_owner_repo(name: &str) -> (String, String) {
        if let Some((owner, repo)) = name.split_once('/') {
            (owner.to_string(), repo.to_string())
        } else {
            (name.to_string(), name.to_string())
        }
    }

    async fn resolve_from_github(&self, identifier: &ToolIdentifier) -> Result<ToolInfo> {
        let (owner, repo) = Self::github_owner_repo(&identifier.name);
        // 尝试从 GitHub Releases 解析：owner/repo 格式
        let github_urls = vec![
            format!("https://api.github.com/repos/{}/{}/releases", owner, repo),
            format!(
                "https://api.github.com/repos/{}/php-{}/releases",
                owner, repo
            ),
            format!(
                "https://api.github.com/repos/php-{}/{}/releases",
                owner, repo
            ),
        ];

        for url in github_urls {
            let client = reqwest::Client::new();
            if let Ok(response) = client.get(&url).send().await {
                if response.status().is_success() {
                    let releases: Vec<GitHubRelease> = response.json().await?;

                    // 找到合适的版本
                    if let Some(release) = self.find_matching_github_release(&releases, identifier)
                    {
                        // 查找 .phar 文件
                        if let Some(asset) =
                            release.assets.iter().find(|a| a.name.ends_with(".phar"))
                        {
                            return Ok(ToolInfo {
                                name: identifier.name.clone(),
                                version: release.tag_name.trim_start_matches('v').to_string(),
                                download_url: asset.browser_download_url.clone(),
                                signature_url: self.find_signature_url(&release.assets),
                                hash: None,
                            });
                        }
                    }
                }
            }
        }

        Err(Error::ToolNotFound(identifier.name.clone()))
    }

    async fn resolve_from_direct_url(&self, identifier: &ToolIdentifier) -> Result<ToolInfo> {
        let (owner, repo) = Self::github_owner_repo(&identifier.name);
        // 尝试常见的直接下载 URL：owner/repo，下载文件名多为 repo.phar 或 vendor-repo.phar
        let direct_urls = vec![
            format!(
                "https://github.com/{}/{}/releases/latest/download/{}.phar",
                owner, repo, repo
            ),
            format!(
                "https://github.com/{}/{}/releases/latest/download/{}-{}.phar",
                owner, repo, owner, repo
            ),
            format!(
                "https://github.com/{}/{}/releases/latest/download/{}.phar",
                owner,
                repo,
                identifier.name.replace('/', "-")
            ),
        ];

        for url in direct_urls {
            let client = reqwest::Client::new();
            let response = client.head(&url).send().await?;

            if response.status().is_success() {
                return Ok(ToolInfo {
                    name: identifier.name.clone(),
                    version: "latest".to_string(),
                    download_url: url.clone(),
                    signature_url: Some(format!("{}.asc", url)),
                    hash: None,
                });
            }
        }

        Err(Error::ToolNotFound(identifier.name.clone()))
    }

    fn find_matching_version(
        &self,
        versions: &HashMap<String, PackagistVersionInfo>,
        identifier: &ToolIdentifier,
    ) -> Result<String> {
        let mut candidate_versions: Vec<Version> = versions
            .keys()
            .filter_map(|v| Version::parse(v).ok())
            .collect();

        candidate_versions.sort();
        candidate_versions.reverse();

        if let Some(constraint) = &identifier.version_constraint {
            for version in &candidate_versions {
                if constraint.matches(version) {
                    return Ok(version.to_string());
                }
            }
        } else if identifier.version.as_deref() == Some("latest") {
            if let Some(latest) = candidate_versions.first() {
                return Ok(latest.to_string());
            }
        } else if let Some(version_str) = &identifier.version {
            if let Ok(version) = Version::parse(version_str) {
                if candidate_versions.contains(&version) {
                    return Ok(version.to_string());
                }
            } else if versions.contains_key(version_str) {
                return Ok(version_str.clone());
            }
        } else {
            // 没有版本约束，使用最新版本
            if let Some(latest) = candidate_versions.first() {
                return Ok(latest.to_string());
            }
        }

        Err(Error::VersionConstraint(
            "No matching version found".to_string(),
        ))
    }

    fn find_matching_github_release<'a>(
        &self,
        releases: &'a [GitHubRelease],
        identifier: &ToolIdentifier,
    ) -> Option<&'a GitHubRelease> {
        for release in releases {
            let version_str = release.tag_name.trim_start_matches('v');

            if let Some(constraint) = &identifier.version_constraint {
                if let Ok(version) = Version::parse(version_str) {
                    if constraint.matches(&version) {
                        return Some(release);
                    }
                }
            } else if identifier.version.as_deref() == Some("latest") {
                return releases.first();
            } else if let Some(version_str) = &identifier.version {
                if release.tag_name == *version_str
                    || release.tag_name == format!("v{}", version_str)
                {
                    return Some(release);
                }
            } else {
                // 没有版本约束，使用最新版本
                return releases.first();
            }
        }

        None
    }

    fn find_signature_url(&self, assets: &[GitHubAsset]) -> Option<String> {
        assets
            .iter()
            .find(|a| a.name.ends_with(".asc") || a.name.ends_with(".sig"))
            .map(|a| a.browser_download_url.clone())
    }
}
