/*
 * AXON - Stateful LSP Session Handler
 * Copyright (C) 2026 dogsinatas
 */

use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use serde_json::Value;
use crate::intelligence::lsp::diagnostics::{LspDiagnostic, LspSeverity};

pub struct WorkspaceManager;

impl WorkspaceManager {
    pub async fn bootstrap_workspace(workspace_root: &Path, lang: &str) -> std::io::Result<()> {
        match lang {
            "c" | "cpp" => {
                let cmake_path = workspace_root.join("CMakeLists.txt");
                if cmake_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&cmake_path) {
                        if !content.contains("CMAKE_EXPORT_COMPILE_COMMANDS") {
                            let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                            lines.insert(0, "set(CMAKE_EXPORT_COMPILE_COMMANDS ON)".to_string());
                            let _ = std::fs::write(&cmake_path, lines.join("\n"));
                            tracing::info!("Injected CMAKE_EXPORT_COMPILE_COMMANDS into CMakeLists.txt");
                        }
                    }
                }
            }
            "rust" => {
                let cargo_toml = workspace_root.join("Cargo.toml");
                if !cargo_toml.exists() {
                    let default_cargo = "[package]\nname = \"axon_sandbox\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n";
                    let _ = std::fs::write(&cargo_toml, default_cargo);
                }
                let src_dir = workspace_root.join("src");
                if !src_dir.exists() {
                    let _ = std::fs::create_dir_all(&src_dir);
                }
            }
            "python" => {
                let pyproject = workspace_root.join("pyproject.toml");
                if !pyproject.exists() {
                    let default_pyproject = "[tool.pyright]\ninclude = [\".\"]\n";
                    let _ = std::fs::write(&pyproject, default_pyproject);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct LspSession {
    child: Child,
    language: String,
}

impl LspSession {
    pub async fn spawn(
        cmd: &str,
        args: &[String],
        workspace_root: &Path,
        language: &str,
    ) -> Result<Self, String> {
        let abs_workspace = workspace_root.canonicalize()
            .map_err(|e| format!("Invalid workspace root canonicalization: {}", e))?;

        let _ = WorkspaceManager::bootstrap_workspace(&abs_workspace, language).await;

        let mut child = Command::new(cmd)
            .args(args)
            .current_dir(&abs_workspace)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn LSP process {}: {}", cmd, e))?;

        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin pipe")?;
        let stdout = child.stdout.as_mut().ok_or("Failed to open stdout pipe")?;

        let mut reader = BufReader::new(stdout);
        let request_id = 1;

        let root_uri = format!("file://{}", abs_workspace.display());
        let init_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootPath": abs_workspace.display().to_string(),
                "rootUri": root_uri,
                "capabilities": {
                    "textDocument": {
                        "publishDiagnostics": {}
                    }
                }
            }
        });

        let req_str = serde_json::to_string(&init_req).unwrap();
        let payload = format!("Content-Length: {}\r\n\r\n{}", req_str.len(), req_str);
        stdin.write_all(payload.as_bytes()).await
            .map_err(|e| format!("Handshake write failed: {}", e))?;
        stdin.flush().await.map_err(|e| format!("Handshake flush failed: {}", e))?;

        let mut line = String::new();
        let mut content_len = 0;
        loop {
            line.clear();
            reader.read_line(&mut line).await
                .map_err(|e| format!("Read error during handshake: {}", e))?;
            if line.trim().is_empty() {
                break;
            }
            if line.to_lowercase().starts_with("content-length:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    content_len = parts[1].trim().parse::<usize>().unwrap_or(0);
                }
            }
        }

        if content_len == 0 {
            return Err("Zero Content-Length response during handshake".to_string());
        }

        let mut body_bytes = vec![0u8; content_len];
        reader.read_exact(&mut body_bytes).await
            .map_err(|e| format!("Failed to read body bytes: {}", e))?;

