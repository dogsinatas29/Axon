use crate::schema::ProjectIR;
use crate::validator::{ValidationError, ValidationKind};
use super::LanguageValidator;

pub struct CValidator;

impl LanguageValidator for CValidator {
    fn validate(&self, ir: &ProjectIR) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for (key, comp) in &ir.components {
            let path_lower = comp.file_path.to_lowercase();
            let is_header = path_lower.ends_with(".h") || path_lower.ends_with(".hpp");
            
            // Header-specific logic: Empty functions allowed in headers (declarations only)
            if comp.functions.is_empty() && !is_header {
                errors.push(ValidationError {
                    kind: ValidationKind::Semantic,
                    target: key.clone(),
                    message: format!("C source file '{}' must have at least one function definition.", comp.file_path),
                });
            }
        }
        errors
    }

    fn check_entry_point(&self, files: &[(String, String)]) -> bool {
        for (name, code) in files {
            let n = name.to_lowercase();
            // C/C++ often uses main.c or the file containing 'int main'
            if (n.ends_with("main.c") || n.ends_with("main.cpp") || n.contains("main")) && 
               (code.contains("int main") || code.contains("void main")) {
                return true;
            }
        }
        false
    }
}
