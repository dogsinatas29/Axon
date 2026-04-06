# LLM Step Execution Visualization & Progress Tracking v0.0.16

---

## 1. Goal

Provide a sequential, human-readable execution trace of the entire pipeline.
Enable real-time understanding of progress and immediate identification of failures.

---

## 2. Core Principle

"Make invisible execution visible"

Every step must be:
- Ordered
- Observable
- Interruptible
- Diagnosable

---

## 3. Execution View Model

Each step must be displayed as:

```
[Step N] <Stage Name>

Task:
<description>

Progress:
[======>       ] 45%

Status:
RUNNING | SUCCESS | FAILED

Error (if any):
<error message>
```

---

## 4. Required Fields

### 4.1 Step Index
- Monotonic increasing
- No skipping

---

### 4.2 Stage Name
- generation
- validation
- planning
- scheduling
- execution

---

### 4.3 Task Description
- Human-readable
- Derived from contract/task

---

### 4.4 Progress Indicator
- Percentage OR step-based
- Must be updated in real-time

---

### 4.5 Status

- PENDING
- RUNNING
- SUCCESS
- FAILED

---

### 4.6 Error Section

- Only visible on failure
- Must include:
  - error type
  - short reason
  - failing component

---

## 5. Sequential Execution Flow

```
Step 1 → Step 2 → Step 3 → ... → Final Step
```

Rules:

- Steps must not execute out of order (view perspective)
- Parallel execution must still be serialized in display

---

## 6. Parallel Handling

If parallel tasks exist:

```
[Step 5] Scheduling

Subtasks:
- Task A [RUNNING]
- Task B [SUCCESS]
- Task C [FAILED]
```

---

## 7. Error Visualization

On failure:

```
[Step 7] Execution

Task:
Render Frame

Progress:
[=====>      ] 60%

Status:
FAILED

Error:
ExecutionError: Missing input state "frame_buffer"
```

---

## 8. Human Intervention Points

System must pause and expose state when:

- FAILED
- FATAL error
- Retry limit exceeded
- Stall detected

---

## 9. Constraints

- No hidden steps
- No silent transitions
- No aggregated logs only (must be step-level)

---

## 10. Success Criteria

- User can trace full execution path
- Failure location immediately visible
- No ambiguity in step responsibility

---

## 11. Failure Conditions

- Missing step logs
- Out-of-order display
- Hidden errors
- Unclear task descriptions

---

## 12. Execution Principle

Show → Update → Explain

---

## 13. Conclusion

This layer transforms the system from a black box into a transparent pipeline.

Without step-level visibility, debugging and human intervention are not possible.
