use crate::intelligence::decision::{Stage, FailureCause};

#[derive(Default)]
pub struct PythonContractValidator;

impl PythonContractValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, _signature: &str) -> super::common::FixResult {
        // Simple python placeholder validator
        super::common::FixResult::Valid
    }
}

pub fn base_prompt(stage: &Stage) -> String {
    match stage {
        Stage::ImplGen => 
            "[ROLE]\nYou implement a Python module.\n\n\
             [TASK]\nImplement the module code in Python.\n\n\
             [CONSTRAINTS]\n- Output ONLY valid, clean Python (.py) code.\n- Keep logic minimal, safe, and correct.\n".to_string(),
        Stage::Build | Stage::Runtime => 
            "[ROLE]\nYou are fixing a broken Python program.\n\n\
             [TASK]\nFix the interpreter or test errors.\n\n\
             [CONSTRAINTS]\n- Output ONLY valid, run-ready Python (.py) code.\n- Focus on the identified runtime/syntax error.\n- Minimal changes only.\n".to_string(),
        _ => "[ROLE]\nAI Agent (Python)\n\n[TASK]\nAssist with Python project.\n".to_string(),
    }
}

pub fn infer_cause(diag_message: &str) -> FailureCause {
    let msg = diag_message.to_lowercase();
    if msg.contains("constitutional_violation") || msg.contains("language mismatch") {
        FailureCause::ConstitutionalViolation
    } else if msg.contains("modulenotfounderror") || msg.contains("import error") {
        FailureCause::MissingHeader
    } else if msg.contains("nameerror") || msg.contains("is not defined") {
        FailureCause::MissingSymbol
    } else if msg.contains("syntaxerror") || msg.contains("indentationerror") {
        FailureCause::SyntaxError
    } else {
        FailureCause::Unknown
    }
}

pub fn generate_hint(cause: &FailureCause) -> &'static str {
    match cause {
        FailureCause::ConstitutionalViolation => "CONSTITUTIONAL VIOLATION: Mismatch between specification and implementation language. Output ONLY valid Python files.",
        FailureCause::MissingHeader => "Focus on resolving import issues. Check if required libraries or modules are installed/present.",
        FailureCause::MissingSymbol => "Check if the variable or function is defined before it is used.",
        FailureCause::SyntaxError => "Fix syntax or indentation errors reported by the Python interpreter.",
        _ => "Analyze runtime logs and apply targeted fixes.",
    }
}

pub fn inject_cause(cause: &FailureCause) -> &'static str {
    match cause {
        FailureCause::MissingHeader => 
            "CAUSE: Module or library not found (ModuleNotFoundError).\n\
             INSTRUCTION: Verify that imports match existing local files or installed packages. Correct import path.",
        FailureCause::MissingSymbol => 
            "CAUSE: Name or symbol is not defined (NameError).\n\
             INSTRUCTION: Define the variable, function, or class before calling it. Ensure correct scoping.",
        FailureCause::SyntaxError => 
            "CAUSE: Syntax/Indentation error.\n\
             INSTRUCTION: Fix the indent level or Python syntax (colons, parentheses) as pointed out by interpreter.",
        _ => "CAUSE: Technical discrepancy detected.\nINSTRUCTION: Re-evaluate module structure and fix the error.",
    }
}
