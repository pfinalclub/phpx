use crate::error::Result;
use crate::runner::Runner;
use crate::ToolOptions;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "phpx")]
#[command(about = "A npx-like tool for PHP - run PHP tools without installation")]
#[command(version, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Tool identifier (e.g., phpstan, php-cs-fixer@^3.0)
    #[arg(required = false)]
    pub tool: Option<String>,

    /// Arguments to pass to the tool
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// Use the given config file instead of ~/.config/phpx/config.toml
    #[arg(long, short = 'c', global = true)]
    pub config: Option<PathBuf>,

    /// Clear this tool's cache (or all cache if no tool) before running
    #[arg(long, global = true)]
    pub clear_cache: bool,

    /// Do not use cache for this run (still caches after download)
    #[arg(long, global = true)]
    pub no_cache: bool,

    /// Skip signature/hash verification for this run
    #[arg(long, global = true)]
    pub skip_verify: bool,

    /// PHP binary path to run the .phar (overrides config default_php_path)
    #[arg(long, global = true)]
    pub php: Option<PathBuf>,

    /// Ignore local vendor/bin and composer global, use cache or remote only
    #[arg(long, short = 'n', global = true)]
    pub no_local: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage cache
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Update phpx to the latest version
    SelfUpdate,
}

#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// Clean cache for a specific tool or all tools
    Clean { tool: Option<String> },

    /// List all cached tools
    List,

    /// Show cache information for a tool
    Info { tool: String },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Get a configuration value
    Get { key: String },

    /// Set a configuration value
    Set { key: String, value: String },
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        if let Some(ref command) = self.command {
            match command {
                Commands::Cache { command } => match command {
                    CacheCommands::Clean { tool } => {
                        tracing::info!("Cleaning cache for tool: {:?}", tool);
                        self.clean_cache(tool.clone())
                    }
                    CacheCommands::List => {
                        tracing::info!("Listing cached tools");
                        self.list_cache()
                    }
                    CacheCommands::Info { tool } => {
                        tracing::info!("Getting cache info for tool: {}", tool);
                        self.cache_info(tool)
                    }
                },
                Commands::Config { command } => match command {
                    ConfigCommands::Get { key } => {
                        tracing::info!("Getting config: {}", key);
                        self.get_config(key)
                    }
                    ConfigCommands::Set { key, value } => {
                        tracing::info!("Setting config: {} = {}", key, value);
                        self.set_config(key, value)
                    }
                },
                Commands::SelfUpdate => {
                    tracing::info!("Updating phpx");
                    self.self_update()
                }
            }
        } else if self.clear_cache && self.tool.is_none() {
            // 仅传入 --clear-cache 时，清理全部缓存（等同 phpx cache clean）
            tracing::info!("Clearing all cache (--clear-cache without tool)");
            self.clean_cache(None)?;
            println!("Cache cleared.");
            Ok(())
        } else if let Some(ref tool) = self.tool {
            tracing::info!("Running tool: {} with args: {:?}", tool, self.args);
            self.run_tool(
                tool,
                &self.args,
                self.clear_cache,
                self.no_cache,
                self.skip_verify,
                self.php.as_ref(),
                self.no_local,
            )
            .await
        } else {
            println!("No command specified. Use --help for usage information.");
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_tool(
        &self,
        tool: &str,
        args: &[String],
        clear_cache: bool,
        no_cache: bool,
        skip_verify: bool,
        php: Option<&PathBuf>,
        no_local: bool,
    ) -> Result<()> {
        let options = ToolOptions {
            clear_cache,
            no_cache,
            skip_verify,
            php: php.cloned(),
            no_local,
        };

        tracing::info!(
            "Running tool: {} with options - clear_cache: {}, no_cache: {}, skip_verify: {}",
            tool,
            options.clear_cache,
            options.no_cache,
            options.skip_verify
        );

        // 创建并运行工具（传入可选配置文件路径以覆盖默认 ~/.config/phpx/config.toml）
        let mut runner = Runner::new(self.config.clone())?;
        runner.run_tool_with_options(tool, args, &options).await
    }

    fn clean_cache(&self, tool: Option<String>) -> Result<()> {
        let mut runner = Runner::new(self.config.clone())?;
        runner.clean_cache(tool)
    }

    fn list_cache(&self) -> Result<()> {
        let runner = Runner::new(self.config.clone())?;
        runner.list_cache()
    }

    fn cache_info(&self, tool: &str) -> Result<()> {
        let runner = Runner::new(self.config.clone())?;
        runner.cache_info(tool)
    }

    fn get_config(&self, key: &str) -> Result<()> {
        println!("Getting config: {}", key);
        println!("(Configuration system not implemented yet)");
        Ok(())
    }

    fn set_config(&self, key: &str, value: &str) -> Result<()> {
        println!("Setting config: {} = {}", key, value);
        println!("(Configuration system not implemented yet)");
        Ok(())
    }

    fn self_update(&self) -> Result<()> {
        println!("Updating phpx to latest version");
        println!("(Self-update functionality not implemented yet)");
        Ok(())
    }
}
