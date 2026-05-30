# AXON Governance Model

## Phase Gating

Tasks execute in strict dependency order:

```
Phase 1 (Headers) → Phase 2 (Implementations) → Phase 3 (Integration)
```

- Each phase must complete before the next begins
- `resolve_target()` closure parses `target_file` from `task.title` when `None`
- Prevents invalid execution ordering

## Transaction Envelope

All generated patches are wrapped in a deterministic transaction boundary:

```
===AXON_PATCH_BEGIN===
BODY
===AXON_PATCH_END===
```

This enables:
- **Patch completeness verification** — BEGIN/BODY/END presence check
- **Truncation detection** — missing END marker = incomplete generation
- **Atomic replay validation** — envelope can be replayed identically
- **Causal rejection** — incomplete generations are explicitly rejected

The envelope is not a formatting convention — it is a **transmission integrity layer**
that provides a causal transaction boundary and patch completeness proof.

### Envelope Validation Rules

| Condition | Result |
|-----------|--------|
| Missing `AXON_PATCH_END` | Reject |
| Empty body | Reject |
| BYTE_COUNT mismatch (if declared) | Reject |
| Malformed structure | Reject |

## Replay Validation

- Every generation is assigned a unique `patch_id`
- Replay identity guarantees deterministic re-execution
- Non-deterministic replay is detected and flagged

## Rejection Semantics

Senior agents provide structured JSON feedback:

```json
{
  "decision": "reject",
  "reason": "violation_type",
  "details": "specific violation description"
}
```

- Causal rejection with specific violation reasons
- Enables precise self-correction loops
- Prevents blind approval patterns
