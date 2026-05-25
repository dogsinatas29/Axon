use serde::{Deserialize, Serialize};
use super::ast::SymbolRange;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AstPatchOperation {
    ReplaceFunction,
    ReplaceMethod,
    ReplaceImplBlock,
    ReplaceTraitImpl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstPatch {
    pub operation: AstPatchOperation,
    pub symbol: String,
    pub language: String,

    pub original_hash: String,
    pub patched_hash: String,

    pub byte_range: SymbolRange,

    pub ownership_verified: bool,
    pub structural_verified: bool,
}

impl AstPatch {
    pub fn new(
        operation: AstPatchOperation,
        symbol: String,
        language: String,
        original_hash: String,
        patched_hash: String,
        byte_range: SymbolRange,
    ) -> Self {
        Self {
            operation,
            symbol,
            language,
            original_hash,
            patched_hash,
            byte_range,
            ownership_verified: false,
            structural_verified: false,
        }
    }
}
