use rustpython_ast::{Stmt, StmtFunctionDef, Suite, Arguments};
use super::types::FunctionSig;

/// Stage 2: Function Extractor (AST Visitor)
pub fn extract_functions(ast: &Suite) -> Vec<FunctionSig> {
    let mut funcs = Vec::new();
    for stmt in ast {
        visit_stmt(stmt, &mut funcs);
    }
    funcs
}

/// Utility for debugging: Pretty prints the AST structure.
pub fn debug_print_ast(code: &str) -> Result<(), String> {
    use rustpython_parser::Parse;
    match Suite::parse(code, "<debug>") {
        Ok(ast) => {
            println!("--- AST PRETTY PRINT ---");
            println!("{:#?}", ast);
            println!("------------------------");
            Ok(())
        }
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

fn visit_stmt(stmt: &Stmt, out: &mut Vec<FunctionSig>) {
    match stmt {
        Stmt::FunctionDef(f) => {
            out.push(extract_signature(f));
            for inner in &f.body {
                visit_stmt(inner, out);
            }
        }
        Stmt::If(i) => {
            for s in &i.body { visit_stmt(s, out); }
            for s in &i.orelse { visit_stmt(s, out); }
        }
        Stmt::While(w) => {
            for s in &w.body { visit_stmt(s, out); }
        }
        Stmt::For(f) => {
            for s in &f.body { visit_stmt(s, out); }
        }
        _ => {}
    }
}

fn extract_signature(f: &StmtFunctionDef) -> FunctionSig {
    let mut params = Vec::new();
    let args: &Arguments = &f.args;

    for arg in &args.args {
        params.push(arg.def.arg.to_string());
    }

    if let Some(vararg) = &args.vararg {
        params.push(format!("*{}", vararg.arg));
    }

    for arg in &args.kwonlyargs {
        params.push(arg.def.arg.to_string());
    }

    if let Some(kwarg) = &args.kwarg {
        params.push(format!("**{}", kwarg.arg));
    }

    FunctionSig {
        name: f.name.to_string(),
        args: params,
    }
}

pub fn match_all(spec: &[FunctionSig], actual: &[FunctionSig]) -> bool {
    spec.iter().all(|s| actual.contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustpython_parser::Parse;
    use rustpython_ast::Suite;

    #[test]
    fn test_pipeline_alive() {
        let code = "def hello(a, *args): pass";
        let ast = Suite::parse(code, "<test>").unwrap();
        let funcs = extract_functions(&ast);
        
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "hello");
        assert_eq!(funcs[0].args, vec!["a".to_string(), "*args".to_string()]);
    }

    #[test]
    fn test_nested_extraction() {
        let code = "if True:\n    def inner(x): pass";
        let ast = Suite::parse(code, "<test>").unwrap();
        let funcs = extract_functions(&ast);
        
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "inner");
    }

    #[test]
    fn test_js_intrusion_fails_parse() {
        let code = "const x = 10; function test() {}";
        let res = Suite::parse(code, "<test>");
        assert!(res.is_err()); // JS syntax is not Python syntax
    }

    #[test]
    fn test_matcher_scenario_matrix() {
        // Scenario 1: Exact Match (PASS)
        let spec = vec![FunctionSig { name: "foo".into(), args: vec!["a".into()] }];
        let actual = vec![FunctionSig { name: "foo".into(), args: vec!["a".into()] }];
        assert!(match_all(&spec, &actual));

        // Scenario 2: Missing Symbol (FAIL)
        let spec = vec![
            FunctionSig { name: "foo".into(), args: vec!["a".into()] },
            FunctionSig { name: "bar".into(), args: vec![] }
        ];
        let actual = vec![FunctionSig { name: "foo".into(), args: vec!["a".into()] }];
        assert!(!match_all(&spec, &actual));

        // Scenario 3: Argument Mismatch (FAIL)
        let spec = vec![FunctionSig { name: "foo".into(), args: vec!["a".into()] }];
        let actual = vec![FunctionSig { name: "foo".into(), args: vec!["a".into(), "b".into()] }];
        assert!(!match_all(&spec, &actual));
    }
}
