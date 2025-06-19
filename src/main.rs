mod cli;
mod config;
mod error;
mod manager;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use error::Result;
use manager::Manager;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let manager = Manager::new()?;
    match cli.command {
        Commands::LsRemote { version } => {
            let versions = manager.list_remote(version.as_deref()).await?;
            println!("\n可用的 Node.js 版本:");
            for version in versions {
                println!("{}", version);
            }
            if version.is_none() {
                println!("\n提示: 使用 `rnm-rs ls-remote <version>` 查看特定版本");
                println!("例如: rnm-rs ls-remote 18 或 rnm-rs ls-remote 18.15\n");
            }
        }
        Commands::Ls => {
            let versions = manager.list_local().await?;
            println!("Installed Node.js versions:");
            if versions.is_empty() {
                println!("No versions installed");
                return Ok(());
            }
            
            // 直接显示版本列表
            for version in versions {
                println!("  {}", version);
            }
        }
        Commands::Install { version } => {
            manager.install(&version).await?;
        }
        Commands::Uninstall { version } => {
            manager.uninstall(&version).await?;
        }
        Commands::Use { version } => {
            manager.use_version(&version)?;
        }
        Commands::Current => {
            match manager.current().await? {
                Some(version) => println!("Current active version: {}", version),
                None => println!("No version is currently active through rnm"),
            }
        }
    }
    Ok(())
}



