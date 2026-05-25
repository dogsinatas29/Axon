use super::semantic_tokens::CanonicalSemanticForm;

/// Layer 1 - Untrusted Parse Layer
pub struct RawAstNode {
    pub language: String,
    pub parser: String, // tree-sitter, syn, swc
    pub node_kind: String,
    // pub byte_range: SymbolRange,
    // pub children: Vec<RawAstNode>,
}

/// The critical mechanism translating Untrusted AST into Canonical Authority.
/// Must be Language-Policy Aware (e.g. C Preprocessor is catastrophic, Python decorators are strict, Rust lifetimes may be harmless).
pub trait SemanticCanonicalizer {
    fn canonicalize(&self, ast: &RawAstNode) -> CanonicalSemanticForm;
}

pub struct RustCanonicalizer;

impl SemanticCanonicalizer for RustCanonicalizer {
    fn canonicalize(&self, _ast: &RawAstNode) -> CanonicalSemanticForm {
        // Stub: Strip whitespace, drop trivial comments, normalize lifetime elision, expand safe macros (or reject unexpanded ones)
        unimplemented!("Rust-specific canonicalization policy to extract SemanticTokens");
    }
}
