use crate::schema::ProjectIR;

pub fn parse_markdown(input: &str) -> Option<ProjectIR> {
    ProjectIR::from_md(input)
}

pub fn to_markdown(ir: &ProjectIR, title: &str) -> String {
    let mut md = format!("# Architecture: {}\n\n", title);
    md.push_str("## Components\n\n");

    let mut keys: Vec<_> = ir.components.keys().collect();
    keys.sort();

    for key in keys {
        let comp = ir.components.get(key).unwrap();
        md.push_str(&format!("### {}\n", key));
        md.push_str(&format!("- **File**: `{}`\n", comp.file_path));
        md.push_str("- **Functions**:\n");

        let mut funcs: Vec<_> = comp.functions.keys().collect();
        funcs.sort();
        for fname in funcs {
            md.push_str(&format!("  - `{}`\n", fname));
        }
        md.push('\n');
    }

    if !ir.constraints.is_empty() {
        md.push_str("## Constraints\n\n");
        for constraint in &ir.constraints {
            md.push_str(&format!("- **{}**: {} ({})\n", constraint.kind, constraint.target, constraint.message));
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_markdown() {
        let ir = ProjectIR::new();
        let md = to_markdown(&ir, "test-project");
        assert!(md.contains("test-project"));
    }
}