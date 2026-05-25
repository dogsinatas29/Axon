use serde::{Deserialize, Serialize};

/// Immutable source of truth for deterministic randomness during replay loops
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplaySeed {
    /// Seed for topology scheduler to prevent non-deterministic order
    pub scheduler_seed: u64,
    /// Seed for internal hash maps to prevent iteration variance
    pub hash_seed: u64,
    /// Deterministic temporary path prefix to avoid unique temp dir variations
    pub temp_path_seed: String,
    /// Global PRNG seed for the mutation campaign
    pub campaign_seed: u64,
}

impl ReplaySeed {
    pub fn new_deterministic(base_seed: u64) -> Self {
        Self {
            scheduler_seed: base_seed,
            hash_seed: base_seed.wrapping_add(1),
            temp_path_seed: format!("axon_replay_{}", base_seed),
            campaign_seed: base_seed.wrapping_add(2),
        }
    }
}
