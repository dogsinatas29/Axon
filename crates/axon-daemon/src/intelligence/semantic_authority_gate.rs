use super::semantic_tokens::CanonicalSemanticForm;
use super::semantic_distance::{SemanticMutationClass, SemanticSeverity};

pub struct SemanticAuthorityGate;

impl SemanticAuthorityGate {
    /// Final Promotion Gate: Inspects the transition from old semantic state to new semantic state.
    /// Notice how we NEVER look at the Raw AST. The Canonical Form is the ONLY truth.
    pub fn evaluate_promotion(
        before: &CanonicalSemanticForm,
        after: &CanonicalSemanticForm,
        mutation_class: &SemanticMutationClass,
    ) -> Result<(), &'static str> {
        
        // Exact semantic match -> Pure formatting drift. Safe.
        if before == after {
            return Ok(());
        }

        match mutation_class.severity() {
            SemanticSeverity::Harmless => Ok(()),
            SemanticSeverity::Inspectable => {
                // Internal body logic changed: Run tests, shadow mutator checks
                Ok(())
            }
            SemanticSeverity::Quarantine => {
                // Signature drift or Visibility drift
                // Isolate the agent, suspend the merge, ping the human Boss
                Err("Quarantine: Semantic Signature/Visibility Mutation requires Boss Arbitration.")
            }
            SemanticSeverity::Reject => {
                // Node broke an architectural boundary
                Err("Reject: Topology integrity compromised. Reverting immediately.")
            }
        }
    }
}
