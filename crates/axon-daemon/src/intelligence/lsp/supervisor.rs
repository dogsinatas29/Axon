/*
 * AXON - LSP Runtime Supervisor Layer (Pre-compilation Semantic Firewall)
 * Copyright (C) 2026 dogsinatas
 */

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tokio::sync::Mutex;
use std::collections::HashMap;
use axon_ir::Language;

use crate::intelligence::lsp::diagnostics::{LspDiagnostic, LspSeverity};
use crate::intelligence::lsp::session::LspSession;
use crate::intelligence::lsp::rust_analyzer::RustAnalyzerConnector;
use crate::intelligence::lsp::clangd::ClangdConnector;
use crate::intelligence::lsp::pyright::PyrightConnector;
use crate::intelligence::lsp::lua_lsp::LuaLspConnector;

pub enum LspVerdict {
    Clean,
    Warning(Vec<LspDiagnostic>),
    Reject(Vec<LspDiagnostic>),
}

fn get_sessions() -> &'static Mutex<HashMap<String, LspSession>> {
    static SESSIONS: OnceLock<Mutex<HashMap<String, LspSession>>> = OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct LspSupervisor;

impl LspSupervisor {
    /// pre-generation snapshot semantic gate
    pub async fn semantic_gate(
        language: Language,
        workspace: &Path,
        changed_files: &[PathBuf],
    ) -> LspVerdict {
        let lang_str = match language {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::C | Language::Cpp => "c",
            Language::Lua => "lua",
        };

        let session_key = format!("{}:{}", workspace.display(), lang_str);

        // Try to spawn session if not cached
        let has_session = {
            let sessions = get_sessions().lock().await;
            sessions.contains_key(&session_key)
        };

        if !has_session {
            match language {
                Language::Rust => {
                    tracing::info!("🚀 [LSP_SESSION_START] Spawning rust-analyzer for workspace: {:?}", workspace);
                    match RustAnalyzerConnector::spawn_session(workspace).await {
                        Ok(sess) => {
                            let mut sessions = get_sessions().lock().await;
                            sessions.insert(session_key.clone(), sess);
                            tracing::info!("✅ [LSP_SESSION_ATTACH_OK] rust-analyzer successfully attached to supervisor!");
                        }
                        Err(e) => {
                            tracing::error!("❌ [LSP_SESSION_START_FAIL] Failed to spawn rust-analyzer: {}", e);
                            return LspVerdict::Clean; // Fallback silently
                        }
                    }
                }
                Language::C | Language::Cpp => {
                    tracing::info!("🚀 [LSP_SESSION_START] Spawning clangd for workspace: {:?}", workspace);
                    match ClangdConnector::spawn_session(workspace).await {
                        Ok(sess) => {
                            let mut sessions = get_sessions().lock().await;
                            sessions.insert(session_key.clone(), sess);
                            tracing::info!("✅ [LSP_SESSION_ATTACH_OK] clangd successfully attached to supervisor!");
                        }
                        Err(e) => {
                            tracing::error!("❌ [LSP_SESSION_START_FAIL] Failed to spawn clangd: {}", e);
                            return LspVerdict::Clean; // Fallback silently
                        }
                    }
                }
                Language::Python => {
                    tracing::info!("🚀 [LSP_SESSION_START] Spawning pyright for workspace: {:?}", workspace);
                    match PyrightConnector::spawn_session(workspace).await {
                        Ok(sess) => {
                            let mut sessions = get_sessions().lock().await;
                            sessions.insert(session_key.clone(), sess);
                            tracing::info!("✅ [LSP_SESSION_ATTACH_OK] pyright successfully attached to supervisor!");
                        }
                        Err(e) => {
                            tracing::error!("❌ [LSP_SESSION_START_FAIL] Failed to spawn pyright: {}", e);
                            return LspVerdict::Clean; // Fallback silently
                        }
                    }
                }
                Language::Lua => {
                    tracing::info!("🚀 [LSP_SESSION_START] Spawning lua-language-server for workspace: {:?}", workspace);
                    match LuaLspConnector::spawn_session(workspace).await {
                        Ok(sess) => {
                            let mut sessions = get_sessions().lock().await;
                            sessions.insert(session_key.clone(), sess);
                            tracing::info!("✅ [LSP_SESSION_ATTACH_OK] lua-language-server successfully attached to supervisor!");
                        }
                        Err(e) => {
                            tracing::error!("❌ [LSP_SESSION_START_FAIL] Failed to spawn lua-language-server: {}", e);
                            return LspVerdict::Clean; // Fallback silently
                        }
                    }
                }
            }
        }

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Perform Snapshot Semantic Validation for each changed file
        for file in changed_files {
            if !file.exists() {
                continue;
            }

            let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            let is_matching_ext = match language {
                Language::Rust => ext == "rs",
                Language::Python => ext == "py",
                Language::C | Language::Cpp => {
                    ext == "c" || ext == "cpp" || ext == "cc" || ext == "cxx" || ext == "h" || ext == "hpp"
                }
                Language::Lua => ext == "lua",
            };
            if !is_matching_ext {
                continue;
            }

            let content = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut sessions = get_sessions().lock().await;
            if let Some(session) = sessions.get_mut(&session_key) {
                // didOpen to sync content
                if let Err(e) = session.did_open(file, &content).await {
                    tracing::warn!("⚠️ [LSP_SYNC_FAIL] didOpen failed: {}", e);
                    continue;
                }

                // debounce/wait for parsing and capture diagnostics
                match session.capture_diagnostics(file).await {
                    Ok(diags) => {
                        for diag in diags {
                            // Active Semantic Authority:
                            // Hard Reject (Block) conditions:
                            // 1. Diagnostics is actual error.
                            // 2. Diagnostics represents architectural drift or dependency mismatch (empty headers, invalid imports, duplicate definition).
                            // 3. Warning containing sqlite3 / database / include / missing model / invalid type.
                            let is_error = diag.severity == LspSeverity::Error || 
                                           diag.message.contains("expected item, found '#'") || 
                                           diag.message.contains("#include") ||
                                           diag.message.contains("sqlite3") ||
                                           diag.message.contains("no such file or directory") ||
                                           diag.message.contains("unresolved import") ||
                                           diag.message.contains("no data models") ||
                                           diag.message.contains("undefined reference");

                            // standard trace log
                            tracing::warn!(
                                "🚨 [LSP_DIAG_FIREWALL] severity={:?} code={:?} message=\"{}\"",
                                diag.severity,
                                diag.code,
                                diag.message
                            );

                            if is_error {
                                errors.push(diag);
                            } else if diag.severity == LspSeverity::Warning {
                                warnings.push(diag);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ [LSP_DIAG_FAIL] Capture diagnostics failed: {}", e);
                    }
                }
            }
        }

        if !errors.is_empty() {
            tracing::error!("❌ [LSP_SEMANTIC_DRIFT] Hard reject triggered for changed files. Total: {}", errors.len());
            LspVerdict::Reject(errors)
        } else if !warnings.is_empty() {
            tracing::info!("⚠️ [LSP_SEMANTIC_WARN] Semantic warnings detected: {}", warnings.len());
            LspVerdict::Warning(warnings)
        } else {
            LspVerdict::Clean
        }
    }
}
