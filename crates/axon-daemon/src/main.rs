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
        Commands::Run { resume } => {
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

            // Architect Recruitment
            #[derive(serde::Serialize, serde::Deserialize)]
            struct AxonConfig {
                arch_brand: String, arch_model: String,
                senior_brand: String, senior_model: String, senior_count: usize,
                junior_brand: String, junior_model: String, junior_count: usize,
            }

            let mut fast_cfg: Option<AxonConfig> = None;
            if std::path::Path::new("axon_config.json").exists() {
                if let Ok(content) = std::fs::read_to_string("axon_config.json") {
                    if let Ok(parsed) = serde_json::from_str::<AxonConfig>(&content) {
                        if resume {
                            fast_cfg = Some(parsed);
                        } else {
                            let choice = prompt("📦 Existing factory settings (axon_config.json) found. Fast Resume? [Y/n]: ");
                            if choice.trim().to_lowercase() != "n" {
                                fast_cfg = Some(parsed);
                            }
                        }
                    }
                }
            }

            let get_drv = |brand: &str, model: &str| -> Arc<dyn axon_model::ModelDriver + Send + Sync> {
                let key = match brand {
                    "Gemini" => std::env::var("GEMINI_API_KEY").unwrap_or_default(),
                    "Claude" => std::env::var("CLAUDE_API_KEY").unwrap_or_default(),
                    "ChatGPT" => std::env::var("OPEN_AI_KEY").unwrap_or_default(),
                    _ => "".to_string(),
                };
                match brand {
                    "Gemini" => Arc::new(axon_model::GeminiDriver::new(key, model.to_string())),
                    "Claude" => Arc::new(axon_model::ClaudeDriver::new(key, model.to_string())),
                    "ChatGPT" => Arc::new(axon_model::OpenAIDriver::new(key, model.to_string())),
                    _ => Arc::new(axon_model::MockDriver),
                }
            };

            let (architect_model, arch_name, senior_model, senior_count_val, junior_model, junior_count_val) = if let Some(cfg) = &fast_cfg {
                println!("✅ Resuming factory operation from saved configuration...");
                (
                    get_drv(&cfg.arch_brand, &cfg.arch_model), cfg.arch_model.clone(),
                    get_drv(&cfg.senior_brand, &cfg.senior_model), cfg.senior_count,
                    get_drv(&cfg.junior_brand, &cfg.junior_model), cfg.junior_count
                )
            } else {
            let (architect_model, arch_brand, arch_name) = 'arch_recruit: loop {
                println!("--- [Stage: Architect (CTO) Recruitment] ---");
                display_models(&available_models);
                let brand_val = prompt("Select Brand for Architect (CTO): ");
                
                if let Ok(idx) = brand_val.parse::<usize>() {
                    if idx > 0 && idx <= available_models.len() {
                        let (name, key) = &available_models[idx - 1];
                        println!("🔍 Fetching available {} versions...", name);
                        let driver: Arc<dyn axon_model::ModelDriver + Send + Sync> = match *name {
                            "Gemini" => Arc::new(axon_model::GeminiDriver::new(key.clone(), "list".into())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                            "Claude" => Arc::new(axon_model::ClaudeDriver::new(key.clone(), "list".into())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                            "ChatGPT" => Arc::new(axon_model::OpenAIDriver::new(key.clone(), "list".into())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                            _ => Arc::new(axon_model::MockDriver) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                        };
                        
                        if let Ok(models) = driver.list_available_models().await {
                            if models.is_empty() {
                                println!("⚠️ No models found for this brand. Using MOCK.");
                                break 'arch_recruit (Arc::new(axon_model::MockDriver) as Arc<dyn axon_model::ModelDriver + Send + Sync>, name.to_string(), "mock".to_string());
                            }
                            let models: Vec<String> = models;
                            for (i, m) in models.iter().enumerate() {
                                println!("  {}. {}", i + 1, m);
                            }
                            loop {
                                let choice = prompt(&format!("Select Version for Architect [1-{}]: ", models.len()));
                                if let Ok(c_idx) = choice.parse::<usize>() {
                                    if c_idx > 0 && c_idx <= models.len() {
                                        let m_name = models[c_idx - 1].clone();
                                        let final_driver: Arc<dyn axon_model::ModelDriver + Send + Sync> = match *name {
                                            "Gemini" => Arc::new(axon_model::GeminiDriver::new(key.clone(), m_name.clone())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                            "Claude" => Arc::new(axon_model::ClaudeDriver::new(key.clone(), m_name.clone())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                            "ChatGPT" => Arc::new(axon_model::OpenAIDriver::new(key.clone(), m_name.clone())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                            _ => Arc::new(axon_model::MockDriver) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                        };
                                        println!("✅ Architect assigned: {} ({})\n", m_name, name);
                                        break 'arch_recruit (final_driver, name.to_string(), m_name);
                                    }
                                }
                                println!("❌ Invalid version. Please choose 1-{}.\n", models.len());
                            }
                        }
                    }
                }
                println!("❌ Invalid brand selection. Try again.\n");
            };

            // Seniors Recruitment
            let (senior_model, senior_brand, s_name) = 'senior_recruit: loop {
                println!("--- [Stage: Seniors Recruitment] ---");
                display_models(&available_models);
                let brand_val = prompt("Select Brand for Seniors: ");
                
                if let Ok(idx) = brand_val.parse::<usize>() {
                    if idx > 0 && idx <= available_models.len() {
                        let (name, key) = &available_models[idx - 1];
                        println!("🔍 Fetching available {} versions...", name);
                        let driver: Arc<dyn axon_model::ModelDriver + Send + Sync> = match *name {
                            "Gemini" => Arc::new(axon_model::GeminiDriver::new(key.clone(), "list".into())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                            "Claude" => Arc::new(axon_model::ClaudeDriver::new(key.clone(), "list".into())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                            "ChatGPT" => Arc::new(axon_model::OpenAIDriver::new(key.clone(), "list".into())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                            _ => Arc::new(axon_model::MockDriver) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                        };
                        if let Ok(models) = driver.list_available_models().await {
                            if models.is_empty() {
                                println!("⚠️ No models found for this brand. Using MOCK.");
                                break 'senior_recruit (Arc::new(axon_model::MockDriver) as Arc<dyn axon_model::ModelDriver + Send + Sync>, name.to_string(), "mock".to_string());
                            }
                            let models: Vec<String> = models;
                            for (i, m) in models.iter().enumerate() {
                                println!("  {}. {}", i + 1, m);
                            }
                            loop {
                                let choice = prompt(&format!("Select Version for Seniors [1-{}]: ", models.len()));
                                if let Ok(c_idx) = choice.parse::<usize>() {
                                    if c_idx > 0 && c_idx <= models.len() {
                                        let m_name = models[c_idx - 1].clone();
                                        let final_driver: Arc<dyn axon_model::ModelDriver + Send + Sync> = match *name {
                                            "Gemini" => Arc::new(axon_model::GeminiDriver::new(key.clone(), m_name.clone())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                            "Claude" => Arc::new(axon_model::ClaudeDriver::new(key.clone(), m_name.clone())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                            "ChatGPT" => Arc::new(axon_model::OpenAIDriver::new(key.clone(), m_name.clone())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                            _ => Arc::new(axon_model::MockDriver) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                        };
                                        break 'senior_recruit (final_driver, name.to_string(), m_name);
                                    }
                                }
                                println!("❌ Invalid version. Please choose 1-{}.\n", models.len());
                            }
                        }
                    }
                }
                println!("❌ Invalid brand selection. Try again.\n");
            };
            let senior_count_val = loop {
                let val = prompt("How many Seniors to hire? (0-10): ");
                if let Ok(num) = val.parse::<usize>() {
                    if num <= 10 {
                        break num;
                    }
                }
                println!("❌ Invalid number. Please enter a number between 0 and 10.\n");
            };
            println!("✅ {} Senior(s) recruited ({}).\n", senior_count_val, s_name);

            // Juniors Recruitment
            let (junior_model, junior_brand, j_name) = 'junior_recruit: loop {
                println!("--- [Stage: Juniors Recruitment] ---");
                display_models(&available_models);
                let brand_val = prompt("Select Brand for Juniors: ");
                
                if let Ok(idx) = brand_val.parse::<usize>() {
                    if idx > 0 && idx <= available_models.len() {
                        let (name, key) = &available_models[idx - 1];
                        println!("🔍 Fetching available {} versions...", name);
                        let driver: Arc<dyn axon_model::ModelDriver + Send + Sync> = match *name {
                            "Gemini" => Arc::new(axon_model::GeminiDriver::new(key.clone(), "list".into())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                            "Claude" => Arc::new(axon_model::ClaudeDriver::new(key.clone(), "list".into())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                            "ChatGPT" => Arc::new(axon_model::OpenAIDriver::new(key.clone(), "list".into())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                            _ => Arc::new(axon_model::MockDriver) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                        };
                        if let Ok(models) = driver.list_available_models().await {
                            if models.is_empty() {
                                println!("⚠️ No models found for this brand. Using MOCK.");
                                break 'junior_recruit (Arc::new(axon_model::MockDriver) as Arc<dyn axon_model::ModelDriver + Send + Sync>, name.to_string(), "mock".to_string());
                            }
                            let models: Vec<String> = models;
                            for (i, m) in models.iter().enumerate() {
                                println!("  {}. {}", i + 1, m);
                            }
                            loop {
                                let choice = prompt(&format!("Select Version for Juniors [1-{}]: ", models.len()));
                                if let Ok(c_idx) = choice.parse::<usize>() {
                                    if c_idx > 0 && c_idx <= models.len() {
                                        let m_name = models[c_idx - 1].clone();
                                        let final_driver: Arc<dyn axon_model::ModelDriver + Send + Sync> = match *name {
                                            "Gemini" => Arc::new(axon_model::GeminiDriver::new(key.clone(), m_name.clone())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                            "Claude" => Arc::new(axon_model::ClaudeDriver::new(key.clone(), m_name.clone())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                            "ChatGPT" => Arc::new(axon_model::OpenAIDriver::new(key.clone(), m_name.clone())) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                            _ => Arc::new(axon_model::MockDriver) as Arc<dyn axon_model::ModelDriver + Send + Sync>,
                                        };
                                        break 'junior_recruit (final_driver, name.to_string(), m_name);
                                    }
                                }
                                println!("❌ Invalid version. Please choose 1-{}.\n", models.len());
                            }
                        }
                    }
                }
                println!("❌ Invalid brand selection. Try again.\n");
            };
            let junior_count_val = loop {
                let val = prompt("How many Juniors to hire? (0-100): ");
                if let Ok(num) = val.parse::<usize>() {
                    if num <= 100 {
                        break num;
                    }
                }
                println!("❌ Invalid number. Please enter a number between 0 and 100.\n");
            };
            println!("✅ {} Junior(s) recruited ({}).\n", junior_count_val, j_name);

            let new_cfg = AxonConfig {
                arch_brand, arch_model: arch_name.clone(),
                senior_brand, senior_model: s_name, senior_count: senior_count_val,
                junior_brand, junior_model: j_name, junior_count: junior_count_val,
            };
            let _ = std::fs::write("axon_config.json", serde_json::to_string_pretty(&new_cfg).unwrap());
            
            (architect_model, arch_name, senior_model, senior_count_val, junior_model, junior_count_val)
            };

            // Stage 4: Factory Initialization (Spec)
            println!("--- [Stage 4: Factory Specification (Bootstrap Menu)] ---");
            let mut skip_bootstrap = false;
            
            if std::path::Path::new("architecture.md").exists() {
                if resume {
                    skip_bootstrap = true;
                    println!("✅ Auto-resuming factory operation from existing database...\n");
                } else {
                    println!("⚠️  'architecture.md' already exists in this workspace.");
                    let choice = prompt("Do you want to [1] Resume (skip spec re-analysis) or [2] Overwrite and Rebuild? [1/2]: ");
                    if choice.trim() == "1" {
                        skip_bootstrap = true;
                        println!("✅ Resuming factory operation from existing database...\n");
                    }
                }
            }

            let spec_path = if !skip_bootstrap {
                prompt("Enter Specification File Path (e.g., GEMINI.md): ")
            } else {
                "".to_string()
            };

            println!("\n====================================================");
            println!("🚀 ALL SYSTEMS GO: Activating Factory Line...");
            println!("   - Target Spec: {}", spec_path);
            println!("   - Studio UI  : http://localhost:8080");
            println!("====================================================\n");

            thread::sleep(Duration::from_millis(1500));

            // Actual Execution
            let storage = Arc::new(axon_storage::Storage::new("axon.db").expect("Failed to open DB"));
            let (worker_tx, worker_rx) = tokio::sync::mpsc::channel(100);
            
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
            println!("AXON: Checking status...");
        }
    }

    Ok(())
}
