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
use clap::Parser;
use cli::{Cli, Commands};
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
            let mock_model: Arc<dyn axon_model::ModelDriver + Send + Sync> =
                Arc::new(axon_model::MockDriver);
            let daemon = Daemon::new(
                storage,
                mock_model.clone(), // Architect
                vec![mock_model.clone()], // Senior
                vec![mock_model.clone()], // Junior
                worker_tx,
                "Standard AXON Protocol".to_string(),
                1.0,
                "en_US".to_string(),
            );
            for line in content.lines() {
                if line.starts_with("## Task:") || line.starts_with("- [ ]") {
                    let title = line
                        .trim_start_matches("## Task:")
                        .trim_start_matches("- [ ]")
                        .trim();
                    if !title.is_empty() {
                        let thread_id = uuid::Uuid::new_v4().to_string();
                        let thread = axon_core::Thread {
                            id: thread_id.clone(),
                            project_id: "default-project".to_string(), 
                            title: title.to_string(),
                            status: axon_core::ThreadStatus::Draft,
                            author: "BOSS".to_string(),
                            milestone_id: None,
                            created_at: chrono::Local::now(),
                            updated_at: chrono::Local::now(),
                        };
                        daemon.storage.save_thread(&thread).expect("Failed to save thread");

                        // v0.0.16: Create a corresponding Task for the Scheduler to pick up
                        let task = axon_core::Task {
                            id: thread_id, // Use same ID for linkage
                            project_id: "default-project".to_string(),
                            title: title.to_string(),
                            description: format!("Automated task generated from spec: {}", title),
                            status: axon_core::TaskStatus::Pending,
                            result: None,
                            created_at: chrono::Local::now(),
                        };
                        daemon.storage.save_task(&task).expect("Failed to save task");
                        
                        tracing::info!("Generated thread & task: {}", title);
                    }
                }
            }
        }
        Commands::Run { resume } => {
            println!("\n====================================================");
            println!("🏭 AXON: Automated Software Factory Bootstrapper");
            println!("====================================================\n");

            #[derive(serde::Serialize, serde::Deserialize, Clone)]
            struct AgentConfig {
                runtime: String,
                provider: Option<String>,
                endpoint: Option<String>,
                model: String,
            }

            #[derive(serde::Serialize, serde::Deserialize, Clone)]
            struct ExecutionConfig {
                review_queue_limit: usize,
                sampling_rate: f32,
                fallback_enabled: bool,
            }

            #[derive(serde::Serialize, serde::Deserialize, Clone)]
            struct AgentsConfig {
                architect: AgentConfig,
                seniors: Vec<AgentConfig>,
                juniors: Vec<AgentConfig>,
            }

            #[derive(serde::Serialize, serde::Deserialize, Clone)]
            struct AxonConfig {
                agents: AgentsConfig,
                execution: ExecutionConfig,
                locale: String,
            }

            fn prompt(msg: &str) -> String {
                use std::io::{self, Write};
                print!("{}", msg);
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                input.trim().to_string()
            }

            // --- PHASE 00: Language Selector ---
            let system_locale = std::env::var("LANG").unwrap_or_else(|_| "en_US".to_string());
            let mut final_locale = if system_locale.contains("ko") { "ko_KR".to_string() }
                                  else if system_locale.contains("ja") { "ja_JP".to_string() }
                                  else { "en_US".to_string() };

            let lang_choice = prompt(&format!("Use detected language ({})? [Y/n]: ", final_locale));
            if lang_choice.to_lowercase() == "n" {
                println!("\nSelect Factory Language:");
                println!("  1. English (en_US)");
                println!("  2. 한국어 (ko_KR)");
                println!("  3. 日本語 (ja_JP)");
                let manual_choice = prompt("Choice [1-3]: ");
                final_locale = match manual_choice.as_str() {
                    "2" => "ko_KR".to_string(),
                    "3" => "ja_JP".to_string(),
                    _ => "en_US".to_string(),
                };
            }
            println!("✅ Language Set to: {}\n", final_locale);

            let mut available_models = Vec::new();
            if let Ok(key) = std::env::var("GEMINI_API_KEY") { available_models.push(("Gemini", key)); }
            if let Ok(key) = std::env::var("CLAUDE_API_KEY") { available_models.push(("Claude", key)); }
            if let Ok(key) = std::env::var("OPEN_AI_KEY") { available_models.push(("ChatGPT", key)); }

            let mut fast_cfg: Option<AxonConfig> = None;
            if std::path::Path::new("axon_config.json").exists() {
                if let Ok(content) = std::fs::read_to_string("axon_config.json") {
                    if let Ok(mut parsed) = serde_json::from_str::<AxonConfig>(&content) {
                        parsed.locale = final_locale.clone();
                        if resume {
                            fast_cfg = Some(parsed);
                        } else {
                            let msg = if final_locale == "ko_KR" { "📦 기존 설정(axon_config.json)을 발견했습니다. 빠른 재개를 사용하시겠습니까? [Y/n]: " } else { "📦 Existing factory settings (axon_config.json) found. Fast Resume? [Y/n]: " };
                            let choice = prompt(msg);
                            if choice.trim().to_lowercase() != "n" { fast_cfg = Some(parsed); }
                        }
                    }
                }
            }

            let get_drv = |cfg: &AgentConfig| -> Arc<dyn axon_model::ModelDriver + Send + Sync> {
                match cfg.runtime.as_str() {
                    "cloud" => {
                        let provider = cfg.provider.as_deref().unwrap_or("gemini");
                        let key = match provider {
                            "gemini" => std::env::var("GEMINI_API_KEY").unwrap_or_default(),
                            "claude" => std::env::var("CLAUDE_API_KEY").unwrap_or_default(),
                            "openai" => std::env::var("OPEN_AI_KEY").unwrap_or_default(),
                            _ => "".to_string(),
                        };
                        match provider {
                            "gemini" => Arc::new(axon_model::GeminiDriver::new(key, cfg.model.clone())),
                            "claude" => Arc::new(axon_model::ClaudeDriver::new(key, cfg.model.clone())),
                            "openai" => Arc::new(axon_model::OpenAIDriver::new(key, cfg.model.clone())),
                            _ => Arc::new(axon_model::MockDriver),
                        }
                    }
                    "local" => {
                        let endpoint = cfg.endpoint.as_deref().unwrap_or("http://localhost:11434");
                        Arc::new(axon_model::OllamaDriver::new(endpoint.to_string(), cfg.model.clone()))
                    }
                    _ => Arc::new(axon_model::MockDriver),
                }
            };

            let (architect_model, _arch_name, senior_models, junior_models) = if let Some(cfg) = &fast_cfg {
                let msg = if final_locale == "ko_KR" { "✅ 저장된 설정으로부터 공장 가동을 재개합니다..." } else { "✅ Resuming factory operation from saved configuration..." };
                println!("{}", msg);
                let arch_drv = get_drv(&cfg.agents.architect);
                let mut s_drvs = Vec::new();
                for s_cfg in &cfg.agents.seniors { s_drvs.push(get_drv(s_cfg)); }
                let mut j_drvs = Vec::new();
                for j_cfg in &cfg.agents.juniors { j_drvs.push(get_drv(j_cfg)); }
                (arch_drv, cfg.agents.architect.model.clone(), s_drvs, j_drvs)
            } else {
                let mut arch_config = None;
                let mut senior_configs = Vec::new();
                let mut junior_configs = Vec::new();

                async fn recruit_agent_async(role: &str, available_models: &Vec<(&str, String)>, locale: &str) -> AgentConfig {
                    let title = if locale == "ko_KR" { format!("{} 모집", role) } else { format!("RECRUITING: {}", role.to_uppercase()) };
                    println!("\n--- [{}] ---", title);
                    if locale == "ko_KR" { println!("🔍 사용 가능한 엔진:"); } else { println!("Q Available Intelligence:"); }
                    for (i, (name, _)) in available_models.iter().enumerate() { println!("  {}. {}", i + 1, name); }
                    println!("  L. LocalAI (Custom Endpoint)");
                    
                    let msg = if locale == "ko_KR" { format!("{}를 위한 제공자 선택 (번호 또는 L): ", role) } else { format!("Select Provider for {} (Number or L): ", role) };
                    let p_idx_str = prompt(&msg);
                    let (runtime, provider, endpoint) = if p_idx_str.to_lowercase() == "l" {
                        let msg_e = if locale == "ko_KR" { "로컬 엔드포인트 입력: " } else { "Enter Local Endpoint: " };
                        ("local".to_string(), None, Some(prompt(&msg_e).trim_end_matches('/').to_string()))
                    } else {
                        let idx: usize = p_idx_str.parse().unwrap_or(1);
                        let name = available_models.get(idx - 1).map(|(n, _)| *n).unwrap_or("Gemini");
                        ("cloud".to_string(), Some(name.to_lowercase()), None)
                    };

                    let drv: Arc<dyn axon_model::ModelDriver + Send + Sync> = if runtime == "cloud" {
                        let provider_name = provider.as_deref().unwrap_or("gemini");
                        let key = match provider_name {
                            "gemini" => std::env::var("GEMINI_API_KEY").unwrap_or_default(),
                            "claude" => std::env::var("CLAUDE_API_KEY").unwrap_or_default(),
                            "openai" => std::env::var("OPEN_AI_KEY").unwrap_or_default(),
                            _ => "".to_string()
                        };
                        match provider_name {
                            "gemini" => Arc::new(axon_model::GeminiDriver::new(key, "".into())),
                            "claude" => Arc::new(axon_model::ClaudeDriver::new(key, "".into())),
                            "openai" => Arc::new(axon_model::OpenAIDriver::new(key, "".into())),
                            _ => Arc::new(axon_model::MockDriver),
                        }
                    } else {
                        Arc::new(axon_model::OllamaDriver::new(endpoint.clone().unwrap_or_default(), "".into()))
                    };

                    let msg_d = if locale == "ko_KR" { format!("🔍 {}를 위한 모델 검색 중...", role) } else { format!("🔍 Discovering models for {}...", role) };
                    println!("{}", msg_d);
                    let mut model_name = String::new();
                    if let Ok(models) = drv.list_available_models().await {
                        if !models.is_empty() {
                            if locale == "ko_KR" { println!("사용 가능한 모델:"); } else { println!("Available Models:"); }
                            for (i, m) in models.iter().enumerate() { println!("  {}. {}", i + 1, m); }
                            let msg_m = if locale == "ko_KR" { "번호 선택 (또는 이름 입력): " } else { "Select Number (or type name): " };
                            let m_idx_str = prompt(&msg_m);
                            if let Ok(m_idx) = m_idx_str.parse::<usize>() { if let Some(m) = models.get(m_idx - 1) { model_name = m.clone(); } }
                            if model_name.is_empty() { model_name = m_idx_str; }
                        }
                    }
                    if model_name.is_empty() { 
                        let msg_man = if locale == "ko_KR" { "모델명 직접 입력: " } else { "Enter Model Name: " };
                        model_name = prompt(&msg_man); 
                    }
                    AgentConfig { runtime, provider, endpoint, model: model_name }
                }

                arch_config = Some(recruit_agent_async("Architect", &available_models, &final_locale).await);
                let msg_s = if final_locale == "ko_KR" { "\n시니어 요원 수 (기본 1): " } else { "\nNumber of Seniors (default 1): " };
                let senior_count: usize = prompt(&msg_s).parse().unwrap_or(1);
                for i in 0..senior_count { senior_configs.push(recruit_agent_async(&format!("Senior #{}", i + 1), &available_models, &final_locale).await); }
                let msg_j = if final_locale == "ko_KR" { "\n주니어 요원 수 (기본 1): " } else { "\nNumber of Juniors (default 1): " };
                let junior_count: usize = prompt(&msg_j).parse().unwrap_or(1);
                for i in 0..junior_count { junior_configs.push(recruit_agent_async(&format!("Junior #{}", i + 1), &available_models, &final_locale).await); }

                let cfg = AxonConfig {
                    agents: AgentsConfig { architect: arch_config.unwrap(), seniors: senior_configs, juniors: junior_configs },
                    execution: ExecutionConfig { review_queue_limit: 5, sampling_rate: 0.3, fallback_enabled: true },
                    locale: final_locale.clone(),
                };

                if let Ok(json) = serde_json::to_string_pretty(&cfg) {
                    let _ = std::fs::write("axon_config.json", json);
                    let msg_save = if final_locale == "ko_KR" { "\n💾 설정 저장 완료." } else { "\n💾 Configuration saved." };
                    println!("{}", msg_save);
                }
                let mut s_drvs = Vec::new();
                for s_cfg in &cfg.agents.seniors { s_drvs.push(get_drv(s_cfg)); }
                let mut j_drvs = Vec::new();
                for j_cfg in &cfg.agents.juniors { j_drvs.push(get_drv(j_cfg)); }
                (get_drv(&cfg.agents.architect), cfg.agents.architect.model.clone(), s_drvs, j_drvs)
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
                    let choice = prompt(
                        "Do you want to [1] Resume (skip spec re-analysis) or [2] Overwrite and Rebuild? [1/2]: ",
                    );
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
            let storage =
                Arc::new(axon_storage::Storage::new("axon.db").expect("Failed to open DB"));
            let (worker_tx, _worker_rx) = tokio::sync::mpsc::channel(100);

            let sampling_rate = fast_cfg.as_ref().map(|c| c.execution.sampling_rate as f64).unwrap_or(0.3);

            let daemon = Arc::new(Daemon::new(
                storage,
                architect_model,
                senior_models,
                junior_models,
                worker_tx,
                "Standard AXON Protocol".to_string(),
                sampling_rate,
                final_locale.clone(),
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
                    let path_for_bootstrap = spec_path.clone();
                    tokio::spawn(async move {
                        if let Err(e) = daemon_bootstrap.bootstrap_from_spec(path_for_bootstrap).await {
                            tracing::error!("Bootstrapping failed: {}", e);
                        }
                    });
                } else {
                    tracing::warn!(
                        "Spec file '{}' not found. Skipping initial bootstrapping.",
                        spec_path
                    );
                }
            }

            daemon.run().await?;
        }
        Commands::Status => {
            println!("AXON: Checking status...");
        }
    }

    Ok(())
}
