use std::env;
use std::fs::File;
use std::io::BufReader;
use axon_daemon::intelligence::topology::replay::{RepairReplaySimulator, RepairSimulationInput};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_repair_simulation_input.json>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let input: RepairSimulationInput = serde_json::from_reader(reader)?;
    
    let output = RepairReplaySimulator::simulate_failure(&input);
    
    let output_json = serde_json::to_string_pretty(&output)?;
    println!("{}", output_json);
    
    Ok(())
}
