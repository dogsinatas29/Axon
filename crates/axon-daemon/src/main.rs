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
use std::thread;
use std::time::Duration;

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
            
            let (worker_tx, _) = tokio::sync::mpsc::channel(1);
            let storage = Arc::new(axon_storage::Storage::new("axon.db")?);
            let mock_model: Arc<dyn axon_model::ModelDriver + Send + Sync> = Arc::new(axon_model::MockDriver);
            let daemon = Daemon::new(
                storage, 
                mock_model.clone(), // Architect
                mock_model.clone(), // Senior
                mock_model.clone(), // Junior
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
            use std::io::{self, Write};

            println!("\n====================================================");
            println!("🏭 AXON: Automated Software Factory Bootstrapper");
            println!("====================================================\n");

            // 1. Discover Models
            let mut available_models = Vec::new();
            if let Ok(key) = std::env::var("GEMINI_API_KEY") {
                available_models.push(("Gemini", key));
            }
            if let Ok(key) = std::env::var("CLAUDE_API_KEY") {
                available_models.push(("Claude", key));
            }
            if let Ok(key) = std::env::var("OPEN_AI_KEY") {
                available_models.push(("ChatGPT", key));
            }

            let display_models = |models: &Vec<(&str, String)>| {
                println!("🔍 Available Intelligence:");
                for (i, (name, _)) in models.iter().enumerate() {
                    println!("  {}. {}", i + 1, name);
                }
                println!("  L. LocalAI (Custom Endpoint)\n");
            };

            // Helper to get user input
            let prompt = |msg: &str| -> String {
                print!("{}", msg);
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                input.trim().to_string()
            };

            // Stage 1: Architect (CTO)
            println!("--- [Stage 1: Architect Recruitment] ---");
            display_models(&available_models);
            let arch_val = prompt("Select Intelligence for Architect (CTO - Fixed: 1): ");
            println!("✅ Architect intelligence assigned.\n");

            // Stage 2: Seniors (Reviewers)
            println!("--- [Stage 2: Senior Recruitment] ---");
            display_models(&available_models);
            let senior_model_val = prompt("Select Intelligence for Seniors: ");
            let senior_count_val = prompt("How many Seniors would you like to hire? (0-10): ");
            println!("✅ {} Senior(s) recruited.\n", senior_count_val);

            // Stage 3: Juniors (Workers)
            println!("--- [Stage 3: Junior Recruitment] ---");
            display_models(&available_models);
            let junior_model_val = prompt("Select Intelligence for Juniors: ");
            let junior_count_val = prompt("How many Juniors would you like to hire? (0-100): ");
            println!("✅ {} Junior(s) recruited.\n", junior_count_val);

            // Stage 4: Factory Initialization (Spec)
            println!("--- [Stage 4: Factory Specification (Bootstrap Menu)] ---");
            println!("To initialize 'architecture.md', please provide the source specification.");
            let spec_path = prompt("Enter Specification File Path (e.g., TEST/mile_stone/v0.0.1.md): ");

            println!("\n====================================================");
            println!("🚀 ALL SYSTEMS GO: Activating Factory Line...");
            println!("   - Target Spec: {}", spec_path);
            println!("   - Studio UI  : http://localhost:8080");
            println!("====================================================\n");

            thread::sleep(Duration::from_millis(1500));

            // Actual Execution
            let storage = Arc::new(axon_storage::Storage::new("axon.db").expect("Failed to open DB"));
            let (worker_tx, worker_rx) = tokio::sync::mpsc::channel(100);
            
            // Model Resolver Helper
            let resolve_model = |val: &str, models: &Vec<(&str, String)>| -> Arc<dyn axon_model::ModelDriver + Send + Sync> {
                if let Ok(idx) = val.parse::<usize>() {
                    if idx > 0 && idx <= models.len() {
                        let (_, key) = &models[idx - 1];
                        return Arc::new(axon_model::GeminiDriver::new(key.clone()));
                    }
                }
                Arc::new(axon_model::MockDriver)
            };

            let architect_model = resolve_model(&arch_val, &available_models);
            let senior_model = resolve_model(&senior_model_val, &available_models);
            let junior_model = resolve_model(&junior_model_val, &available_models);

            let daemon = Arc::new(Daemon::new(
                storage, 
                architect_model,
                senior_model,
                junior_model,
                worker_tx, 
                "Standard AXON Protocol".to_string()
            ));

            let daemon_clone = daemon.clone();
            tokio::spawn(async move {
                if let Err(e) = axon_daemon::server::start_server(daemon_clone).await {
                    tracing::error!("Server error: {}", e);
                }
            });

            // Start the production loop
            let daemon_bootstrap = daemon.clone();
            if !spec_path.is_empty() {
                if std::path::Path::new(&spec_path).exists() {
                    let spec_content = std::fs::read_to_string(&spec_path).expect("Failed to read spec file");
                    tokio::spawn(async move {
                        if let Err(e) = daemon_bootstrap.bootstrap_from_spec(spec_content).await {
                            tracing::error!("Bootstrapping failed: {}", e);
                        }
                    });
                } else {
                    tracing::warn!("Spec file '{}' not found. Skipping initial bootstrapping.", spec_path);
                }
            }

            daemon.run(worker_rx).await?;
        }
        Commands::Status => {
            tracing::info!("Checking AXON status...");
        }
    }

    Ok(())
}
