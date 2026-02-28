use crate::config::AxonConfig;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "axon")]
#[command(about = "AXON: The Automated Software Factory CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new AXON project in the current directory
    Init {
        #[arg(short, long, default_value = "New Project")]
        name: String,
        #[arg(short, long, default_value_t = 2)]
        juniors: u32,
    },
    /// Start the AXON factory daemon
    Start,
}

pub fn handle_init(name: String, juniors: u32) -> std::io::Result<()> {
    println!("🚀 Initializing AXON project: {}", name);

    // 1. Create config
    let mut config = AxonConfig::default();
    config.project_name = name;
    config.agents.juniors = juniors;
    config.save()?;
    println!("✅ Created axon_config.json");

    // 2. Capture context and generate ARCHITECTURE_AXON.md
    let arch_path = Path::new("ARCHITECTURE_AXON.md");
    if !arch_path.exists() {
        let mut context = String::new();
        // Simple context capture: find the first .md file that isn't this one or GEMINI.md
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md")
                    && path.file_name().and_then(|s| s.to_str()) != Some("ARCHITECTURE_AXON.md")
                    && path.file_name().and_then(|s| s.to_str()) != Some("GEMINI.md")
                {
                    if let Ok(content) = fs::read_to_string(&path) {
                        context = content;
                        println!("📖 Captured context from {}", path.display());
                        break;
                    }
                }
            }
        }

        let arch_content = format!(
            "# ARCHITECTURE_AXON: {}\n\n## [SNR] 👴 Senior: Initializing from captured context.\n\n{}\n\n## Current Status\n- [ ] System Setup\n",
            config.project_name,
            if context.is_empty() {
                "No initial context found. Please define your architecture here."
            } else {
                &context
            }
        );
        fs::write(arch_path, arch_content)?;
        println!("✅ Generated ARCHITECTURE_AXON.md");
    }

    // 3. Create agent MD placeholders
    fs::write(
        "senior.md",
        "# 👴 Senior Engineer Node\n\nRole: Architecture Review & Approval\n",
    )?;
    for i in 1..=juniors {
        fs::write(
            format!("junior_{}.md", i),
            format!(
                "# 🐣 Junior Coder Node {}\n\nRole: Module Implementation\n",
                i
            ),
        )?;
    }
    println!(
        "✅ Created agent workspaces (senior.md, junior_1..{}.md)",
        juniors
    );

    // 4. Create Nogari lounge
    if !Path::new("노가리.md").exists() {
        fs::write(
            "노가리.md",
            "# 🗨️ AXON Lounge (노가리.md)\n\n**[SYSTEM]:** Factory initialization complete. Lounge is open.\n",
        )?;
        println!("✅ Created 노가리.md");
    }

    println!("\n🎉 AXON workspace is ready! Run `axon start` to begin.");
    Ok(())
}
