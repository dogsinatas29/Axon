//! IR types - DEPRECATED in v0.0.28
//!
//! WARNING: This module is deprecated. Use `axon_ir` crate instead.
//!
//! Migration:
//!   - Replace `use axon_core::ir::ProjectIR` with `use axon_ir::schema::ProjectIR`
//!   - Replace `use axon_core::ir::canonicalize_path` with `use axon_ir::canonicalize_path`
//!   - Use `axon_ir::load_ir()` instead of `ProjectIR::load_from_file()`

#![deprecated(since = "0.0.30", note = "Use axon_ir crate instead")]

pub use axon_ir::schema::{ProjectIR, Component, Function};
pub use axon_ir::{
    canonicalize_path,
    canonical_ir_name,
    normalize_paths,
    is_compound_path,
    sanitize_llm_output,
    load_ir as load_from_file,
    save_ir,
    validate_ir,
    validate_language_capability,
    link_dependencies,
    DependencyGraph,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_deprecated_re_exports() {
        assert_eq!(canonicalize_path("/main.c"), "main.c");
        assert_eq!(canonical_ir_name("database.c"), "database");
    }
}