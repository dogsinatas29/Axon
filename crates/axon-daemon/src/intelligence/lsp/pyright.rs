/*
 * AXON - Pyright Connector
 * Copyright (C) 2026 dogsinatas
 */

use std::path::Path;
use crate::intelligence::lsp::session::LspSession;

pub struct PyrightConnector;

impl PyrightConnector {
    pub async fn spawn_session(workspace_root: &Path) -> Result<LspSession, String> {
        let mut cmd = "pyright-langserver".to_string();
        let mut args = vec!["--stdio".to_string()];

        // Load custom axon_lsp.json if available
        if std::path::Path::new("axon_lsp.json").exists() {
            if let Ok(content) = std::fs::read_to_string("axon_lsp.json") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(py_cfg) = val.get("python") {
                        if py_cfg.get("enabled").and_then(|e| e.as_bool()).unwrap_or(false) {
                            if let Some(c) = py_cfg.get("command").and_then(|c| c.as_str()) {
                                cmd = c.to_string();
                            }
                            if let Some(arr) = py_cfg.get("args").and_then(|a| a.as_array()) {
                                args = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                            }
                        }
                    }
                }
            }
        }

        LspSession::spawn(&cmd, &args, workspace_root, "python").await
    }
}
