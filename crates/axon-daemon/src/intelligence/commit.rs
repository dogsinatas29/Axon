use crate::intelligence::staging::ConstraintProposal;
use axon_core::ir::ProjectIR;
use axon_ir::schema::Constraint;

pub struct IRCommitLayer;

impl IRCommitLayer {
    /// Applies a list of constraint proposals to the ProjectIR atomically.
    /// This is the SINGLE entry point for modifying IR constraints.
    pub fn apply_proposals(ir: &mut ProjectIR, proposals: Vec<ConstraintProposal>) {
        println!(
            "=== [COMMIT] Applying {} Constraint Proposals ===",
            proposals.len()
        );

        for proposal in proposals {
            let id = Self::calculate_id(&proposal.constraint);

            if !ir.constraint_ids.contains(&id) {
                ir.constraint_ids.insert(id);
                ir.constraints.push(proposal.constraint);
                println!(
                    "   -> ADDED Constraint (ID: {:X}): {:?}",
                    id,
                    ir.constraints.last().unwrap()
                );
            } else {
                println!("   -> SKIPPED Duplicate Constraint (ID: {:X})", id);
            }
        }
    }

    fn calculate_id(c: &Constraint) -> u64 {
        c.id as u64
    }
}
