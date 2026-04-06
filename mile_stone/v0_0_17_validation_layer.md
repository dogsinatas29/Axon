# v0.0.17 Validation & Interaction Layer (Protocol Edition)

---

## 1. Objective

Convert validation from a static guideline into an executable protocol.

System goals:
- Prevent execution deadlock
- Maintain continuous progress
- Control validation cost
- Enable deterministic behavior across agents

---

## 2. Core Principle

Perfect validation is not the goal.

Non-blocking execution is the priority.

---

## 3. System Roles

### Validator
- Performs staged validation
- Produces bounded feedback
- Cannot block execution beyond defined rules

### Executor (Junior)
- Executes tasks
- Fixes only explicitly reported issues
- Cannot redesign or expand scope

### Architect
- Handles structural failures (Stage B/C)
- Resolves systemic issues

---

## 4. Validation Protocol

### Stage A — Sanity Check

Input:
- Task
- Current State

Process:
- Validate basic executability

Reject ONLY if:
- Execution impossible
- Data corruption risk
- Infinite loop risk

Output:
- PASS | REJECT
- reason (1 line)

---

### Stage B — Structural Check

Input:
- Execution Plan
- State Snapshot

Process:
- Validate consistency
- Check idempotency

Output:
- PASS | SOFT_REJECT

Rules:
- MUST NOT block execution
- Failures are reported to Architect

---

### Stage C — Adversarial Check

Input:
- Near-final output

Process:
- Simulate failure scenarios

Output:
- PASS | SOFT_REJECT

Rules:
- Execute only once before release
- MUST NOT block execution

---

## 5. Output Contract (Validator)

Format:

[RESULT]
PASS | REJECT | SOFT_REJECT

[ISSUES]
1. ...
2. ...
3. ...

Constraints:
- Max 3 issues
- Max 5 lines per issue

[FIX_SCOPE]
- Exact components or lines only

---

## 6. Executor Contract

Execution Rules:

IF Stage A == PASS:
→ Execute immediately

IF REJECT:
→ Fix ONLY reported issues
→ Retry

Constraints:
- No redesign
- No scope expansion
- Modify only specified areas

---

## 7. Loop Controller

Track:
- retry_count per task
- error_signature

Rules:
- Max retry: 3

IF retry_count > 3:
→ FORCE STOP
→ Escalate to Architect

IF same error repeats:
→ Immediate STOP

---

## 8. Failure Routing

Stage A Failure:
→ Executor fixes

Stage B/C Failure:
→ Execution continues
→ Report to Architect (async)

---

## 9. Operational Strategy

Design Phase:
- Allow ~70% correctness

Implementation Phase:
- Prioritize speed

Execution Phase:
- Correct based on real errors

---

## 10. Safety Constraints

Validator MUST NOT:
- Block execution beyond Stage A
- Produce unbounded feedback

Executor MUST NOT:
- Modify unrelated code
- Attempt architectural changes

---

## 11. Termination Condition

System stops ONLY when:
- Task completed
OR
- Loop Controller triggers FORCE STOP

---

## 12. Conclusion

A system that continues executing is more valuable than a system that validates perfectly.

Progressive Validation ensures:
- Deadlock prevention
- Controlled cost
- Continuous execution
