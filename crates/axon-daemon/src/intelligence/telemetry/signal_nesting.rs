use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SignalFrame {
    pub signal_id: u64,
    pub widget_ptr: usize,
    pub depth: u32,
    pub parent_seq: Option<u64>,
}

/// 1. Signal Nesting Depth Tracker
/// Physically tracks recursive `g_signal_emit` loops. 
/// Captures the actual depth of signal reentrancy which frequently causes GTK stack overflows.
pub struct SignalNestingTracker {
    pub current_depth: u32,
    pub active_frames: Vec<SignalFrame>,
    pub max_depth_seen: u32,
}

impl SignalNestingTracker {
    pub fn new() -> Self {
        Self {
            current_depth: 0,
            active_frames: Vec::new(),
            max_depth_seen: 0,
        }
    }

    pub fn enter_signal(&mut self, signal_id: u64, widget_ptr: usize, parent_seq: Option<u64>) {
        self.current_depth += 1;
        if self.current_depth > self.max_depth_seen {
            self.max_depth_seen = self.current_depth;
        }
        self.active_frames.push(SignalFrame {
            signal_id,
            widget_ptr,
            depth: self.current_depth,
            parent_seq,
        });
    }

    pub fn exit_signal(&mut self) -> Result<(), String> {
        if self.current_depth == 0 {
            return Err("SIGNAL_UNDERFLOW_DETECTED".to_string());
        }
        self.current_depth -= 1;
        self.active_frames.pop();
        Ok(())
    }
}
