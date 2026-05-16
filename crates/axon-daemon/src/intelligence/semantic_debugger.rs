// # encoding: utf-8
use axon_core::validator::{SemanticRisk, SemanticClosure};
use axon_core::ir::ProjectIR;
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
        
        let mut closure = crate::intelligence::decision::load_sealed_ir(&self.project_root)
            .unwrap_or_default();

        let ir_hash = crate::intelligence::decision::calculate_ir_hash(ir);

        // 1. Ghost Struct Detection
        self.detect_ghost_structs(ir, &mut risks);

        // 2. Dependency Escalation Detection
        self.detect_dependency_escalation(ir, &mut risks);

        // 3. ABI Mismatch Detection (Physical check)
        self.detect_abi_mismatch(ir, &mut risks).await;

        // 4. Authority Conflict Detection (Spec vs Sealed)
        self.detect_authority_conflict(ir, &mut risks, &closure);

        closure.risks = risks;
        closure.is_sealed = !closure.has_critical_risks(&ir_hash);
        closure
    }

    fn detect_authority_conflict(&self, ir: &ProjectIR, risks: &mut Vec<SemanticRisk>, _closure: &axon_core::validator::SemanticClosure) {
        // v0.0.30: [AUTHORITY_CONFLICT] Spec mutation attempt on LOCKED components
        for (path, comp) in &ir.components {
            if comp.locked {
                // 1. Check if the physical file actually exists and matches the locked status
                // 2. Check if there are conflicting instructions in spec.md (Simplified: any change to a locked comp is a conflict)
                
                // This is a placeholder for a more complex semantic diff engine.
                // For now, we detect if a component is marked LOCKED but has an InterfaceDrift risk.
                if risks.iter().any(|r| r.kind == axon_core::validator::SemanticRiskKind::InterfaceDrift && &r.target == path) {
                     let kind = axon_core::validator::SemanticRiskKind::InterfaceDrift; // Or a new Conflict kind
                     let message = format!("AUTHORITY CONFLICT: Locked Component '{}' is drifting!", comp.name);
                     let cause = "Spec/Implementation mutation attempt on a [✅ LOCKED] component.".to_string();
                     let impact = "System integrity collapse. The factory is trying to change a 'Sacred' contract.".to_string();
                     let options = vec![
                         "Unlock the component (Unseal)".to_string(),
                         "Revert changes to match the Lock".to_string(),
                         "Escalate to Boss for Arbitration".to_string()
                     ];
                     let recommendation = "Either revert the code or explicitly Unseal the component to allow changes.".to_string();
                     let context = format!("Target: {} (LOCKED)", path);

                     risks.push(SemanticRisk {
                         id: format!("CONFLICT:{}", path),
                         kind,
                         level: axon_core::validator::SemanticRiskLevel::Critical,
                         target: path.clone(),
                         message,
                         cause,
                         impact,
                         options,
                         recommendation,
                         context,
                         blast_radius: Some(self.calculate_blast_radius(ir, path, &kind)),
                         conflict_source: Some("Sealed Contract vs Spec/Impl Mutation".to_string()),
                         fingerprint: Some(self.infer_fingerprint(ir, path, &kind)),
                     });
                }
            }
        }
    }

    fn generate_id(kind: &axon_core::validator::SemanticRiskKind, target: &str, context: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        context.hash(&mut hasher);
        format!("{:?}:{}:{:x}", kind, target, hasher.finish())
    }

    fn infer_fingerprint(&self, ir: &ProjectIR, target: &str, kind: &axon_core::validator::SemanticRiskKind) -> axon_core::validator::SemanticFingerprint {
        let name_lower = target.to_lowercase();
        
        let role = if name_lower.contains("auth") || name_lower.contains("perm") || name_lower.contains("cred") {
            axon_core::validator::SemanticRole::Auth
        } else if name_lower.contains("db") || name_lower.contains("storage") || name_lower.contains("record") {
            axon_core::validator::SemanticRole::Persistence
        } else if name_lower.contains("api") || name_lower.contains("transport") || name_lower.contains("net") {
            axon_core::validator::SemanticRole::Transport
        } else if name_lower.contains("cache") || name_lower.contains("temp") {
            axon_core::validator::SemanticRole::Cache
        } else {
            axon_core::validator::SemanticRole::Unknown
        };

        let mut has_pointers = false;
        let mut field_count = 0;
        
        // IR에서 해당 타겟의 상세 구조 탐색 (간략화된 예시)
        if let Some(comp) = ir.components.get(target).or_else(|| ir.components.values().find(|c| c.name == target)) {
             field_count = comp.functions.len(); // 함수 개수를 복잡도 지표로 활용
             has_pointers = comp.functions.values().any(|f| f.signature.contains('*'));
        }

        axon_core::validator::SemanticFingerprint {
            role,
            has_pointers,
            security_sensitive: name_lower.contains("auth") || name_lower.contains("secret") || name_lower.contains("key"),
            is_persistent: name_lower.contains("db") || name_lower.contains("record"),
            field_count,
            blast_radius_score: self.calculate_blast_radius(ir, target, kind).total_impact_score,
        }
    }

    fn calculate_blast_radius(&self, ir: &ProjectIR, target: &str, kind: &axon_core::validator::SemanticRiskKind) -> axon_core::validator::BlastRadius {
        let mut affected_components = Vec::new();
        let mut affected_functions = Vec::new();
        let mut score = 0;

        match kind {
            axon_core::validator::SemanticRiskKind::GhostStruct => {
                // 이 구조체를 사용하는 모든 함수와 컴포넌트 추적
                for (path, comp) in &ir.components {
                    let mut comp_affected = false;
                    for func in comp.functions.values() {
                        if func.signature.contains(&format!("struct {}", target)) || func.signature.contains(target) {
                            affected_functions.push(format!("{}::{}", comp.name, func.name));
                            comp_affected = true;
                            score += 1;
                        }
                    }
                    if comp_affected {
                        affected_components.push(path.clone());
                        score += 5;
                    }
                }
            }
            axon_core::validator::SemanticRiskKind::InterfaceDrift => {
                // 이 인터페이스(컴포넌트)를 의존하는 모든 컴포넌트 추적
                for (path, comp) in &ir.components {
                    if comp.imports.contains(&target.to_string()) {
                        affected_components.push(path.clone());
                        score += 10;
                    }
                }
                affected_components.push(target.to_string());
            }
            _ => {
                score = 1;
            }
        }

        axon_core::validator::BlastRadius {
            affected_components,
            affected_functions,
            total_impact_score: score,
        }
    }

    fn detect_ghost_structs(&self, ir: &ProjectIR, risks: &mut Vec<SemanticRisk>) {
        for (path, comp) in &ir.components {
            for func in comp.functions.values() {
                let sig = &func.signature;
                if sig.contains("struct ") {
                    let parts: Vec<&str> = sig.split("struct ").collect();
                    for i in 1..parts.len() {
                        let struct_name = parts[i].split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("");
                        if !struct_name.is_empty() {
                            let known = ir.components.values().any(|c| {
                                c.name.to_lowercase().contains(&struct_name.to_lowercase()) || 
                                c.metadata.contains_key(&format!("struct:{}", struct_name))
                            });

                            if !known {
                                let kind = axon_core::validator::SemanticRiskKind::GhostStruct;
                                let message = format!("Ghost Struct Detected: '{}'", struct_name);
                                let cause = format!("Function in {} uses 'struct {}' but no layout is defined in Architecture IR.", path, struct_name);
                                let impact = "Junior agent will guess the memory layout, leading to field hallucination and memory corruption.".to_string();
                                let options = vec![
                                    "Define struct layout in architecture.md".to_string(),
                                    "Seal current assumptions as a Contract".to_string(),
                                    "Use an opaque pointer if layout is internal".to_string()
                                ];
                                let recommendation = "Define the struct layout to ensure ABI stability.".to_string();
                                let context = format!("Signature: {}", sig);
                                
                                risks.push(SemanticRisk {
                                    id: Self::generate_id(&kind, struct_name, &context),
                                    kind,
                                    level: axon_core::validator::SemanticRiskLevel::Critical,
                                    target: struct_name.to_string(),
                                    message,
                                    cause,
                                    impact,
                                    options,
                                    recommendation,
                                    context,
                                    blast_radius: Some(self.calculate_blast_radius(ir, struct_name, &kind)),
                                    conflict_source: None,
                                    fingerprint: Some(self.infer_fingerprint(ir, struct_name, &kind)),
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
            if comp.is_blocking { 
                for dep_path in &comp.imports {
                    if let Some(dep_comp) = ir.components.get(dep_path) {
                        if !dep_comp.is_blocking { 
                            let kind = axon_core::validator::SemanticRiskKind::DependencyEscalation;
                            let message = format!("Dependency Escalation: '{}' -> '{}'", path, dep_path);
                            let cause = format!("Core module '{}' depends on Optional module '{}'.", path, dep_path);
                            let impact = "If the optional module fails to build, the entire core project will be blocked.".to_string();
                            let options = vec![
                                "Promote dependency to Core status".to_string(),
                                "Refactor to remove the dependency".to_string(),
                                "Approve as an exceptional bypass".to_string()
                            ];
                            let recommendation = "Promote the dependency to Core or refactor to maintain graph purity.".to_string();
                            let context = "Core graph must not depend on Optional components.".to_string();
                            
                            risks.push(SemanticRisk {
                                id: Self::generate_id(&kind, dep_path, &context),
                                kind,
                                level: axon_core::validator::SemanticRiskLevel::Critical,
                                target: dep_path.clone(),
                                message,
                                cause,
                                impact,
                                options,
                                recommendation,
                                context,
                                blast_radius: Some(self.calculate_blast_radius(ir, dep_path, &kind)),
                                conflict_source: None,
                                fingerprint: Some(self.infer_fingerprint(ir, dep_path, &kind)),
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
                            let sig_clean = func.signature.replace(";", "").trim().to_string();
                            if !content.contains(&sig_clean) {
                                let kind = axon_core::validator::SemanticRiskKind::InterfaceDrift;
                                let message = format!("ABI Drift: '{}' in {}", func.name, path);
                                let cause = format!("Physical signature in {} differs from the Architectural IR.", path);
                                let impact = "Linker errors or runtime crashes due to mismatched argument registers/stack layout.".to_string();
                                let options = vec![
                                    "Update IR to match physical code".to_string(),
                                    "Regenerate header from IR".to_string(),
                                    "Manually reconcile signatures".to_string()
                                ];
                                let recommendation = "Regenerate the header to maintain SSOT integrity.".to_string();
                                let context = format!("IR Expects: {}\nFile: {}", func.signature, path);
                                
                                risks.push(SemanticRisk {
                                    id: Self::generate_id(&kind, &func.name, &context),
                                    kind,
                                    level: axon_core::validator::SemanticRiskLevel::Critical,
                                    target: path.clone(),
                                    message,
                                    cause,
                                    impact,
                                    options,
                                    recommendation,
                                    context,
                                    blast_radius: Some(self.calculate_blast_radius(ir, path, &kind)),
                                    conflict_source: None,
                                    fingerprint: Some(self.infer_fingerprint(ir, path, &kind)),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
