use super::symbol_graph::SymbolDependencyGraph;

pub struct TopologyDeltaValidator;

impl TopologyDeltaValidator {
    pub fn validate_delta(before: &SymbolDependencyGraph, after: &SymbolDependencyGraph, symbol: &str) -> Result<(), String> {
        let before_deps = before.outgoing.get(symbol).cloned().unwrap_or_default();
        let after_deps = after.outgoing.get(symbol).cloned().unwrap_or_default();

        let mut removed_edges = Vec::new();
        let mut added_edges = Vec::new();

        for dep in &before_deps {
            if !after_deps.contains(dep) {
                removed_edges.push(dep.clone());
            }
        }

        for dep in &after_deps {
            if !before_deps.contains(dep) {
                added_edges.push(dep.clone());
            }
        }

        if !removed_edges.is_empty() || !added_edges.is_empty() {
            return Err(format!(
                "Topology Delta Detected for '{}': removed {:?}, added {:?}",
                symbol, removed_edges, added_edges
            ));
        }

        Ok(())
    }
}
