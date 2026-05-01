use super::contract::ExecutionArtifact;
use std::collections::HashSet;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref COVERAGE: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

pub fn update(artifact: &ExecutionArtifact) {
    let mut cov = COVERAGE.lock().unwrap();

    // Track encountered failure patterns
    for err in &artifact.errors {
        let pattern = format!("{:?}", err);
        cov.insert(pattern);
    }

    println!("=== [DEBUG] EXTRACTOR COVERAGE UPDATED ===");
    println!("Unique Failure Patterns Encountered: {}", cov.len());
}
