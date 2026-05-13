use super::common::*;

#[derive(Default)]
pub struct CContractValidator;

impl CContractValidator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate(&self, signature: &str) -> FixResult {
        let tokens = tokenize(signature);
        let mut auto_fixes = Vec::new();
        let mut soft_warnings = Vec::new();
        let mut hard_fails = Vec::new();

        for token in &tokens {
            let token_str = token.as_str();

            // 1. Check for hard-banned generic syntax
            if has_generic_syntax(token_str) {
                let type_name = token_str.split('<').next().unwrap_or(token_str);
                hard_fails.push(SignatureIssue::GenericType {
                    type_name: type_name.to_string(),
                });
                continue;
            }

            // 2. Check for Java collection types
            if is_java_collection(token_str) {
                hard_fails.push(SignatureIssue::JavaCollection {
                    type_name: token_str.to_string(),
                });
                continue;
            }

            // 3. Check for primitive alias replacements
            for (java_type, c_type) in C_PRIMITIVE_REPLACEMENTS {
                if token_str == *java_type {
                    auto_fixes.push(SignatureIssue::PrimitiveAlias {
                        original: java_type.to_string(),
                        replacement: c_type.to_string(),
                    });
                    break;
                }
            }

            // 4. Check for PascalCase identifiers (soft warning only)
            if is_pascal_case(token_str) && !is_c_builtin(token_str) && !token_str.contains('<') {
                soft_warnings.push(SignatureIssue::NamingConvention {
                    name: token_str.to_string(),
                });
            }

            // 5. Check for soft-banned patterns (camelCase in names)
            let token_lower = token_str.to_lowercase();
            for pattern in C_SOFT_BANNED_PATTERNS {
                if token_lower.contains(pattern) {
                    soft_warnings.push(SignatureIssue::NamingConvention {
                        name: token_str.to_string(),
                    });
                    break;
                }
            }
        }

        // Generate fixed signature if auto-fixes exist
        let fixed = if !auto_fixes.is_empty() {
            let mut result = signature.to_string();
            for issue in &auto_fixes {
                if let SignatureIssue::PrimitiveAlias { original, replacement } = issue {
                    result = safe_replace(&result, original, replacement);
                }
            }
            Some(result)
        } else {
            None
        };

        // Decision logic
        if !hard_fails.is_empty() {
            return FixResult::HardFail(hard_fails);
        }

        if let Some(fixed_sig) = fixed {
            let fix = SignatureFix {
                original: signature.to_string(),
                fixed: fixed_sig,
                issues: auto_fixes.clone(),
            };
            // Include soft warnings in result
            let mut all_issues = auto_fixes;
            all_issues.extend(soft_warnings);
            FixResult::AutoFixed(SignatureFix {
                original: signature.to_string(),
                fixed: fix.fixed,
                issues: all_issues,
            })
        } else if !soft_warnings.is_empty() {
            FixResult::SoftWarning(soft_warnings)
        } else {
            FixResult::Valid
        }
    }
    
    pub fn regeneration_feedback() -> String {
        r#"
[LANGUAGE CONTRACT VIOLATION]
C projects MUST NOT use:
- generic syntax (<T>)
- List<T>, Map<K,V>, Optional<T>, Set<T>
- Java collection types (ArrayList, HashMap, etc.)
- Java-specific types (String, boolean, Integer, Object)

REQUIRED:
- Use char* for strings
- Use int for boolean
- Use struct for data grouping
- Pure C99 idioms only

Regenerate using C semantics only.
"#.trim().to_string()
    }
}

fn is_c_builtin(s: &str) -> bool {
    matches!(s, "void" | "int" | "char" | "float" | "double" | "short" | "long" 
             | "unsigned" | "signed" | "const" | "static" | "struct" | "enum" | "typedef"
             | "bool" | "size_t" | "ssize_t" | "FILE" | "NULL")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_c_signature() {
        let validator = CContractValidator::new();
        let result = validator.validate("void process_input(char* input)");
        assert!(matches!(result, FixResult::Valid));
    }

    #[test]
    fn test_string_contamination() {
        let validator = CContractValidator::new();
        let result = validator.validate("void print_message(String msg)");
        match result {
            FixResult::AutoFixed(fix) => {
                assert_eq!(fix.fixed, "void print_message(char* msg)");
            }
            _ => panic!("Expected AutoFixed"),
        }
    }

    #[test]
    fn test_generic_hard_fail() {
        let validator = CContractValidator::new();
        let result = validator.validate("void process(List<DataModel> input)");
        assert!(matches!(result, FixResult::HardFail(_)));
    }

    #[test]
    fn test_pascal_soft_warning() {
        let validator = CContractValidator::new();
        let result = validator.validate("void initialize(UserData data)");
        match result {
            FixResult::SoftWarning(issues) => {
                assert!(issues.iter().any(|i| matches!(i, SignatureIssue::NamingConvention { name } if name == "UserData")));
            }
            _ => {}
        }
    }
}