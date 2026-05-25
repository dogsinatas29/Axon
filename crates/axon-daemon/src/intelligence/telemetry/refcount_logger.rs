use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum RefcountEvent {
    Ref,
    Unref,
    FloatingSink,
    Dispose,
    Finalize,
    DestroyWithoutStabilization, // Crucial GTK physical pathology
}

#[derive(Debug, Serialize, Clone)]
pub struct RefcountTransition {
    pub widget_ptr: usize,
    pub event: RefcountEvent,
    pub count_after: i32,
    pub timestamp_ms: u64,
}

/// 2. Refcount Transition Logger
/// Logs the raw, physical `GObject` refcount mutations to detect leaks,
/// use-after-free, and floating reference collapses.
pub struct RefcountLogger {
    pub transitions: Vec<RefcountTransition>,
}

impl RefcountLogger {
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
        }
    }

    pub fn log_transition(&mut self, ptr: usize, event: RefcountEvent, count_after: i32, time: u64) -> Result<(), String> {
        self.transitions.push(RefcountTransition {
            widget_ptr: ptr,
            event: event.clone(),
            count_after,
            timestamp_ms: time,
        });

        if event == RefcountEvent::DestroyWithoutStabilization {
            // Immediate red flag for legacy GTK C code
            return Err("DESTROY_WITHOUT_REF_STABILIZATION".to_string());
        }
        Ok(())
    }
}
