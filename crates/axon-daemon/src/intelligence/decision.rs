use serde::{Deserialize, Serialize};
// # encoding: utf-8
use axon_core::validator::{SemanticClosure, SemanticDecision};
use axon_core::PatchContract;
use std::path::Path;
use axon_ir::ProjectIR;
use crate::intelligence::language_contract;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureCause {
    MissingHeader,
    MissingSymbol,
    UndefinedReference,
    SegFault,
    SyntaxError,
    ConstitutionalViolation,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Stage {
    SpecAnalysis,
    Skeleton,
    HeaderGen,
    ImplGen,
    IntegratorGen,
    Build,
    BuildRepair,
    Runtime,
    Sync,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryScope {
    Skeleton,
    Stage,
    HeaderOnly,
    ImplementationOnly,
    BuildRepair,
    Full,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
}

pub struct StageRouter;

impl StageRouter {
    pub fn next_stage(current: &Stage) -> Stage {
        match current {
            Stage::SpecAnalysis => Stage::Skeleton,
            Stage::Skeleton => Stage::HeaderGen,
            Stage::HeaderGen => Stage::ImplGen,
            Stage::ImplGen => Stage::IntegratorGen,
            Stage::IntegratorGen => Stage::Build,
            Stage::Build => Stage::Runtime,
            Stage::Runtime => Stage::Sync,
            Stage::Sync => Stage::Complete,
            Stage::Complete => Stage::Complete,
            Stage::BuildRepair => Stage::Build,
        }
    }

    pub fn route_retry(scope: &RetryScope, current: &Stage) -> Stage {
        match scope {
            RetryScope::Skeleton => Stage::Skeleton,
            RetryScope::Stage => *current,
            RetryScope::HeaderOnly => Stage::HeaderGen,
            RetryScope::ImplementationOnly => Stage::ImplGen,
            RetryScope::BuildRepair => Stage::BuildRepair,
            RetryScope::Full => Stage::Skeleton,
            RetryScope::None => Stage::Complete,
        }
    }
}

fn load_current_expected_lang() -> String {
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let const_path = path.join("immutable_constraints.json");
                if const_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(const_path) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(lang) = val["language"].as_str() {
                                return lang.to_lowercase();
                            }
                        }
                    }
                }
            }
        }
    }
    "c".to_string()
}

pub fn infer_cause(diag: &Diagnostic) -> FailureCause {
    let msg = diag.message.to_lowercase();
    
    // 1. Rust 특정 키워드 감지 시 Rust 모듈 위임
    if msg.contains("use of undeclared crate") || msg.contains("unresolved import") || msg.contains("cargo") {
        return language_contract::rust::infer_cause(&diag.message);
    }
    
    // 2. Python 특정 에러 감지 시 Python 모듈 위임
    if msg.contains("modulenotfounderror") || msg.contains("nameerror") || msg.contains("syntaxerror") {
        return language_contract::python::infer_cause(&diag.message);
    }
    
    // 3. 현재 활성화된 프로젝트 언어를 기반으로 타겟 위임
    let lang = load_current_expected_lang();
    match lang.as_str() {
        "rust" => language_contract::rust::infer_cause(&diag.message),
        "python" => language_contract::python::infer_cause(&diag.message),
        _ => language_contract::c::infer_cause(&diag.message),
    }
}

pub fn determine_scope(cause: &FailureCause) -> RetryScope {
    match cause {
        FailureCause::ConstitutionalViolation => RetryScope::Skeleton,
        FailureCause::MissingHeader | FailureCause::MissingSymbol => RetryScope::HeaderOnly,
        FailureCause::UndefinedReference => RetryScope::ImplementationOnly,
        FailureCause::SyntaxError => RetryScope::Stage,
        FailureCause::SegFault => RetryScope::BuildRepair,
        _ => RetryScope::Full,
    }
}

pub fn generate_hint(cause: &FailureCause) -> &'static str {
    let lang = load_current_expected_lang();
    match lang.as_str() {
        "rust" => language_contract::rust::generate_hint(cause),
        "python" => language_contract::python::generate_hint(cause),
        _ => language_contract::c::generate_hint(cause),
    }
}

pub struct PromptContext {
    pub project_root: String,
    pub stage: Stage,
    pub target: String,
    pub files: Vec<(String, String)>,
    pub cause: Option<FailureCause>,
}

pub struct PromptBuilder {
    pub project_root: String,
}

impl PromptBuilder {
    pub fn new(project_root: String) -> Self {
        Self { project_root }
    }

