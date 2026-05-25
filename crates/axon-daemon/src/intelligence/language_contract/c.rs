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

use crate::intelligence::decision::{Stage, FailureCause};

/// v0.0.31.xx: Generate #include statements for dependencies
/// This ensures header files include necessary library headers (e.g., sqlite3.h for sqlite3)
pub fn dependency_includes(dependencies: &[String]) -> String {
    if dependencies.is_empty() {
        return String::new();
    }

    let mut includes = String::from("\n[DEPENDENCY_INCLUDES]\n");
    for dep in dependencies {
        let include = match dep.as_str() {
            "sqlite3" => "#include <sqlite3.h>",
            "curl" => "#include <curl/curl.h>",
            "openssl" => "#include <openssl/ssl.h>",
            "json" => "#include <cjson/cjson.h>",
            "zlib" => "#include <zlib.h>",
            "pthread" => "#include <pthread.h>",
            "unistd" => "#include <unistd.h>",
            "sys_socket" => "#include <sys/socket.h>",
            _ => continue, // Skip unknown dependencies
        };
        includes.push_str(&format!("{}\n", include));
    }
    includes
}

pub fn base_prompt(stage: &Stage, dependencies: &[String]) -> String {
    let dep_includes = dependency_includes(dependencies);
    
    match stage {
        Stage::HeaderGen => 
            format!(
                "[ROLE]\nYou generate a C/C++ header file.\n\n\
                 [TASK]\nDeclare public interfaces only.\n\n\
                 [CONSTRAINTS]\n- No implementation\n- Use include guards\n- Use simple types\n- Signatures must ending with ';'{}\n",
                dep_includes
            ),
        Stage::ImplGen => 
            "[ROLE]\nYou implement a C/C++ source file.\n\n\
             [TASK]\nImplement functions declared in the header.\n\n\
             [CONSTRAINTS]\n- Must include its own header\n- Do not change signatures from header\n- Keep logic minimal and safe\n".to_string(),
        Stage::Build | Stage::Runtime => 
            "[ROLE]\nYou are fixing a broken C/C++ program.\n\n\
             [TASK]\nFix the error without rewriting the entire module.\n\n\
             [CONSTRAINTS]\n- Minimal changes only\n- Do not modify unrelated code\n- Focus on the identified failure cause\n".to_string(),
        _ => "[ROLE]\nAI Agent (C/C++)\n\n[TASK]\nAssist with C/C++ project.\n".to_string(),
    }
}

pub fn infer_cause(diag_message: &str) -> FailureCause {
    let msg = diag_message.to_lowercase();
    if msg.contains("constitutional_violation") || msg.contains("language mismatch") {
        FailureCause::ConstitutionalViolation
    } else if msg.contains("no such file or directory") || msg.contains("missing header") {
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

pub fn generate_hint(cause: &FailureCause) -> &'static str {
    match cause {
        FailureCause::ConstitutionalViolation => "CONSTITUTIONAL VIOLATION: Mismatch between specification and implementation language. Output ONLY valid C/C++ files.",
        FailureCause::MissingHeader => "Focus on generating the header file first. Check include paths.",
        FailureCause::MissingSymbol => "Declare the missing symbol in the corresponding header.",
        FailureCause::UndefinedReference => "Provide the implementation body for the declared function.",
        FailureCause::SegFault => "Add null pointer checks and verify memory allocation.",
        FailureCause::SyntaxError => "Fix semicolons, braces, or type mismatches reported by GCC/Clang.",
        _ => "Analyze compiler logs and apply minimal targeted fixes.",
    }
}

pub fn inject_cause(cause: &FailureCause) -> &'static str {
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