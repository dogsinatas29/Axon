use axon_core::ir::ProjectIR;

pub struct ArchitectureWriter;

impl ArchitectureWriter {
    /// Generates a full architecture.md string from the ProjectIR.
    /// This is the SINGLE entry point for writing the architectural specification.
    pub fn generate(ir: &ProjectIR) -> String {
        let mut out = String::new();

        out.push_str("# AXON Architecture Specification\n\n");
        out.push_str("## Components\n\n");

        // 1. Render Components & Functions (Spec Block)
        let spec_json = Self::render_spec_json(ir);
        out.push_str("<!-- AXON:SPEC:COMPONENTS\n");
        out.push_str(&spec_json);
        out.push_str("\n-->\n\n");

        // 2. Render Human-Readable Section (Optional but good for UX)
        for comp in ir.components.values() {
            out.push_str(&format!("### {}\n", comp.name));
            out.push_str(&format!("- File: `{}`\n", comp.file_path));
            for func in comp.functions.values() {
                out.push_str(&format!("  - `{}`\n", func.name));
            }
            out.push_str("\n");
        }

        // 3. Render Constraints (The NEW Intelligence Block)
        if !ir.constraints.is_empty() {
            out.push_str("## AXON:CONSTRAINTS\n\n");
            out.push_str("```json\n");
            let constraints_json = serde_json::to_string_pretty(&ir.constraints).unwrap_or_else(|_| "[]".to_string());
            out.push_str(&constraints_json);
            out.push_str("\n```\n");
            
            // Hidden machine-readable tag for parser consistency
            out.push_str("\n<!-- AXON:CONSTRAINTS\n");
            out.push_str(&serde_json::to_string(&ir.constraints).unwrap_or_else(|_| "[]".to_string()));
            out.push_str("\n-->\n");
        }

        out
    }

    fn render_spec_json(ir: &ProjectIR) -> String {
        let mut components = Vec::new();
        for comp in ir.components.values() {
            components.push(serde_json::json!({
                "name": comp.name,
                "file": comp.file_path,
                "type": "component",
                "symbols": comp.functions.keys().cloned().collect::<Vec<_>>()
            }));
        }
        serde_json::to_string_pretty(&serde_json::json!({ "components": components })).unwrap_or_default()
    }
}
