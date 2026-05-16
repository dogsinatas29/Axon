// # encoding: utf-8
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SemanticRiskLevel {
    Critical, // 즉시 중단 (Interrupt)
    Warning,  // 주의 필요
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SemanticRiskKind {
    GhostStruct,         // 정의되지 않은 구조체 사용
    InterfaceDrift,      // 명세와 구현 간 시그니처 불일치
    DependencyEscalation, // 선택적 의존성의 핵심 그래프 침범
    OwnershipGap,        // 메모리 소유권 정의 누락
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum DecisionAction {
    #[serde(alias = "SEAL")]
    Seal,
    #[serde(alias = "APPROVE")]
    Approve,
    #[serde(alias = "EXCLUDE")]
    Exclude,
    #[serde(alias = "STOP")]
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDecision {
    pub risk_id: String,     // Fingerprint ID
    pub ir_hash: String,     // IR hash at decision time
    pub action: DecisionAction,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SemanticRole {
    Auth,
    Persistence,
    Transport,
    Cache,
    Logic,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticFingerprint {
    pub role: SemanticRole,
    pub has_pointers: bool,
    pub security_sensitive: bool,
    pub is_persistent: bool,
    pub field_count: usize,
    pub blast_radius_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadius {
    pub affected_components: Vec<String>,
    pub affected_functions: Vec<String>,
    pub total_impact_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRisk {
    pub id: String,          // Fingerprint: kind:target:context_hash
    pub kind: SemanticRiskKind,
    pub level: SemanticRiskLevel,
    pub target: String,
    pub message: String,
    pub cause: String,
    pub impact: String,
    pub options: Vec<String>,
    pub recommendation: String,
    pub context: String,
    pub blast_radius: Option<BlastRadius>,
    pub conflict_source: Option<String>,
    pub fingerprint: Option<SemanticFingerprint>, // NEW: v0.0.30 Stage 3.2
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SemanticClosure {
    pub risks: Vec<SemanticRisk>,
    pub decisions: Vec<SemanticDecision>,
    pub is_sealed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrecedentTrustLevel {
    Experimental,      // 검증 중 (확인 필요)
    LocalStable,       // 프로젝트 내 안정 (자동 적용)
    Constitutional,    // 시스템 규약 (절대 적용)
    Deprecated,        // 폐기됨 (사용 중단)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPrecedent {
    pub id: String,
    pub risk_kind: SemanticRiskKind,
    pub target_pattern: String,
    pub action: DecisionAction,
    pub comment: String,
    pub trust_level: PrecedentTrustLevel,
    pub success_count: u32,
    pub failure_count: u32,
    pub severity_override: Option<SemanticRiskLevel>,
    pub fingerprint_requirement: Option<SemanticFingerprint>, // NEW: v0.0.30 Stage 3.2
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JurisprudenceDB {
    pub precedents: Vec<DecisionPrecedent>,
    pub global_policies: BTreeMap<String, String>,
}

impl SemanticClosure {
    pub fn has_critical_risks(&self, current_ir_hash: &str) -> bool {
        self.risks.iter().any(|r| {
            if r.level != SemanticRiskLevel::Critical { return false; }
            
            // Unresolved if no valid decision exists for this specific risk fingerprint
            !self.decisions.iter().any(|d| 
                d.risk_id == r.id && 
                d.ir_hash == current_ir_hash &&
                matches!(d.action, DecisionAction::Seal | DecisionAction::Approve | DecisionAction::Exclude)
            )
        })
    }

    pub fn risk_score(&self) -> usize {
        self.risks.iter().map(|r| match r.level {
            SemanticRiskLevel::Critical => 10,
            SemanticRiskLevel::Warning => 1,
        }).sum()
    }
}
