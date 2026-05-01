use rustpython_ast::{Stmt, Suite};
use rustpython_parser::Parse;

pub fn print_ast(code: &str) {
    match Suite::parse(code, "<debug>") {
        Ok(ast) => {
            println!("--- AST HUMAN READABLE ---");
            for stmt in &ast {
                print_stmt(stmt, 0);
            }
            println!("---------------------------");
        }
        Err(e) => println!("Parse failed: {}", e),
    }
}

fn print_stmt(stmt: &Stmt, indent: usize) {
    let pad = "  ".repeat(indent);
    match stmt {
        Stmt::FunctionDef(f) => {
            println!("{}Function: {}", pad, f.name);
            let args: Vec<String> = f.args.args.iter().map(|a| a.def.arg.to_string()).collect();
            println!("{}  Args: {:?}", pad, args);
            for inner in &f.body {
                print_stmt(inner, indent + 1);
            }
        }
        Stmt::If(i) => {
            println!("{}If", pad);
            for s in &i.body { print_stmt(s, indent + 1); }
            for s in &i.orelse { print_stmt(s, indent + 1); }
        }
        Stmt::For(f) => {
            println!("{}For", pad);
            for s in &f.body { print_stmt(s, indent + 1); }
        }
        Stmt::While(w) => {
            println!("{}While", pad);
            for s in &w.body { print_stmt(s, indent + 1); }
        }
        _ => {
            println!("{}Other Stmt", pad);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_printer() {
        let code = "def foo(a, b):\n    if a:\n        pass";
        print_ast(code);
    }
}
