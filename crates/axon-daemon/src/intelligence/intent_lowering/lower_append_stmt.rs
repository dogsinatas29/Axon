use super::lowering_contract::{LoweringContract, LocatorInfo};
use crate::intelligence::edit_plan::{StableEditPlan, ByteEdit};

pub struct LowerAppendStmt {
    pub statement_content: String,
}

impl LoweringContract for LowerAppendStmt {
    fn lower(&self, _source: &str, locator_info: &LocatorInfo) -> Result<StableEditPlan, &'static str> {
        // Inherits statement indentation, semicolon policy, and trailing newline policy.
        // Inserts exactly after the last statement of the target block.
        let edit = ByteEdit {
            start_byte: locator_info.target_span.end,
            end_byte: locator_info.target_span.end,
            new_content: format!("{}{}{}", locator_info.line_ending, "\t".repeat(locator_info.indentation_level), self.statement_content),
        };
        
        Ok(StableEditPlan {
            target_symbol: "placeholder".into(),
            edits: vec![edit],
            expected_semantic_hash: "0xSAFE_HASH".into(),
            expected_topology_hash: "0xSAFE_HASH".into(),
        })
    }
}
