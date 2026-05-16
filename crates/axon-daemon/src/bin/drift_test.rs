use axon_daemon::intelligence::decision::{load_project_ir, calculate_ir_hash};
use axon_daemon::intelligence::semantic_debugger::SemanticRiskExtractor;
use axon_daemon::intelligence::jurisprudence::JurisprudenceMatcher;
use axon_core::validator::{JurisprudenceDB, DecisionPrecedent, SemanticRiskKind, DecisionAction};
use std::collections::BTreeMap;

#[tokio::main]
async fn main() {
    let project_root = "./default";
    println!("--- [LONG-RUN DRIFT SIMULATION] ---");

    // 1. Setup Initial Jurisprudence
    let db = JurisprudenceDB {
        precedents: vec![
            DecisionPrecedent {
                id: "P001".to_string(),
                risk_kind: SemanticRiskKind::GhostStruct,
                target_pattern: "user_.*".to_string(), // user_로 시작하는 구조체 자동 승인 패턴
                action: DecisionAction::Seal,
                comment: "Auto-sealing common user structures".to_string(),
                trust_level: axon_core::validator::PrecedentTrustLevel::LocalStable,
                success_count: 0,
                failure_count: 0,
                severity_override: None,
                fingerprint_requirement: None,
                metadata: BTreeMap::new(),
            }
        ],
        global_policies: BTreeMap::new(),
    };
    let db_path = std::path::Path::new(project_root).join("contracts/jurisprudence.json");
    std::fs::write(db_path, serde_json::to_string_pretty(&db).unwrap()).unwrap();

    // 2. Load Project State
    let ir = load_project_ir(project_root).expect("Failed to load IR");
    let ir_hash = calculate_ir_hash(&ir);
    let extractor = SemanticRiskExtractor::new(project_root);
    let matcher = JurisprudenceMatcher::load(project_root);

    // 3. Iterative Testing
    println!("Iteration 1: Baseline inspection...");
    let closure = extractor.extract_risks(&ir).await;
    println!("Total Risks: {}", closure.risks.len());

    let mut auto_count = 0;
    for risk in &closure.risks {
        if let Some(decision) = matcher.auto_arbitrate(risk, &ir_hash) {
            println!("✅ Auto-arbitrated: {} -> {:?}", risk.id, decision.action);
            auto_count += 1;
        } else {
            println!("❌ Manual required: {}", risk.id);
        }
    }

    println!("\n[RESULT]");
    println!("Total Potential Interrupts: {}", closure.risks.len());
    println!("Jurisprudence Hits: {}", auto_count);
    println!("Human Intervention Reduction: {}%", (auto_count as f32 / closure.risks.len() as f32) * 100.0);
}
