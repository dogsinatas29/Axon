use axon_core::validator::debug::contract::ExecutionArtifact;
use axon_core::validator::debug::analysis_contract::{Analyzer, FailureEvent, AnalysisResult};
use axon_core::validator::debug::{ast_printer, test_gen, coverage, clusterer, rule_generator};

/// Hook called by the daemon when an agent task fails validation.
/// It orchestrates the automated debugging and learning loop.
pub fn on_validation_failure(artifact: ExecutionArtifact) {
    println!("--- [DEBUG] AXON AUTOMATED DEBUG HOOK TRIGGERED ---");
    
    // 1. Human-Readable AST Visualization
    ast_printer::print_ast(&artifact.code);

    // 2. Automated Test Case Generation
    let _tc = test_gen::generate(&artifact);

    // 3. Extractor Coverage & Pattern Tracking
    coverage::update(&artifact);

    // 4. Advanced Failure Analysis (Analyzers)
    let event = FailureEvent {
        code: artifact.code.clone(),
        errors: artifact.errors.clone(),
        functions: Vec::new(), // In a real scenario, this would be the extracted funcs
    };

    let analyzers: Vec<Box<dyn Analyzer>> = vec![
        Box::new(clusterer::FailureClusterer),
        Box::new(rule_generator::RuleGenerator),
    ];

    for analyzer in analyzers {
        let results = analyzer.analyze(&event);
        for res in results {
            match res {
                AnalysisResult::Cluster(info) => {
                    println!("   -> CLUSTER DETECTED: {:X}", info.id);
                }
                AnalysisResult::Rule(cand) => {
                    println!("   -> RULE SUGGESTED: {}", cand.text);
                }
                _ => {}
            }
        }
    }
    
    println!("--------------------------------------------------");
}