    pub fn build(ctx: &PromptContext) -> String {
        let mut prompt = String::new();

        // v0.0.30: [CRITICAL_CONTRACT] Injection
        if let Some(sealed) = load_sealed_ir(&ctx.project_root) {
            let current_ir = load_project_ir(&ctx.project_root);
            let current_hash = current_ir.map(|ir| calculate_ir_hash(&ir)).unwrap_or_default();
            
            let mut valid_decisions = Vec::new();
            for decision in &sealed.decisions {
                if decision.ir_hash == current_hash {
                    valid_decisions.push(decision);
                }
            }

            if !valid_decisions.is_empty() {
                prompt.push_str("\n[CRITICAL_CONTRACT]\n");
                prompt.push_str("The following semantic decisions are BINDING and IMMUTABLE:\n\n");
                for decision in valid_decisions {
                    prompt.push_str(&format!("- {}: {:?} ({})\n", decision.risk_id, decision.action, decision.comment));
                }
                prompt.push_str("\nViolation of these contracts will result in immediate rejection.\n\n");
            }
        }

        // Load language from immutable_constraints.json if available
        let expected_lang = {
            let path = Path::new(&ctx.project_root).join("immutable_constraints.json");
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        val["language"].as_str().unwrap_or("c").to_lowercase()
                    } else {
                        "c".to_string()
                    }
                } else {
                    "c".to_string()
                }
            } else {
                "c".to_string()
            }
        };

        prompt.push_str(&Self::base(&ctx.stage, &expected_lang, &Vec::<String>::new()));

        if let Some(ref cause) = ctx.cause {
            prompt.push_str("\n[ERROR CONTEXT]\n");
            prompt.push_str(Self::inject_cause(&expected_lang, cause));
        }

        prompt.push_str("\n[INPUT]\n");
        for (name, content) in &ctx.files {
            prompt.push_str(&format!("--- {} ---\n{}\n\n", name, content));
        }

        prompt.push_str(&format!("\nTARGET FILE: {}\n", ctx.target));
        prompt.push_str("\nGenerate the code NOW:");

        prompt
    }

    fn base(stage: &Stage, expected_lang: &str, dependencies: &[String]) -> String {
        match expected_lang {
            "rust" => language_contract::rust::base_prompt(stage),
            "python" => language_contract::python::base_prompt(stage),
            _ => language_contract::c::base_prompt(stage, dependencies),
        }
    }

    fn inject_cause(expected_lang: &str, cause: &FailureCause) -> &'static str {
        match expected_lang {
            "rust" => language_contract::rust::inject_cause(cause),
            "python" => language_contract::python::inject_cause(cause),
            _ => language_contract::c::inject_cause(cause),
        }
    }
}

