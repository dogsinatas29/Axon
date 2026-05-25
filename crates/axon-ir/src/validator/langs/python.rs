use crate::schema::ProjectIR;
use crate::validator::{ValidationError, ValidationKind};
use super::LanguageValidator;

pub struct PythonValidator;

impl LanguageValidator for PythonValidator {
    fn validate(&self, ir: &ProjectIR) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for (key, comp) in &ir.components {
            let path_lower = comp.file_path.to_lowercase();
            if path_lower.ends_with(".c") || path_lower.ends_with(".h") || path_lower.contains("include/") || path_lower == "cmakelists.txt" {
                errors.push(ValidationError {
                    kind: ValidationKind::Semantic,
                    target: key.clone(),
                    message: format!("[CONSTITUTIONAL_VIOLATION] C-centric file '{}' detected in Python project. Python projects must use Python files (.py) only.", comp.file_path),
                });
            }

            // Python specific: must have functions or be a package init/script
            if comp.functions.is_empty() && 
               !comp.file_path.ends_with("__init__.py") &&
               !path_lower.ends_with(".c") &&
               !path_lower.ends_with(".h") {
                 errors.push(ValidationError {
                    kind: ValidationKind::Semantic,
                    target: key.clone(),
                    message: format!("Python file '{}' has no functions and is not a package init.", comp.file_path),
                });
            }
        }
        errors
    }

    fn check_entry_point(&self, files: &[(String, String)]) -> bool {
        for (name, code) in files {
            let n = name.to_lowercase();
            if n.ends_with("main.py") || n.ends_with("app.py") || 
               code.contains("if __name__ == '__main__'") || 
               code.contains("if __name__ == \"__main__\"") ||
               code.contains("def main():") {
                return true;
            }
        }
        false
    }
}
