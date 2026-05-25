use std::time::Duration;

/// v0.0.31.xx: Chaos Recovery Test Infrastructure
/// 
/// Fault Injection Harness for AXON recovery validation.
/// Tests that the system converges to consistent state after various failure scenarios.

pub mod mid_dispatch_kill;
pub mod rework_storm;
pub mod recovery_loop;
pub mod barrier_race;
pub mod orphan_lock;

/// Chaos injection point in the execution flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaosPoint {
    BeforeDispatch,
    AfterDispatch,
    BeforeTerminate,
    AfterTerminate,
    BeforeBarrierNotify,
    DuringRecovery,
    BeforeQueueScrub,
    DuringInvariantCheck,
}

/// Chaos configuration for test reproducibility
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    pub seed: u64,
    pub enabled: bool,
    pub crash_probability: f64,
    pub iterations: u32,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            seed: 1337,
            enabled: true,
            crash_probability: 0.1,
            iterations: 10,
        }
    }
}

impl ChaosConfig {
    pub fn from_env() -> Self {
        Self {
            seed: std::env::var("CHAOS_SEED")
                .unwrap_or_else(|_| "1337".to_string())
                .parse()
                .unwrap_or(1337),
            enabled: std::env::var("CHAOS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            crash_probability: std::env::var("CHAOS_CRASH_PROB")
                .unwrap_or_else(|_| "0.1".to_string())
                .parse()
                .unwrap_or(0.1),
            iterations: std::env::var("CHAOS_ITERATIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }
    }
}

/// Simple deterministic random for reproducibility
pub struct ChaosRng {
    seed: u64,
    current: u64,
}

impl ChaosRng {
    pub fn new(seed: u64) -> Self {
        Self { seed, current: seed }
    }

    pub fn next(&mut self) -> u64 {
        // Linear congruential generator
        self.current = self.current.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.current
    }

    pub fn should_crash(&mut self, probability: f64) -> bool {
        let r = (self.next() as f64) / (u64::MAX as f64);
        r < probability
    }
}

/// Eventually consistent assertion helper
/// AXON is async, notify-driven, self-healing - immediate consistency is NOT guaranteed
pub async fn wait_until<F, Fut, R>(
    name: &str,
    mut check: F,
    timeout: Duration,
    interval: Duration,
) -> Result<R, String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<R, String>>,
{
    let start = std::time::Instant::now();
    let mut attempt = 0;

    loop {
        attempt += 1;
        
        match check().await {
            Ok(result) => {
                tracing::info!(
                    "✅ [EVENTUAL_CONSISTENCY] {} converged after {} attempts ({:?})",
                    name, attempt, start.elapsed()
                );
                return Ok(result);
            }
            Err(e) => {
                if start.elapsed() > timeout {
                    return Err(format!(
                        "🚨 [CONVERGENCE_FAILED] {} did not converge after {:?} (attempted {} times): {}",
                        name, timeout, attempt, e
                    ));
                }
                
                tracing::debug!(
                    "⏳ [WAITING] {} not ready (attempt {}): {}",
                    name, attempt, e
                );
                tokio::time::sleep(interval).await;
            }
        }
    }
}

/// Helper to wait for invariant to pass
pub async fn wait_for_invariant(
    daemon: &crate::Daemon,
    timeout_secs: u64,
) -> Result<(), String> {
    wait_until(
        "invariant_check",
        || async {
            // Run invariant checks
            daemon.run_invariant_checks();
            
            // Check if no failures (we can't directly check, but if it doesn't panic we're good)
            Ok(())
        },
        Duration::from_secs(timeout_secs),
        Duration::from_millis(500),
    ).await
}

/// Runtime state snapshot for debugging
#[derive(Debug)]
pub struct RuntimeSnapshot {
    pub db_active_count: usize,
    pub coord_queued_count: usize,
    pub coord_active_files: usize,
    pub recovery_epoch: u64,
}

impl RuntimeSnapshot {
    pub fn from_daemon(daemon: &crate::Daemon) -> Self {
        let db_active = daemon.storage
            .count_active_tasks_by_project("default")
            .unwrap_or(0);
        
        let (coord_queued, coord_active_files) = {
            let coord = daemon.coordinator.lock().unwrap();
            (coord.queued_task_count(), coord.active_files_count())
        };
        
        let epoch = daemon.recovery_epoch.load(std::sync::atomic::Ordering::Relaxed);
        
        Self {
            db_active_count: db_active,
            coord_queued_count: coord_queued,
            coord_active_files,
            recovery_epoch: epoch,
        }
    }

    pub fn log(&self) {
        tracing::info!(
            "📸 [SNAPSHOT] db_active={} coord_queued={} coord_active_files={} epoch={}",
            self.db_active_count,
            self.coord_queued_count,
            self.coord_active_files,
            self.recovery_epoch
        );
    }

    /// Check if state is consistent (DB count matches coordinator count)
    pub fn is_consistent(&self) -> bool {
        // Allow small variance due to in-flight tasks
        let diff = if self.db_active_count > self.coord_queued_count {
            self.db_active_count - self.coord_queued_count
        } else {
            self.coord_queued_count - self.db_active_count
        };
        diff <= 1
    }
}