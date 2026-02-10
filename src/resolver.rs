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

/// 解析结果：要么是 phar（下载即跑），要么是 Composer 包（需在隔离目录安装后跑 vendor/bin）
#[derive(Debug, Clone)]
pub enum ResolvedTool {
    Phar(ToolInfo),
    Composer(ComposerPackage),
}

#[derive(Debug, Clone)]
pub struct ComposerPackage {
    pub package: String,
    pub version: String,
    pub bin_names: Vec<String>,
}

// Packagist 相关类型
#[derive(Deserialize)]
struct PackagistVersionInfo {
    dist: PackagistDist,
    #[serde(default)]
    bin: Option<Vec<String>>,
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

    pub async fn resolve_tool(&self, identifier: &ToolIdentifier) -> Result<ResolvedTool> {
        // 内置 composer：从 getcomposer.org 下载 composer.phar
        if identifier.name == "composer" {
            return Ok(ResolvedTool::Phar(
                self.resolve_builtin_composer(identifier),
            ));
        }

        // 首先尝试从 Packagist 解析（path → Phar，zip → Composer）
        if let Ok(resolved) = self.resolve_from_packagist(identifier).await {
            return Ok(resolved);
        }

        // 然后尝试从 GitHub Releases 解析
        if let Ok(tool_info) = self.resolve_from_github(identifier).await {
            return Ok(ResolvedTool::Phar(tool_info));
        }

        // 仅当用户未指定版本约束且未指定具体版本（或明确 @latest）时，才尝试直接 URL（latest）
        let use_direct_url = identifier.version_constraint.is_none()
            && identifier
                .version
                .as_deref()
                .map(|v| v == "latest")
                .unwrap_or(true);
        if use_direct_url {
            if let Ok(tool_info) = self.resolve_from_direct_url(identifier).await {
                return Ok(ResolvedTool::Phar(tool_info));
            }
        }

        Err(Error::ToolNotFound(identifier.name.clone()))
    }

    /// 内置 composer 工具：getcomposer.org 的 composer.phar
    fn resolve_builtin_composer(&self, identifier: &ToolIdentifier) -> ToolInfo {
        let version = identifier
            .version
            .as_deref()
            .filter(|v| *v != "latest")
            .unwrap_or("latest");
        let url = "https://getcomposer.org/download/latest-stable/composer.phar";
        ToolInfo {
            name: "composer".to_string(),
            version: version.to_string(),
            download_url: url.to_string(),
            signature_url: None,
            hash: None,
        }
    }

    async fn resolve_from_packagist(&self, identifier: &ToolIdentifier) -> Result<ResolvedTool> {
        #[derive(Deserialize)]
        struct PackagistResponse {
            package: Package,
        }

        #[derive(Deserialize)]
        struct Package {
            versions: HashMap<String, PackagistVersionInfo>,
        }

        // 单段名（如 rector）时先试 vendor/package（rector/rector），避免 /packages/rector.json 返回 HTML 重定向页
        let names_to_try: Vec<String> = if identifier.name.contains('/') {
            vec![identifier.name.clone()]
        } else {
            vec![
                format!("{}/{}", identifier.name, identifier.name),
                identifier.name.clone(),
            ]
        };

        let client = reqwest::Client::new();
        for packagist_name in names_to_try {
            let url = format!("https://packagist.org/packages/{}.json", packagist_name);
            let response = client.get(&url).send().await?;
            if !response.status().is_success() {
                continue;
            }

            // 响应可能为 HTML（如单段名重定向页），解析失败则尝试下一个包名
            let packagist_response: PackagistResponse = match response.json().await {
                Ok(p) => p,
                Err(_) => continue,
            };

            let version =
                match self.find_matching_version(&packagist_response.package.versions, identifier) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

            let version_info = &packagist_response.package.versions[&version];
            let dist = &version_info.dist;

            return match dist.dist_type.as_str() {
                "path" => Ok(ResolvedTool::Phar(ToolInfo {
                    name: identifier.name.clone(),
                    version: version.clone(),
                    download_url: dist.url.clone(),
                    signature_url: None,
                    hash: None,
                })),
                "zip" => {
                    let bin_names = version_info
                        .bin
                        .clone()
                        .filter(|b| !b.is_empty())
                        .unwrap_or_else(|| {
                            let default = packagist_name
                                .split('/')
                                .last()
                                .unwrap_or("tool")
                                .to_string();
                            vec![default]
                        });
                    // 标准化 bin：Packagist 可能为 "bin/rector"，取最后一段
                    let bin_names: Vec<String> = bin_names
                        .into_iter()
                        .map(|b| {
                            b.split('/')
                                .last()
                                .map(String::from)
                                .unwrap_or(b)
                        })
                        .collect();
                    Ok(ResolvedTool::Composer(ComposerPackage {
                        package: packagist_name,
                        version,
                        bin_names,
                    }))
                }
                _ => continue,
            };
        }

        Err(Error::ToolNotFound(identifier.name.clone()))
    }

