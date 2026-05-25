use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRange {
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedSymbol {
    pub symbol: String,
    pub kind: String,
    pub language: String,
    pub parser: String,
    pub parser_confidence: f32,
    pub range: SymbolRange,
    pub owner_task: Option<String>,
    pub phase: Option<String>,
    pub hash: Option<String>,
    pub composite_hash: Option<hash_types::CompositeSymbolHash>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShadowDivergenceKind {
    MissingSymbolRegex,
    MissingSymbolTreeSitter,
    RangeMismatch,
    NestedImplMismatch,
    DecoratorMismatch,
    MacroExpansionMismatch,
    TraitImplMismatch,
    GenericScopeMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowDivergence {
    pub kind: ShadowDivergenceKind,
    pub symbol: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationComparison {
    pub file: String,
    pub regex_count: usize,
    pub treesitter_count: usize,
    pub divergences: Vec<ShadowDivergence>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShadowValidationMetrics {
    pub total_validations: u64,
    pub parser_agreements: u64,
    pub parser_divergences: u64,
    pub regex_false_negatives: u64,
    pub treesitter_false_negatives: u64,
    pub divergence_kinds: std::collections::HashMap<String, u64>,
}

pub struct ValidationContext<'a> {
    pub project_id: &'a str,
    pub task_id: &'a str,
    pub patch_content: &'a str,
    pub language: axon_ir::schema::Language,
    pub registry: &'a axon_core::SymbolOwnershipRegistry,
}

#[derive(Debug, Clone)]
pub struct OwnershipValidationResult {
    pub is_valid: bool,
    pub violations: Vec<String>,
}

pub trait AstOwnershipValidator: Send + Sync {
    fn extract_symbols(
        &self,
        source: &str,
        lang: axon_ir::schema::Language,
    ) -> anyhow::Result<Vec<OwnedSymbol>>;

    fn validate_patch(
        &self,
        ctx: &ValidationContext,
    ) -> anyhow::Result<OwnershipValidationResult>;
}

pub struct RegexAstValidator;

impl AstOwnershipValidator for RegexAstValidator {
    fn extract_symbols(
        &self,
        source: &str,
        _lang: axon_ir::schema::Language,
    ) -> anyhow::Result<Vec<OwnedSymbol>> {
        let mut symbols = Vec::new();
        // Fallback to the existing regex-based symbol extraction (extract_symbol_bodies) logic here.
        // We will adapt Daemon::extract_symbol_bodies functionality.
        if let Ok(re_rust) = regex::Regex::new(r"(?:pub\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(") {
            for cap in re_rust.captures_iter(source) {
                let sym = cap[1].to_string();
                let m = cap.get(0).unwrap();
                let start_idx = m.start();
                let mut brace_depth = 0;
                let mut in_string = false;
                let mut in_comment = false;
                let mut end_idx = start_idx;
                let mut started = false;

                let bytes = source.as_bytes();
                let mut i = m.end();
                while i < bytes.len() {
                    let c = bytes[i];
                    if !in_string && !in_comment {
                        if c == b'{' {
                            brace_depth += 1;
                            started = true;
                        } else if c == b'}' {
                            brace_depth -= 1;
                            if started && brace_depth == 0 {
                                end_idx = i + 1;
                                break;
                            }
                        } else if c == b'"' {
                            in_string = true;
                        } else if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                            in_comment = true;
                        }
                    } else if in_string {
                        if c == b'"' && bytes[i - 1] != b'\\' {
                            in_string = false;
                        }
                    } else if in_comment {
                        if c == b'\n' {
                            in_comment = false;
                        }
                    }
                    i += 1;
                }

                if started && brace_depth == 0 {
                    let body = &source[start_idx..end_idx];
                    let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
                    sha2::Digest::update(&mut hasher, body.as_bytes());
                    let hash_bytes = sha2::Digest::finalize(hasher);
                    let current_hash = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();

                    symbols.push(OwnedSymbol {
                        symbol: sym,
                        kind: "function".to_string(),
                        language: "rust".to_string(),
                        parser: "regex".to_string(),
                        parser_confidence: 0.8,
                        range: SymbolRange {
                            start_byte: start_idx,
                            end_byte: end_idx,
                        },
                        owner_task: None, // Will be filled by registry
                        phase: None,
                        hash: Some(current_hash),
                        composite_hash: None,
                    });
                }
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
                        violations.push(format!("`{}` (Immutable, owned by Task {})", sym.symbol, owner));
                    } else if hash_drift && owner != ctx.task_id {
                        violations.push(format!("`{}` (owned by Task {})", sym.symbol, owner));
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

pub mod hash_types;
pub mod treesitter;

pub fn dual_run_shadow_validation(
    project_root: &std::path::Path,
    regex_validator: &RegexAstValidator,
    ts_validator: &treesitter::TreeSitterAstValidator,
    ctx: &ValidationContext,
) {
    let regex_symbols = regex_validator.extract_symbols(ctx.patch_content, ctx.language.clone()).unwrap_or_default();
    let ts_symbols = ts_validator.extract_symbols(ctx.patch_content, ctx.language.clone()).unwrap_or_default();

    let regex_names: std::collections::HashSet<String> = regex_symbols.iter().map(|s| s.symbol.clone()).collect();
    let ts_names: std::collections::HashSet<String> = ts_symbols.iter().map(|s| s.symbol.clone()).collect();

    let missing_in_ts: Vec<String> = regex_names.difference(&ts_names).cloned().collect();
    let missing_in_regex: Vec<String> = ts_names.difference(&regex_names).cloned().collect();

    let mut divergences = Vec::new();

    for sym in &missing_in_ts {
        divergences.push(ShadowDivergence {
            kind: ShadowDivergenceKind::MissingSymbolTreeSitter,
            symbol: sym.clone(),
            details: "Found by Regex but missed by TreeSitter".to_string(),
        });
    }

    for sym in &missing_in_regex {
        // Here we could implement more advanced heuristic checks for Decorator, NestedImpl, etc.
        // For now, classify broadly as missed by Regex
        divergences.push(ShadowDivergence {
            kind: ShadowDivergenceKind::MissingSymbolRegex,
            symbol: sym.clone(),
            details: "Found by TreeSitter but missed by Regex".to_string(),
        });
    }

    for r_sym in &regex_symbols {
        if let Some(t_sym) = ts_symbols.iter().find(|s| s.symbol == r_sym.symbol) {
            if r_sym.range.start_byte != t_sym.range.start_byte || r_sym.range.end_byte != t_sym.range.end_byte {
                divergences.push(ShadowDivergence {
                    kind: ShadowDivergenceKind::RangeMismatch,
                    symbol: r_sym.symbol.clone(),
                    details: format!("regex[{}..{}] vs ts[{}..{}]", r_sym.range.start_byte, r_sym.range.end_byte, t_sym.range.start_byte, t_sym.range.end_byte),
                });
            }
        }
    }

    let comparison = ValidationComparison {
        file: "in-memory-patch".to_string(), // In context of a patch validation
        regex_count: regex_symbols.len(),
        treesitter_count: ts_symbols.len(),
        divergences,
    };

    if let Ok(json_line) = serde_json::to_string(&comparison) {
        use std::io::Write;
        let log_path = project_root.join("debug/ast_shadow_validation.jsonl");
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(file, "{}", json_line);
        }
    }

    // Materialize Metrics
    let metrics_path = project_root.join("debug/ast_shadow_metrics.json");
    let mut metrics: ShadowValidationMetrics = if let Ok(json) = std::fs::read_to_string(&metrics_path) {
        serde_json::from_str(&json).unwrap_or_default()
    } else {
        ShadowValidationMetrics::default()
    };

    metrics.total_validations += 1;
    if comparison.divergences.is_empty() {
        metrics.parser_agreements += 1;
    } else {
        metrics.parser_divergences += 1;
        for div in &comparison.divergences {
            match div.kind {
                ShadowDivergenceKind::MissingSymbolRegex => metrics.regex_false_negatives += 1,
                ShadowDivergenceKind::MissingSymbolTreeSitter => metrics.treesitter_false_negatives += 1,
                _ => {}
            }
            let key = format!("{:?}", div.kind);
            *metrics.divergence_kinds.entry(key).or_insert(0) += 1;
        }
    }

    if let Ok(metrics_json) = serde_json::to_string_pretty(&metrics) {
        let _ = std::fs::write(&metrics_path, metrics_json);
    }
}
