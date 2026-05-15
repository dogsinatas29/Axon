// # encoding: utf-8
use serde::{Deserialize, Serialize};
use axon_ir::ProjectIR;
use axon_core::validator::{SemanticClosure, SemanticDecision};
use std::path::Path;

pub fn load_project_ir(project_root: &str) -> Option<ProjectIR> {
    let path = Path::new(project_root).join("contracts/project_ir.json");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            return serde_json::from_str(&content).ok();
        }
    }
    None
}

pub fn load_sealed_ir(project_root: &str) -> Option<SemanticClosure> {
    let path = Path::new(project_root).join("contracts/sealed_ir.json");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            return serde_json::from_str(&content).ok();
        }
    }
    None
}

pub fn save_sealed_ir(project_root: &str, closure: &SemanticClosure) -> anyhow::Result<()> {
    let path = Path::new(project_root).join("contracts/sealed_ir.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(closure)?;
    std::fs::write(path, content)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Stage {
    SpecAnalysis, // v0.0.29.25: Extract immutable constraints from spec.md
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RetryScope {
    Skeleton,
    Stage,
    HeaderOnly,
    ImplementationOnly,
    BuildRepair,
    Full,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FixStrategy {
    Patch,
    RegenerateFile,
    RetryStage,
    FullReset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationState {
    pub strategy: FixStrategy,
    pub attempts: usize,
    pub last_error_hash: Option<String>,
    pub error_repeat_count: usize,
}

impl Default for EscalationState {
    fn default() -> Self {
        Self {
            strategy: FixStrategy::Patch,
            attempts: 0,
            last_error_hash: None,
            error_repeat_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FailureCause {
    // Header-related
    MissingHeader,
    MissingInclude,
    
    // Symbol-related
    MissingSymbol,
    SignatureMismatch,
    
    // Linker-related
    UndefinedReference,
    MultipleDefinition,
    
    // Logic/Syntax
    SyntaxError,
    TypeError,
    
    // Runtime
    SegFault,
    AssertionFailed,
    
    // Parsing
    JsonError,
    
    // Others
    Unknown,
}

pub struct StageRouter;

impl StageRouter {
    pub fn next_stage(current: &Stage) -> Stage {
        match current {
            Stage::SpecAnalysis => Stage::Skeleton,
            Stage::Skeleton => Stage::HeaderGen,
            Stage::HeaderGen => Stage::ImplGen,
            Stage::ImplGen => Stage::IntegratorGen, // v0.0.28: Top-down logic
            Stage::IntegratorGen => Stage::Build,
            Stage::Build => Stage::Runtime,
            Stage::BuildRepair => Stage::Build,
            Stage::Runtime => Stage::Sync,
            Stage::Sync => Stage::Complete,
            Stage::Complete => Stage::Complete,
        }
    }

    pub fn route_retry(scope: &RetryScope, current: &Stage) -> Stage {
        let target = match scope {
            RetryScope::Skeleton => Stage::Skeleton,
            RetryScope::Stage => current.clone(),
            RetryScope::HeaderOnly => Stage::HeaderGen,
            RetryScope::ImplementationOnly => Stage::ImplGen,
            RetryScope::BuildRepair => Stage::BuildRepair,
            RetryScope::Full => Stage::Skeleton,
            RetryScope::None => current.clone(),
        };

        // v0.0.28: Prevent forward jumps on retry. 
        // If the suggested retry stage is ahead of current stage, stay in current stage.
        let target_rank = match target {
            Stage::SpecAnalysis => 0,
            Stage::Skeleton => 1,
            Stage::HeaderGen => 2,
            Stage::ImplGen => 3,
            Stage::IntegratorGen => 4,
            Stage::Build => 5,
            Stage::BuildRepair => 5,
            Stage::Runtime => 6,
            Stage::Sync => 7,
            Stage::Complete => 8,
        };
        let current_rank = match current {
            Stage::SpecAnalysis => 0,
            Stage::Skeleton => 1,
            Stage::HeaderGen => 2,
            Stage::ImplGen => 3,
            Stage::IntegratorGen => 4,
            Stage::Build => 5,
            Stage::BuildRepair => 5,
            Stage::Runtime => 6,
            Stage::Sync => 7,
            Stage::Complete => 8,
        };

        if target_rank > current_rank {
            current.clone()
        } else {
            target
        }
    }
}

pub fn normalize_log(log: &str) -> String {
    log.to_lowercase()
       .replace("\r", "")
       .replace("\x1b[0m", "")
       .replace("\x1b[31m", "")
       .replace("\x1b[32m", "")
}

pub fn infer_cause(diag: &Diagnostic) -> FailureCause {
    let log = normalize_log(&diag.message);

    if (log.contains("no such file") || log.contains("not found") || log.contains("missing .h headers")) && log.contains(".h") {
        FailureCause::MissingHeader
    } else if log.contains("undefined reference") {
        FailureCause::UndefinedReference
    } else if log.contains("multiple definition") {
        FailureCause::MultipleDefinition
    } else if log.contains("was not declared") || log.contains("unknown type name") {
        FailureCause::MissingSymbol
    } else if log.contains("conflicting types") || log.contains("does not match") {
        FailureCause::SignatureMismatch
    } else if log.contains("expected") && (log.contains(";") || log.contains("declaration")) {
        FailureCause::SyntaxError
    } else if log.contains("invalid conversion") || log.contains("cannot convert") {
        FailureCause::TypeError
    } else if log.contains("segmentation fault") {
        FailureCause::SegFault
    } else if log.contains("assertion") && log.contains("failed") {
        FailureCause::AssertionFailed
    } else if log.contains("json") || log.contains("parse error") || log.contains("expected value") || 
              log.contains("eof while parsing") || log.contains("trailing characters") || log.contains("control character") ||
              log.contains("envelope") || log.contains("stabilization") || log.contains("failed to find") {
        FailureCause::JsonError
    } else if log.contains("cmake error") || log.contains("build error") || log.contains("no rule to make target") {
        FailureCause::Unknown // For now map to Unknown but we can add CMAKE_FAIL
    } else {
        FailureCause::Unknown
    }
}

pub fn determine_scope(cause: &FailureCause) -> RetryScope {
    match cause {
        // v0.0.28: Structural omissions are Skeleton-level failures
        FailureCause::MissingHeader => RetryScope::Skeleton, 
        FailureCause::MissingInclude => RetryScope::HeaderOnly,
        FailureCause::MissingSymbol => RetryScope::HeaderOnly,
        FailureCause::SignatureMismatch => RetryScope::HeaderOnly,

        FailureCause::UndefinedReference => RetryScope::ImplementationOnly,
        FailureCause::MultipleDefinition => RetryScope::ImplementationOnly,

        FailureCause::SyntaxError => RetryScope::ImplementationOnly,
        FailureCause::TypeError => RetryScope::ImplementationOnly,

        FailureCause::SegFault => RetryScope::ImplementationOnly,
        FailureCause::AssertionFailed => RetryScope::ImplementationOnly,

        FailureCause::JsonError => RetryScope::Stage,
        FailureCause::Unknown => {
            // v0.0.28: If we fail in Build, try BuildRepair first instead of Full reset
            RetryScope::BuildRepair
        }
    }
}

pub fn generate_hint(cause: &FailureCause) -> &'static str {
    match cause {
        FailureCause::MissingHeader => 
            "Previous architecture rejected: Missing header component (*.h).\n\
             C/C++ projects REQUIRE a modular structure:\n\
             - At least one .h (header) component for interfaces.\n\
             - Matching .c/.cpp (logic) components.\n\
             - A clear entrypoint (main.c).\n\
             Please redesign the Skeleton with these components.",
        FailureCause::MissingSymbol => 
            "HINT: Declare the missing symbol (function/type) in the appropriate header file.",
        FailureCause::SignatureMismatch => 
            "HINT: Ensure function signatures (return type, parameters) exactly match between .h and .c/.cpp files.",
        FailureCause::UndefinedReference => 
            "HINT: The function is declared but its implementation is missing or not linked. Check your .c files.",
        FailureCause::MultipleDefinition => 
            "HINT: Duplicate implementation found. Remove redundant code or use 'static' for internal functions.",
        FailureCause::SyntaxError => 
            "HINT: Check for missing semicolons, unbalanced braces, or invalid C structure.",
        FailureCause::SegFault => 
            "HINT: Memory corruption detected. Check for null pointers, invalid array indexing, or uninitialized memory.",
        FailureCause::AssertionFailed => 
            "HINT: A logical condition failed. Review your algorithm and edge cases.",
        FailureCause::JsonError => 
            "HINT: Output ONLY the requested JSON data. Ensure it is wrapped between <JSON_START> and <JSON_END>. Do not include any conversational text.",
        _ => 
            "HINT: The model produced an unrecognizable response. Ensure your environment can handle the prompt size and check the RAW output in the logs.",
    }
}

pub fn escalate(state: &mut EscalationState, file: &str) {
    state.attempts += 1;

    // Safety Guard: Header Protection
    // Headers are critical; skip RegenerateFile and go to RetryStage
    if file.ends_with(".h") && state.strategy == FixStrategy::Patch && state.attempts >= 2 {
        state.strategy = FixStrategy::RetryStage;
        state.attempts = 0;
        return;
    }

    state.strategy = match state.strategy {
        FixStrategy::Patch if state.attempts >= 2 => {
            state.attempts = 0;
            FixStrategy::RegenerateFile
        },
        FixStrategy::RegenerateFile if state.attempts >= 2 => {
            state.attempts = 0;
            FixStrategy::RetryStage
        },
        FixStrategy::RetryStage if state.attempts >= 2 => {
            state.attempts = 0;
            FixStrategy::FullReset
        },
        other => other,
    };
}

// --- PROMPT ENGINE ---

pub struct PromptContext {
    pub stage: Stage,
    pub cause: Option<FailureCause>,
    pub files: Vec<(String, String)>,
    pub target: String,
    pub project_root: String, // v0.0.30: Required for contract injection
}

pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build(ctx: &PromptContext) -> String {
        let mut prompt = String::new();

        // v0.0.30: [CRITICAL_CONTRACT] Injection
        if let Some(sealed) = load_sealed_ir(&ctx.project_root) {
            prompt.push_str("\n[CRITICAL_CONTRACT]\n");
            prompt.push_str("The following semantic decisions are BINDING and IMMUTABLE:\n\n");
            for decision in &sealed.decisions {
                prompt.push_str(&format!("- {}: {} ({})\n", decision.risk_id, decision.action, decision.comment));
            }
            prompt.push_str("\nViolation of these contracts will result in immediate rejection.\n\n");
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
