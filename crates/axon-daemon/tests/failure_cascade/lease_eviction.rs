#[cfg(test)]
mod tests {
    use axon_daemon::governance::lease::{OwnershipLease, LeaseManager};

    #[test]
    fn test_lease_zombie_eviction() {
        let current_time = 1000;
        let mut lease = OwnershipLease {
            owner_task: "task_1".to_string(),
            symbol: "core_func".to_string(),
            acquired_at: 0,
            expires_at: 2000,
            last_heartbeat_at: 0,
        };

        // Assert not expired yet
        assert!(!LeaseManager::is_lease_expired(&lease, current_time));

        // Advance time to simulate zombie (5 min heartbeat timeout is 300)
        let zombie_time = 1301;
        assert!(LeaseManager::is_lease_expired(&lease, zombie_time));

        // Evict
        let eviction_result = LeaseManager::evict_zombie_lease(&lease);
        assert!(eviction_result.is_ok());

        // Pseudo: Verify no double ownership and topology locks cleared
    }
}
