use serde::Serialize;
use std::collections::VecDeque;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum QueueKind {
    Idle,
    Timeout,
    SignalEmit,
}

/// A physical, canonical runtime event in the GTK/Win32 event queue.
#[derive(Debug, Serialize, Clone)]
pub struct QueueEvent {
    pub seq: u64,
    pub callback_ptr: usize,
    pub widget_ptr: usize,
    pub queue_kind: QueueKind,
    pub ownership_anchor: u64,
    pub enqueue_tick: u64,
    pub dispatch_tick: Option<u64>,
}

/// STEP 3: Queue Drift Detector
/// Detects event queue ordering drift which is the true root cause of legacy GUI collapses.
pub struct QueueDriftDetector {
    pub active_queue: VecDeque<QueueEvent>,
    pub dispatched_sequence: Vec<QueueEvent>,
    pub current_tick: u64,
}

impl QueueDriftDetector {
    pub fn new() -> Self {
        Self {
            active_queue: VecDeque::new(),
            dispatched_sequence: Vec::new(),
            current_tick: 0,
        }
    }

    /// Captures the actual enqueue bytes (latency, pointer, ownership)
    pub fn register_enqueue(&mut self, callback_ptr: usize, widget_ptr: usize, kind: QueueKind, anchor: u64) -> u64 {
        self.current_tick += 1;
        let seq = self.current_tick;
        self.active_queue.push_back(QueueEvent {
            seq,
            callback_ptr,
            widget_ptr,
            queue_kind: kind,
            ownership_anchor: anchor,
            enqueue_tick: self.current_tick,
            dispatch_tick: None,
        });
        seq
    }

    /// Validates dispatch ordering. Any inversion triggers QUEUE_ORDERING_DRIFT.
    pub fn register_dispatch(&mut self, callback_ptr: usize) -> Result<(), String> {
        self.current_tick += 1;
        
        if let Some(pos) = self.active_queue.iter().position(|e| e.callback_ptr == callback_ptr) {
            // Enqueue(A) -> Enqueue(B) -> Dispatch(B) -> Dispatch(A) == DRIFT
            if pos != 0 {
                return Err("QUEUE_ORDERING_DRIFT".to_string());
            }
            let mut event = self.active_queue.remove(pos).unwrap();
            event.dispatch_tick = Some(self.current_tick);
            self.dispatched_sequence.push(event);
            Ok(())
        } else {
            Err("ORPHAN_DISPATCH_DETECTED".to_string())
        }
    }
}
