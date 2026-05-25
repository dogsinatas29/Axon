use crate::intelligence::edit_plan::StableEditPlan;

/// The absolute deterministic contract for transforming a SemanticIntent into a StableEditPlan.
/// Violation of this contract leads to immediate rejection in the Replay Observatory.
pub trait LoweringContract {
    /// 1. 1 Intent -> Exactly 1 StableEditPlan.
    /// 2. Must NEVER invoke a formatter or AST printer.
    /// 3. Must preserve trivia (whitespace, trailing commas, line endings) strictly.
    /// 4. signature_hash delta == 0, topology_hash delta == 0
    fn lower(&self, source: &str, locator_info: &LocatorInfo) -> Result<StableEditPlan, &'static str>;
}

/// Represents the high-precision boundaries extracted by the Tree-sitter locator.
pub struct LocatorInfo {
    /// Exact byte span of the targeted interior (not the whole node)
    pub target_span: std::ops::Range<usize>,
    pub indentation_level: usize,
    pub line_ending: &'static str, // "\n" or "\r\n"
}
