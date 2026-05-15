// # encoding: utf-8
use axon_core::validator::{SemanticRisk, SemanticRiskKind, SemanticRiskLevel, SemanticClosure};
use axon_core::ir::ProjectIR;
use std::sync::Arc;
use std::path::Path;

pub struct SemanticRiskExtractor {
    project_root: String,
}

impl SemanticRiskExtractor {
    pub fn new(project_root: &str) -> Self {
        Self {
            project_root: project_root.to_string(),
        }
    }

    /// Extract semantic risks from current IR and physical file state
    pub async fn extract_risks(&self, ir: &ProjectIR) -> SemanticClosure {
        let mut risks = Vec::new();
        
        // v0.0.30: Load existing decisions to filter risks
        let mut closure = crate::intelligence::decision::load_sealed_ir(&self.project_root)
            .unwrap_or_default();

        // 1. Ghost Struct Detection
        self.detect_ghost_structs(ir, &mut risks);

        // 2. Dependency Escalation Detection
        self.detect_dependency_escalation(ir, &mut risks);

        // 3. ABI Mismatch Detection (Physical check)
        self.detect_abi_mismatch(ir, &mut risks).await;

        closure.risks = risks;
        closure.is_sealed = !closure.has_critical_risks();
        closure
    }

    fn detect_ghost_structs(&self, ir: &ProjectIR, risks: &mut Vec<SemanticRisk>) {
        // Simple heuristic: Look for "struct X" or types used in signatures that aren't defined
        // For C projects, we check if struct names used in functions are defined in metadata or components
        for (path, comp) in &ir.components {
            for func in comp.functions.values() {
                let sig = &func.signature;
                if sig.contains("struct ") {
                    let parts: Vec<&str> = sig.split("struct ").collect();
                    for i in 1..parts.len() {
                        let struct_name = parts[i].split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("");
                        if !struct_name.is_empty() {
                            // Check if this struct is "known" in IR metadata or is defined in some .h
                            let known = ir.components.values().any(|c| {
                                c.name.to_lowercase().contains(&struct_name.to_lowercase()) || 
                                c.metadata.contains_key(&format!("struct:{}", struct_name))
                            });

                            if !known {
                                risks.push(SemanticRisk {
                                    kind: SemanticRiskKind::GhostStruct,
                                    level: SemanticRiskLevel::Critical,
                                    target: struct_name.to_string(),
                                    message: format!("Ghost Struct Detected: '{}' is used in {} but never defined.", struct_name, path),
                                    context: format!("Signature: {}", sig),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn detect_dependency_escalation(&self, ir: &ProjectIR, risks: &mut Vec<SemanticRisk>) {
        for (path, comp) in &ir.components {
            if comp.is_blocking { // Core component
                for dep_path in &comp.imports {
                    if let Some(dep_comp) = ir.components.get(dep_path) {
                        if !dep_comp.is_blocking { // Optional dependency
                            risks.push(SemanticRisk {
                                kind: SemanticRiskKind::DependencyEscalation,
                                level: SemanticRiskLevel::Critical,
                                target: dep_path.clone(),
                                message: format!("Dependency Escalation: Core module '{}' depends on Optional module '{}'.", path, dep_path),
                                context: "Core graph must not depend on Optional components.".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    async fn detect_abi_mismatch(&self, ir: &ProjectIR, risks: &mut Vec<SemanticRisk>) {
        for (path, comp) in &ir.components {
            if path.ends_with(".h") {
                let physical_path = Path::new(&self.project_root).join(path);
                if physical_path.exists() {
                    if let Ok(content) = tokio::fs::read_to_string(&physical_path).await {
                        for func in comp.functions.values() {
                            // Simple substring match for signature (KISS)
                            // In a real C parser this would be more robust
                            let sig_clean = func.signature.replace(";", "").trim().to_string();
                            if !content.contains(&sig_clean) {
                                risks.push(SemanticRisk {
                                    kind: SemanticRiskKind::InterfaceDrift,
                                    level: SemanticRiskLevel::Critical,
                                    target: path.clone(),
                                    message: format!("ABI Drift: Signature for '{}' in IR does not match physical header.", func.name),
                                    context: format!("IR Expects: {}\nFile: {}", func.signature, path),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
