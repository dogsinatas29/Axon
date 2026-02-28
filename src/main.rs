mod cli;
mod config;
mod core;
mod protocol;
mod web;

use crate::cli::{Cli, Commands};
use clap::Parser;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    match args.command {
        Some(Commands::Init { name, juniors }) => {
            cli::handle_init(name, juniors)?;
            return Ok(());
        }
        Some(Commands::Start) | None => {
            run_factory().await?;
        }
    }

    Ok(())
}

async fn run_factory() -> Result<(), Box<dyn std::error::Error>> {
    info!("🚀 AXON v0.0.1: The Automated Software Factory is starting...");

    // Load config
    let config = config::AxonConfig::load();
    info!("📂 Loaded config: {}", config.project_name);

    // Initialize FactoryDaemon
    let daemon = std::sync::Arc::new(core::FactoryDaemon::new());

    // Start Axum Web Server in a background task
    let daemon_web = daemon.clone();
    tokio::spawn(async move {
        if let Err(e) = web::start_web_server(daemon_web).await {
            tracing::error!("❌ Web server error: {}", e);
        }
    });

    // Start File Watcher
    if let Err(e) = daemon.start_watcher("ARCHITECTURE_AXON.md") {
        tracing::error!("❌ Watcher error: {}", e);
    }

    info!("🏭 Factory is online and waiting for agents.");

    // Keep the main thread alive
    tokio::signal::ctrl_c().await?;
    info!("🛑 AXON is shutting down...");

    Ok(())
}
