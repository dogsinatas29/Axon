use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SemanticSeverity {
    Harmless,
    Inspectable,
    Quarantine,
    Reject,
}

/// Replaces simple float "distances" with directional, topology-aware severities.
/// "Non-commutative" - adding a parameter is Breaking, removing a parameter is also Breaking,
/// but adding a comment is HarmlessNormalization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SemanticMutationClass {
    HarmlessNormalization,
    InternalBodyMutation,
    VisibilityExpansion,
    SignatureBreakingChange,
    TopologyBreakingChange,
}

impl SemanticMutationClass {
    pub fn severity(&self) -> SemanticSeverity {
        match self {
            Self::HarmlessNormalization => SemanticSeverity::Harmless,
            Self::InternalBodyMutation => SemanticSeverity::Inspectable,
            Self::VisibilityExpansion => SemanticSeverity::Quarantine,
            Self::SignatureBreakingChange => SemanticSeverity::Quarantine, // Needs Boss
            Self::TopologyBreakingChange => SemanticSeverity::Reject, // Kills the build
        }
    }
}
