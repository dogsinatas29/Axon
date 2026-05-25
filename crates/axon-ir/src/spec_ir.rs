use crate::schema::Language;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticSpecIR {
    pub language: Language,
    pub nodes: Vec<LogicalNode>,
    pub interfaces: Vec<InterfaceContract>,
    pub transitions: Vec<StateTransition>,
    pub retry_loops: Vec<RetryLoop>,
    pub invariants: Vec<ArchitecturalInvariant>,
    pub dependencies: Vec<DependencyConstraint>,
}

impl Default for SemanticSpecIR {
    fn default() -> Self {
        Self {
            language: Language::Rust,
            nodes: Vec::new(),
            interfaces: Vec::new(),
            transitions: Vec::new(),
            retry_loops: Vec::new(),
            invariants: Vec::new(),
            dependencies: Vec::new(),
        }
    }
}

impl SemanticSpecIR {
    pub fn to_human_readable_summary(&self) -> String {
        let mut out = String::new();
        out.push_str("[AXON Semantic Interpretation]\n\n");
        
        out.push_str("Language:\n");
        out.push_str(&format!("- {:?}\n\n", self.language));
        
        out.push_str("Logical Nodes:\n");
        out.push_str(&format!("- {} detected\n\n", self.nodes.len()));
        
        out.push_str("Physical Files:\n");
        let mut files: Vec<String> = self.nodes.iter().map(|n| n.file_path.clone()).collect();
        files.sort();
        files.dedup();
        out.push_str(&format!("- {} planned\n\n", files.len()));
        
        out.push_str("Interface Contracts:\n");
        for interface in &self.interfaces {
            out.push_str(&format!("- {}\n", interface.signature));
        }
        out.push_str("\n");
        
        out.push_str("Runtime Retry Loops:\n");
        for retry in &self.retry_loops {
            out.push_str(&format!("- {} -> {}\n", retry.trigger_node, retry.target_node));
        }
        out.push_str("\n");
        
        out.push_str("External Dependencies:\n");
        for dep in &self.dependencies {
            out.push_str(&format!("- {}\n", dep.target));
        }
        out.push_str("\n");
        
        out.push_str("Architectural Constraints:\n");
        for inv in &self.invariants {
            out.push_str(&format!("- {}\n", inv.rule));
        }
        out.push_str("\n");
        
        out.push_str("Transition Graph:\nSTART\n");
        for t in &self.transitions {
            if let Some(cond) = &t.condition {
                out.push_str(&format!(" -> {} [{}]\n", t.to, cond));
            } else {
                out.push_str(&format!(" -> {}\n", t.to));
            }
        }
        
        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogicalNode {
    pub id: String,         // e.g. "INPUT_YEAR"
    pub tier: String,       // e.g. "input", "control"
    pub file_path: String,  // e.g. "src/input.rs"
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InterfaceContract {
    pub node_id: String,     // e.g. "INPUT_YEAR"
    pub symbol: String,      // e.g. "input_year"
    pub signature: String,   // e.g. "pub fn input_year() -> i32"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateTransition {
    pub from: String,       // NodeId (e.g. "VALID_YEAR")
    pub to: String,         // NodeId (e.g. "INPUT_YEAR")
    pub condition: Option<String>,
    pub kind: TransitionKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransitionKind {
    Forward,
    Conditional,
    RetryLoop,
    Rollback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryLoop {
    pub trigger_node: String, // e.g. "VALID_YEAR"
    pub target_node: String,  // e.g. "INPUT_YEAR"
    pub condition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchitecturalInvariant {
    pub id: String,
    pub rule: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyConstraint {
    pub target: String,
    pub allowed_dependencies: Vec<String>,
    pub forbidden_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalContract {
    pub semantic_ir_hash: String,
    pub transition_graph_hash: String,
    pub contract_hash: String,
    
    pub human_summary: String,
    
    pub logical_nodes_count: usize,
    pub interfaces_count: usize,
    pub transitions_count: usize,
    pub retry_loops_count: usize,
    pub dependencies: Vec<String>,
}

impl ApprovalContract {
    pub fn from_ir(ir: &SemanticSpecIR, base_hash: &str) -> Self {
        Self {
            semantic_ir_hash: base_hash.to_string(),
            transition_graph_hash: base_hash.to_string(),
            contract_hash: base_hash.to_string(),
            human_summary: ir.to_human_readable_summary(),
            logical_nodes_count: ir.nodes.len(),
            interfaces_count: ir.interfaces.len(),
            transitions_count: ir.transitions.len(),
            retry_loops_count: ir.retry_loops.len(),
            dependencies: ir.dependencies.iter().map(|d| d.target.clone()).collect(),
        }
    }
}
