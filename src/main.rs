use clap::Parser;
use phpx::cli::Cli;

fn main() -> phpx::Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 解析命令行参数
    let cli = Cli::parse();

    // 执行命令
    cli.execute()
}