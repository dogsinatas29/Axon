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