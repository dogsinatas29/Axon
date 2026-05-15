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
use tracing_subscriber::{fmt, Registry, prelude::*};
use axon_daemon::observability::EventBusLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // v0.0.29: [DYNAMIC_OBSERVABILITY] Reloadable filter for Stage-aware logging
    let filter = tracing_subscriber::EnvFilter::new("info");
    let (filter, reload_handle) = tracing_subscriber::reload::Layer::new(filter);

    let subscriber = Registry::default()
        .with(filter)
        .with(fmt::Layer::default())
        .with(EventBusLayer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // v0.0.29: Pass handle to EventBus or keep it available for later
    let logger_handle = Arc::new(reload_handle);
    
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
                "mock-architect".into(),
                vec![mock_model.clone()], // Senior
                vec!["mock-senior".into()],
                vec![mock_model.clone()], // Junior
                vec!["mock-junior".into()],
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
                            task_kind: None,
                            rejection_count: 0,
                            created_at: chrono::Local::now(),
                            updated_at: chrono::Local::now(),
                        };
                        daemon.storage.save_thread(thread).await.expect("Failed to save thread");

                        // v0.0.28: Create a corresponding Task for the Scheduler to pick up
                        let task = axon_core::Task {
                            id: thread_id, // Use same ID for linkage
                            project_id: "default-project".to_string(),
                            title: title.to_string(),
                            description: format!("Automated task generated from spec: {}", title),
                            status: axon_core::TaskStatus::Pending,
                            dependencies: Vec::new(),
                            result: None,
                            target_file: None,
                            lock_files: Vec::new(),
                            error_feedback: None,
                            senior_comment: None,
                            rework_count: 0,
                            base_hash: None,
                            parent_task: None,
                            reason: None,
                            kind: "rust".to_string(),
                            retries: 0,
                            assigned_worker: None,
                            created_at: chrono::Local::now(),
                            ir_path: None,
                            task_kind: None,
                            signature: None,
                        };
                        daemon.storage.save_task(task).await.expect("Failed to save task");
                        
                        tracing::info!("Generated thread & task: {}", title);
                    }
                }
            }
        }
        Commands::Run { resume, spec } => {
            println!("\n====================================================
🏭 AXON: Automated Software Factory v0.0.28_HARDENED
====================================================
======================\n");

            // ... (rest of the config logic remains same)

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

            // v0.0.29: Restore Language Selection Prompt
            let system_locale = std::env::var("LANG").unwrap_or_else(|_| "en_US".to_string());
            let detected_lang = if system_locale.contains("ko") { "ko_KR" }
                                  else if system_locale.contains("ja") { "ja_JP" }
                                  else { "en_US" };
            
            println!("🌐 Detected System Language: {}", detected_lang);
            let use_detected = prompt("Use detected language? [Y/n]: ");
            
            let final_locale = if use_detected.trim().to_lowercase() == "n" {
                println!("\nSelect Language / 언어 선택 / 言語選択:");
                println!("  1. English (en_US)");
                println!("  2. 한국어 (ko_KR)");
                println!("  3. 日本語 (ja_JP)");
                let lang_choice = prompt("Choice (1-3): ");
                match lang_choice.trim() {
                    "2" => "ko_KR".to_string(),
                    "3" => "ja_JP".to_string(),
                    _ => "en_US".to_string(),
                }
            } else {
                detected_lang.to_string()
            };
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
                        let msg = if final_locale == "ko_KR" { 
                            "📦 기존 설정(axon_config.json)을 발견했습니다. 빠른 재개를 사용하시겠습니까? [Y/n]: " 
                        } else if final_locale == "ja_JP" {
                            "📦 既存の設定(axon_config.json)が見つかりました。高速再開を使用しますか？ [Y/n]: "
                        } else { 
                            "📦 Existing factory settings (axon_config.json) found. Fast Resume? [Y/n]: " 
                        };
                        let choice = prompt(msg);
                        if choice.trim().to_lowercase() != "n" { fast_cfg = Some(parsed); }
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


            let (architect_model, arch_name, senior_models, senior_model_names, junior_models, junior_model_names) = if let Some(cfg) = &fast_cfg {
                let msg = if final_locale == "ko_KR" { "✅ 저장된 설정으로부터 공장 가동을 재개합니다..." } else if final_locale == "ja_JP" { "✅ 保存された設定から工場の稼働を再開します..." } else { "✅ Resuming factory operation..." };
                println!("{}", msg);
                let arch_drv = get_drv(&cfg.agents.architect);
                let mut s_drvs = Vec::new();
                let mut s_names = Vec::new();
                for s_cfg in &cfg.agents.seniors { s_drvs.push(get_drv(s_cfg)); s_names.push(s_cfg.model.clone()); }
                let mut j_drvs = Vec::new();
                let mut j_names = Vec::new();
                for j_cfg in &cfg.agents.juniors { j_drvs.push(get_drv(j_cfg)); j_names.push(j_cfg.model.clone()); }
                (arch_drv, cfg.agents.architect.model.clone(), s_drvs, s_names, j_drvs, j_names)
            } else {
                let arch_config: AgentConfig;
                let mut senior_configs = Vec::new();
                let mut junior_configs = Vec::new();
                let mut global_local_endpoint: Option<String> = None;
                let mut use_global_endpoint = false;

                async fn recruit_agent_async(role: &str, _available_models: &Vec<(&str, String)>, locale: &str, cached_endpoint: &mut Option<String>, use_cached: &mut bool) -> AgentConfig {
                    let recruit_header = if locale == "ko_KR" { format!("\n--- [{} 모집] ---", role) } else if locale == "ja_JP" { format!("\n--- [{} 募集] ---", role) } else { format!("\n--- [Recruiting {}] ---", role) };
                    println!("{}", recruit_header);
                    
                    let engine_list = if locale == "ko_KR" { "🔍 사용 가능한 엔진: (1. Gemini, L. LocalAI)" } else if locale == "ja_JP" { "🔍 使用可能なエンジン: (1. Gemini, L. LocalAI)" } else { "🔍 Available Engines: (1. Gemini, L. LocalAI)" };
                    println!("{}", engine_list);
                    
                    let provider_prompt = if locale == "ko_KR" { format!("{}를 위한 제공자 선택 (번호 또는 L): ", role) } else if locale == "ja_JP" { format!("{}のためのプロバイダー選択 (番号または L): ", role) } else { format!("Select provider for {} (Number or L): ", role) };
                    let p_idx_str = prompt(&provider_prompt);
                    
                    let (runtime, provider, endpoint) = if p_idx_str.to_lowercase() == "l" {
                        let ep = if *use_cached && cached_endpoint.is_some() { cached_endpoint.clone().unwrap() } else {
                            loop {
                                let ep_prompt = if locale == "ko_KR" { "로컬 엔드포인트 입력: " } else if locale == "ja_JP" { "ローカルエンドポイント入力: " } else { "Enter local endpoint: " };
                                let input_ep = prompt(ep_prompt).trim_end_matches('/').to_string();
                                
                                let wait_msg = if locale == "ko_KR" { format!("⏳ {} 연결 확인 중...", input_ep) } else if locale == "ja_JP" { format!("⏳ {} 接続確認中...", input_ep) } else { format!("⏳ Checking connection to {}...", input_ep) };
                                println!("{}", wait_msg);
                                
                                if reqwest::get(&input_ep).await.is_ok() { 
                                    let success_msg = if locale == "ko_KR" { "✅ [SUCCESS] 접속 가능합니다.\n" } else if locale == "ja_JP" { "✅ [SUCCESS] 接続可能です。\n" } else { "✅ [SUCCESS] Connection established.\n" };
                                    println!("{}", success_msg); 
                                    break input_ep; 
                                }
                                
                                let fail_msg = if locale == "ko_KR" { "❌ [FAILED] 접속 실패. 다시 입력하세요." } else if locale == "ja_JP" { "❌ [FAILED] 接続失敗。再入力してください。" } else { "❌ [FAILED] Connection failed. Please retry." };
                                println!("{}", fail_msg);
                            }
                        };
                        if cached_endpoint.is_none() {
                            *cached_endpoint = Some(ep.clone());
                            let apply_all_prompt = if locale == "ko_KR" { "이후 모든 요원에게 이 주소를 동일하게 적용할까요? [Y/n]: " } else if locale == "ja_JP" { "以降のすべてのエージェントにこのアドレスを適用しますか？ [Y/n]: " } else { "Apply this endpoint to all future agents? [Y/n]: " };
                            if prompt(apply_all_prompt).to_lowercase() != "n" { *use_cached = true; }
                        }
                        ("local".to_string(), None, Some(ep))
                    } else { ("cloud".to_string(), Some("gemini".to_string()), None) };

                    let drv: Arc<dyn axon_model::ModelDriver + Send + Sync> = if runtime == "local" {
                        Arc::new(axon_model::OllamaDriver::new(endpoint.clone().unwrap_or_default(), "".into()))
                    } else { Arc::new(axon_model::MockDriver) };

                    let mut model_name = String::new();
                    if let Ok(models) = drv.list_available_models().await {
                        let avail_models_msg = if locale == "ko_KR" { "사용 가능한 모델:" } else if locale == "ja_JP" { "使用可能なモデル:" } else { "Available models:" };
                        println!("{}", avail_models_msg);
                        for (i, m) in models.iter().enumerate() { println!("  {}. {}", i + 1, m); }
                        
                        let select_msg = if locale == "ko_KR" { "번호 선택 (또는 이름 입력): " } else if locale == "ja_JP" { "番号選択 (または名前入力): " } else { "Select number (or enter name): " };
                        let m_idx_str = prompt(select_msg);
                        if let Ok(m_idx) = m_idx_str.parse::<usize>() { if let Some(m) = models.get(m_idx - 1) { model_name = m.clone(); } }
                        if model_name.is_empty() { model_name = m_idx_str; }
                    }
                    AgentConfig { runtime, provider, endpoint, model: model_name }
                }

                arch_config = recruit_agent_async("Architect", &available_models, &final_locale, &mut global_local_endpoint, &mut use_global_endpoint).await;
                
                let senior_count_prompt = if final_locale == "ko_KR" { "\n시니어 요원 수 (기본 1): " } else if final_locale == "ja_JP" { "\nシニアエージェント数 (基本 1): " } else { "\nNumber of Senior Agents (Default 1): " };
                let senior_count: usize = prompt(senior_count_prompt).parse().unwrap_or(1);
                for i in 0..senior_count { senior_configs.push(recruit_agent_async(&format!("Senior #{}", i + 1), &available_models, &final_locale, &mut global_local_endpoint, &mut use_global_endpoint).await); }
                
                let junior_count_prompt = if final_locale == "ko_KR" { "\n주니어 요원 수 (기본 1): " } else if final_locale == "ja_JP" { "\nジュニアエージェント数 (基本 1): " } else { "\nNumber of Junior Agents (Default 1): " };
                let junior_count: usize = prompt(junior_count_prompt).parse().unwrap_or(1);
                for i in 0..junior_count { junior_configs.push(recruit_agent_async(&format!("Junior #{}", i + 1), &available_models, &final_locale, &mut global_local_endpoint, &mut use_global_endpoint).await); }

                let cfg = AxonConfig {
                    agents: AgentsConfig { architect: arch_config, seniors: senior_configs, juniors: junior_configs },
                    execution: ExecutionConfig { review_queue_limit: 5, sampling_rate: 0.3, fallback_enabled: true },
                    locale: final_locale.clone(),
                };
                if let Ok(json) = serde_json::to_string_pretty(&cfg) { let _ = std::fs::write("axon_config.json", json); }
                
                let mut s_drvs = Vec::new(); let mut s_names = Vec::new();
                for s_cfg in &cfg.agents.seniors { s_drvs.push(get_drv(s_cfg)); s_names.push(s_cfg.model.clone()); }
                let mut j_drvs = Vec::new(); let mut j_names = Vec::new();
                for j_cfg in &cfg.agents.juniors { j_drvs.push(get_drv(j_cfg)); j_names.push(j_cfg.model.clone()); }
                (get_drv(&cfg.agents.architect), cfg.agents.architect.model.clone(), s_drvs, s_names, j_drvs, j_names)
            };

            // --- Configuration Briefing (v0.0.28) ---
            println!("\n📋 --------------------------------------");
            let briefing_title = if final_locale == "ko_KR" { "현재 공장 가동 설정 요약" } else if final_locale == "ja_JP" { "現在の工場稼働設定の要約" } else { "Factory Configuration Briefing" };
            println!("   [{}]", briefing_title);
            println!("   - Architect : {}", arch_name);
            println!("   - Seniors   : {}", senior_model_names.join(", "));
            println!("   - Juniors   : {}", junior_model_names.join(", "));
            println!("   - Locale    : {}", final_locale);
            println!("------------------------------------------\n");

            // Stage 4: Factory Initialization (Spec)
            let stage4_title = if final_locale == "ko_KR" { "--- [Stage 4: 공장 사양 설정 (부트스트랩 메뉴)] ---" } else if final_locale == "ja_JP" { "--- [Stage 4: 工場仕様設定 (ブートストラップメニュー)] ---" } else { "--- [Stage 4: Factory Specification (Bootstrap Menu)] ---" };
            println!("{}", stage4_title);
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

            let mut spec_path = if !skip_bootstrap {
                if let Some(s) = spec {
                    s
                } else {
                    let msg = if final_locale == "ko_KR" {
                        "공장 가동을 위한 요구사항 명세서 경로를 입력하세요 (예: GEMINI.md)\n[이미 진행 중인 작업을 이어서 하려면 아무것도 입력하지 말고 엔터를 누르세요]: "
                    } else if final_locale == "ja_JP" {
                        "工場の稼働に必要な要件定義書のパスを入力してください (例: spec.md)\n[既存のプロジェクトを再開する場合は、何も入力せずにエンターキーを押してください]: "
                    } else {
                        "Enter Specification File Path (e.g., GEMINI.md)\n[Press Enter to SKIP if resuming an existing project]: "
                    };
                    prompt(msg)
                }
            } else {
                "".to_string()
            };
            
            // v0.0.29: Input Validation Guard
            if !skip_bootstrap && !spec_path.is_empty() && !std::path::Path::new(&spec_path).exists() {
                println!("❌ Spec file '{}' not found. Falling back to manual input.", spec_path);
                loop {
                    let msg = if final_locale == "ko_KR" {
                        "공장 가동을 위한 요구사항 명세서 경로를 다시 입력하세요: "
                    } else if final_locale == "ja_JP" {
                        "工場の稼働に必要な要件定義書のパスを再入力してください: "
                    } else {
                        "Please re-enter Specification File Path: "
                    };
                    let input = prompt(msg);
                    if input.is_empty() || std::path::Path::new(&input).exists() {
                        spec_path = input;
                        break;
                    }
                    println!("❌ File '{}' not found.", input);
                }
            }

            println!("\n====================================================");
            let msg_all_systems = if final_locale == "ko_KR" { "🚀 모든 시스템 가동 준비 완료: 공장 라인 활성화 중..." } else if final_locale == "ja_JP" { "🚀 全システム稼働準備完了: 工場ラインを活性化中..." } else { "🚀 ALL SYSTEMS GO: Activating Factory Line..." };
            println!("{}", msg_all_systems);
            let msg_target_spec = if final_locale == "ko_KR" { format!("   - 대상 명세서: {}", spec_path) } else if final_locale == "ja_JP" { format!("   - ターゲット仕様書: {}", spec_path) } else { format!("   - Target Spec: {}", spec_path) };
            println!("{}", msg_target_spec);
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
                arch_name, // v0.0.28: Use the explicitly selected name
                senior_models,
                senior_model_names, // v0.0.28: Use the explicitly selected names
                junior_models,
                junior_model_names, // v0.0.28: Use the explicitly selected names
                worker_tx,
                "Standard AXON Protocol".to_string(),
                sampling_rate,
                final_locale.clone(),
            ));

            EventBusLayer::init(daemon.event_bus.clone());

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

            // v0.0.29: [RELEASE_THE_TRACE] Configuration complete. Activate full observability for factory run.
            let _ = logger_handle.modify(|filter| {
                *filter = tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("trace"));
            });

            daemon.run().await?;
        }
        Commands::Status => {
            println!("AXON: Checking status...");
        }
    }

    Ok(())
}
