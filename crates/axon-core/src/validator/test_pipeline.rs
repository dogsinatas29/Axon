use rustpython_parser::Parse;
use rustpython_ast::Suite;
use crate::validator::analysis::extract_functions;

fn main() {
    let samples = vec![
        // 1. Pure Python
        "def hello(name, *args, **kwargs):\n    pass",
        // 2. Nested Python
        "if True:\n    def inner(x, y=10):\n        pass",
        // 3. JavaScript (Should Fail)
        "const x = 10;\nfunction js_func() {}"
    ];

    for (i, code) in samples.iter().enumerate() {
        println!("--- Sample {} ---", i + 1);
        println!("Code:\n{}", code);
        
        match Suite::parse(code, "<test>") {
            Ok(ast) => {
                let funcs = extract_functions(&ast);
                println!("Extracted: {:?}", funcs);
            }
            Err(e) => {
                println!("Parse Failed: {}", e);
            }
        }
        println!();
    }
}
