use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationIntent {
    ReplaceFunctionBody { symbol_name: String, new_body: String },
    ReplaceMatchArm { symbol_name: String, pattern: String, new_arm: String },
    InsertStructField { struct_name: String, field_def: String },
    RenameParameter { function_name: String, old_name: String, new_name: String },
}

impl MutationIntent {
    /// Determines the expected ceiling of AST node modifications.
    /// Used by the Structural Minimality Constraint to prevent printer normalization explosion.
    pub fn expected_node_change_ceiling(&self) -> usize {
        match self {
            MutationIntent::ReplaceFunctionBody { .. } => 1,
            MutationIntent::ReplaceMatchArm { .. } => 1,
            MutationIntent::InsertStructField { .. } => 1,
            MutationIntent::RenameParameter { .. } => 2,
        }
    }
}
