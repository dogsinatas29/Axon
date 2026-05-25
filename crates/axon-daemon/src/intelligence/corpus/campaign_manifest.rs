use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GrammarSnapshot {
    pub parser_version: String,
    pub rules_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormatterSnapshot {
    pub tool_name: String, // e.g., "rustfmt"
    pub tool_version: String,
    pub config_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntropyBudget {
    pub max_locality_collapse: f32,
    pub max_byte_drift: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApprovalPolicy {
    pub require_semantic_identity: bool,
    pub allowed_ripple_radius: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MutationClass {
    SafeSubsetV1,
    FormattingOnly,
    ImportOrdering,
    SignatureMutation,
}

#[derive(PartialEq)]
pub struct CampaignManifest {
    pub corpus_id: String,
    pub commit_hash: String,
    pub grammar_snapshot: GrammarSnapshot,
    pub formatter_snapshot: FormatterSnapshot,

    pub enabled_mutations: Vec<MutationClass>,
    pub replay_iterations: u32,

    pub entropy_budget: EntropyBudget,
    pub approval_policy: ApprovalPolicy,
}

impl CampaignManifest {
    /// Generates a deterministic hash representing this exact experimental contract
    pub fn fingerprint(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.corpus_id, 
            self.commit_hash, 
            self.grammar_snapshot.rules_hash,
            self.formatter_snapshot.config_hash
        )
    }
}
