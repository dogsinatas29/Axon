/*
 * AXON - Clangd Connector
 * Copyright (C) 2026 dogsinatas
 */

use std::path::Path;
use crate::intelligence::lsp::session::LspSession;

pub struct ClangdConnector;

impl ClangdConnector {
    pub async fn spawn_session(workspace_root: &Path) -> Result<LspSession, String> {
        let mut cmd = "clangd".to_string();
        let mut args = Vec::new();

        // Check if nvim mason clangd exists as fallback
        let mason_clangd = "/home/dogsinatas/.local/share/nvim/mason/bin/clangd";
        if std::path::Path::new(mason_clangd).exists() {
            cmd = mason_clangd.to_string();
        }

        // Load custom axon_lsp.json if available
        if std::path::Path::new("axon_lsp.json").exists() {
            if let Ok(content) = std::fs::read_to_string("axon_lsp.json") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(c_cfg) = val.get("c").or_else(|| val.get("cpp")) {
                        if c_cfg.get("enabled").and_then(|e| e.as_bool()).unwrap_or(false) {
                            if let Some(c) = c_cfg.get("command").and_then(|c| c.as_str()) {
                                cmd = c.to_string();
                            }
                            if let Some(arr) = c_cfg.get("args").and_then(|a| a.as_array()) {
                                args = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                            }
                        }
                    }
                }
            }
        }

        // v0.0.31.40: [LSP_GATEKEEPER] Point clangd to compile_commands.json
        let abs_workspace = workspace_root.canonicalize()
            .map_err(|e| format!("Invalid workspace root: {}", e))?;
        args.push(format!("--compile-commands-dir={}", abs_workspace.display()));

        // Default args for clangd if not configured
        if args.len() == 1 {
            // Only --compile-commands-dir was added, add defaults
            args.push("--background-index".to_string());
            args.push("--clang-tidy".to_string());
            args.push("--header-insertion=never".to_string());
        }

        LspSession::spawn(&cmd, &args, workspace_root, "c").await
    }
}
