use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_ast_shadow_validation.jsonl>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut total = 0;
    let mut clean_runs = 0;
    let mut divergences = 0;
    let mut missing_in_ts = 0;
    let mut missing_in_regex = 0;
    let mut range_mismatches = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let parsed: Value = serde_json::from_str(&line)?;
        total += 1;

        if let Some(divs) = parsed.get("divergences").and_then(|d| d.as_array()) {
            if divs.is_empty() {
                clean_runs += 1;
            } else {
                divergences += 1;
                for d in divs {
                    if let Some(kind) = d.get("kind").and_then(|k| k.as_str()) {
                        match kind {
                            "MissingSymbolTreeSitter" => missing_in_ts += 1,
                            "MissingSymbolRegex" => missing_in_regex += 1,
                            "RangeMismatch" => range_mismatches += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    println!("=== Shadow Validation Replay Report ===");
    println!("Total Validations Replayed: {}", total);
    println!("Clean Runs:                 {}", clean_runs);
    println!("Divergent Runs:             {}", divergences);
    println!();
    println!("--- Divergence Breakdown ---");
    println!("Missing in Tree-Sitter: {}", missing_in_ts);
    println!("Missing in Regex:       {}", missing_in_regex);
    println!("Range Mismatches:       {}", range_mismatches);
    
    if total > 0 {
        println!();
        let div_rate = (divergences as f64 / total as f64) * 100.0;
        println!("Overall Divergence Rate:  {:.2}%", div_rate);
    }

    Ok(())
}
