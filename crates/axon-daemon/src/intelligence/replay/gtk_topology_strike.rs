use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum GtkTopologyViolation {
    SignalTopologyViolation(String),
    GtkWidgetLifecycleViolation(String),
    MacroContamination(String),
}

/// GTK Runtime Topology Gate
/// Enforces GTK-specific lifecycle, ownership, and signal topology invariants.
pub struct GtkTopologyGate {
    pub widget_owners: HashMap<String, String>,
    pub active_signals: HashMap<String, Vec<String>>,
}

impl GtkTopologyGate {
    pub fn new() -> Self {
        Self {
            widget_owners: HashMap::new(),
            active_signals: HashMap::new(),
        }
    }

    pub fn register_widget(&mut self, widget_id: &str, owner: &str) {
        self.widget_owners.insert(widget_id.to_string(), owner.to_string());
    }

    /// STAGE 2: Signal Topology Corruption detection
    /// Ensures that an AI task cannot maliciously or accidentally re-wire signals on unowned widgets.
    pub fn attempt_signal_connect(&mut self, widget: &str, signal: &str, task_id: &str) -> Result<(), GtkTopologyViolation> {
        if let Some(owner) = self.widget_owners.get(widget) {
            if owner != task_id {
                return Err(GtkTopologyViolation::SignalTopologyViolation(
                    format!("SIGNAL_TOPOLOGY_VIOLATION: Task '{}' attempted to wire signal '{}' on widget owned by '{}'", task_id, signal, owner)
                ));
            }
        } else {
             return Err(GtkTopologyViolation::SignalTopologyViolation(
                format!("SIGNAL_TOPOLOGY_VIOLATION: Attempted to wire signal on unregistered widget '{}'", widget)
            ));
        }
        self.active_signals.entry(widget.to_string()).or_default().push(signal.to_string());
        Ok(())
    }

    /// STAGE 3: GTK Runtime Lifecycle Validation
    /// Ensures that widget destruction follows strict ownership rules to prevent UAF or Segfaults.
    pub fn attempt_widget_destroy(&mut self, widget: &str, task_id: &str) -> Result<(), GtkTopologyViolation> {
        if let Some(owner) = self.widget_owners.get(widget) {
            if owner != task_id {
                return Err(GtkTopologyViolation::GtkWidgetLifecycleViolation(
                    format!("GTK_WIDGET_LIFECYCLE_VIOLATION: Task '{}' attempted to destroy widget owned by '{}'", task_id, owner)
                ));
            }
        } else {
            return Err(GtkTopologyViolation::GtkWidgetLifecycleViolation(
                "GTK_WIDGET_LIFECYCLE_VIOLATION: Attempted to destroy unknown or unowned widget".to_string()
            ));
        }
        Ok(())
    }
    
    /// STAGE 2: Macro Contamination (Shadow Macro)
    pub fn attempt_shadow_macro(&self, macro_name: &str, task_id: &str) -> Result<(), GtkTopologyViolation> {
        // Enforce that fundamental GTK type macros cannot be redefined/shadowed.
        if macro_name.starts_with("GTK_") || macro_name.starts_with("G_") {
            return Err(GtkTopologyViolation::MacroContamination(
                format!("MACRO_CONTAMINATION: Task '{}' attempted to shadow core GTK macro '{}'", task_id, macro_name)
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gtk_signal_topology() {
        let mut gate = GtkTopologyGate::new();
        gate.register_widget("btn_submit", "Task_A");

        // Task B tries to wire a signal to Task A's widget
        let result = gate.attempt_signal_connect("btn_submit", "clicked", "Task_B");
        assert!(matches!(result, Err(GtkTopologyViolation::SignalTopologyViolation(_))));
    }

    #[test]
    fn test_gtk_widget_lifecycle() {
        let mut gate = GtkTopologyGate::new();
        gate.register_widget("window_main", "Task_A");

        // Task B tries to destroy Task A's window
        let result = gate.attempt_widget_destroy("window_main", "Task_B");
        assert!(matches!(result, Err(GtkTopologyViolation::GtkWidgetLifecycleViolation(_))));
    }

    #[test]
    fn test_gtk_macro_contamination() {
        let gate = GtkTopologyGate::new();
        
        // Task tries to redefine a core GTK cast macro
        let result = gate.attempt_shadow_macro("GTK_WIDGET", "Task_A");
        assert!(matches!(result, Err(GtkTopologyViolation::MacroContamination(_))));
    }
}
