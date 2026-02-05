use crate::error::{Error, Result};

pub struct SecurityManager {
    /// 是否默认跳过签名/哈希验证（来自配置）
    skip_verify: bool,
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new(false)
    }
}

impl SecurityManager {
    pub fn new(skip_verify: bool) -> Self {
        Self { skip_verify }
    }

    pub fn verify_signature(
        &self,
        _file_path: &std::path::Path,
        _signature_url: Option<&str>,
    ) -> Result<()> {
        // TODO: 实现 GPG 签名验证
        tracing::warn!("GPG signature verification not implemented yet");
        Ok(())
    }

    pub fn verify_hash(&self, file_path: &std::path::Path, expected_hash: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let actual_hash = format!("{:x}", md5::compute(&buffer));

        if actual_hash == expected_hash {
            tracing::info!("File hash verification successful");
            Ok(())
        } else {
            Err(Error::Security(format!(
                "Hash mismatch: expected {}, got {}",
                expected_hash, actual_hash
            )))
        }
    }

    pub fn skip_verification(&self) -> bool {
        self.skip_verify
    }
}
