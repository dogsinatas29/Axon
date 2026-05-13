use crate::ir::ProjectIR;
use crate::validator::error::ValidationError;
use crate::validator::types::FunctionSig;
use crate::rules::Constraint as RuleConstraint;

pub struct ValidationInput {
    pub ir: ProjectIR,
    pub extracted: Vec<FunctionSig>,
    pub constraints: Vec<RuleConstraint>,
}

pub struct DeterministicValidator;

impl DeterministicValidator {
    /// Validates the extracted code structure against the constraints.
    /// This is purely deterministic and does not rely on LLM logic.
    pub fn validate(input: ValidationInput) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for constraint in &input.constraints {
            match constraint {
                RuleConstraint::ExactFunctionExists { name } => {
                    if !input.extracted.iter().any(|f| &f.name == name) {
                        errors.push(ValidationError::MissingFunction(name.clone()));
                    }
                }

                RuleConstraint::ExactSignatureMatch { name, args } => {
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

                RuleConstraint::NoExtraFunctions => {
                    let mut known_functions = std::collections::HashSet::new();
                    for comp in input.ir.components.values() {
                        for func_name in comp.functions.keys() {
                            known_functions.insert(func_name);
                        }
                    }

                    for func in &input.extracted {
                        if !known_functions.contains(&func.name) {
                            // Placeholder for extra function detection
                        }
                    }
                }

                _ => {}
            }
        }

        errors
    }
}