        let initialized_notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });
        let notif_str = serde_json::to_string(&initialized_notif).unwrap();
        let notif_payload = format!("Content-Length: {}\r\n\r\n{}", notif_str.len(), notif_str);
        stdin.write_all(notif_payload.as_bytes()).await
            .map_err(|e| format!("Failed to send initialized: {}", e))?;
        stdin.flush().await.map_err(|e| format!("Failed to flush initialized: {}", e))?;

        tracing::info!("🧠 [LSP_SESSION_ATTACH] {} attached to workspace", language);

        Ok(Self {
            child,
            language: language.to_string(),
        })
    }

    pub async fn did_open(&mut self, file_path: &Path, content: &str) -> Result<(), String> {
        let stdin = self.child.stdin.as_mut().ok_or("Stdin closed")?;
        let abs_file = file_path.canonicalize()
            .unwrap_or_else(|_| file_path.to_path_buf());
        let uri = format!("file://{}", abs_file.display());
        
        let lang_id = match self.language.as_str() {
            "rust" => "rust",
            "python" => "python",
            "c" => "c",
            "cpp" => "cpp",
            _ => "rust",
        };

        let did_open_params = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": lang_id,
                    "version": 1,
                    "text": content
                }
            }
        });

        let payload_str = serde_json::to_string(&did_open_params).unwrap();
        let payload = format!("Content-Length: {}\r\n\r\n{}", payload_str.len(), payload_str);
        stdin.write_all(payload.as_bytes()).await
            .map_err(|e| format!("didOpen write failed: {}", e))?;
        stdin.flush().await.map_err(|e| format!("didOpen flush failed: {}", e))?;

        tracing::debug!("📂 [LSP_SYNC] didOpen sent for: {:?}", file_path.file_name());
        Ok(())
    }

    pub async fn capture_diagnostics(
        &mut self,
        file_path: &Path,
    ) -> Result<Vec<LspDiagnostic>, String> {
        let stdout = self.child.stdout.as_mut().ok_or("Stdout closed")?;
        let mut reader = BufReader::new(stdout);

        let mut line = String::new();
        let mut content_len = 0;

        let read_future = async {
            loop {
                line.clear();
                if reader.read_line(&mut line).await.is_err() {
                    return None;
                }
                if line.trim().is_empty() {
                    break;
                }
                if line.to_lowercase().starts_with("content-length:") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        content_len = parts[1].trim().parse::<usize>().unwrap_or(0);
                    }
                }
            }
            if content_len == 0 {
                return None;
            }
            let mut body_bytes = vec![0u8; content_len];
            if reader.read_exact(&mut body_bytes).await.is_err() {
                return None;
            }
            serde_json::from_slice::<Value>(&body_bytes).ok()
        };

        // Snapshot Semantic Validation: Allow up to 400ms to wait for analysis
        let lsp_msg = match tokio::time::timeout(tokio::time::Duration::from_millis(400), read_future).await {
            Ok(Some(msg)) => msg,
            _ => return Ok(vec![]), // Timeout or read failed
        };

        let mut lsp_diagnostics = Vec::new();
        if let Some(method) = lsp_msg.get("method").and_then(|v| v.as_str()) {
            if method == "textDocument/publishDiagnostics" {
                if let Some(params) = lsp_msg.get("params") {
                    if let Some(diags) = params.get("diagnostics").and_then(|v| v.as_array()) {
                        for diag in diags {
                            let message = diag.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let severity_num = diag.get("severity").and_then(|v| v.as_u64()).unwrap_or(1);
                            let severity = LspSeverity::from_num(severity_num);

                            let start_line = diag.pointer("/range/start/line").and_then(|v| v.as_u64()).unwrap_or(0) as usize + 1;
                            let start_col = diag.pointer("/range/start/character").and_then(|v| v.as_u64()).unwrap_or(0) as usize + 1;

                            let fname = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("file").to_string();
                            
                            let lsp_diag = LspDiagnostic {
                                source: match self.language.as_str() {
                                    "rust" => "rust-analyzer",
                                    "python" => "pyright",
                                    _ => "clangd",
                                }.to_string(),
                                severity,
                                file: fname,
                                line: start_line,
                                column: start_col,
                                code: diag.get("code").and_then(|c| c.as_str()).map(|s| s.to_string()),
                                message,
                            };
                            lsp_diagnostics.push(lsp_diag);
                        }
                    }
                }
            }
        }

        Ok(lsp_diagnostics)
    }
}
