use crate::error::{Error, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct Executor;

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

        tracing::info!(
            "Executing {} with PHP: {:?}",
            phar_path.display(),
            php_binary
        );

        let mut command = Command::new(php_binary);
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
            Err(Error::Execution(format!(
                "Tool execution failed with exit code: {}",
                status.code().unwrap_or(-1)
            )))
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

    pub fn detect_project_php_version(&self) -> Option<String> {
        // TODO: 从 composer.json 检测 PHP 版本约束
        None
    }
}
