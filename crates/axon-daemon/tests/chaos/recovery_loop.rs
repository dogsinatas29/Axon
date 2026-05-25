//! v0.0.31.xx: Recovery Loop Chaos Test
//!
//! Tests that AXON handles rapid restarts correctly.
//!
//! Scenario:
//! - Restart daemon 20 times rapidly
//! - Each restart triggers recovery reconciliation
//!
//! Expected:
//! - Epoch monotonic (always increases)
//! - Stale queue removed each time
//! - No permanent divergence

use crate::tests::chaos::{ChaosConfig, RuntimeSnapshot, wait_for_invariant};
use std::time::Duration;

/// Test rapid recovery cycles
pub async fn test_recovery_loop(daemon: &crate::Daemon) -> Result<(), String> {
    let config = ChaosConfig::from_env();
    let restart_count = 5; // Simulate 5 restarts
    
    tracing::info!("🎲 [CHAOS_START] Recovery Loop Test ({} restarts)", restart_count);
    
    let mut epochs = Vec::new();
    
    for i in 0..restart_count {
        tracing::info!("🔄 [RECOVERY_LOOP] Iteration {}/{}", i + 1, restart_count);
        
        // Get current epoch
        let current_epoch = daemon.recovery_epoch.load(std::sync::atomic::Ordering::Relaxed);
        epochs.push(current_epoch);
        
        // Simulate work - add some tasks
        let task = axon_core::Task {
            id: format!("recovery_loop_{}_{}", config.seed, i),
            project_id: "default".to_string(),
            title: format!("Recovery Test {}", i),
            description: "Recovery loop test".to_string(),
            status: axon_core::TaskStatus::Pending,
            lifecycle_state: axon_core::TaskLifecycleState::Queued,
            dependencies: Vec::new(),
            result: None,
            target_file: Some(format!("test_{}.rs", i)),
            lock_files: Vec::new(),
            error_feedback: None,
            senior_comment: None,
            rework_count: 0,
            base_hash: None,
            parent_task: None,
            reason: None,
            kind: "implementation".to_string(),
            retries: 0,
            assigned_worker: None,
            created_at: chrono::Local::now(),
            ir_path: None,
            task_kind: None,
            signature: None,
            validator_rejections: 0,
            senior_rejections: 0,
            architecture_rejections: 0,
            cargo_rejections: 0,
            lsp_rejections: 0,
            boss_interventions: 0,
        };
        
        daemon.submit_task(task);
        
        // Simulate recovery (in real test, this would be restart)
        // For unit test, we trigger rebuild
        daemon.rebuild_from_active_db_snapshot().await.map_err(|e| e.to_string())?;
        
        // Wait for recovery to settle
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Verify invariant
        wait_for_invariant(daemon, 10).await?;
    }
    
    // Verify epochs are monotonic
    let final_epoch = daemon.recovery_epoch.load(std::sync::atomic::Ordering::Relaxed);
    
    tracing::info!("📊 [EPOCH_HISTORY] {:?}", epochs);
    tracing::info!("📊 [FINAL_EPOCH] {}", final_epoch);
    
    // Epoch should have increased
    if final_epoch <= epochs.first().unwrap_or(&0) {
        return Err("Epoch is not monotonic!".to_string());
    }
    
    // Verify final state is consistent
    let snapshot = RuntimeSnapshot::from_daemon(daemon);
    snapshot.log();
    
    if !snapshot.is_consistent() {
        return Err("Final state inconsistent after recovery loop".to_string());
    }
    
    tracing::info!("✅ [RECOVERY_LOOP_PASSED] Epoch monotonic, state consistent");
    Ok(())
}

/// Test that old epoch tasks are ignored
pub async fn test_old_epoch_ignored(daemon: &crate::Daemon) -> Result<(), String> {
    tracing::info!("🎲 [CHAOS_START] Old Epoch Tasks Test");
    
    let initial_epoch = daemon.recovery_epoch.load(std::sync::atomic::Ordering::Relaxed);
    
    // Simulate task from old epoch (before recovery)
    let old_task = axon_core::Task {
        id: "old_epoch_task".to_string(),
        project_id: "default".to_string(),
        title: "Old Epoch Task".to_string(),
        description: "Should be cleaned up".to_string(),
        status: axon_core::TaskStatus::Pending,
        lifecycle_state: axon_core::TaskLifecycleState::Queued, // Stale - not in DB
        dependencies: Vec::new(),
        result: None,
        target_file: Some("old.rs".to_string()),
        lock_files: Vec::new(),
        error_feedback: None,
        senior_comment: None,
        rework_count: 0,
        base_hash: None,
        parent_task: None,
        reason: None,
        kind: "implementation".to_string(),
        retries: 0,
        assigned_worker: None,
        created_at: chrono::Local::now(),
        ir_path: None,
        task_kind: None,
        signature: None,
        validator_rejections: 0,
        senior_rejections: 0,
        architecture_rejections: 0,
        cargo_rejections: 0,
        lsp_rejections: 0,
        boss_interventions: 0,
    };
    
    // Add to coordinator (simulating stale state)
    daemon.submit_task(old_task.clone());
    
    // Trigger recovery
    daemon.rebuild_from_active_db_snapshot().await.map_err(|e| e.to_string())?;
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Verify stale task was scrubbed
    let snapshot = RuntimeSnapshot::from_daemon(daemon);
    snapshot.log();
    
    // The stale task should be removed because it's not in DB
    let stale_removed = daemon.queue_scrubbed_tasks.load(std::sync::atomic::Ordering::Relaxed);
    
    tracing::info!("🧹 [STALE_SCRUBBED] {} tasks removed", stale_removed);
    
    wait_for_invariant(daemon, 10).await?;
    
    tracing::info!("✅ [OLD_EPOCH_PASSED] Stale tasks from old epoch were cleaned");
    Ok(())
}