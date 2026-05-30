# AXON Lineage Taxonomy

## Causal Family

- Generation lineage tracking via `patch_id`
- Parent-child patch relationships
- Similarity scoring across generations

## Normalization

- IR normalization rules per language
- Cross-language constraint mapping
- Symbol ownership resolution

## Active IR Validators

| Language | Validator Path | Status |
|----------|---------------|--------|
| C (GTK4) | `crates/axon-ir/src/validator/langs/c.rs` | Active |
| Rust | `crates/axon-ir/src/validator/langs/rust.rs` | Active |
| Python | `crates/axon-ir/src/validator/langs/python.rs` | Active |
