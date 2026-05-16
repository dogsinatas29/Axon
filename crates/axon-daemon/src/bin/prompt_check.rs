use axon_daemon::intelligence::decision::{PromptBuilder, PromptContext, Stage, calculate_ir_hash};
// # encoding: utf-8

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        println!("Usage: prompt_check <project_root> <component_name> <file_path>");
        return;
    }

    let project_root = &args[1];
    let _component_name = &args[2];
    let file_path = &args[3];

    let ir = axon_daemon::intelligence::decision::load_project_ir(project_root).expect("Failed to load IR");
    let ir_hash = calculate_ir_hash(&ir);
    println!("DEBUG: project_ir.json loaded from {}/contracts/project_ir.json", project_root);
    println!("DEBUG: CURRENT_IR_HASH: {}", ir_hash);

    let ctx = PromptContext {
        project_root: project_root.to_string(),
        stage: Stage::ImplGen,
        target: file_path.to_string(),
        files: Vec::new(), 
        cause: None,
    };
    
    let sealed_path = std::path::Path::new(project_root).join("contracts/sealed_ir.json");
    if sealed_path.exists() {
        let content = std::fs::read_to_string(&sealed_path).unwrap();
        println!("DEBUG: sealed_ir.json content length: {}", content.len());
        match serde_json::from_str::<axon_core::validator::SemanticClosure>(&content) {
            Ok(c) => println!("DEBUG: Manual parse SUCCESS. Decisions: {}", c.decisions.len()),
            Err(e) => println!("DEBUG: Manual parse FAILED: {}", e),
        }
    }

    let prompt = PromptBuilder::build(&ctx);

    println!("\n--- [STRESS TEST: PROMPT INJECTION] ---");
    if prompt.contains("[CRITICAL_CONTRACT]") {
        println!("✅ [CRITICAL_CONTRACT] Section Injected Successfully!");
        if let Some(start) = prompt.find("[CRITICAL_CONTRACT]") {
            println!("\n{}", &prompt[start..]);
        }
    } else {
        println!("❌ [CRITICAL_CONTRACT] Section MISSING!");
        if let Some(sealed) = axon_daemon::intelligence::decision::load_sealed_ir(project_root) {
            println!("Sealed decisions count: {}", sealed.decisions.len());
            for d in sealed.decisions {
                println!("- RiskID: {}, DecisoinHash: {}, CurrentHash: {}", d.risk_id, d.ir_hash, ir_hash);
            }
        }
    }
}
