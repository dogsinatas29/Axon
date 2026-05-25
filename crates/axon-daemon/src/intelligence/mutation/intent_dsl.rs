use serde::Serialize;

/// 1. Mutation Intent Layer (Topology Mutation DSL)
/// Declaratively specifies the topological intent of the mutation before any code is written.
/// This prevents unconstrained abstraction additions or architectural hallucination by LLMs.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum TopologyMutationIntent {
    AddIdleCallback {
        target_flow: String,
        owner_widget_ptr: usize,
        queue_class: String,
    },
    AttachSignal {
        from_widget_ptr: usize,
        signal_name: String,
        to_handler_flow: String,
    },
    AddTimeout {
        target_flow: String,
        owner_widget_ptr: usize,
        interval_ms: u32,
    }
}
