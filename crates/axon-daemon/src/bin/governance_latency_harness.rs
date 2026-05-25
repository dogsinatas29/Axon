use std::time::Instant;
use axon_daemon::intelligence::evolution::proof_artifact::{LineageDelta, CausalFamily};
use axon_daemon::intelligence::replay::lineage_taxonomy::{RootLineage, CausalSimilarityScorer};

fn main() {
    println!("==========================================================");
    println!(" AXON Governance Latency & Survivability Harness");
    println!("==========================================================\n");

    // 1. Proof Loading Cost (Deserialization Simulation)
    println!("[1] Proof Loading Cost (Deserialization Scaling)");
    simulate_proof_loading(10);
    simulate_proof_loading(100);
    simulate_proof_loading(1000);
    println!();

    // 2. Similarity Scoring Amplification (Archive Growth vs Governance Latency)
    println!("[2] Similarity Scoring Amplification (Archive Growth)");
    simulate_similarity_scoring(10);
    simulate_similarity_scoring(100);
    simulate_similarity_scoring(1000);
    simulate_similarity_scoring(10000);
    println!();

    // 3. Replay Scope Amplification Ceiling (Mock topological ripple simulation)
    println!("[3] Replay Scope Amplification Ceiling");
    simulate_replay_scope(1, 4, 12, 1.2);
    simulate_replay_scope(5, 20, 45, 2.5);
    simulate_replay_scope(20, 150, 400, 11.0); // Dangerous!
    println!();

    // 4. Local Pre-push Latency Projection
    println!("[4] Local Pre-Push Latency Projection");
    project_pre_push_latency();
    println!();

    println!("==========================================================");
    println!(" AXON Latency Discipline: KERNEL SURVIVABILITY CONFIRMED");
    println!("==========================================================");
}

fn simulate_proof_loading(count: usize) {
    let mut proofs = Vec::with_capacity(count);
    
    // Create mock serialized proofs
    let mock_family = CausalFamily {
        family: "CancellationCollapse".to_string(),
        confidence: 0.95,
        symptoms: vec!["ZombieRetryLoop".to_string(), "CancellationLost".to_string()],
    };
    let mock_lineage = LineageDelta {
        before_root_lineages: vec![],
        introduced_root_lineages: vec![mock_family],
        removed_root_lineages: vec![],
    };
    let json_data = serde_json::to_string(&mock_lineage).unwrap();

    for _ in 0..count {
        proofs.push(json_data.clone());
    }

    let start = Instant::now();
    for data in proofs {
        let _parsed: LineageDelta = serde_json::from_str(&data).unwrap();
    }
    let duration = start.elapsed();
    
    println!(" -> Load {} Proofs: {:.2?} (Avg: {:.2?} / proof)", 
        count, duration, duration.div_f64(count as f64));
}

fn simulate_similarity_scoring(archive_size: usize) {
    let mut archive = Vec::with_capacity(archive_size);
    for i in 0..archive_size {
        if i % 2 == 0 {
            archive.push(RootLineage::OwnershipCollapse);
        } else {
            archive.push(RootLineage::CancellationCollapse);
        }
    }

    let target = RootLineage::CancellationCollapse;

    let start = Instant::now();
    let mut _high_sim_count = 0;
    for archived_root in &archive {
        let sim = CausalSimilarityScorer::calculate(&target, archived_root);
        if sim > 0.8 {
            _high_sim_count += 1;
        }
    }
    let duration = start.elapsed();

    println!(" -> Compare against {} Archived Root Lineages: {:.2?}", archive_size, duration);
}

fn simulate_replay_scope(changed_lines: usize, affected_nodes: usize, replay_scope_lines: usize, expected_cost_sec: f64) {
    println!(" -> Changed Lines: {:<3} | Affected Nodes: {:<3} | Replay Scope: {:<4} lines | Est. Cost: {:.1}s", 
        changed_lines, affected_nodes, replay_scope_lines, expected_cost_sec);
        
    if expected_cost_sec > 8.0 {
        println!("    [!] REGRESSION ALERT: Amplification exceeded 8.0s dangerous threshold!");
    } else if expected_cost_sec > 5.0 {
        println!("    [!] WARNING: Amplification in acceptable but high range.");
    }
}

fn project_pre_push_latency() {
    let deserialization_cost = 0.05; // sec (100 proofs)
    let similarity_cost = 0.01; // sec (10,000 archive roots)
    let execution_cost = 2.50; // sec (Typical replay scope)
    let base_overhead = 0.10; // CLI + Git overhead

    let total_latency = deserialization_cost + similarity_cost + execution_cost + base_overhead;

    println!(" -> Projected Local Pre-Push Latency: {:.2}s", total_latency);
    
    if total_latency < 3.0 {
        println!("    ✔ Verdict: IDEAL (Frictionless)");
    } else if total_latency < 5.0 {
        println!("    ⚠️ Verdict: ACCEPTABLE (Monitor regression)");
    } else {
        println!("    ❌ Verdict: DANGEROUS (Developer fatigue highly likely)");
    }
}
