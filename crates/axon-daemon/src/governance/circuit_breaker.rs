use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchFingerprint {
    pub topology_delta_hash: String,
    pub signature_delta_hash: String,
    pub ownership_delta_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureBudget {
    pub symbol: String,
    pub max_attempts: usize,
    pub attempts: usize,
    pub cooldown_seconds: u64,
    pub last_attempt_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CircuitState {
    Healthy,
    CoolingDown,
    EscalatedToHuman,
}

impl FailureBudget {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            max_attempts: 5,
            attempts: 0,
            cooldown_seconds: 300,
            last_attempt_at: 0,
        }
    }

    pub fn record_attempt(&mut self, _fingerprint: &PatchFingerprint) -> CircuitState {
        // If the fingerprint is identical to previous failed attempts, we could count it double.
        // For now, we increment linearly.
        self.attempts += 1;
        self.last_attempt_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        if self.attempts >= self.max_attempts {
            CircuitState::EscalatedToHuman
        } else {
            CircuitState::Healthy
        }
    }
}
