use crate::error::{Error, Result};
use crate::runner::Runner;
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

    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub clear_cache: bool,

    #[arg(long)]
    pub no_cache: bool,

    #[arg(long)]
    pub skip_verify: bool,

    #[arg(long)]
    pub php: Option<PathBuf>,

    #[arg(long, short = 'n')]
    pub no_local: bool,
}

#[derive(Subcommand)]
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

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Clean cache for a specific tool or all tools
    Clean {
        tool: Option<String>,
    },

    /// List all cached tools
    List,

    /// Show cache information for a tool
    Info {
        tool: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Get a configuration value
    Get {
        key: String,
    },

    /// Set a configuration value
    Set {
        key: String,
        value: String,
    },
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
            ).await
        } else {
            // 显示帮助信息
            println!("No command specified. Use --help for usage information.");
            Ok(())
        }
    }

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
        tracing::info!(
            "Running tool: {} with options - clear_cache: {}, no_cache: {}, skip_verify: {}",
            tool,
            clear_cache,
            no_cache,
            skip_verify
        );

        // 创建并运行工具
        let mut runner = Runner::new()?;
        runner.run_tool(tool, args, clear_cache, no_cache, skip_verify, php, no_local).await
    }

    fn clean_cache(&self, tool: Option<String>) -> Result<()> {
        match tool {
            Some(tool_name) => {
                println!("Cleaning cache for tool: {}", tool_name);
            }
            None => {
                println!("Cleaning all cache");
            }
        }
        Ok(())
    }

    fn list_cache(&self) -> Result<()> {
        println!("Listing cached tools:");
        println!("(No cached tools found)");
        Ok(())
    }

    fn cache_info(&self, tool: &str) -> Result<()> {
        println!("Cache info for tool: {}", tool);
        println!("(No cache information available)");
        Ok(())
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