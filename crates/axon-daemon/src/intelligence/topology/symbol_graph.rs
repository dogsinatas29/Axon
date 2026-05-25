use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type SymbolId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolNode {
    pub symbol: String,
    pub owner_task_id: Option<String>,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SymbolDependencyGraph {
    // caller -> callees
    pub outgoing: HashMap<SymbolId, Vec<SymbolId>>,

    // callee -> callers
    pub incoming: HashMap<SymbolId, Vec<SymbolId>>,

    pub nodes: HashMap<SymbolId, SymbolNode>,
}

impl SymbolDependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract deterministic static dependencies by analyzing symbol ranges.
    pub fn extract_static_dependencies(&mut self, source_code: &str, symbol_ranges: &[crate::intelligence::ast::OwnedSymbol]) {
        let symbol_names: Vec<String> = symbol_ranges.iter().map(|s| s.symbol.clone()).collect();
        
        for sym in symbol_ranges {
            // Guard against out of bounds if source_code changed but ranges didn't
            if sym.range.end_byte <= source_code.len() {
                let body_text = &source_code[sym.range.start_byte..sym.range.end_byte];
                let mut deps = Vec::new();

                for target in &symbol_names {
                    if target != &sym.symbol && body_text.contains(target) {
                        deps.push(target.clone());
                        
                        self.outgoing.entry(sym.symbol.clone()).or_default().push(target.clone());
                        self.incoming.entry(target.clone()).or_default().push(sym.symbol.clone());
                    }
                }
                
                let node = self.nodes.entry(sym.symbol.clone()).or_insert_with(|| SymbolNode {
                    symbol: sym.symbol.clone(),
                    owner_task_id: sym.owner_task.clone(),
                    dependencies: Vec::new(),
                    dependents: Vec::new(),
                });
                node.dependencies.extend(deps.clone());
            }
        }

        // populate dependents back
        for (callee, callers) in &self.incoming {
            if let Some(node) = self.nodes.get_mut(callee) {
                node.dependents.extend(callers.clone());
                node.dependents.sort();
                node.dependents.dedup();
            }
        }
        for node in self.nodes.values_mut() {
            node.dependencies.sort();
            node.dependencies.dedup();
        }
    }
}
