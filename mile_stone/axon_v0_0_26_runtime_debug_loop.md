# AXON v0.0.26 — Runtime Feedback Auto Debugging Loop

## Goal
Use runtime execution results to guide LLM toward minimal, localized fixes.

---

## Core Philosophy
- Do not regenerate everything
- Fix only the failing scope
- Convert runtime errors into structured feedback

---

## Pipeline
Build → Run → Capture Output → Analyze → Generate Hint → Retry (Scoped)

---

## Execution Stage
- Compile using CMake
- Run executable
- Capture:
  - stdout
  - stderr
  - exit code

---

## Error Classification

### Types
- Crash (segfault, abort)
- Assertion failure
- Runtime exception
- Incorrect output

---

## Diagnostic Structure
```
Diagnostic {
  type: RuntimeError,
  message: "...",
  file: optional,
  line: optional
}
```

---

## Cause Mapping

| Error | Cause |
|------|------|
| Segmentation fault | Invalid memory access |
| Assertion failed | Logic error |
| Wrong output | Algorithm error |

---

## Retry Scope

| Cause | Scope |
|------|------|
| Crash | ImplementationOnly |
| Assertion | ImplementationOnly |
| Wrong output | ImplementationOnly |
| Unknown | Full |

---

## Hint Generation

Example:
```
[HINT]
Check for null pointer before dereferencing.
Do not modify header.
```

---

## Retry Plan
```
RetryPlan {
  scope: ImplementationOnly,
  target_files: ["UserService.cpp"]
}
```

---

## Loop Control
- Max retry limit
- Stop on success
- Escalate scope if repeated failure

---

## Full Flow
```
Build
 ↓
Run
 ↓
Capture error
 ↓
Classify cause
 ↓
Generate hint
 ↓
Retry scoped fix
```

---

## Key Insight
Runtime errors are the most reliable signal for guiding LLM correction.

---

## Summary
- Runtime → Structured signal
- Signal → Scoped retry
- Scoped retry → Stability
