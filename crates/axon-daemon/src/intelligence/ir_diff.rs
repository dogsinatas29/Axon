use axon_core::rules::Constraint;
use axon_core::validator::types::FunctionSig;
use axon_core::ir::ProjectIR;
use super::staging::ConstraintProposal;
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────
// 1. Diff 정의 (IR spec vs extracted code)
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum IRDiff {
    MissingFunction { name: String },
    ExtraFunction   { name: String },
    SignatureMismatch {
        name:     String,
        expected: Vec<String>,
        actual:   Vec<String>,
    },
}

// ─────────────────────────────────────────────────────────────
// 2. Deterministic diff 생성 (IR spec vs 실제 추출된 함수들)
// ─────────────────────────────────────────────────────────────

pub fn diff(ir: &ProjectIR, extracted: &[FunctionSig]) -> Vec<IRDiff> {
    let mut result = Vec::new();

    // Collect every function name declared in the IR spec
    let mut spec_fns: HashMap<String, Vec<String>> = HashMap::new();
    for comp in ir.components.values() {
        for (fname, func) in &comp.functions {
            // Parse simple "name(a, b)" signature into arg list
            let args = parse_sig_args(&func.signature);
            spec_fns.insert(fname.clone(), args);
        }
    }

    let extracted_map: HashMap<&str, &FunctionSig> =
        extracted.iter().map(|f| (f.name.as_str(), f)).collect();

    // MissingFunction / SignatureMismatch
    for (name, expected_args) in &spec_fns {
        match extracted_map.get(name.as_str()) {
            None => {
                result.push(IRDiff::MissingFunction { name: name.clone() });
            }
            Some(found) if &found.args != expected_args => {
                result.push(IRDiff::SignatureMismatch {
                    name:     name.clone(),
                    expected: expected_args.clone(),
                    actual:   found.args.clone(),
                });
            }
            _ => {}
        }
    }

    // ExtraFunction
    for f in extracted {
        if !spec_fns.contains_key(&f.name) {
            result.push(IRDiff::ExtraFunction { name: f.name.clone() });
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────
// 3. 빈도 기반 Proposal 생성 (임계치 N 이상만 승격 후보)
// ─────────────────────────────────────────────────────────────

const PROMOTION_THRESHOLD: u32 = 3; // 같은 diff가 N번 이상 나와야 제안

pub fn propose_from_diff(
    diffs: &[IRDiff],
    freq_store: &mut HashMap<String, u32>,
) -> Vec<ConstraintProposal> {
    let mut proposals = Vec::new();

    for d in diffs {
        let (key, candidate): (String, Option<ConstraintProposal>) = match d {
            IRDiff::MissingFunction { name } => {
                let key = format!("missing:{}", name);
                let proposal = ConstraintProposal {
                    constraint: Constraint::ExactFunctionExists { name: name.clone() },
                    source_rule: format!("MissingFunction({}) repeated", name),
                };
                (key, Some(proposal))
            }
            IRDiff::SignatureMismatch { name, expected, .. } => {
                let key = format!("sig:{}", name);
                let proposal = ConstraintProposal {
                    constraint: Constraint::ExactSignatureMatch {
                        name: name.clone(),
                        args: expected.clone(),
                    },
                    source_rule: format!("SignatureMismatch({}) repeated", name),
                };
                (key, Some(proposal))
            }
            IRDiff::ExtraFunction { name } => {
                // ExtraFunction은 단순 경고만; NoExtraFunctions는 전역이라
                // 빈도 높으면 추가 가능
                let key = format!("extra:{}", name);
                (key, None)
            }
        };

        let count = freq_store.entry(key).or_insert(0);
        *count += 1;

        if *count >= PROMOTION_THRESHOLD {
            if let Some(p) = candidate {
                proposals.push(p);
            }
        }
    }

    proposals
}

// ─────────────────────────────────────────────────────────────
// helpers
// ─────────────────────────────────────────────────────────────

fn parse_sig_args(sig: &str) -> Vec<String> {
    // "foo(a, b, c)" → ["a", "b", "c"]
    if let (Some(l), Some(r)) = (sig.find('('), sig.rfind(')')) {
        let inner = &sig[l + 1..r];
        if inner.trim().is_empty() {
            return vec![];
        }
        return inner.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    vec![]
}
