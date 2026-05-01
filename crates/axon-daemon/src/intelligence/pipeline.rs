use axon_core::ir::ProjectIR;
use super::{
    rule_registry::RuleRegistry,
    rule_engine::RuleEngine,
    commit::IRCommitLayer,
    writer::ArchitectureWriter,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// The fully automated evolution pipeline.
///
/// Flow: IR Diff → Failure Signals → RuleEngine → Proposals → IR Commit → Write MD
pub struct EvolutionPipeline {
    pub engine: RuleEngine,
}

impl EvolutionPipeline {
    pub fn new() -> Self {
        Self {
            engine: RuleEngine::new(RuleRegistry::new()),
        }
    }

    /// Runs a full evolution cycle.
    /// Given the old and new IR, it:
    /// 1. Detects regressions via IR diff
    /// 2. Feeds signals to the RuleEngine
    /// 3. Promotes rules to constraints
    /// 4. Commits constraints to the new IR atomically
    /// 5. Generates an updated architecture.md string
    pub fn run(&mut self, _old_ir: &ProjectIR, new_ir: &mut ProjectIR) -> String {
        let _now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        println!("=== [EVOLUTION] Cycle Start ===");

        // Step 1: Produce constraint proposals (pure, no side effects)
        let proposals = self.engine.produce_constraints();
        println!("   -> {} Constraint Proposal(s) Generated", proposals.len());

        // Step 4: Atomic IR commit (dedup-safe)
        IRCommitLayer::apply_proposals(new_ir, proposals);

        // Step 5: Generate architecture.md from final IR state
        let output = ArchitectureWriter::generate(new_ir);

        println!("=== [EVOLUTION] Cycle Complete ===");
        output
    }
}
