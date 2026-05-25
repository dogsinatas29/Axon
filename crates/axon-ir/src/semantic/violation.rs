use crate::schema::Language;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SemanticViolation {
    ForbiddenVocabulary {
        term: String,
        language: Language,
    },
    CrossLanguageContamination {
        detected: String,
        expected: Language,
    },
    InvalidBuildSystem {
        system: String,
        language: Language,
    },
    InvalidTaskVocabulary {
        task: String,
        language: Language,
    },
}

impl std::fmt::Display for SemanticViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticViolation::ForbiddenVocabulary { term, language } => {
                write!(f, "Forbidden vocabulary '{}' for language {:?}", term, language)
            }
            SemanticViolation::CrossLanguageContamination { detected, expected } => {
                write!(f, "Cross-language contamination: detected '{}', expected {:?}", detected, expected)
            }
            SemanticViolation::InvalidBuildSystem { system, language } => {
                write!(f, "Invalid build system '{}' for language {:?}", system, language)
            }
            SemanticViolation::InvalidTaskVocabulary { task, language } => {
                write!(f, "Invalid task vocabulary '{}' for language {:?}", task, language)
            }
        }
    }
}

impl std::error::Error for SemanticViolation {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticCategory {
    Syntax,
    TypeMismatch,
    MissingImport,
    ForbiddenPattern,
    FfiViolation,
    LanguageContamination,
    ContractViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDiagnostic {
    pub source: String,
    pub severity: DiagnosticSeverity,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub code: Option<String>,
    pub message: String,
    pub category: DiagnosticCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityLock {
    pub supports_diagnostics: bool,
    pub supports_semantic_tokens: bool,
    pub supports_code_action: bool,
}