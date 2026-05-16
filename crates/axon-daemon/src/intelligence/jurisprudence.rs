use axon_core::validator::{SemanticRisk, SemanticDecision, JurisprudenceDB};
use regex::Regex;
use std::path::Path;

pub struct JurisprudenceMatcher {
    db: JurisprudenceDB,
}

impl JurisprudenceMatcher {
    pub fn load(project_root: &str) -> Self {
        let path = Path::new(project_root).join("contracts/jurisprudence.json");
        let db = if path.exists() {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            JurisprudenceDB::default()
        };
        Self { db }
    }

    pub fn auto_arbitrate(&self, risk: &SemanticRisk, ir_hash: &str) -> Option<SemanticDecision> {
        for precedent in &self.db.precedents {
            if precedent.trust_level == axon_core::validator::PrecedentTrustLevel::Deprecated {
                continue;
            }

            if precedent.risk_kind == risk.kind {
                if let Ok(re) = Regex::new(&precedent.target_pattern) {
                    if re.is_match(&risk.target) {
                        // v0.0.30 Stage 3.2: Semantic Fingerprint Validation
                        if let (Some(req), Some(actual)) = (&precedent.fingerprint_requirement, &risk.fingerprint) {
                            let similarity = self.calculate_similarity(req, actual);
                            
                            if similarity < 0.8 {
                                tracing::warn!("⚖️ [JURISPRUDENCE] Semantic Drift Detected! Name matched '{}' but similarity is only {:.2}. Auto-arbitration HALTED.", risk.target, similarity);
                                // [CONSTITUTIONAL_RISK]: Pattern matches but semantics differ.
                                // Return None to trigger manual arbitration (Boss intervention)
                                return None;
                            }
                            
                            tracing::info!("⚖️ [JURISPRUDENCE] Semantic Match Confirmed ({:.2}). Applying precedent '{}'", similarity, precedent.id);
                        }

                        // Only auto-seal if trust is high
                        if precedent.trust_level == axon_core::validator::PrecedentTrustLevel::LocalStable 
                           || precedent.trust_level == axon_core::validator::PrecedentTrustLevel::Constitutional {
                            
                            return Some(SemanticDecision {
                                risk_id: risk.id.clone(),
                                action: precedent.action.clone(),
                                comment: format!("[AUTO:PRECEDENT:{}] {}", precedent.id, precedent.comment),
                                ir_hash: ir_hash.to_string(),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    fn calculate_similarity(&self, req: &axon_core::validator::SemanticFingerprint, actual: &axon_core::validator::SemanticFingerprint) -> f32 {
        let mut score = 0.0;
        let mut total = 0.0;

        // Role check (Critical)
        total += 2.0;
        if req.role == actual.role { score += 2.0; }

        // Trait checks
        total += 1.0;
        if req.has_pointers == actual.has_pointers { score += 1.0; }
        
        total += 1.0;
        if req.security_sensitive == actual.security_sensitive { score += 1.0; }

        total += 1.0;
        if req.is_persistent == actual.is_persistent { score += 1.0; }

        // Field count similarity (Linear)
        total += 1.0;
        let diff = (req.field_count as i32 - actual.field_count as i32).abs();
        if diff <= 2 { score += 1.0; }
        else if diff <= 5 { score += 0.5; }

        score / total
    }

    pub fn record_success(&mut self, precedent_id: &str) {
        if let Some(p) = self.db.precedents.iter_mut().find(|p| p.id == precedent_id) {
            p.success_count += 1;
            // Upgrade if enough success
            if p.trust_level == axon_core::validator::PrecedentTrustLevel::Experimental && p.success_count >= 5 {
                p.trust_level = axon_core::validator::PrecedentTrustLevel::LocalStable;
                tracing::info!("🏆 [JURISPRUDENCE] Precedent '{}' upgraded to LocalStable", p.id);
            }
        }
    }
}
