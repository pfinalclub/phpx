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

pub struct ToolResolver;

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
        let version = self.find_matching_version(
            &packagist_response.package.versions,
            identifier,
        )?;

        let version_info = &packagist_response.package.versions[&version];
        
        if version_info.dist.dist_type != "zip" {
            return Err(Error::ToolNotFound(
                "Only zip distributions are supported".to_string(),
            ));
        }

        Ok(ToolInfo {
            name: identifier.name.clone(),
            version,
            download_url: version_info.dist.url.clone(),
            signature_url: None,
            hash: None,
        })
    }

    async fn resolve_from_github(&self, identifier: &ToolIdentifier) -> Result<ToolInfo> {
        // TODO: 实现 GitHub Releases 解析
        Err(Error::ToolNotFound(
            "GitHub resolution not implemented yet".to_string(),
        ))
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
}

// 为 Packagist 响应定义的类型
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