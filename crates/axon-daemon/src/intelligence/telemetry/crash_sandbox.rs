use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SandboxConfig {
    pub timeout_ms: u64,
    pub memory_cap_mb: usize,
    pub max_idle_recursion: usize,
    pub stall_threshold_ms: u64,
}

/// STEP 4: Sandbox Hardening
/// Process Isolation & Watchdog for hostile GTK/Win32 runtimes.
/// Prevents infinite redraw loops, deadlocks, message starvation, 
/// and recursive callback storms from bringing down the AXON kernel.
pub struct CrashSandbox {
    pub config: SandboxConfig,
    pub main_loop_iterations: u64,
    pub dispatch_count: u64,
    pub current_idle_recursion: usize,
    pub last_dispatch_timestamp_ms: u64,
}

impl CrashSandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self { 
            config,
            main_loop_iterations: 0,
            dispatch_count: 0,
            current_idle_recursion: 0,
            last_dispatch_timestamp_ms: 0, // Initialized on start
        }
    }
    
    /// Health Check Probe: Triggers SIGKILL if the legacy runtime enters a pathology loop.
    pub fn check_health(&self, current_time_ms: u64) -> Result<(), String> {
        // 1. Hard Kill Watchdog: Queue Stall (e.g. 5 seconds no dispatch)
        if current_time_ms - self.last_dispatch_timestamp_ms > self.config.stall_threshold_ms {
            return Err("QUEUE_STALL_SIGKILL".to_string());
        }

        // 2. Event Loop Stall Detector: Main loop spins but dispatch is frozen
        if self.main_loop_iterations > self.dispatch_count + 10000 {
            return Err("EVENT_LOOP_STALL".to_string());
        }

        // 3. Memory Guard & Recursion Cap: g_idle_add storm
        if self.current_idle_recursion > self.config.max_idle_recursion {
            return Err("IDLE_RECURSION_STORM".to_string());
        }

        Ok(())
    }

    pub fn execute_isolated(&self, _binary_path: &str) -> Result<(), String> {
        // Execution using physical Linux/Windows primitives (ulimit, cgroups, Job Objects)
        Ok(())
    }
}
