use super::common::*;

#[derive(Default)]
pub struct RustContractValidator;

impl RustContractValidator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate(&self, signature: &str) -> FixResult {
        let tokens = tokenize(signature);
        let mut hard_fails = Vec::new();
        let mut soft_warnings = Vec::new();

        for token in &tokens {
            let token_str = token.as_str();
            
            if token_str.starts_with("def ") || token_str.starts_with("function(") || token_str == "import" {
                hard_fails.push(SignatureIssue::NamingConvention {
                    name: token_str.to_string(),
                });
            }
            
            if is_pascal_case(token_str) && !is_rust_keyword(token_str) {
                soft_warnings.push(SignatureIssue::NamingConvention {
                    name: token_str.to_string(),
                });
            }
        }

        if !hard_fails.is_empty() {
            FixResult::HardFail(hard_fails)
        } else if !soft_warnings.is_empty() {
            FixResult::SoftWarning(soft_warnings)
        } else {
            FixResult::Valid
        }
    }
    
    pub fn regeneration_feedback() -> String {
        r#"
[LANGUAGE CONTRACT VIOLATION]
Rust projects should use snake_case for functions (e.g., process_data not processData).
Avoid Python/JS patterns like: def, function, import
Use Rust idioms: fn, let, impl, struct
"#.trim().to_string()
    }
}

fn is_rust_keyword(s: &str) -> bool {
    matches!(s, "fn" | "let" | "mut" | "impl" | "struct" | "enum" | "trait" 
             | "use" | "mod" | "crate" | "self" | "Self" | "async" | "await"
             | "move" | "ref" | "type" | "where" | "loop" | "match" | "if" | "else"
             | "return" | "break" | "continue" | "true" | "false")
}

use crate::intelligence::decision::{Stage, FailureCause};

pub fn base_prompt(stage: &Stage) -> String {
    let critical_header = "\
         CRITICAL:\n\
         You are inside a PURE RUST workspace.\n\n\
         ABSOLUTELY FORBIDDEN:\n\
         - #include\n\
         - .h headers\n\
         - stdio.h\n\
         - malloc\n\
         - free\n\
         - printf\n\
         - scanf\n\n\
         Use ONLY valid Rust syntax.\n\
         Generate compilable Rust source files only.\n\n\
         If you emit C/C++ syntax, the task is considered FAILED.\n\n";

    match stage {
        Stage::Skeleton => 
            format!("{}{}", critical_header,
             "[ROLE]\nYou implement the Skeleton (Phase 1) of a Rust module.\n\n\
              [TASK]\nDeclare ONLY the topology and signatures.\n\n\
              [CONSTRAINTS]\n\
              - OUTPUT ONLY: pub struct, pub enum, pub trait, type aliases, and function signatures.\n\
              - ABSOLUTELY NO IMPLEMENTATION BODY. Use `todo!()` or `unimplemented!()` if braces are syntactically required.\n\
              - DO NOT write business logic, algorithms, IO, async runtime wiring, or thread spawning.\n\
              - This phase establishes Semantic Ownership. Implementation will be done later.\n"),
        Stage::ImplGen => 
            format!("{}{}", critical_header,
             "[ROLE]\nYou implement a Rust module.\n\n\
              [TASK]\nImplement the module code in Rust.\n\n\
              [CONSTRAINTS]\n- Output ONLY valid, clean Rust (.rs) code.\n- Do not use raw C SQLite APIs or extern blocks. Use standard Cargo/Rust libraries.\n- Keep logic minimal, safe, and correct.\n"),
        Stage::Build | Stage::Runtime => 
            format!("{}{}", critical_header,
             "[ROLE]\nYou are fixing a broken Rust program.\n\n\
              [TASK]\nFix the compiler/linter error reported in Cargo check/build.\n\n\
              [CONSTRAINTS]\n- Output ONLY valid, compile-ready Rust (.rs) code.\n- Focus on the identified compiler error.\n- Minimal changes only.\n"),
        _ => format!("{}{}", critical_header,
             "[ROLE]\nAI Agent (Rust)\n\n[TASK]\nAssist with Rust project.\n"),
    }
}

