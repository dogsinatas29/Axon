use axon_core::ir::ProjectIR;
use axon_core::validator::types::FunctionSig;
use axon_core::rules::Constraint;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::commit::IRCommitLayer;
use super::constraint_meta::ConstraintMeta;
use super::ir_diff::{diff, propose_from_diff, IRDiff};

const PRUNE_INTERVAL: u32 = 10;   // every N commits
const PRUNE_TTL_SECS: u64 = 86400; // 24h

// ─────────────────────────────────────────────────────────────
// Persistent system state across validation cycles
// ─────────────────────────────────────────────────────────────

pub struct SystemState {
    pub commit_count: u32,
    pub diff_freq: HashMap<String, u32>,                  // key → occurrence count
    pub constraint_meta: HashMap<String, ConstraintMeta>, // constraint_key → meta
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            commit_count: 0,
            diff_freq: HashMap::new(),
            constraint_meta: HashMap::new(),
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Core orchestration function
// ─────────────────────────────────────────────────────────────

/// Single entry point for one validation cycle.
///
/// Order:
///   1. diff(IR, extracted)        → what's wrong right now
///   2. propose_from_diff(diffs)   → frequency-gated proposals
///   3. apply_proposals(ir, …)     → single atomic write
///   4. update_all_meta(…)         → priority / confidence
///   5. prune(ir, …) every N       → remove stale constraints
pub fn on_validation_cycle(
    state: &mut SystemState,
    ir: &mut ProjectIR,
    extracted: &[FunctionSig],
) {
    let now = epoch_secs();

    // ── Step 1: Structural diff ───────────────────────────────
    let diffs = diff(ir, extracted);

    if !diffs.is_empty() {
        println!("[ORCH] {} diff(s) detected this cycle", diffs.len());
    }

    // ── Step 2: Frequency-gated proposals ────────────────────
    let proposals = propose_from_diff(&diffs, &mut state.diff_freq);

    // ── Step 3: Atomic IR write (dedup inside) ────────────────
    IRCommitLayer::apply_proposals(ir, proposals);
    state.commit_count += 1;

    // ── Step 4: Update meta for every live constraint ─────────
    update_all_meta(state, ir, &diffs, now);

    // ── Step 5: Periodic pruning ──────────────────────────────
    if state.commit_count % PRUNE_INTERVAL == 0 {
        prune(state, ir, now);
    }
}

// ─────────────────────────────────────────────────────────────
// Meta update
// ─────────────────────────────────────────────────────────────

fn update_all_meta(
    state: &mut SystemState,
    ir: &ProjectIR,
    diffs: &[IRDiff],
    now: u64,
) {
    // Build a set of constraint keys that were violated this cycle
    let violated_keys: std::collections::HashSet<String> = diffs.iter().map(|d| match d {
        IRDiff::MissingFunction { name } =>
            constraint_key(&Constraint::ExactFunctionExists { name: name.clone() }),
        IRDiff::SignatureMismatch { name, expected, .. } =>
            constraint_key(&Constraint::ExactSignatureMatch { name: name.clone(), args: expected.clone() }),
        IRDiff::ExtraFunction { .. } =>
            constraint_key(&Constraint::NoExtraFunctions),
    }).collect();

    for c in &ir.constraints {
        let key = constraint_key(c);
        let meta = state.constraint_meta.entry(key.clone()).or_default();
        let violated = violated_keys.contains(&key);
        meta.update(true, violated, now);
    }
}

// ─────────────────────────────────────────────────────────────
// Pruning
// ─────────────────────────────────────────────────────────────

fn prune(state: &mut SystemState, ir: &mut ProjectIR, now: u64) {
    let before = ir.constraints.len();

    ir.constraints.retain(|c| {
        let key = constraint_key(c);
        let should_drop = state.constraint_meta
            .get(&key)
            .map(|m| m.should_prune(now, PRUNE_TTL_SECS))
            .unwrap_or(false);
        if should_drop {
            state.constraint_meta.remove(&key);
            ir.constraint_ids.remove(&ir_hash(c));
        }
        !should_drop
    });

    let removed = before - ir.constraints.len();
    if removed > 0 {
        println!("[ORCH] Pruned {} stale constraint(s)", removed);
    }
}

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

fn constraint_key(c: &Constraint) -> String {
    format!("{:?}", c)
}

fn ir_hash(c: &Constraint) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut h = DefaultHasher::new();
    c.hash(&mut h);
    h.finish()
}

fn epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
