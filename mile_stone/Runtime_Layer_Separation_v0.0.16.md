# Runtime Layer Separation & Scheduler Positioning v0.0.16

---

## 1. Goal

Define strict separation between Architecture, Planning, and Runtime layers.
Ensure Scheduler does NOT interfere with architecture generation.

---

## 2. Core Principle

"WHAT / HOW / WHEN must be separated"

---

## 3. Layer Definitions

### 3.1 Architecture Layer (CTO)

Purpose:
- Define systems
- Generate contracts
- Establish constraints

Output:
- architecture.md

Scope:
- WHAT to build

Restrictions:
- No execution logic
- No scheduling logic
- No runtime assumptions

---

### 3.2 Planning Layer (PM / Planner)

Purpose:
- Convert contracts into task DAG

Output:
- execution plan

Scope:
- HOW to execute

Restrictions:
- No timing decisions
- No retry logic
- No runtime control

---

### 3.3 Runtime Layer

Components:
- Scheduler
- Executor
- State

Scope:
- WHEN and HOW execution happens in real-time

---

## 4. Scheduler Definition

Role:
- Decide execution timing
- Resolve dependencies
- Manage retries
- Handle parallelization

Scope:
- Runtime ONLY

Restrictions:
- Cannot modify contracts
- Cannot influence architecture
- Cannot generate new tasks

---

## 5. Allowed Interaction (Upstream Influence)

Scheduler may influence Architecture ONLY via constraints:

Example:

```json
{
  "constraints": [
    "must_support_parallel_execution",
    "max_latency_100ms"
  ]
}
```

This must be defined during contract generation, not runtime.

---

## 6. Forbidden Flows

❌ Scheduler → Architecture  
❌ Runtime → Contract Mutation  
❌ Planner → Runtime Control Logic  

---

## 7. Correct Pipeline

```text
[CTO]
Fragments
→ Systems
→ Normalized Systems
→ Contracts
→ Validation
→ architecture.md

[PM]
→ Execution Planner (Task DAG)

[Runtime]
→ Scheduler
→ Executor
→ State
```

---

## 8. Failure Cases

### Case 1: Scheduler leaks into Architecture
- Structure becomes execution-dependent
- Loss of generality

---

### Case 2: Planner handles runtime logic
- Retry loops embedded incorrectly
- Non-deterministic planning

---

### Case 3: Runtime mutates contracts
- System becomes unstable
- Validation meaningless

---

## 9. Success Criteria

- Clear layer boundaries
- No cross-layer logic leakage
- Deterministic architecture generation
- Stable runtime execution

---

## 10. Execution Principle

Separate → Isolate → Enforce

---

## 11. Conclusion

Scheduler is NOT a global orchestrator.

It is a strictly bounded runtime component that must not interfere
with architecture or planning stages.
