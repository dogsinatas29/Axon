use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SemanticToken {
    FunctionDefinition { name: String },
    AsyncModifier,
    Parameter { name: String, ty: String },
    ReturnType { ty: String },
    Visibility { scope: String },
    Decorator { name: String },
    // Core semantic identities, ignoring syntactic trivia
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    Calls,
    Inherits,
    Instantiates,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TopologyEdge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SignatureVector {
    pub params: Vec<SemanticToken>,
    pub ret: Option<SemanticToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VisibilityScope {
    pub is_public: bool,
    pub module_path: String,
}

/// Layer 3 - The Ultimate Authority.
/// Parser/Formatting drift is eliminated here.
/// Two distinct text ASTs with identical CanonicalSemanticForm are 100% equivalent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CanonicalSemanticForm {
    pub semantic_tokens: Vec<SemanticToken>,
    pub topology_edges: Vec<TopologyEdge>,
    pub signature_vector: SignatureVector,
    pub visibility_scope: VisibilityScope,
}
