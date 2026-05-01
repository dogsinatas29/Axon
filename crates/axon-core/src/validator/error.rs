use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ValidationError {
    // Stage 1: Parse
    SyntaxError(String),

    // Stage 2: Extract
    NoFunctionFound,

    // Stage 3 & 4: Match
    MissingFunction(String),
    MissingComponent(String),
    MissingDependency {
        component: String,
        function: String,
        dependency: String,
    },
    SignatureMismatch {
        name: String,
        expected: Vec<String>,
        actual: Vec<String>,
    },

    // Constraint Violation
    ConstraintViolation(String),
    SpecError(String),
}
