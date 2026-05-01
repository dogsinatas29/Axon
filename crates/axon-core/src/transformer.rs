use crate::ir::*;
use crate::ir_change::*;
use crate::patch::*;
use std::collections::HashSet;

pub fn patch_to_ir_changes(patch: Patch) -> Vec<IRChange> {
    let mut changes = Vec::new();

    for file in patch.files {
        let component_name = file_path_to_component(&file.path);

        match file.action {
            PatchAction::Delete => {
                changes.push(IRChange::DeleteComponent {
                    name: component_name,
                });
            }

            PatchAction::Rewrite => {
                let component = build_component(component_name, file.path, &file.code);
                changes.push(IRChange::ReplaceComponent { component });
            }

            PatchAction::Append => {
                let functions = extract_functions(&file.code);

                for f in functions {
                    changes.push(IRChange::AddOrUpdateFunction {
                        component: component_name.clone(),
                        function: f,
                    });
                }
            }
        }
    }

    changes
}

fn file_path_to_component(path: &str) -> String {
    path.replace(".py", "").replace("/", ".")
}

fn build_component(name: String, path: String, code: &str) -> Component {
    let mut functions = std::collections::HashMap::new();
    let extracted = extract_functions(code);
    for f in extracted {
        functions.insert(f.name.clone(), f);
    }

    Component {
        name,
        file_path: path,
        functions,
        imports: extract_imports(code),
    }
}

fn extract_functions(code: &str) -> Vec<Function> {
    let mut functions = Vec::new();
    // Minimal regex-like parsing for v1.0
    for line in code.lines() {
        if line.starts_with("def ") {
            if let Some(end) = line.find(':') {
                let sig = line[4..end].trim().to_string();
                if let Some(name_end) = sig.find('(') {
                    let name = sig[..name_end].trim().to_string();
                    functions.push(Function {
                        name,
                        signature: format!("{}(...)", sig), // Simplified
                        dependencies: HashSet::new(),
                        body_hash: None,
                    });
                }
            }
        }
    }
    functions
}

fn extract_imports(code: &str) -> HashSet<String> {
    let mut imports = HashSet::new();
    for line in code.lines() {
        if line.starts_with("import ") {
            imports.insert(line[7..].trim().to_string());
        } else if line.starts_with("from ") {
            if let Some(mid) = line.find(" import ") {
                imports.insert(line[5..mid].trim().to_string());
            }
        }
    }
    imports
}
