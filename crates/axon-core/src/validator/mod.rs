pub mod error;
pub mod types;
pub mod analysis;
pub mod debug;
pub mod integration;

use rustpython_parser::Parse;
use rustpython_ast::Suite;
use crate::ir::*;
use crate::spec::*;
use crate::rules::Constraint;
pub use error::ValidationError;
pub use types::{FunctionSig, ValidationResult};
use analysis::*;

pub struct Validator;

impl Validator {
    /// Validates code against a set of constraints and an architectural spec.
    pub fn validate_code(
        code: &str,
        _spec: &Spec,
        constraints: &[Constraint],
    ) -> ValidationResult {
        let mut errors = Vec::new();

        // 1. Parse (Stage 1)
        let ast = match Suite::parse(code, "<axon_validator>") {
            Ok(ast) => ast,
            Err(e) => {
                errors.push(ValidationError::SyntaxError(format!("{}", e)));
                return ValidationResult::failure(errors);
            }
        };

        // 2. Extract (Stage 2)
        let funcs = extract_functions(&ast);
        if funcs.is_empty() {
            errors.push(ValidationError::NoFunctionFound);
        }

        // 3. Constraint-based Orchestration (Stage 3 & 4)
        for constraint in constraints {
            match constraint {
                Constraint::PythonOnly => {
                    // Handled by parse step
                }
                Constraint::MustImplementAllSymbols => {
                    // Handled by project validation or higher level logic
                }
                Constraint::ExactFunctionExists { name } => {
                    if !funcs.iter().any(|f| &f.name == name) {
                        errors.push(ValidationError::MissingFunction(name.clone()));
                    }
                }
                Constraint::ExactSignatureMatch { name, args } => {
                    match funcs.iter().find(|f| &f.name == name) {
                        Some(f) if &f.args != args => {
                            errors.push(ValidationError::SignatureMismatch {
                                name: name.clone(),
                                expected: args.clone(),
                                actual: f.args.clone(),
                            });
                        }
                        None => {
                            errors.push(ValidationError::MissingFunction(name.clone()));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if errors.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::failure(errors)
        }
    }

    /// Global project validation (Components, Dependencies)
    pub fn validate_project(ir: &ProjectIR, spec: &Spec) -> ValidationResult {
        let mut errors = Vec::new();
        
        // Component check
        for required in &spec.components {
            if !ir.components.contains_key(&required.name) {
                errors.push(ValidationError::MissingComponent(required.name.clone()));
            }
        }

        // Dependency check
        for (comp_name, comp) in &ir.components {
            for func in comp.functions.values() {
                for dep in &func.dependencies {
                    let exists = ir.components.values().any(|c| {
                        c.functions.contains_key(dep)
                    });

                    if !exists {
                        errors.push(ValidationError::MissingDependency {
                            component: comp_name.clone(),
                            function: func.name.clone(),
                            dependency: dep.clone(),
                        });
                    }
                }
            }
        }

        if errors.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::failure(errors)
        }
    }
}
