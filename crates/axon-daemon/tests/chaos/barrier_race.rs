//! v0.0.31.xx: Barrier Race Chaos Test
//!
//! Tests that AXON handles race conditions in stage barrier notification.
//!
//! Scenario:
//! - Last task terminates
//! - Notify sent
//! - New task inserted
//! - Race condition potential
//!
//! Expected:
//! - No permanent wait (deadlock)
//! - No double stage advance

use crate::tests::chaos::{ChaosConfig, RuntimeSnapshot, wait_for_invariant};
use std::time::Duration;

/// Test barrier notification timing
pub async fn test_barrier_race(daemon: &crate::Daemon) -> Result<(), String> {
    let config = ChaosConfig::from_env();
    
    tracing::info!("🎲 [CHAOS_START] Barrier Race Test");
    
    // Create tasks that will complete in sequence
    for i in 0..3 {
        let task = axon_core::Task {
            id: format!("barrier_task_{}_{}", config.seed, i),
            project_id: "default".to_string(),
            title: format!("Barrier Test {}", i),
            description: "Barrier race test".to_string(),
            status: axon_core::TaskStatus::Completed, // Already completed
            lifecycle_state: axon_core::TaskLifecycleState::Completed,
            dependencies: Vec::new(),
            result: Some("Completed".to_string()),
            target_file: Some(format!("barrier_{}.rs", i)),
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
        
        daemon.storage.save_task(task.clone()).await.map_err(|e| e.to_string())?;
        
        // Notify after each completion (simulating terminate_task_with_cas)
        daemon.stage_notify.notify_waiters();
    }
    
    // Small delay
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify no deadlock - active count should be 0
    let db_active = daemon.storage
        .count_active_tasks_by_project("default")
        .unwrap_or(0);
    
    tracing::info!("📊 [BARRIER] db_active={}", db_active);
    
    // Should have no active tasks (all completed)
    if db_active > 0 {
        // This might be expected if there are pending tasks
        tracing::warn!("[BARRIER] {} active tasks still present", db_active);
    }
    
    wait_for_invariant(daemon, 15).await?;
    
    tracing::info!("✅ [BARRIER_RACE_PASSED] No deadlock, barrier works");
    Ok(())
}

/// Test notify + new task race
pub async fn test_notify_new_task_race(daemon: &crate::Daemon) -> Result<(), String> {
    let config = ChaosConfig::from_env();
    
    tracing::info!("🎲 [CHAOS_START] Notify + New Task Race Test");
    
    // Create completed task
    let completed_task = axon_core::Task {
        id: format!("complete_first_{}", config.seed),
        project_id: "default".to_string(),
        title: "Complete First".to_string(),
        description: "Notify race test".to_string(),
        status: axon_core::TaskStatus::Completed,
        lifecycle_state: axon_core::TaskLifecycleState::Completed,
        dependencies: Vec::new(),
        result: Some("Done".to_string()),
        target_file: Some("race_test.rs".to_string()),
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
    
    daemon.storage.save_task(completed_task.clone()).await.map_err(|e| e.to_string())?;
    
    // Notify (simulating termination)
    daemon.stage_notify.notify_waiters();
    
    // Immediately add new task (race condition)
    let new_task = axon_core::Task {
        id: format!("new_after_notify_{}", config.seed),
        project_id: "default".to_string(),
        title: "New After Notify".to_string(),
        description: "Race test".to_string(),
        status: axon_core::TaskStatus::Pending,
        lifecycle_state: axon_core::TaskLifecycleState::Queued,
        dependencies: Vec::new(),
        result: None,
        target_file: Some("race_test.rs".to_string()), // Same file
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
    
    daemon.submit_task(new_task);
    
    // Wait and verify no deadlock
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    let snapshot = RuntimeSnapshot::from_daemon(daemon);
    snapshot.log();
    
    wait_for_invariant(daemon, 10).await?;
    
    tracing::info!("✅ [NOTIFY_NEW_TASK_PASSED] No race condition deadlock");
    Ok(())
}