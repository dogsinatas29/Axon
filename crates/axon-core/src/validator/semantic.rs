// # encoding: utf-8
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SemanticRiskLevel {
    Critical, // 즉시 중단 (Interrupt)
    Warning,  // 주의 필요
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SemanticRiskKind {
    GhostStruct,         // 정의되지 않은 구조체 사용
    InterfaceDrift,      // 명세와 구현 간 시그니처 불일치
    DependencyEscalation, // 선택적 의존성의 핵심 그래프 침범
    OwnershipGap,        // 메모리 소유권 정의 누락
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRisk {
    pub kind: SemanticRiskKind,
    pub level: SemanticRiskLevel,
    pub target: String,      // 문제의 파일명 또는 심볼명
    pub message: String,     // 사용자에게 보여줄 메시지
    pub context: String,     // 관련 코드나 명세 발췌본
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDecision {
    pub risk_id: String,
    pub action: String, // SEAL, EXCLUDE, APPROVE, STOP
    pub comment: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SemanticClosure {
    pub risks: Vec<SemanticRisk>,
    pub decisions: Vec<SemanticDecision>,
    pub is_sealed: bool,
}

impl SemanticClosure {
    pub fn has_critical_risks(&self) -> bool {
        // A risk is unresolved if there's no decision for it
        self.risks.iter().any(|r| {
            r.level == SemanticRiskLevel::Critical && 
            !self.decisions.iter().any(|d| d.risk_id == r.target)
        })
    }

    pub fn risk_score(&self) -> usize {
        self.risks.iter().map(|r| match r.level {
            SemanticRiskLevel::Critical => 10,
            SemanticRiskLevel::Warning => 1,
        }).sum()
    }
}
