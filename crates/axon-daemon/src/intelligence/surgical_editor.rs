use super::edit_plan::StableEditPlan;

pub struct SurgicalEditor;

impl SurgicalEditor {
    /// Executes the actual mutation as minimal byte-range replacements.
    /// Completely bypasses AST printers, thereby preserving untouched indentation, whitespace, and trivia.
    /// Prevents the "Formatter Entropy Explosion" common in AST-native rewrites.
    pub fn execute_surgery(source: &str, plan: &StableEditPlan) -> Result<String, &'static str> {
        let mut source_bytes = source.as_bytes().to_vec();
        
        // Edits should be sorted descending by start_byte to prevent offset invalidation
        let mut sorted_edits = plan.edits.clone();
        sorted_edits.sort_by(|a, b| b.start_byte.cmp(&a.start_byte));

        for edit in sorted_edits {
            if edit.end_byte > source_bytes.len() || edit.start_byte > edit.end_byte {
                return Err("Invalid byte edit range");
            }
            source_bytes.splice(
                edit.start_byte..edit.end_byte, 
                edit.new_content.as_bytes().iter().copied()
            );
        }

        String::from_utf8(source_bytes).map_err(|_| "Surgical edit produced invalid UTF-8")
    }
}
