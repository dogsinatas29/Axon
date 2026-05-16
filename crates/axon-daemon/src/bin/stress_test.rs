use axon_daemon::intelligence::semantic_debugger::SemanticRiskExtractor;
use axon_daemon::intelligence::decision::load_project_ir;

#[tokio::main]
async fn main() {
    println!("Current working directory: {:?}", std::env::current_dir().unwrap());
    let mut project_root = std::env::current_dir().unwrap();
    project_root.pop(); // axon-daemon
    project_root.pop(); // crates
    project_root.push("TEST2");
    let project_root_str = project_root.to_str().unwrap();
    
    println!("Loading IR from: {}", project_root_str);
    let ir = load_project_ir(project_root_str).expect("Failed to load IR");
    let extractor = SemanticRiskExtractor::new(project_root_str);
    
    let closure = extractor.extract_risks(&ir).await;
    let matcher = axon_daemon::intelligence::jurisprudence::JurisprudenceMatcher::load(project_root_str);
    let ir_hash = axon_daemon::intelligence::decision::calculate_ir_hash(&ir);

    println!("--- [TEST A: SEMANTIC JURISPRUDENCE EXPERIMENT] ---");
    println!("Risks detected: {}", closure.risks.len());
    
    for risk in &closure.risks {
        println!("\n[ID: {}] Target: {}", risk.id, risk.target);
        if let Some(fp) = &risk.fingerprint {
            println!("  Fingerprint: Role={:?}, Security={}", fp.role, fp.security_sensitive);
        }
        
        match matcher.auto_arbitrate(risk, &ir_hash) {
            Some(decision) => {
                println!("  ✅ AUTO_SEAL SUCCESS: Matched precedent with high confidence.");
                println!("  Decision: {:?}", decision.action);
            },
            None => {
                println!("  ❌ AUTO_SEAL REJECTED: Semantic Drift or No Match.");
                println!("  Status: CONSTITUTIONAL INTERRUPT TRIGGERED");
            }
        }
        println!("-----------------------------------");
    }
    
    let ir_hash = axon_daemon::intelligence::decision::calculate_ir_hash(&ir);
    println!("Current IR Hash: {}", ir_hash);
    println!("Is Sealed: {}", !closure.has_critical_risks(&ir_hash));
}
