//! v0.0.31.xx: Orphan Lock Chaos Test
//!
//! Tests that AXON handles orphaned active_files correctly.
//!
//! Scenario:
//! - Task is in active_files
//! - Task terminates (or crashes) without calling complete_task()
//! - active_file remains orphaned
//!
//! Expected:
//! - scrub_orphaned_active_files() removes orphan locks
//! - No permanent file lock starvation

use crate::tests::chaos::{ChaosConfig, RuntimeSnapshot, wait_for_invariant};
use std::time::Duration;

/// Test orphan active_file cleanup
pub async fn test_orphan_lock(daemon: &crate::Daemon) -> Result<(), String> {
    let config = ChaosConfig::from_env();
    
    tracing::info!("🎲 [CHAOS_START] Orphan Lock Test");
    
    // Manually add an orphan active_file to coordinator
    // (simulating crash before complete_task was called)
    {
        let mut coord = daemon.coordinator.lock().unwrap();
        coord.active_files.insert("orphan_file.rs".to_string());
        tracing::info!("🔒 [TEST] Added orphan lock: orphan_file.rs");
    }
    
    // Verify it's there
    let before = {
        let coord = daemon.coordinator.lock().unwrap();
        coord.active_files_count()
    };
    tracing::info!("📊 [BEFORE_SCRUB] active_files={}", before);
    
    // Run scrubber
    let removed = daemon.scrub_orphaned_active_files();
    tracing::info!("🧹 [SCRUB_RESULT] removed={} orphan files", removed);
    
    // Verify it's gone
    let after = {
        let coord = daemon.coordinator.lock().unwrap();
        coord.active_files_count()
    };
    tracing::info!("📊 [AFTER_SCRUB] active_files={}", after);
    
    // Check metrics
    let orphan_count = daemon.orphan_active_files_removed.load(std::sync::atomic::Ordering::Relaxed);
    tracing::info!("📊 [METRICS] orphan_active_files_removed={}", orphan_count);
    
    if removed > 0 && after < before {
        tracing::info!("✅ [ORPHAN_LOCK_PASSED] Scrubber removed orphan locks");
    } else {
        return Err("Scrubber failed to remove orphan locks".to_string());
    }
    
    wait_for_invariant(daemon, 10).await?;
    
    Ok(())
}

/// Test multiple orphan locks
pub async fn test_multiple_orphan_locks(daemon: &crate::Daemon) -> Result<(), String> {
    let config = ChaosConfig::from_env();
    
    tracing::info!("🎲 [CHAOS_START] Multiple Orphan Locks Test");
    
    // Add multiple orphan active_files
    {
        let mut coord = daemon.coordinator.lock().unwrap();
        coord.active_files.insert("orphan1.rs".to_string());
        coord.active_files.insert("orphan2.rs".to_string());
        coord.active_files.insert("orphan3.rs".to_string());
    }
    
    tracing::info!("🔒 [TEST] Added 3 orphan locks");
    
    // Run scrubber
    let removed = daemon.scrub_orphaned_active_files();
    
    if removed != 3 {
        return Err(format!("Expected to remove 3 orphan locks, got {}", removed));
    }
    
    // Verify all gone
    let after = {
        let coord = daemon.coordinator.lock().unwrap();
        coord.active_files_count()
    };
    
    if after != 0 {
        return Err(format!("Expected 0 active_files after scrub, got {}", after));
    }
    
    let orphan_count = daemon.orphan_active_files_removed.load(std::sync::atomic::Ordering::Relaxed);
    tracing::info!("📊 [METRICS] total orphan files removed={}", orphan_count);
    
    wait_for_invariant(daemon, 10).await?;
    
    tracing::info!("✅ [MULTIPLE_ORPHAN_PASSED] All orphan locks removed");
    Ok(())
}

/// Test active_file with stale DB task
pub async fn test_stale_db_task_active_file(daemon: &crate::Daemon) -> Result<(), String> {
    let config = ChaosConfig::from_env();
    
    tracing::info!("🎲 [CHAOS_START] Stale DB Task + Active File Test");
    
    // Create task in DB as terminal (Completed)
    let stale_task = axon_core::Task {
        id: format!("stale_task_{}", config.seed),
        project_id: "default".to_string(),
        title: "Stale Task".to_string(),
        description: "Already completed but still in queue".to_string(),
        status: axon_core::TaskStatus::Completed,
        lifecycle_state: axon_core::TaskLifecycleState::Completed, // Terminal!
        dependencies: Vec::new(),
        result: Some("Done".to_string()),
        target_file: Some("stale_file.rs".to_string()),
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
    
    // Save to DB as completed
    daemon.storage.save_task(stale_task.clone()).await.map_err(|e| e.to_string())?;
    
    // Also add to coordinator queue (stale)
    daemon.submit_task(stale_task.clone());
    
    // Add active_file for this task (simulating crash during execution)
    {
        let mut coord = daemon.coordinator.lock().unwrap();
        coord.active_files.insert("stale_file.rs".to_string());
    }
    
    // Run both scrubbers
    let queue_removed = daemon.validate_and_scrub_stale_tasks();
    let file_removed = daemon.scrub_orphaned_active_files();
    
    tracing::info!(
        "🧹 [SCRUB_RESULT] queue_removed={} file_removed={}",
        queue_removed, file_removed
    );
    
    // Verify stale task and orphan file are gone
    let snapshot = RuntimeSnapshot::from_daemon(daemon);
    snapshot.log();
    
    wait_for_invariant(daemon, 10).await?;
    
    tracing::info!("✅ [STALE_DB_TASK_PASSED] Stale task and orphan file cleaned");
    Ok(())
}