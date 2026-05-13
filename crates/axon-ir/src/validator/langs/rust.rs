use crate::schema::ProjectIR;
use crate::validator::{ValidationError, ValidationKind};
use super::LanguageValidator;

pub struct RustValidator;

impl LanguageValidator for RustValidator {
    fn validate(&self, ir: &ProjectIR) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for (key, comp) in &ir.components {
            // Rust specific: files must have functions unless they are module roots with re-exports
            if comp.functions.is_empty() && 
               !comp.file_path.ends_with("lib.rs") && 
               !comp.file_path.ends_with("mod.rs") &&
               !comp.file_path.ends_with("main.rs") {
                 errors.push(ValidationError {
                    kind: ValidationKind::Semantic,
                    target: key.clone(),
                    message: format!("Rust file '{}' has no functions and is not a module root.", comp.file_path),
                });
            }
        }
        errors
    }

    fn check_entry_point(&self, files: &[(String, String)]) -> bool {
        for (name, code) in files {
            let n = name.to_lowercase();
            if n.ends_with("main.rs") || code.contains("fn main()") {
                return true;
            }
        }
        false
    }
}
