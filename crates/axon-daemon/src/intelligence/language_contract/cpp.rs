use super::common::*;

#[derive(Default)]
pub struct CppContractValidator;

impl CppContractValidator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate(&self, signature: &str) -> FixResult {
        let tokens = tokenize(signature);
        let mut issues = Vec::new();

        for token in &tokens {
            let token_str = token.as_str();
            
            if token_str == "String" || token_str == "Integer" || token_str == "Boolean" {
                issues.push(SignatureIssue::PrimitiveAlias {
                    original: token_str.to_string(),
                    replacement: "std::string".to_string(),
                });
            }
        }

        if issues.is_empty() {
            FixResult::Valid
        } else {
            FixResult::SoftWarning(issues)
        }
    }
    
    pub fn regeneration_feedback() -> String {
        r#"
[LANGUAGE CONTRACT VIOLATION]
C++ projects should use STL containers (std::vector, std::map) instead of Java collections.
Avoid: ArrayList, HashMap, List, etc.
Use: std::vector, std::map, std::unordered_map, std::list
"#.trim().to_string()
    }
}