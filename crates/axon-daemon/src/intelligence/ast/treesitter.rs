use super::{AstOwnershipValidator, OwnedSymbol, SymbolRange, ValidationContext, OwnershipValidationResult};
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

pub struct TreeSitterAstValidator;

impl AstOwnershipValidator for TreeSitterAstValidator {
    fn extract_symbols(
        &self,
        source: &str,
        lang: axon_ir::schema::Language,
    ) -> anyhow::Result<Vec<OwnedSymbol>> {
        let mut parser = Parser::new();
        let (language, query_str, lang_name) = match lang {
            axon_ir::schema::Language::Rust => (
                Language::from(tree_sitter_rust::LANGUAGE),
                "(function_item name: (identifier) @name) @function",
                "rust",
            ),
            axon_ir::schema::Language::C | axon_ir::schema::Language::Cpp => (
                Language::from(tree_sitter_c::LANGUAGE),
                "(function_definition declarator: (function_declarator declarator: (identifier) @name)) @function",
                "c",
            ),
            axon_ir::schema::Language::Python => (
                Language::from(tree_sitter_python::LANGUAGE),
                "(function_definition name: (identifier) @name) @function",
                "python",
            ),
        };

        parser.set_language(&language)?;
        let tree = parser.parse(source, None).ok_or_else(|| anyhow::anyhow!("Tree-sitter parsing failed"))?;

        let query = Query::new(&language, query_str)?;
        let mut query_cursor = QueryCursor::new();
        let mut matches = query_cursor.matches(&query, tree.root_node(), source.as_bytes());

        let mut symbols = Vec::new();

        while let Some(m) = matches.next() {
            let mut func_node = None;
            let mut name_node = None;

            for capture in m.captures {
                let capture_name = &query.capture_names()[capture.index as usize];
                if *capture_name == "function" {
                    func_node = Some(capture.node);
                } else if *capture_name == "name" {
                    name_node = Some(capture.node);
                }
            }

            if let (Some(func), Some(name)) = (func_node, name_node) {
                let symbol_name = name.utf8_text(source.as_bytes())?.to_string();
                let start_byte = func.start_byte();
                let end_byte = func.end_byte();

                let body = &source[start_byte..end_byte];
                let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
                sha2::Digest::update(&mut hasher, body.as_bytes());
                let hash_bytes = sha2::Digest::finalize(hasher);
                let current_hash = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();

                symbols.push(OwnedSymbol {
                    symbol: symbol_name,
                    kind: "function".to_string(),
                    language: lang_name.to_string(),
                    parser: "tree-sitter".to_string(),
                    parser_confidence: 0.99,
                    range: SymbolRange {
                        start_byte,
                        end_byte,
                    },
                    owner_task: None,
                    phase: None,
                    hash: Some(current_hash),
                    composite_hash: None,
                });
            }
        }

        Ok(symbols)
    }

    fn validate_patch(
        &self,
        ctx: &ValidationContext,
    ) -> anyhow::Result<OwnershipValidationResult> {
        let extracted = self.extract_symbols(ctx.patch_content, ctx.language.clone())?;
        let mut violations = Vec::new();

        for sym in extracted {
            let mut ownership_opt = None;
            for file_map in ctx.registry.files.values() {
                if let Some(own) = file_map.get(&sym.symbol) {
                    ownership_opt = Some(own);
                    break;
                }
            }

            if let Some(ownership) = ownership_opt {
                let mut hash_drift = false;
                if let Some(ref validated_hash) = ownership.last_validated_hash {
                    if Some(validated_hash) != sym.hash.as_ref() {
                        hash_drift = true;
                    }
                } else {
                    hash_drift = true;
                }

                if let Some(ref owner) = ownership.owner_task_id {
                    if ownership.immutable && hash_drift && owner != ctx.task_id {
                        violations.push(format!("`{}` (Immutable, owned by Task {}) [TreeSitter]", sym.symbol, owner));
                    } else if hash_drift && owner != ctx.task_id {
                        violations.push(format!("`{}` (owned by Task {}) [TreeSitter]", sym.symbol, owner));
                    }
                }
            }
        }

        Ok(OwnershipValidationResult {
            is_valid: violations.is_empty(),
            violations,
        })
    }
}
