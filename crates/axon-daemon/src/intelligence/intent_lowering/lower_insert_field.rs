use super::lowering_contract::{LoweringContract, LocatorInfo};
use crate::intelligence::edit_plan::{StableEditPlan, ByteEdit};

pub struct LowerInsertField {
    pub field_name: String,
    pub field_type: String,
}

impl LoweringContract for LowerInsertField {
    fn lower(&self, _source: &str, locator_info: &LocatorInfo) -> Result<StableEditPlan, &'static str> {
        // Strict SAFE_SUBSET constraints:
        // Plain struct field ONLY. No attributes, no generics, no proc-macro expansions.
        let new_field = format!("{}{}{}: {},", locator_info.line_ending, "\t".repeat(locator_info.indentation_level), self.field_name, self.field_type);
        
        let edit = ByteEdit {
            start_byte: locator_info.target_span.end,
            end_byte: locator_info.target_span.end,
            new_content: new_field,
        };
        
        Ok(StableEditPlan {
            target_symbol: "placeholder".into(),
            edits: vec![edit],
            expected_semantic_hash: "0xSAFE_HASH".into(),
            expected_topology_hash: "0xSAFE_HASH".into(),
        })
    }
}
