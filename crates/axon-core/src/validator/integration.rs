use crate::ir::ProjectIR;
use crate::validator::error::ValidationError;
use crate::validator::types::FunctionSig;
use crate::rules::Constraint;

pub struct ValidationInput {
    pub ir: ProjectIR,
    pub extracted: Vec<FunctionSig>,
}

pub struct DeterministicValidator;

impl DeterministicValidator {
    /// Validates the extracted code structure against the constraints defined in the IR.
    /// This is purely deterministic and does not rely on LLM logic.
    pub fn validate(input: ValidationInput) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for constraint in &input.ir.constraints {
            match constraint {
                Constraint::ExactFunctionExists { name } => {
                    if !input.extracted.iter().any(|f| &f.name == name) {
                        errors.push(ValidationError::MissingFunction(name.clone()));
                    }
                }

                Constraint::ExactSignatureMatch { name, args } => {
                    match input.extracted.iter().find(|f| &f.name == name) {
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

                Constraint::NoExtraFunctions => {
                    // Check if there are functions in 'extracted' that are not in IR components
                    let mut known_functions = std::collections::HashSet::new();
                    for comp in input.ir.components.values() {
                        for func_name in comp.functions.keys() {
                            known_functions.insert(func_name);
                        }
                    }

                    for func in &input.extracted {
                        if !known_functions.contains(&func.name) {
                            // This would need a new ValidationError variant or custom error
                            // For now, using a placeholder
                        }
                    }
                }

                _ => {}
            }
        }

        errors
    }
}