pub fn load_project_ir(project_root: &str) -> Option<ProjectIR> {
    let path = Path::new(project_root).join("contracts/project_ir.json");
    if path.exists() {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

pub fn save_project_ir(project_root: &str, ir: &ProjectIR) -> std::io::Result<()> {
    // 1. Save JSON
    let path = Path::new(project_root).join("contracts/project_ir.json");
    let content = serde_json::to_string_pretty(ir).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(&path, content)?;

    // 2. Sync Architecture Markdown (LOCKED status)
    crate::intelligence::writer::ArchitectureWriter::write_architecture(project_root, ir)?;

    Ok(())
}

pub fn load_sealed_ir(project_root: &str) -> Option<SemanticClosure> {
    let path = Path::new(project_root).join("contracts/sealed_ir.json");
    if path.exists() {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

pub fn save_sealed_ir(project_root: &str, closure: &SemanticClosure) -> std::io::Result<()> {
    let path = Path::new(project_root).join("contracts/sealed_ir.json");
    let content = serde_json::to_string_pretty(closure).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(&path, content)
}

pub fn calculate_ir_hash(ir: &ProjectIR) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    let json = serde_json::to_string(ir).unwrap_or_default();
    json.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn archive_decision(project_root: &str, decision: &SemanticDecision) -> std::io::Result<()> {
    let history_dir = std::path::Path::new(project_root).join("history");
    if !history_dir.exists() {
        std::fs::create_dir_all(&history_dir)?;
    }
    let history_path = history_dir.join("semantic_history.jsonl");
    
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(history_path)?;
    
    // v0.0.30: Deep Analytics for Governance Drift
    let entry = serde_json::json!({
        "timestamp": chrono::Local::now().to_rfc3339(),
        "decision": decision,
        "metrics": {
             "ir_hash": decision.ir_hash,
             "is_auto": decision.comment.contains("[JURISPRUDENCE]"), // 자동 판결 여부 판별
        }
    });
    
    use std::io::Write;
    writeln!(file, "{}", entry.to_string())?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairScope {
    pub file: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseInfo {
    pub r#type: String,
    pub symbol: Option<String>,
    pub required_include: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredRepairContract {
    pub repair_scope: RepairScope,
    pub root_cause: RootCauseInfo,
    pub hard_constraints: Vec<String>,
    pub forbidden_patterns: Vec<String>,
    pub allowed_changes: Vec<String>,
}

impl StructuredRepairContract {
    pub fn new(file: &str, raw_errors: &[String]) -> Self {
        let primary_error = raw_errors.first().cloned().unwrap_or_else(|| "Unknown compiler error".to_string());
        
        let diag = Diagnostic {
            code: "PRIMARY_FAIL".to_string(),
            message: primary_error.clone(),
        };
        let cause = infer_cause(&diag);
        let cause_type = match cause {
            FailureCause::MissingHeader => "missing_header",
            FailureCause::MissingSymbol => "missing_symbol",
            FailureCause::UndefinedReference => "undefined_reference",
            FailureCause::SegFault => "segfault",
            FailureCause::SyntaxError => "syntax_error",
            FailureCause::ConstitutionalViolation => "constitutional_violation",
            _ => "unknown",
        };

        let mut symbol = None;
        let mut required_include = None;
        
        let error_lower = primary_error.to_lowercase();
        if error_lower.contains("sqlite3") {
            symbol = Some("sqlite3".to_string());
            required_include = Some("<sqlite3.h>".to_string());
        } else if error_lower.contains("curl") {
            symbol = Some("curl".to_string());
            required_include = Some("<curl/curl.h>".to_string());
        } else if error_lower.contains("openssl") || error_lower.contains("ssl") {
            symbol = Some("openssl".to_string());
            required_include = Some("<openssl/ssl.h>".to_string());
        } else if error_lower.contains("cjson") || error_lower.contains("json") {
            symbol = Some("json".to_string());
            required_include = Some("<cjson/cjson.h>".to_string());
        } else if error_lower.contains("zlib") || error_lower.contains("deflate") {
            symbol = Some("zlib".to_string());
            required_include = Some("<zlib.h>".to_string());
        } else if error_lower.contains("pthread") {
            symbol = Some("pthread".to_string());
            required_include = Some("<pthread.h>".to_string());
        }

        let mut hard_constraints = vec![
            "MUST preserve existing public API signatures".to_string(),
        ];
        let mut forbidden_patterns = Vec::new();
        let mut allowed_changes = vec![
            "targeted bug fix".to_string(),
        ];

        if file.ends_with(".c") || file.ends_with(".cpp") {
            hard_constraints.push("MUST compile under C99 or later".to_string());
            forbidden_patterns.push("implicit function declaration".to_string());
            forbidden_patterns.push("undeclared identifier".to_string());
        } else if file.ends_with(".h") || file.ends_with(".hpp") {
            hard_constraints.push("MUST NOT include function bodies (declarations only)".to_string());
            hard_constraints.push("MUST include proper header guards".to_string());
        } else if file.ends_with(".rs") {
            hard_constraints.push("MUST follow strict safe Rust borrow check rules".to_string());
        }

        if cause == FailureCause::MissingHeader {
            if let Some(ref inc) = required_include {
                hard_constraints.push(format!("MUST include {}", inc));
                allowed_changes.push("include section".to_string());
            }
        } else if cause == FailureCause::MissingSymbol {
            hard_constraints.push("MUST declare the missing symbol or function prototype before use".to_string());
        } else if cause == FailureCause::UndefinedReference {
            hard_constraints.push("MUST provide the implementation/body for the missing symbol".to_string());
        } else if cause == FailureCause::SyntaxError {
            hard_constraints.push("MUST correct unbalanced braces, parentheses, or missing semicolons".to_string());
        }

        if raw_errors.len() > 1 {
            let ignored_count = raw_errors.len() - 1;
            hard_constraints.push(format!("Focus ONLY on the root cause error. Ignored {} cascade compiler errors.", ignored_count));
        }

        Self {
            repair_scope: RepairScope {
                file: file.to_string(),
                mode: "patch_only".to_string(),
            },
            root_cause: RootCauseInfo {
                r#type: cause_type.to_string(),
                symbol,
                required_include,
                message: primary_error,
            },
            hard_constraints,
            forbidden_patterns,
            allowed_changes,
        }
    }

    pub fn to_patch_contract(&self) -> PatchContract {
        PatchContract {
            target_file: self.repair_scope.file.clone(),
            symbol: self.root_cause.symbol.clone(),
            error_line: None,
            error_message: self.root_cause.message.clone(),
            hard_constraints: self.hard_constraints.clone(),
            forbidden_patterns: self.forbidden_patterns.clone(),
            allowed_changes: self.allowed_changes.clone(),
            allowed_regions: vec![],
        }
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

