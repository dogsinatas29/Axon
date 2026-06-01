use crate::schema::ProjectIR;
use crate::validator::{ValidationError, ValidationKind};
use super::LanguageValidator;

pub struct LuaValidator;

impl LanguageValidator for LuaValidator {
    fn validate(&self, ir: &ProjectIR) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for (key, comp) in &ir.components {
            let path_lower = comp.file_path.to_lowercase();
            if path_lower.ends_with(".c") || path_lower.ends_with(".h") || path_lower.ends_with(".rs") || path_lower.ends_with(".py") || path_lower == "cmakelists.txt" || path_lower == "cargo.toml" {
                errors.push(ValidationError {
                    kind: ValidationKind::Semantic,
                    target: key.clone(),
                    message: format!("[CONSTITUTIONAL_VIOLATION] Non-Lua file '{}' detected in Lua project. Lua projects must use Lua files (.lua) only.", comp.file_path),
                });
            }

            if path_lower.ends_with(".lua") && comp.functions.is_empty() {
                errors.push(ValidationError {
                    kind: ValidationKind::Semantic,
                    target: key.clone(),
                    message: format!("Lua file '{}' has no functions.", comp.file_path),
                });
            }
        }
        errors
    }

    fn check_entry_point(&self, files: &[(String, String)]) -> bool {
        for (name, code) in files {
            let n = name.to_lowercase();
            if n.ends_with("main.lua") || n.ends_with("init.lua") || 
               code.contains("function main(") ||
               code.contains("return {") {
                return true;
            }
        }
        false
    }
}
