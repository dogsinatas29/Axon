# SYNAPSE Senior Gate Reject Rule Set v0.1

## 1. Purpose

Defines strict rejection criteria for the final human (Senior) validation stage
after logical (Axon) and physical validation layers.

Senior Gate MUST remain minimal, deterministic, and high-signal.

---

## 2. Gate Philosophy

- Reject only structural or systemic risks
- Do NOT re-validate logic already proven by Axon
- Do NOT re-check items covered by automated validators
- Focus on: architecture integrity, side-effects, and long-term maintainability

---

## 3. Reject Conditions (Critical)

### R1: Architecture Violation
- ECS principles broken
- Modules not isolated
- Hidden coupling between layers

→ Immediate REJECT

---

### R2: Undeclared Side Effects
- File writes outside expected scope
- Hidden global state mutation
- Non-explicit DB writes

→ REJECT

---

### R3: Entry Point Inconsistency
- Missing or incorrect entry function
- Execution path ambiguity

→ REJECT

---

### R4: Environment Dependency Leak
- Hardcoded paths
- OS-specific assumptions
- Missing dependency declaration

→ REJECT

---

### R5: Non-Deterministic Behavior
- Randomness without seed
- Time-based logic without control
- Order-dependent execution

→ REJECT

---

## 4. Reject Conditions (High Risk)

### R6: Over-Engineering
- Unnecessary abstraction layers
- Trait/Generic overuse (Rust)
- Premature optimization

→ REJECT if it impacts readability or maintainability

---

### R7: Incomplete Error Handling
- unwrap() in critical path
- panic usage
- missing Result propagation

→ REJECT

---

### R8: Inconsistent State Flow
- State transitions not matching IR
- Missing validation before compute

→ REJECT

---

## 5. Soft Reject / Review Required

### R9: Naming Inconsistency
- Non-descriptive identifiers
- Mixed conventions

---

### R10: Minor Structural Drift
- Slight deviation from spec but safe

---

## 6. Explicit Allowances

Senior MUST NOT reject for:

- Coding style preferences
- Formatting differences
- Equivalent alternative implementations
- Performance micro-optimizations (unless harmful)

---

## 7. Decision Output Format

```json
{
  "decision": "APPROVE | REJECT",
  "reasons": ["R1", "R4"],
  "notes": "Optional explanation"
}
```

---

## 8. Operational Constraints

- Max review time: 2 minutes
- Max reject reasons: 3
- If unclear → APPROVE (default bias toward flow)

---

## 9. Summary

Senior Gate acts as:

- Final anomaly detector
- Not a re-validator
- Not a bottleneck

Goal: Catch what automation cannot, and nothing more.
