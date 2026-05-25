use std::ops::Range;

pub struct TreeSitterLocator;

impl TreeSitterLocator {
    /// Tree-sitter is strictly a Precision Locator, NOT a mutation authority.
    /// Finds the target symbol in the AST and extracts its byte boundaries and anchor hash.
    pub fn locate_symbol_byte_range(_source: &str, _symbol: &str) -> (Range<usize>, u64) {
        // Stub: Walk Tree-sitter node, return byte bounds and structural hash
        (0..0, 0)
    }

    /// Locates an inner target like a match arm or struct field.
    pub fn locate_inner_target(_source: &str, _symbol: &str, _target: &str) -> (Range<usize>, u64) {
        (0..0, 0)
    }
}
