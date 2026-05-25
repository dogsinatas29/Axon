use serde::{Deserialize, Serialize};

/// Represents the evolutionary linkage between an AXON policy update and a new failure pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureLineage {
    pub lineage_id: String,
    
    /// The cluster of failures (fingerprint) being tracked
    pub failure_fingerprint: String,

    /// Which policy version or configuration change triggered this?
    pub causal_policy_version: String,
    
    /// Description of the policy change that is suspected
    pub policy_delta_description: String,

    /// Is this a verified regression?
    pub verified_regression: bool,
}

impl FailureLineage {
    pub fn new(
        fingerprint: String,
        policy_version: String,
        description: String,
    ) -> Self {
        Self {
            lineage_id: format!("{}-{}", policy_version, fingerprint),
            failure_fingerprint: fingerprint,
            causal_policy_version: policy_version,
            policy_delta_description: description,
            verified_regression: false,
        }
    }
}
