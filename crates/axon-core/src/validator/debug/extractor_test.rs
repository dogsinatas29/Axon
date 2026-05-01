use rustpython_parser::Parse;
use rustpython_ast::Suite;
use crate::validator::analysis::extract_functions;
use crate::validator::types::FunctionSig;

pub fn test_case(code: &str, expected: Vec<FunctionSig>) {
    let ast = match Suite::parse(code, "<test>") {
        Ok(ast) => ast,
        Err(e) => {
            println!("❌ FAIL: Parse error: {}", e);
            return;
        }
    };
    
    let result = extract_functions(&ast);

    if result != expected {
        println!("❌ FAIL");
        println!("Expected: {:?}", expected);
        println!("Actual:   {:?}", result);
    } else {
        println!("✅ PASS");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_extractor_tests() {
        test_case(
            "def foo(a): pass",
            vec![FunctionSig {
                name: "foo".into(),
                args: vec!["a".into()],
            }],
        );

        test_case(
            "def bar(a, b=10, *args): pass",
            vec![FunctionSig {
                name: "bar".into(),
                args: vec!["a".into(), "b".into(), "*args".into()],
            }],
        );
    }
}