pub fn infer_cause(diag_message: &str) -> FailureCause {
    let msg = diag_message.to_lowercase();
    if msg.contains("constitutional_violation") || msg.contains("language mismatch") || msg.contains("c-centric file pollution") {
        FailureCause::ConstitutionalViolation
    } else if msg.contains("cannot find") || msg.contains("unresolved import") || msg.contains("not found in") {
        FailureCause::MissingSymbol
    } else if msg.contains("expected") || msg.contains("syntax error") {
        FailureCause::SyntaxError
    } else {
        FailureCause::Unknown
    }
}

pub fn generate_hint(cause: &FailureCause) -> &'static str {
    match cause {
        FailureCause::ConstitutionalViolation => "CONSTITUTIONAL VIOLATION: Mismatch between specification and implementation language, or C files created for Rust project. DO NOT create any .c/.h/CMakeLists.txt files. Output ONLY valid Rust files as constrained.",
        FailureCause::MissingSymbol => "Declare or import the missing symbol. Check if the module is correctly declared in main.rs/lib.rs (e.g. mod db;).",
        FailureCause::SyntaxError => "Fix syntax issues, semicolons, brackets, or mismatched types reported by cargo check.",
        _ => "Analyze cargo check diagnostic logs and apply minimal targeted fixes.",
    }
}

pub fn inject_cause(cause: &FailureCause) -> &'static str {
    match cause {
        FailureCause::MissingSymbol => 
            "CAUSE: A symbol (module, struct, function, or crate) is missing or unresolved.\n\
             INSTRUCTION: Declare the missing module using 'pub mod <name>;' or import it via 'use <crate>::...;'. Check project module structure.",
        FailureCause::SyntaxError => 
            "CAUSE: Syntax error (missing semicolon, mismatched braces, or invalid compiler tokens).\n\
             INSTRUCTION: Correct the syntax according to compiler error line and column info.",
        _ => "CAUSE: Compile/Lint diagnostic detected.\nINSTRUCTION: Re-evaluate the Rust module design and resolve the error.",
    }
}

/// Skeleton Validator (Step 2 of Rust Constitution)
/// Prevents the LLM from outputting implementation bodies during Phase 1.
pub fn validate_skeleton(content: &str) -> Result<(), String> {
    let forbidden_patterns = [
        " loop ", " loop{", " loop\n", 
        " if ", " if{", " if\n", 
        " match ", " match{", " match\n",
        "async move", "tokio::spawn", "println!", "vec![", 
        "Box::new", "Arc::new", "Mutex::new", "RwLock::new", 
        "std::fs::", "std::io::", "std::thread::"
    ];

    for pattern in &forbidden_patterns {
        if content.contains(pattern) {
            return Err(format!("SKELETON_VIOLATION: Forbidden implementation logic detected ('{}'). Phase 1 requires declarations only, absolutely no implementation bodies or runtime wiring.", pattern));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_validator_blocks_implementation() {
        // Phase A - Skeleton Constitution 검증
        let valid_skeleton = "pub fn parse_user(id: u32) -> Result<(), Error>;";
        assert!(validate_skeleton(valid_skeleton).is_ok());

        let invalid_skeleton_1 = "pub fn parse_user(id: u32) { loop { break; } }";
        assert!(validate_skeleton(invalid_skeleton_1).is_err());

        let invalid_skeleton_2 = "async fn fetch() { tokio::spawn(async move {}); }";
        assert!(validate_skeleton(invalid_skeleton_2).is_err());

        let invalid_skeleton_3 = "fn debug() { println!(\"debug\"); }";
        assert!(validate_skeleton(invalid_skeleton_3).is_err());
    }
}