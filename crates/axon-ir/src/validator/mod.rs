pub mod langs;
use crate::schema::ProjectIR;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub kind: ValidationKind,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationKind {
    Syntax,
    Semantic,
    RuntimeContract,
    OrphanDependency,
    CircularDispatch,
    MissingArtifact,
    InvalidBackend,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}: {}", self.kind, self.target, self.message)
    }
}

pub fn validate_ir(ir: &ProjectIR) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    errors.extend(validate_structure(ir));
    errors.extend(validate_semantic(ir));
    
    // v0.0.31: Explicit language semantic binding
    let lang_str = match ir.language {
        crate::schema::Language::C => "c",
        crate::schema::Language::Cpp => "cpp",
        crate::schema::Language::Rust => "rust",
        crate::schema::Language::Python => "python",
    };
    
    if let Some(validator) = langs::get_validator(lang_str) {
        errors.extend(validator.validate(ir));
    } else {
        errors.push(ValidationError {
            kind: ValidationKind::InvalidBackend,
            target: "ProjectIR".to_string(),
            message: format!("No validator found for language: {:?}", ir.language),
        });
    }

    // Circular Dependency check
    if let Err(cycle_msg) = crate::linker::link_dependencies(ir) {
        errors.push(ValidationError {
            kind: ValidationKind::CircularDispatch,
            target: "ProjectGraph".to_string(),
            message: cycle_msg,
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_structure(ir: &ProjectIR) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for (key, comp) in &ir.components {
        if key.is_empty() {
            errors.push(ValidationError {
                kind: ValidationKind::Syntax,
                target: key.clone(),
                message: "Empty component key".to_string(),
            });
        }

        if comp.file_path.is_empty() {
            errors.push(ValidationError {
                kind: ValidationKind::Syntax,
                target: comp.name.clone(),
                message: "Empty file_path".to_string(),
            });
        }
    }

    errors
}

fn validate_semantic(ir: &ProjectIR) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for (key, comp) in &ir.components {
        /* Moved to language-specific validators */

        for (fname, _func) in &comp.functions {
            if fname.is_empty() || fname.starts_with('_') {
                errors.push(ValidationError {
                    kind: ValidationKind::Semantic,
                    target: format!("{}.{}", key, fname),
                    message: "Invalid function name".to_string(),
                });
            }
        }
    }

    errors
}

pub fn validate_runtime_contract(ir: &ProjectIR, task_graph: &[String]) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    let component_keys: std::collections::BTreeSet<_> = ir.components.keys().collect();

    for task_id in task_graph {
        let key = task_id;
        if !component_keys.contains(&key) && !key.starts_with("task_") {
            errors.push(ValidationError {
                kind: ValidationKind::OrphanDependency,
                target: task_id.clone(),
                message: format!("Task references non-existent component: {}", key),
            });
        }
    }

    for (key, comp) in &ir.components {
        for func in comp.functions.values() {
            for dep in &func.dependencies {
                if !component_keys.contains(&dep) && !dep.starts_with("task_") {
                    errors.push(ValidationError {
                        kind: ValidationKind::OrphanDependency,
                        target: format!("{}.depends_on.{}", key, dep),
                        message: format!("Function dependency points to non-existent component: {}", dep),
                    });
                }
            }
        }
    }

    errors
}

pub fn validate_language_capability(language: &str, _required_features: &[&str]) -> Result<(), ValidationError> {
    let supported = matches!(language, "c" | "cpp" | "rust" | "python" | "javascript" | "typescript");

    if !supported {
        return Err(ValidationError {
            kind: ValidationKind::InvalidBackend,
            target: language.to_string(),
            message: format!("Unsupported language backend: {}", language),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use crate::schema::Component;

    #[test]
    fn test_validate_empty_component() {
        let mut ir = ProjectIR::new();
        ir.components.insert("".to_string(), Component {
            name: "".to_string(),
            file_path: "".to_string(),
            functions: std::collections::BTreeMap::new(),
            imports: std::collections::BTreeSet::new(),
            associated_files: Vec::new(),
            is_entrypoint: false,
            data_models: Vec::new(),
            metadata: std::collections::BTreeMap::new(),
            allowed_includes: std::collections::BTreeSet::new(),
            forbidden_includes: std::collections::BTreeSet::new(),
            forbidden_symbols: std::collections::BTreeSet::new(),
            tier: crate::schema::ComponentTier::Core,
            is_blocking: true,
            locked: false,
            component_type: crate::schema::ComponentType::ProjectModule,
            subsystem: None,
            dll_imports: std::collections::BTreeSet::new(),
            ownership: crate::schema::OwnershipMetadata::generator_patchable(),
        });

        let result = validate_ir(&ir);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_functions() {
        let mut ir = ProjectIR::new();
        ir.components.insert("test.c".to_string(), Component {
            name: "test".to_string(),
            file_path: "test.c".to_string(),
            functions: std::collections::BTreeMap::new(),
            imports: std::collections::BTreeSet::new(),
            associated_files: Vec::new(),
            is_entrypoint: false,
            data_models: Vec::new(),
            metadata: std::collections::BTreeMap::new(),
            allowed_includes: std::collections::BTreeSet::new(),
            forbidden_includes: std::collections::BTreeSet::new(),
            forbidden_symbols: std::collections::BTreeSet::new(),
            tier: crate::schema::ComponentTier::Core,
            is_blocking: true,
            locked: false,
            component_type: crate::schema::ComponentType::ProjectModule,
            subsystem: None,
            dll_imports: std::collections::BTreeSet::new(),
            ownership: crate::schema::OwnershipMetadata::generator_patchable(),
        });

        let result = validate_ir(&ir);
        assert!(result.is_err());
    }
}