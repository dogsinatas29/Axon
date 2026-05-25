/*
 * AXON - Standard Semantic Diagnostic Layer
 * Copyright (C) 2026 dogsinatas
 */

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LspDiagnostic {
    pub source: String,
    pub severity: LspSeverity,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub code: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum LspSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl LspSeverity {
    pub fn from_num(num: u64) -> Self {
        match num {
            1 => LspSeverity::Error,
            2 => LspSeverity::Warning,
            3 => LspSeverity::Information,
            _ => LspSeverity::Hint,
        }
    }
}
