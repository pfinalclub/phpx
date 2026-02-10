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

    /// Pass --no-interaction to the tool (e.g. rector, composer) to avoid interactive prompts
    #[arg(long, global = true)]
    pub no_interaction: bool,
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

    /// Install a library package in override dir for "seamless version switch" (no bin required).
    /// Prints the install path; use it as vendor/autoload.php prefix or run with --bootstrap.
    Add {
        /// Package spec (e.g. guzzlehttp/guzzle@^7.8)
        package: String,

        /// Generate override_autoload.php in current dir and print run command
        #[arg(long)]
        bootstrap: bool,
    },

    /// Remove override install(s) for a package. Omit version to remove all versions.
    Remove {
        /// Package name (e.g. guzzlehttp/guzzle)
        package: String,

        /// Version to remove (e.g. 7.10.0); omit to remove all versions of the package
        version: Option<String>,
    },

    /// List override-installed packages (from phpx add).
    List,
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
                Commands::Add { package, bootstrap } => {
                    self.add_override_package(&package, *bootstrap).await
                }
                Commands::Remove { package, version } => {
                    self.remove_override_package(&package, version.as_deref())
                }
                Commands::List => self.list_override_packages(),
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
            no_interaction: self.no_interaction,
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

    async fn add_override_package(&self, package: &str, bootstrap: bool) -> Result<()> {
        let mut runner = Runner::new(self.config.clone())?;
        let install_dir = runner
            .install_override_package(package, self.php.as_ref())
            .await?;
        let autoload_path = install_dir.join("vendor").join("autoload.php");
        println!("{}", autoload_path.display());
        if bootstrap {
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let bootstrap_path = cwd.join("override_autoload.php");
            Runner::write_override_bootstrap(&install_dir, &bootstrap_path)?;
            println!(
                "Wrote {}. Run with: php -d auto_prepend_file=override_autoload.php your_script.php",
                bootstrap_path.display()
            );
        }
        Ok(())
    }

    fn remove_override_package(
        &self,
        package: &str,
        version: Option<&str>,
    ) -> Result<()> {
        let runner = Runner::new(self.config.clone())?;
        let removed = runner.remove_override_package(package, version)?;
        if removed.is_empty() {
            if let Some(v) = version {
                println!("No override found for {}@{}", package, v);
            } else {
                println!("No override found for {}", package);
            }
        } else {
            for path in &removed {
                println!("Removed {}", path.display());
            }
        }
        Ok(())
    }

    fn list_override_packages(&self) -> Result<()> {
        let runner = Runner::new(self.config.clone())?;
        let items = runner.list_override_packages()?;
        if items.is_empty() {
            println!("No override packages installed. Use 'phpx add <package>' to add one.");
            return Ok(());
        }
        for (package, version, path) in items {
            println!("{}@{}  {}", package, version, path.display());
        }
        Ok(())
    }
}
