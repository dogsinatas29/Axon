use axon_daemon::{DeterministicKernel, KernelConfig};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::io::{self, Write};
use std::sync::Arc;
use clap::Parser;
mod cli;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "info") };
    }
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            tracing_subscriber::fmt::init();
            run_init_wizard();
        }
        Commands::Run { resume, spec } => {
            let event_bus = Arc::new(axon_core::events::EventBus::new(256));
            let storage = Arc::new(axon_storage::Storage::new("runtime/state.db")
                .map_err(|e| format!("Failed to initialize Storage: {}", e))?);

            axon_daemon::observability::EventBusLayer::init(event_bus.clone());

            use tracing_subscriber::layer::SubscriberExt;
            use tracing_subscriber::Registry;
            let subscriber = Registry::default()
                .with(tracing_subscriber::EnvFilter::new(
                    &std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
                ))
                .with(tracing_subscriber::fmt::Layer::default())
                .with(axon_daemon::observability::EventBusLayer);
            tracing::subscriber::set_global_default(subscriber)?;

            let (config, spec_path, skip_bootstrap) = resolve_bootstrap_config(*resume, spec.as_deref());

            let kernel = DeterministicKernel::new(config, storage, event_bus);

            if skip_bootstrap {
                kernel.run().await?;
            } else if let Some(path) = &spec_path {
                if path.is_empty() {
                    kernel.run().await?;
                } else {
                    kernel.start_with_spec(path).await?;
                }
            } else {
                kernel.run().await?;
            }
        }
        Commands::Read { path } => {
            tracing_subscriber::fmt::init();
            println!("Reading blueprint from: {}", path);
        }
        Commands::Status => {
            tracing_subscriber::fmt::init();
            println!("AXON Daemon is offline.");
        }
    }
    
    Ok(())
}

fn resolve_bootstrap_config(resume: bool, spec_arg: Option<&str>) -> (KernelConfig, Option<String>, bool) {
    // 1. Check existing axon_config.json
    let config_exists = std::path::Path::new("axon_config.json").exists();

    if !config_exists && !resume {
        run_init_wizard();
    } else if config_exists && !resume {
        prompt_config_reuse();
    }

    // 2. Load config
    let axon_config = axon_daemon::AxonConfig::load("axon_config.json")
        .expect("Failed to load axon_config.json");

    let mut skip_bootstrap = false;

    // 3. Check existing architecture.md
    let arch_path = std::path::Path::new("architecture.md");
    if arch_path.exists() && !resume {
        println!("⚠️  'architecture.md' already exists in this workspace.");
        print!("Do you want to [1] Resume (skip spec re-analysis) or [2] Overwrite and Rebuild? [1/2]: ");
        io::stdout().flush().unwrap();
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        if choice.trim() == "1" {
            skip_bootstrap = true;
            println!("✅ Resuming from existing architecture...\n");
        }
    }

    // 4. Resolve spec path (from --spec arg or prompt)
    let spec_path = if skip_bootstrap {
        None
    } else if let Some(s) = spec_arg {
        if !s.is_empty() && std::path::Path::new(s).exists() {
            Some(s.to_string())
        } else if !s.is_empty() {
            eprintln!("⚠️  Spec file '{}' not found.", s);
            None
        } else {
            None
        }
    } else if !skip_bootstrap {
        Some(prompt_spec_file())
    } else {
        None
    };

    let config = KernelConfig {
        temp_dir: PathBuf::from("runtime/temp"),
        thread_count: 1,
        replay_seed: 42,
        feature_flags: BTreeSet::new(),
        axon_config,
    };

    (config, spec_path, skip_bootstrap)
}

fn prompt_config_reuse() {
    println!("\n📦 Existing factory settings (axon_config.json) found.");
    println!("   [1] Fast Resume — use current config and resume");
    println!("   [2] New Spec — keep current config, load new specification");
    println!("   [3] Reconfigure — run full setup wizard again");
    print!("Choice [1/2/3] (default: 1): ");
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    match choice.trim() {
        "3" => {
            println!("♻️  Running setup wizard to reconfigure...\n");
            run_init_wizard();
        }
        "2" => {
            println!("📄 Keeping current config. Will prompt for new specification file.\n");
        }
        _ => {
            println!("✅ Fast Resume: Using current configuration.\n");
        }
    }
}

