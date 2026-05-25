//! AXON IR Compiler - Universal Build Graph Runtime IR
//!
//! This crate provides the IR Compiler subsystem for AXON.
//! It handles:
//! - Input format parsing (Markdown, JSON, YAML, TOML)
//! - Semantic validation
//! - Dependency linking
//! - Path canonicalization
//! - IR serialization
//! - Spec extraction (tolerant parsing)
//!
//! IMPORTANT: Runtime core should only interact with validated IR,
//! never directly with raw input formats like markdown.

pub mod schema;
pub mod parser;
pub mod validator;
pub mod linker;
pub mod canonicalizer;
pub mod emitter;
pub mod spec_extractor;
pub mod semantic;
pub mod spec_ir;
pub mod spec_parser;

pub use schema::{Language, Platform, Subsystem, EntrypointType, RuntimeModel, Win32ComponentType, ProjectIR, Component, Function, Constraint, ComponentTier, ComponentType, default_true};
pub use schema::{ProjectTopology, ModuleTopology, TopologyMeta};
pub use schema::{FileAuthority, PatchRegion, OwnershipMetadata};
pub use parser::{parse, InputFormat, detect_format};
pub use validator::{validate_ir, validate_runtime_contract, validate_language_capability, ValidationError, ValidationKind};
pub use linker::{link_dependencies, DependencyGraph};
pub use canonicalizer::{canonicalize_path, canonical_ir_name, normalize_paths, is_compound_path, sanitize_llm_output};
pub use emitter::{save_ir, load_ir, load_ir_from_path};
pub use spec_extractor::{SpecExtractor, SpecKind, ExtractedSpec};
pub use semantic::{SemanticViolation, SemanticDiagnostic, DiagnosticSeverity, DiagnosticCategory, CapabilityLock};

pub const IR_VERSION: &str = "0.0.28";
pub const MIN_IR_VERSION: &str = "0.0.28";

pub fn check_version(ir: &ProjectIR) -> Result<(), String> {
    if ir.components.is_empty() {
        return Err("Empty IR - no components found".to_string());
    }
    Ok(())
}

pub fn create_empty_ir() -> ProjectIR {
    ProjectIR::new()
}

pub fn extract_spec(text: &str) -> Option<ExtractedSpec> {
    SpecExtractor::extract(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_ir() {
        let ir = create_empty_ir();
        assert!(ir.components.is_empty());
    }

    #[test]
    fn test_canonicalizer() {
        assert_eq!(canonicalize_path("/main.c"), "main.c");
        assert_eq!(canonical_ir_name("path/to/test.c"), "test");
    }

    #[test]
    fn test_spec_extraction() {
        let input = "# Architecture\n## Components\n- File: test.c";
        let spec = extract_spec(input);
        assert!(spec.is_some());
    }

    #[test]
    fn test_from_md_language_extraction() {
        let md = "<!-- AXON:SPEC:COMPONENTS\n{\n  \"components\": []\n}\n-->\nlanguage: rust";
        let ir = ProjectIR::from_md(md);
        assert!(ir.is_some());
        assert_eq!(ir.unwrap().language, Language::Rust);
    }
}