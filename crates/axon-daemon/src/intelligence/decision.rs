use serde::{Deserialize, Serialize};
// # encoding: utf-8
use axon_core::validator::{SemanticClosure, SemanticDecision};
use std::path::Path;
use axon_ir::ProjectIR;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureCause {
    MissingHeader,
    MissingSymbol,
    UndefinedReference,
    SegFault,
    SyntaxError,
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

pub fn infer_cause(diag: &Diagnostic) -> FailureCause {
    let msg = diag.message.to_lowercase();
    if msg.contains("no such file or directory") || msg.contains("missing header") {
        FailureCause::MissingHeader
    } else if msg.contains("undeclared") || msg.contains("not declared") {
        FailureCause::MissingSymbol
    } else if msg.contains("undefined reference") {
        FailureCause::UndefinedReference
    } else if msg.contains("segmentation fault") || msg.contains("segfault") {
        FailureCause::SegFault
    } else if msg.contains("syntax error") || msg.contains("expected") {
        FailureCause::SyntaxError
    } else {
        FailureCause::Unknown
    }
}

pub fn determine_scope(cause: &FailureCause) -> RetryScope {
    match cause {
        FailureCause::MissingHeader | FailureCause::MissingSymbol => RetryScope::HeaderOnly,
        FailureCause::UndefinedReference => RetryScope::ImplementationOnly,
        FailureCause::SyntaxError => RetryScope::Stage,
        FailureCause::SegFault => RetryScope::BuildRepair,
        _ => RetryScope::Full,
    }
}

pub fn generate_hint(cause: &FailureCause) -> &'static str {
    match cause {
        FailureCause::MissingHeader => "Focus on generating the header file first. Check include paths.",
        FailureCause::MissingSymbol => "Declare the missing symbol in the corresponding header.",
        FailureCause::UndefinedReference => "Provide the implementation body for the declared function.",
        FailureCause::SegFault => "Add null pointer checks and verify memory allocation.",
        FailureCause::SyntaxError => "Fix semicolons, braces, or type mismatches reported by GCC.",
        _ => "Analyze the diagnostic logs and apply minimal targeted fixes.",
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

        prompt.push_str(&Self::base(&ctx.stage));

        if let Some(ref cause) = ctx.cause {
            prompt.push_str("\n[ERROR CONTEXT]\n");
            prompt.push_str(Self::inject_cause(cause));
        }

        prompt.push_str("\n[INPUT]\n");
        for (name, content) in &ctx.files {
            prompt.push_str(&format!("--- {} ---\n{}\n\n", name, content));
        }

        prompt.push_str(&format!("\nTARGET FILE: {}\n", ctx.target));
        prompt.push_str("\nGenerate the code NOW:");

        prompt
    }

    fn base(stage: &Stage) -> String {
        match stage {
            Stage::HeaderGen => 
                "[ROLE]\nYou generate a C/C++ header file.\n\n\
                 [TASK]\nDeclare public interfaces only.\n\n\
                 [CONSTRAINTS]\n- No implementation\n- Use include guards\n- Use simple types\n- Signatures must ending with ';'\n".to_string(),
            Stage::ImplGen => 
                "[ROLE]\nYou implement a C/C++ source file.\n\n\
                 [TASK]\nImplement functions declared in the header.\n\n\
                 [CONSTRAINTS]\n- Must include its own header\n- Do not change signatures from header\n- Keep logic minimal and safe\n".to_string(),
            Stage::Build | Stage::Runtime => 
                "[ROLE]\nYou are fixing a broken C/C++ program.\n\n\
                 [TASK]\nFix the error without rewriting the entire module.\n\n\
                 [CONSTRAINTS]\n- Minimal changes only\n- Do not modify unrelated code\n- Focus on the identified failure cause\n".to_string(),
            _ => "[ROLE]\nAI Agent\n\n[TASK]\nAssist with project.\n".to_string(),
        }
    }

    fn inject_cause(cause: &FailureCause) -> &'static str {
        match cause {
            FailureCause::MissingHeader => 
                "CAUSE: Header file is missing or not found.\n\
                 INSTRUCTION: Generate the required header file first. Ensure it's correctly linked.",
            FailureCause::MissingSymbol => 
                "CAUSE: A symbol (function/variable) is used but not declared.\n\
                 INSTRUCTION: Declare the missing symbol in the header file. Do not touch implementation yet.",
            FailureCause::UndefinedReference => 
                "CAUSE: Function is declared but its definition (body) is missing.\n\
                 INSTRUCTION: Implement the missing function in the .c/.cpp file.",
            FailureCause::SegFault => 
                "CAUSE: Segmentation fault detected (Memory Error).\n\
                 INSTRUCTION: Check for null pointers and array boundaries. Add safety checks.",
            FailureCause::SyntaxError => 
                "CAUSE: Syntax error (missing semicolon or unbalanced braces).\n\
                 INSTRUCTION: Fix the specific syntax error reported in the logs.",
            _ => "CAUSE: Technical discrepancy detected.\nINSTRUCTION: Re-evaluate module structure and fix the error.",
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
