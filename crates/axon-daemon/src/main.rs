/*
 * AXON - The Automated Software Factory
 * Copyright (C) 2026 dogsinatas
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

mod cli;

use axon_daemon::Daemon;
use cli::{Cli, Commands};
use clap::Parser;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            tracing::info!("Initializing AXON project...");
            // Logic for init
        }
        Commands::Read { path } => {
            tracing::info!("Reading blueprint from: {}", path);
            let content = std::fs::read_to_string(&path)?;
            
            let (worker_tx, _) = tokio::sync::mpsc::channel(1); // Dummy for Read
            let storage = Arc::new(axon_storage::Storage::new("axon.db")?);
            let model = Arc::new(axon_model::MockDriver);
            let daemon = Daemon::new(
                storage, 
                model, 
                worker_tx, 
                "Standard AXON Protocol".to_string()
            );
            for line in content.lines() {
                if line.starts_with("## Task:") || line.starts_with("- [ ]") {
                    let title = line.trim_start_matches("## Task:").trim_start_matches("- [ ]").trim();
                    if !title.is_empty() {
                        let thread = axon_core::Thread {
                            id: uuid::Uuid::new_v4().to_string(),
                            project_id: "default-project".to_string(), // Default for CLI
                            title: title.to_string(),
                            status: axon_core::ThreadStatus::Draft,
                            author: "BOSS".to_string(),
                            milestone_id: None,
                            created_at: chrono::Local::now(),
                            updated_at: chrono::Local::now(),
                        };
                        daemon.storage.save_thread(&thread).expect("Failed to save thread");
                        tracing::info!("Generated thread: {}", title);
                    }
                }
            }
        }
        Commands::Run => {
            let storage = Arc::new(axon_storage::Storage::new("axon.db").expect("Failed to open DB"));
            let (worker_tx, worker_rx) = tokio::sync::mpsc::channel(100);
            let model = Arc::new(axon_model::MockDriver);
            let daemon = Arc::new(Daemon::new(
                storage, 
                model, 
                worker_tx, 
                "Standard AXON Protocol".to_string()
            ));
            let daemon_clone = daemon.clone();
            tokio::spawn(async move {
                if let Err(e) = axon_daemon::server::start_server(daemon_clone).await {
                    tracing::error!("Server error: {}", e);
                }
            });
            daemon.run(worker_rx).await?;
        }
        Commands::Status => {
            tracing::info!("Checking AXON status...");
        }
    }

    Ok(())
}
