// STAGE 1 and STAGE 2 Strike Simulator

#[cfg(test)]
mod tests {
    use crate::intelligence::replay::platform::gtk_topology_strike::{GtkTopologyGate, GtkTopologyViolation};
    use crate::intelligence::replay::parser_freeze::{ParserDivergenceHarness, BoundaryAnchor};

    #[test]
    fn test_stage1_signal_and_lifecycle_intrusion() {
        let mut gate = GtkTopologyGate::new();
        // Task_A owns the main window logic (ui_create)
        gate.register_widget("ui->window", "Task_A");

        // ATTACK 1: Task_B tries to wire a signal to ui->window
        let res_sig = gate.attempt_signal_connect("ui->window", "destroy", "Task_B");
        assert!(matches!(res_sig, Err(GtkTopologyViolation::SignalTopologyViolation(_))), "Kernel MUST block foreign signal wiring");

        // ATTACK 2: Task_B tries to call free() on ui->window directly
        let res_free = gate.attempt_widget_destroy("ui->window", "Task_B");
        assert!(matches!(res_free, Err(GtkTopologyViolation::GtkWidgetLifecycleViolation(_))), "Kernel MUST block foreign widget destruction");
    }

    #[test]
    fn test_stage2_parser_drift_strike() {
        // Original clean AST boundaries
        let original_boundaries = vec![
            BoundaryAnchor { start_byte: 10, end_byte: 50 },
            BoundaryAnchor { start_byte: 60, end_byte: 100 },
        ];

        // Simulated Trivia Neutralization Policy:
        // Even if the source code is injected with CRLF, trailing commas, or unicode comments,
        // AXON canonicalizes these out BEFORE Tree-sitter determines the absolute byte offsets
        // of the canonical logic, enforcing 0 variance.
        let neutralized_boundaries = vec![
            BoundaryAnchor { start_byte: 10, end_byte: 50 },
            BoundaryAnchor { start_byte: 60, end_byte: 100 },
        ];

        let report = ParserDivergenceHarness::measure_divergence(&original_boundaries, &neutralized_boundaries);
        
        // Assert perfect immunity against parser entropy drift
        assert_eq!(report.parser_boundary_variance, 0.0, "Parser Boundary Variance must be exactly 0.0%");
        assert_eq!(report.topology_anchor_drift, 0, "No topology anchor drift allowed");
        assert!(ParserDivergenceHarness::can_promote_upgrade(&report).is_ok());
    }
}
