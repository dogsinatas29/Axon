# AXON Failure Model

## PATCH_TRUNCATED

**Trigger:** Empty or incomplete generation

**Evidence:**
- Empty validator response (`{}`)
- Missing `===AXON_PATCH_END===` marker
- Context exhaustion (output window starvation)

**Action:** Hard reject — not neutral, not retry

## PROOF_CORRUPTED

**Trigger:** Integrity validation failure

**Evidence:**
- BYTE_COUNT mismatch
- CHECKSUM validation failure
- Malformed envelope structure

**Action:** Reject with causal evidence

## CATASTROPHIC

**Trigger:** Pipeline-level failure

**Evidence:**
- Pipeline deadlock
- IR seal violation
- Boss interrupt signal

**Action:** Immediate abort, rollback to last stable state

## Replay Divergence

**Trigger:** Non-deterministic behavior across iterations

**Evidence:**
- Same input produces different output
- Generation drift across replays
- Token usage anomaly

**Action:** Flag for investigation, quarantine patch
