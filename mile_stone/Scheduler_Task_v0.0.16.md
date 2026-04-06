# LLM Task Contract: Scheduler v0.0.16

---

## 1. Goal

Define a deterministic scheduler that consumes a prioritized task list
and ensures monotonic task completion (work strictly decreases over time).

---

## 2. Core Principle

"Pre-determined priority → Continuous draining execution"

- Task list may grow
- Executable workload must strictly decrease

---

## 3. Input

```json
{
  "tasks": [],
  "state": {},
  "retry_queue": []
}
```

---

## 4. Output

```json
{
  "next_tasks": [],
  "execution_mode": "sequential | parallel"
}
```

---

## 5. Scheduler Model

### 5.1 Priority-Driven Execution

- Each task has priority
- Higher priority executes first
- No dynamic priority mutation at runtime

---

### 5.2 Ready Queue

A task is READY if:
- All dependencies are resolved
- Required state exists

---

### 5.3 Draining Guarantee

Scheduler must ensure:

- Total unresolved dependency count decreases
- Completed tasks are never re-added
- Retry is bounded

---

## 6. Retry Policy

```yaml
max_retry: 2
```

- Retry only on FIX-type failures
- Retry does not increase priority
- Retry queue processed after primary queue

---

## 7. Execution Modes

### Sequential
- Dependency chain exists

### Parallel
- No shared dependencies
- No state conflict

---

## 8. Constraints

- No task creation
- No contract modification
- No priority reassignment

---

## 9. Success Criteria

- Task queue monotonically drains
- No infinite loops
- Deterministic task selection

---

## 10. Failure Conditions

- Task reappears after completion
- Dependency deadlock
- Infinite retry loop
- Priority inversion

---

## 11. Execution Flow

```text
Task Queue (Prioritized)
   ↓
Filter READY tasks
   ↓
Select highest priority
   ↓
Dispatch to Executor
   ↓
Update State
   ↓
Repeat
```

---

## 12. Execution Principle

Select → Dispatch → Reduce

---

## 13. Conclusion

Scheduler enforces convergence.

Even if tasks are dynamically added, the system must always trend toward completion.
