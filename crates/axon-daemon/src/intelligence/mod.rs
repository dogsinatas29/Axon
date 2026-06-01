pub(crate) mod rule_registry;
pub(crate) mod rule_engine;
pub(crate) mod promotion;
pub(crate) mod selection;
pub(crate) mod global_registry;
pub mod decision;
pub(crate) mod staging;
pub(crate) mod commit;
pub(crate) mod writer;
pub(crate) mod priority;
pub(crate) mod ir_diff;
pub(crate) mod pipeline;
pub(crate) mod constraint_meta;
pub(crate) mod orchestrator;
pub(crate) mod planner;
pub(crate) mod coordinator;
pub(crate) mod language_contract;
pub(crate) mod include_path_normalizer;
pub mod semantic_debugger;
pub(crate) mod lsp;
pub mod jurisprudence;
pub(crate) mod ast;
pub mod topology;
pub(crate) mod patch_ir;
pub(crate) mod signature_drift;
pub(crate) mod mutation_sandbox;
pub(crate) mod provenance;
pub(crate) mod observatory;
pub(crate) mod shadow_mutator;
pub(crate) mod roundtrip_validator;
pub(crate) mod stability_matrix;
pub(crate) mod heatmap;
pub mod causality;
pub(crate) mod determinism_harness;
pub(crate) mod rollback;
pub(crate) mod mutation_intent;
pub(crate) mod semantic_tokens;
pub(crate) mod canonicalizer;
pub(crate) mod semantic_distance;
pub(crate) mod semantic_authority_gate;
pub(crate) mod edit_plan;
pub(crate) mod tree_sitter_locator;
pub(crate) mod surgical_editor;
pub(crate) mod anchor_validator;
pub(crate) mod surgical_replay;
pub(crate) mod intent_lowering;
pub mod replay;
pub mod corpus;
pub(crate) mod telemetry;
pub mod common;
pub(crate) mod mutation;
pub mod evolution;

/// AXON Intelligence Gate Facade
pub struct IntelligenceEngine;

impl IntelligenceEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_spec(&self) {
        println!("IntelligenceEngine analyzing active spec.");
    }
}
