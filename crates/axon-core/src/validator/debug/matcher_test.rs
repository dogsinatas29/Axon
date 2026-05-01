use crate::validator::types::FunctionSig;
use crate::validator::analysis::match_all;

pub fn test_match(spec: Vec<FunctionSig>, actual: Vec<FunctionSig>) {
    let result = match_all(&spec, &actual);

    if result {
        println!("✅ MATCH");
    } else {
        println!("❌ MISMATCH");
        println!("Spec:   {:?}", spec);
        println!("Actual: {:?}", actual);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_matcher_tests() {
        // PASS
        test_match(
            vec![FunctionSig { name: "foo".into(), args: vec!["a".into()] }],
            vec![FunctionSig { name: "foo".into(), args: vec!["a".into()] }],
        );

        // FAIL (missing function)
        test_match(
            vec![
                FunctionSig { name: "foo".into(), args: vec!["a".into()] },
                FunctionSig { name: "bar".into(), args: vec![] },
            ],
            vec![FunctionSig { name: "foo".into(), args: vec!["a".into()] }],
        );
    }
}
