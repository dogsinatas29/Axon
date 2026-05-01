use super::contract::{ExecutionArtifact, TestCase};

pub fn generate(artifact: &ExecutionArtifact) -> TestCase {
    let expected = artifact.spec.clone().unwrap_or_default();

    let tc = TestCase {
        code: artifact.code.clone(),
        expected,
    };

    println!("=== [DEBUG] GENERATED TEST CASE ===");
    println!("Stage of Failure: {:?}", artifact.stage);
    println!("Errors Found: {:?}", artifact.errors);
    println!("Reproducible Code:\n{}", tc.code);
    
    tc
}