    /// 将工具名解析为 GitHub (owner, repo)。支持 vendor/package 如 laravel/pint -> (laravel, pint)
    fn github_owner_repo(name: &str) -> (String, String) {
        if let Some((owner, repo)) = name.split_once('/') {
            (owner.to_string(), repo.to_string())
        } else {
            (name.to_string(), name.to_string())
        }
    }

    /// 生成 (owner, repo) 的多种写法，用于应对 GitHub 仓库名大小写（如 PHP-CS-Fixer）
    fn github_owner_repo_variants(name: &str) -> Vec<(String, String)> {
        let (owner, repo) = Self::github_owner_repo(name);
        let mut out = vec![(owner.clone(), repo.clone())];
        // 各段首字母大写，如 php-cs-fixer -> Php-Cs-Fixer
        let title: String = name
            .split('-')
            .map(|s| {
                let mut c = s.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().chain(c).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join("-");
        if title != name {
            let (to, tr) = if name.contains('/') {
                let (a, b) = name.split_once('/').unwrap();
                let at: String = a
                    .split('-')
                    .map(|s| {
                        let mut c = s.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().chain(c).collect(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("-");
                let bt: String = b
                    .split('-')
                    .map(|s| {
                        let mut c = s.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().chain(c).collect(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("-");
                (at, bt)
            } else {
                (title.clone(), title.clone())
            };
            out.push((to, tr));
        }
        // 短段（≤3 字符）全大写，如 php-cs-fixer -> PHP-CS-Fixer
        let acronym: String = name
            .split('-')
            .map(|s| {
                if s.len() <= 3 {
                    s.to_uppercase()
                } else {
                    let mut c = s.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().chain(c).collect(),
                    }
                }
            })
            .collect::<Vec<_>>()
            .join("-");
        if acronym != name && (acronym != title || title == name) {
            let (ao, ar) = if name.contains('/') {
                let (a, b) = name.split_once('/').unwrap();
                let aa: String = a
                    .split('-')
                    .map(|s| {
                        if s.len() <= 3 {
                            s.to_uppercase()
                        } else {
                            let mut c = s.chars();
                            match c.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().chain(c).collect(),
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("-");
                let ab: String = b
                    .split('-')
                    .map(|s| {
                        if s.len() <= 3 {
                            s.to_uppercase()
                        } else {
                            let mut c = s.chars();
                            match c.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().chain(c).collect(),
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("-");
                (aa, ab)
            } else {
                (acronym.clone(), acronym)
            };
            if !out.iter().any(|(o, r)| o == &ao && r == &ar) {
                out.push((ao, ar));
            }
        }
        out
    }

    async fn resolve_from_github(&self, identifier: &ToolIdentifier) -> Result<ToolInfo> {
        // GitHub API 要求带 User-Agent，且部分仓库使用大写（如 PHP-CS-Fixer）
        let client = reqwest::Client::builder()
            .user_agent("phpx/0.1")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let base_urls: Vec<String> = Self::github_owner_repo_variants(&identifier.name)
            .into_iter()
            .flat_map(|(owner, repo)| {
                vec![
                    format!("https://api.github.com/repos/{}/{}/releases", owner, repo),
                    format!(
                        "https://api.github.com/repos/{}/php-{}/releases",
                        owner, repo
                    ),
                    format!(
                        "https://api.github.com/repos/php-{}/{}/releases",
                        owner, repo
                    ),
                ]
            })
            .collect();

        for url in base_urls {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_caret_version_sets_constraint() {
        let resolver = ToolResolver::new();
        let id = resolver.parse_identifier("php-cs-fixer@^3.14").unwrap();
        assert!(
            id.version_constraint.is_some(),
            "^3.14 should be parsed as version_constraint, got version={:?}",
            id.version
        );
    }
}