fn prompt_spec_file() -> String {
    loop {
        print!("Enter specification file path [default: spec.md]: ");
        io::stdout().flush().unwrap();
        let mut sf = String::new();
        io::stdin().read_line(&mut sf).unwrap();
        let sf = if sf.trim().is_empty() { "spec.md".to_string() } else { sf.trim().to_string() };

        if std::path::Path::new(&sf).exists() {
            println!("    -> Found specification: {}", sf);
            return sf;
        }
        if sf == "spec.md" {
            println!("    -> spec.md not found. Skipping file bootstrap; use API to submit spec.");
            return String::new();
        }
        println!("    -> Error: File '{}' not found. Try again or leave empty to skip.", sf);
    }
}

fn run_init_wizard() {
    use std::io::{self, Write};

    println!("==========================================================");
    println!(" AXON Initialization Wizard (Manual Setup)");
    println!("==========================================================\n");
    
    // 1. Language Detection & Selection
    let os_lang = std::env::var("LANG").unwrap_or_else(|_| "en_US".to_string());
    println!("[1] Detected OS Language: {}", os_lang);
    print!("Use this language for LLM Output and UI? (y/N): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let target_lang = if input.trim().eq_ignore_ascii_case("y") {
        if os_lang.contains("ko") { "Korean".to_string() }
        else if os_lang.contains("ja") { "Japanese".to_string() }
        else { "English".to_string() }
    } else {
        println!("Select Language:");
        println!("1. English");
        println!("2. Korean (한국어)");
        println!("3. Japanese (日本語)");
        print!("Choice (1/2/3): ");
        io::stdout().flush().unwrap();
        
        let mut lang_choice = String::new();
        io::stdin().read_line(&mut lang_choice).unwrap();
        match lang_choice.trim() {
            "2" => "Korean".to_string(),
            "3" => "Japanese".to_string(),
            _ => "English".to_string(),
        }
    };
    println!("[*] Global LLM Output & UI Language locked to: {}\n", target_lang);

    // 2. LSP Detection
    println!("[2] Detecting Local LSPs...");
    
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "".to_string());
    let mut found_lsps = Vec::new();
    let mut lsps_json = Vec::new();

    let check_lsp = |binary: &str, custom_path: &str| -> Option<String> {
        if !custom_path.is_empty() && std::path::Path::new(custom_path).exists() {
            Some(custom_path.to_string())
        } else if let Ok(output) = std::process::Command::new("which").arg(binary).output() {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    };

    // Check Rust
    let ra_path = format!("{}/.cargo/bin/rust-analyzer", home_dir);
    if let Some(path) = check_lsp("rust-analyzer", &ra_path) {
        found_lsps.push(format!("Rust (rust-analyzer) -> {}", path));
        lsps_json.push(serde_json::json!({
            "language": "rust",
            "command": path,
            "args": []
        }));
    }
    
    // Check Python
    let pyright_path = format!("{}/.local/share/nvim/mason/bin/pyright-langserver", home_dir);
    if let Some(path) = check_lsp("pyright-langserver", &pyright_path) {
        found_lsps.push(format!("Python (pyright) -> {}", path));
        lsps_json.push(serde_json::json!({
            "language": "python",
            "command": path,
            "args": ["--stdio"]
        }));
    }

    // Check C/C++
    let clangd_path = format!("{}/.local/share/nvim/mason/bin/clangd", home_dir);
    if let Some(path) = check_lsp("clangd", &clangd_path) {
        found_lsps.push(format!("C/C++ (clangd) -> {}", path));
        lsps_json.push(serde_json::json!({
            "language": "c",
            "command": path,
            "args": ["--background-index", "--clang-tidy"]
        }));
    }

    if found_lsps.is_empty() {
        println!("    -> No supported LSPs detected. (AXON will fallback to regex parsing)");
    } else {
        for lsp in &found_lsps {
            println!("    -> Found: {}", lsp);
        }
    }
    
    print!("(Press Enter to continue)");
    io::stdout().flush().unwrap();
    let mut _dummy = String::new();
    io::stdin().read_line(&mut _dummy).unwrap();

    // 3. LLM Configuration (Manual Setup with Auto-Detection)
    println!("\n[3] LLM Persona Binding");

    // Auto-detect API Keys from shell config
    let bashrc = std::fs::read_to_string(format!("{}/.bashrc", home_dir)).unwrap_or_default();
    let zshrc = std::fs::read_to_string(format!("{}/.zshrc", home_dir)).unwrap_or_default();
    let combined_rc = format!("{}\n{}", bashrc, zshrc);

    let mut detected_apis = Vec::new();
    if combined_rc.contains("GEMINI_API_KEY") { detected_apis.push("Google Gemini"); }
    if combined_rc.contains("ANTHROPIC_API_KEY") { detected_apis.push("Anthropic Claude"); }
    if combined_rc.contains("OPENAI_API_KEY") { detected_apis.push("OpenAI"); }

    if !detected_apis.is_empty() {
        println!("    -> Detected Cloud APIs in ~/.bashrc / ~/.zshrc: {}", detected_apis.join(", "));
    } else {
        println!("    -> No Cloud APIs detected in shell configs.");
    }

    let mut shared_local_endpoint: Option<String> = None;

    let configure_agent = |role: &str, detected_apis: &[&str], shared_endpoint: &mut Option<String>| -> serde_json::Value {
        println!("Configure LLM for '{}'", role);
        println!("1. Cloud LLM");
        println!("2. Local LLM (e.g., Ollama)");
        print!("Choice (1/2): ");
        io::stdout().flush().unwrap();
        
        let mut llm_type = String::new();
        io::stdin().read_line(&mut llm_type).unwrap();
        
        if llm_type.trim() == "1" {
            let has_gemini = detected_apis.contains(&"Google Gemini");
            let has_claude = detected_apis.contains(&"Anthropic Claude");
            let has_openai = detected_apis.contains(&"OpenAI");

            println!("Select Cloud Provider:");
            println!("1. Google Gemini {}", if has_gemini { "[Detected]" } else { "" });
            println!("2. Anthropic Claude {}", if has_claude { "[Detected]" } else { "" });
            println!("3. OpenAI {}", if has_openai { "[Detected]" } else { "" });
            print!("Choice: ");
            io::stdout().flush().unwrap();
            
            let mut provider_choice = String::new();
            io::stdin().read_line(&mut provider_choice).unwrap();
            let provider = match provider_choice.trim() {
                "2" => "claude",
                "3" => "openai",
                _ => "gemini",
            };

            let model = match provider {
                "gemini" => {
                    println!("Select Gemini Model:");
                    println!("1. gemini-1.5-pro-preview-0409 (Advanced Reasoning)");
                    println!("2. gemini-1.5-flash-latest (Fast/Cheap)");
                    println!("3. gemini-1.0-pro");
                    print!("Choice: ");
                    io::stdout().flush().unwrap();
                    let mut m = String::new();
                    io::stdin().read_line(&mut m).unwrap();
                    match m.trim() {
                        "1" => "gemini-1.5-pro-preview-0409",
                        "2" => "gemini-1.5-flash-latest",
                        "3" => "gemini-1.0-pro",
                        _ => "gemini-1.5-pro-preview-0409",
                    }
                },
                "claude" => {
                    println!("Select Claude Model:");
                    println!("1. claude-3-opus-20240229");
                    println!("2. claude-3-sonnet-20240229");
                    println!("3. claude-3-haiku-20240307");
                    print!("Choice: ");
                    io::stdout().flush().unwrap();
                    let mut m = String::new();
                    io::stdin().read_line(&mut m).unwrap();
                    match m.trim() {
                        "1" => "claude-3-opus-20240229",
                        "2" => "claude-3-sonnet-20240229",
                        "3" => "claude-3-haiku-20240307",
                        _ => "claude-3-opus-20240229",
                    }
                },
                "openai" => {
                    println!("Select OpenAI Model:");
                    println!("1. gpt-4o");
                    println!("2. gpt-4-turbo");
                    println!("3. gpt-3.5-turbo");
                    print!("Choice: ");
                    io::stdout().flush().unwrap();
                    let mut m = String::new();
                    io::stdin().read_line(&mut m).unwrap();
                    match m.trim() {
                        "1" => "gpt-4o",
                        "2" => "gpt-4-turbo",
                        "3" => "gpt-3.5-turbo",
                        _ => "gpt-4o",
                    }
                },
                _ => "default"
            };

            serde_json::json!({
                "runtime": "cloud",
                "provider": provider,
                "endpoint": null,
                "model": model
            })
        } else {
            let endpoint = if let Some(prev_ep) = shared_endpoint {
                prev_ep.clone()
            } else {
                print!("Enter Local Endpoint [default: http://127.0.0.1:11434]: ");
                io::stdout().flush().unwrap();
                let mut ep = String::new();
                io::stdin().read_line(&mut ep).unwrap();
                let new_ep = if ep.trim().is_empty() { "http://127.0.0.1:11434".to_string() } else { ep.trim().trim_end_matches('/').to_string() };
                
                print!("Use this Local Endpoint for all subsequent agents? (Y/n): ");
                io::stdout().flush().unwrap();
                let mut auto_use_str = String::new();
                io::stdin().read_line(&mut auto_use_str).unwrap();
                if !auto_use_str.trim().eq_ignore_ascii_case("n") {
                    *shared_endpoint = Some(new_ep.clone());
                }
                new_ep
            };

            println!("Fetching models from {}...", endpoint);
            let mut available_models = Vec::new();
            if let Ok(output) = std::process::Command::new("curl")
                .arg("-s")
                .arg(format!("{}/api/tags", endpoint))
                .output() 
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = output_str.split("\"name\":\"").collect();
                for part in parts.iter().skip(1) {
                    if let Some(end_idx) = part.find("\"") {
                        available_models.push(part[..end_idx].to_string());
                    }
                }
            }

            let model = if available_models.is_empty() {
                println!("Failed to fetch models from endpoint. Please enter manually.");
                print!("Enter Model Name (e.g., qwen2.5:7b-instruct-q4_K_M): ");
                io::stdout().flush().unwrap();
                let mut m = String::new();
                io::stdin().read_line(&mut m).unwrap();
                if m.trim().is_empty() { "qwen2.5:7b-instruct-q4_K_M".to_string() } else { m.trim().to_string() }
            } else {
                println!("Select Model:");
                for (i, m) in available_models.iter().enumerate() {
                    println!("{}. {}", i + 1, m);
                }
                print!("Choice: ");
                io::stdout().flush().unwrap();
                let mut choice_str = String::new();
                io::stdin().read_line(&mut choice_str).unwrap();
                if let Ok(choice) = choice_str.trim().parse::<usize>() {
                    if choice > 0 && choice <= available_models.len() {
                        available_models[choice - 1].clone()
                    } else {
                        available_models[0].clone()
                    }
                } else {
                    available_models[0].clone()
                }
            };
            
            serde_json::json!({
                "runtime": "local",
                "provider": null,
                "endpoint": endpoint,
                "model": model
            })
        }
    };

    let detected_slice: Vec<&str> = detected_apis.iter().map(|s| *s).collect();

    println!();
    let architect_config = configure_agent("Architect", &detected_slice, &mut shared_local_endpoint);
    
    println!();
    print!("How many Seniors to hire? [default: 1]: ");
    io::stdout().flush().unwrap();
    let mut num_seniors_str = String::new();
    io::stdin().read_line(&mut num_seniors_str).unwrap();
    let num_seniors = num_seniors_str.trim().parse::<usize>().unwrap_or(1).max(1);
    
    let mut senior_configs = Vec::new();
    for i in 1..=num_seniors {
        println!();
        senior_configs.push(configure_agent(&format!("Senior ({}/{})", i, num_seniors), &detected_slice, &mut shared_local_endpoint));
    }

    println!();
    print!("How many Juniors to hire? [default: 3]: ");
    io::stdout().flush().unwrap();
    let mut num_juniors_str = String::new();
    io::stdin().read_line(&mut num_juniors_str).unwrap();
    let num_juniors = num_juniors_str.trim().parse::<usize>().unwrap_or(3).max(1);
    
    let mut junior_configs = Vec::new();
    for i in 1..=num_juniors {
        println!();
        junior_configs.push(configure_agent(&format!("Junior ({}/{})", i, num_juniors), &detected_slice, &mut shared_local_endpoint));
    }
    println!();

    // 4. Scaffolding
    println!("[4] Generating Workspace...");
    std::fs::create_dir_all(".axon/personas").unwrap_or_default();
    
    let config_json = serde_json::json!({
        "locale": if target_lang == "Korean" { "ko_KR" } else if target_lang == "Japanese" { "ja_JP" } else { "en_US" },
        "lsps": lsps_json,
        "agents": {
            "architect": architect_config,
            "seniors": senior_configs,
            "juniors": junior_configs
        },
        "execution": {
            "review_queue_limit": 5,
            "sampling_rate": 0.3,
            "fallback_enabled": true
        }
    });

    std::fs::write("axon_config.json", serde_json::to_string_pretty(&config_json).unwrap()).unwrap_or_default();
    
    println!("    -> Created axon_config.json");
    println!("\nAXON Initialization Complete.\n");
}

