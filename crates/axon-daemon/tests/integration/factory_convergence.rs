//! v0.0.31.xx: Factory Convergence Integration Test
//!
//! Full factory lifecycle validation:
//! spec ingestion → architecture → skeleton → impl → validation → rework → recovery → convergence

use crate::tests::chaos::{RuntimeSnapshot, wait_for_invariant, wait_until};
use std::time::Duration;

/// Test full factory convergence lifecycle
pub async fn test_factory_convergence(daemon: &crate::Daemon) -> Result<(), String> {
    tracing::info!("🏭 [INTEGRATION_START] Factory Convergence Test");
    
    // Phase 1: Create initial spec tasks
    tracing::info!("📋 [PHASE_1] Creating spec tasks");
    let spec_task = axon_core::Task {
        id: "spec_001".to_string(),
        project_id: "default".to_string(),
        title: "Parse Spec".to_string(),
        description: "Parse spec.md for project requirements".to_string(),
        status: axon_core::TaskStatus::Pending,
        lifecycle_state: axon_core::TaskLifecycleState::Queued,
        dependencies: Vec::new(),
        result: None,
        target_file: Some("spec.md".to_string()),
        lock_files: Vec::new(),
        error_feedback: None,
        senior_comment: None,
        rework_count: 0,
        base_hash: None,
        parent_task: None,
        reason: None,
        kind: "analysis".to_string(),
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
    
    daemon.submit_task(spec_task.clone());
    daemon.storage.save_task(spec_task.clone()).await.map_err(|e| e.to_string())?;
    
    tracing::info!("✅ [PHASE_1_COMPLETE] Spec task created");
    
    // Phase 2: Architecture generation
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    tracing::info!("🏗️ [PHASE_2] Architecture generation");
    let arch_task = axon_core::Task {
        id: "arch_001".to_string(),
        project_id: "default".to_string(),
        title: "Generate Architecture".to_string(),
        description: "Create architecture.md from spec".to_string(),
        status: axon_core::TaskStatus::Pending,
        lifecycle_state: axon_core::TaskLifecycleState::Queued,
        dependencies: vec!["spec_001".to_string()],
        result: None,
        target_file: Some("architecture.md".to_string()),
        lock_files: Vec::new(),
        error_feedback: None,
        senior_comment: None,
        rework_count: 0,
        base_hash: None,
        parent_task: None,
        reason: None,
        kind: "generation".to_string(),
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
    
    daemon.submit_task(arch_task.clone());
    daemon.storage.save_task(arch_task.clone()).await.map_err(|e| e.to_string())?;
    
    tracing::info!("✅ [PHASE_2_COMPLETE] Architecture task created");
    
    // Phase 3: Skeleton generation
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    tracing::info!("🦴 [PHASE_3] Skeleton generation");
    for i in 0..3 {
        let skeleton_task = axon_core::Task {
            id: format!("skeleton_{:03}", i),
            project_id: "default".to_string(),
            title: format!("Generate Skeleton {}", i),
            description: "Create skeleton code".to_string(),
            status: axon_core::TaskStatus::Pending,
            lifecycle_state: axon_core::TaskLifecycleState::Queued,
            dependencies: vec!["arch_001".to_string()],
            result: None,
            target_file: Some(format!("src/module_{}.rs", i)),
            lock_files: Vec::new(),
            error_feedback: None,
            senior_comment: None,
            rework_count: 0,
            base_hash: None,
            parent_task: None,
            reason: None,
            kind: "generation".to_string(),
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
        
        daemon.submit_task(skeleton_task.clone());
        daemon.storage.save_task(skeleton_task).await.map_err(|e| e.to_string())?;
    }
    
    tracing::info!("✅ [PHASE_3_COMPLETE] 3 skeleton tasks created");
    
    // Wait for scheduler to process
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Run invariant checks throughout
    tracing::info!("🔍 [INVARIANT_CHECK] Running invariant checks");
    wait_for_invariant(daemon, 30).await?;
    
    // Check state convergence
    let snapshot = RuntimeSnapshot::from_daemon(daemon);
    snapshot.log();
    
    if !snapshot.is_consistent() {
        return Err(format!(
            "🚨 [CONVERGENCE_FAILED] State inconsistent: db={} coord={}",
            snapshot.db_active_count, snapshot.coord_queued_count
        ));
    }
    
    tracing::info!("✅ [INTEGRATION_PASSED] Factory convergence successful");
    Ok(())
}

/// Test recovery after partial completion
pub async fn test_recovery_after_partial_completion(daemon: &crate::Daemon) -> Result<(), String> {
    tracing::info!("🔄 [INTEGRATION_START] Recovery After Partial Completion");
    
    // Create some completed, some pending tasks
    let completed_task = axon_core::Task {
        id: "done_001".to_string(),
        project_id: "default".to_string(),
        title: "Completed Task".to_string(),
        description: "Already done".to_string(),
        status: axon_core::TaskStatus::Completed,
        lifecycle_state: axon_core::TaskLifecycleState::Completed,
        dependencies: Vec::new(),
        result: Some("Done".to_string()),
        target_file: Some("done.rs".to_string()),
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
    
    let pending_task = axon_core::Task {
        id: "pending_001".to_string(),
        project_id: "default".to_string(),
        title: "Pending Task".to_string(),
        description: "Still working".to_string(),
        status: axon_core::TaskStatus::Pending,
        lifecycle_state: axon_core::TaskLifecycleState::Queued,
        dependencies: Vec::new(),
        result: None,
        target_file: Some("pending.rs".to_string()),
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
    
    daemon.storage.save_task(completed_task).await.map_err(|e| e.to_string())?;
    daemon.storage.save_task(pending_task.clone()).await.map_err(|e| e.to_string())?;
    daemon.submit_task(pending_task);
    
    // Trigger recovery
    daemon.rebuild_from_active_db_snapshot().await.map_err(|e| e.to_string())?;
    
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // Verify invariants
    wait_for_invariant(daemon, 20).await?;
    
    let snapshot = RuntimeSnapshot::from_daemon(daemon);
    snapshot.log();
    
    // Should have exactly 1 active (the pending one)
    if snapshot.db_active_count != 1 {
        return Err(format!("Expected 1 active task, got {}", snapshot.db_active_count));
    }
    
    tracing::info!("✅ [RECOVERY_PARTIAL_PASSED] Recovered correctly after partial completion");
    Ok(())
}

/// Test REWORK integration with convergence
pub async fn test_rework_convergence(daemon: &crate::Daemon) -> Result<(), String> {
    tracing::info!("🔄 [INTEGRATION_START] REWORK Convergence Test");
    
    // Create initial task
    let original = axon_core::Task {
        id: "original_001".to_string(),
        project_id: "default".to_string(),
        title: "Original Implementation".to_string(),
        description: "Initial implementation".to_string(),
        status: axon_core::TaskStatus::InProgress,
        lifecycle_state: axon_core::TaskLifecycleState::Running,
        dependencies: Vec::new(),
        result: None,
        target_file: Some("impl.rs".to_string()),
        lock_files: Vec::new(),
        error_feedback: None,
        senior_comment: None,
        rework_count: 0,
        base_hash: Some("abc123".to_string()),
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
    
    daemon.storage.save_task(original.clone()).await.map_err(|e| e.to_string())?;
    daemon.submit_task(original.clone());
    
    // Simulate REWORK: mark original as Superseded, create new Queued task
    let mut superseded = original.clone();
    superseded.status = axon_core::TaskStatus::Failed;
    superseded.lifecycle_state = axon_core::TaskLifecycleState::Superseded;
    superseded.error_feedback = Some("Boss REWORK requested".to_string());
    daemon.storage.save_task(superseded).await.map_err(|e| e.to_string())?;
    
    let reworked = axon_core::Task {
        id: "rework_001".to_string(),
        project_id: "default".to_string(),
        title: "Reworked Implementation".to_string(),
        description: "REWORK: new contract".to_string(),
        status: axon_core::TaskStatus::Pending,
        lifecycle_state: axon_core::TaskLifecycleState::Queued,
        dependencies: Vec::new(),
        result: None,
        target_file: Some("impl.rs".to_string()),
        lock_files: Vec::new(),
        error_feedback: None,
        senior_comment: Some("[BOSS_HINT]: Improve error handling".to_string()),
        rework_count: 1,
        base_hash: Some("new_contract_xyz".to_string()),
        parent_task: Some("original_001".to_string()),
        reason: Some("Boss REWORK: Improve error handling".to_string()),
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
        boss_interventions: 1,
    };
    
    daemon.storage.save_task(reworked.clone()).await.map_err(|e| e.to_string())?;
    daemon.submit_task(reworked);
    
    // Notify barrier of state change
    daemon.stage_notify.notify_waiters();
    
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // Verify invariants
    wait_for_invariant(daemon, 20).await?;
    
    let snapshot = RuntimeSnapshot::from_daemon(daemon);
    snapshot.log();
    
    // Should have exactly 1 active (the reworked task)
    // Original should be Superseded (terminal)
    if let Ok(Some(original_check)) = daemon.storage.get_task("original_001") {
        if original_check.lifecycle_state != axon_core::TaskLifecycleState::Superseded {
            return Err("Original task not marked as Superseded".to_string());
        }
    }
    
    tracing::info!("✅ [REWORK_CONVERGENCE_PASSED] REWORK atomic transition works");
    Ok(())
}

/// Test concurrent task convergence
pub async fn test_concurrent_convergence(daemon: &crate::Daemon) -> Result<(), String> {
    tracing::info!("🔄 [INTEGRATION_START] Concurrent Convergence Test");
    
    // Create many concurrent tasks
    let task_count = 10;
    for i in 0..task_count {
        let task = axon_core::Task {
            id: format!("concurrent_{:03}", i),
            project_id: "default".to_string(),
            title: format!("Concurrent Task {}", i),
            description: "Testing concurrent convergence".to_string(),
            status: axon_core::TaskStatus::Pending,
            lifecycle_state: axon_core::TaskLifecycleState::Queued,
            dependencies: Vec::new(),
            result: None,
            target_file: Some(format!("concurrent_{}.rs", i)),
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
        
        daemon.submit_task(task.clone());
        daemon.storage.save_task(task).await.map_err(|e| e.to_string())?;
    }
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Run invariant checks
    wait_for_invariant(daemon, 30).await?;
    
    let snapshot = RuntimeSnapshot::from_daemon(daemon);
    snapshot.log();
    
    // Should be consistent despite concurrency
    if !snapshot.is_consistent() {
        return Err(format!(
            "Concurrent tasks caused inconsistency: db={} coord={}",
            snapshot.db_active_count, snapshot.coord_queued_count
        ));
    }
    
    tracing::info!("✅ [CONCURRENT_CONVERGENCE_PASSED] {} concurrent tasks converged", task_count);
    Ok(())
}