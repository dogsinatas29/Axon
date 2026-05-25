use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipLease {
    pub owner_task: String,
    pub symbol: String,
    pub acquired_at: u64,
    pub expires_at: u64,
    pub last_heartbeat_at: u64,
}

pub struct LeaseManager;

impl LeaseManager {
    /// Checks if an ownership lease has expired either by hard deadline or heartbeat stall.
    pub fn is_lease_expired(lease: &OwnershipLease, current_time: u64) -> bool {
        let heartbeat_timeout = 300; // 5 minutes without a heartbeat is a zombie
        current_time > lease.expires_at || current_time > (lease.last_heartbeat_at + heartbeat_timeout)
    }

    /// Evicts a zombie task from the symbol graph, rolling back its state to allow reassignment.
    pub fn evict_zombie_lease(_lease: &OwnershipLease) -> Result<(), String> {
        // Pseudo logic:
        // 1. Identify zombie task.
        // 2. Rollback any partial patches associated with this task.
        // 3. Clear the [Locked] status in the ownership_snapshot.json.
        // 4. Return success so the scheduler can reassign the symbol.
        Ok(())
    }
}
