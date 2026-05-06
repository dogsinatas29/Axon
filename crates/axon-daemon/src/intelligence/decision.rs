// # encoding: utf-8
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Stage {
    Skeleton,
    HeaderGen,
    ImplGen,
    Build,
    Runtime,
    Sync,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RetryScope {
    Skeleton,
    HeaderOnly,
    ImplementationOnly,
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
            Stage::Skeleton => Stage::HeaderGen,
            Stage::HeaderGen => Stage::ImplGen,
            Stage::ImplGen => Stage::Build,
            Stage::Build => Stage::Runtime,
            Stage::Runtime => Stage::Sync,
            Stage::Sync => Stage::Complete,
            Stage::Complete => Stage::Complete,
        }
    }

    pub fn route_retry(scope: &RetryScope, current: &Stage) -> Stage {
        match scope {
            RetryScope::Skeleton => Stage::Skeleton,
            RetryScope::HeaderOnly => Stage::HeaderGen,
            RetryScope::ImplementationOnly => Stage::ImplGen,
            RetryScope::Full => Stage::Skeleton,
            RetryScope::None => current.clone(),
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

    if (log.contains("no such file") || log.contains("not found")) && log.contains(".h") {
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
              log.contains("eof while parsing") || log.contains("trailing characters") || log.contains("control character") {
        FailureCause::JsonError
    } else {
        FailureCause::Unknown
    }
}

pub fn determine_scope(cause: &FailureCause) -> RetryScope {
    match cause {
        FailureCause::MissingHeader => RetryScope::HeaderOnly,
        FailureCause::MissingInclude => RetryScope::HeaderOnly,
        FailureCause::MissingSymbol => RetryScope::HeaderOnly,
        FailureCause::SignatureMismatch => RetryScope::HeaderOnly,

        FailureCause::UndefinedReference => RetryScope::ImplementationOnly,
        FailureCause::MultipleDefinition => RetryScope::ImplementationOnly,

        FailureCause::SyntaxError => RetryScope::ImplementationOnly,
        FailureCause::TypeError => RetryScope::ImplementationOnly,

        FailureCause::SegFault => RetryScope::ImplementationOnly,
        FailureCause::AssertionFailed => RetryScope::ImplementationOnly,

        FailureCause::JsonError => RetryScope::Skeleton,
        FailureCause::Unknown => RetryScope::Full,
    }
}

pub fn generate_hint(cause: &FailureCause) -> &'static str {
    match cause {
        FailureCause::MissingHeader => 
            "HINT: Create or include the required header (.h) file. Ensure all modules have matching interfaces.",
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
            "HINT: Return ONLY valid JSON format. Do not include markdown code blocks or explanations.",
        _ => 
            "HINT: Re-evaluate the module structure and dependencies. Check the logs for specific errors.",
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
}

pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build(ctx: &PromptContext) -> String {
        let mut prompt = String::new();

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
