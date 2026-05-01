use axon_core::rules::Constraint;

#[derive(Debug, Clone)]
pub struct ConstraintProposal {
    pub constraint: Constraint,
    pub source_rule: String,
}
