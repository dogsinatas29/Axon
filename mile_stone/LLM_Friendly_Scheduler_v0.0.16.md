# LLM-Friendly Scheduler Design v0.0.16

---

## 1. Goal

Design a scheduler that remains stable under LLM uncertainty.
Ensure convergence, prevent infinite loops, and maximize completion reliability.

---

## 2. Core Principle

"Stability over optimality"

The scheduler must prioritize:
- Completion certainty
- State convergence
- Loop prevention

---

## 3. Key Concepts

### 3.1 Completion-Driven Scheduling

- Prefer tasks that can fully complete
- Prioritize finishing independent chains

---

### 3.2 Uncertainty Minimization

- Favor tasks with higher success probability
- Avoid tasks likely to fail repeatedly

---

### 3.3 State Convergence

- Select tasks that reduce unresolved dependencies
- Favor tasks that produce new valid state

---

## 4. Task Scoring Model

```text
score =
  priority_weight
+ success_probability
+ dependency_resolution_score
+ state_progress_score
- retry_penalty
```

---

## 5. Ready Queue Processing

- Do NOT use FIFO
- Evaluate all READY tasks
- Select highest score

---

## 6. Progress Monitoring

### 6.1 Stall Detection

Detect when:
- No state change over N steps
- Repeated outputs occur

Action:
- Force FAIL
- Stop retries

---

## 7. Retry Strategy

### Rules:
- Max retry: 2
- Retry only with modifications
- No blind re-execution

### Retry must include:
- Reduced ambiguity
- Explicit fix instructions

---

## 8. LLM Interaction Model

LLM is:
- A scoring assistant
- Not a decision authority

Input example:

```json
{
  "tasks": [],
  "state_summary": "",
  "objective": "maximize_completion"
}
```

---

## 9. Safety Mechanisms

### 9.1 Deterministic Fallback
- If LLM fails → fallback to static priority

---

### 9.2 Hard Limits
- max_retry
- max_steps
- max_stall_window

---

### 9.3 Task Finality
- Completed tasks cannot re-enter queue

---

## 10. Constraints

- No task creation
- No contract mutation
- No priority reassignment

---

## 11. Success Criteria

- System converges to completion
- No infinite loops
- Deterministic behavior maintained

---

## 12. Failure Conditions

- Infinite retry loop
- No state progression
- Task repetition
- Priority inversion

---

## 13. Execution Principle

Score → Select → Execute → Converge

---

## 14. Conclusion

This scheduler is not designed for maximum efficiency,
but for maximum stability under uncertainty.

It ensures the system moves toward completion even in imperfect conditions.
