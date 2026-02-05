use clap::Parser;
use phpx::cli::Cli;
use phpx::Error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();

    if let Err(e) = cli.execute().await {
        // 工具因自身逻辑退出（如 lint 报错）时只传播退出码，不再打印冗余错误
        if let Error::ExecutionFailed(code) = e {
            std::process::exit(code);
        }
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
