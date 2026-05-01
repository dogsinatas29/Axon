# SYNAPSE Physical Validator Checklist (Automation-Level) v0.1

## 1. Purpose

Defines an automated validation checklist executed AFTER materialization (real file system)
and BEFORE Senior Gate.

This layer ensures that sandbox success is reproducible in the real environment.

---

## 2. Validation Scope

- File system integrity
- Build / execution validity
- Dependency resolution
- Entry point correctness
- Runtime determinism (basic level)

---

## 3. Checklist (Automatable)

### F1: File Materialization

- All expected files exist
- No missing modules
- No unexpected extra files (drift)

Check:
- fs.exists(path)
- compare with IR file list

---

### F2: File Integrity

- File is readable
- Encoding is valid (UTF-8)
- No zero-byte files

---

### F3: Entry Point Validation

- Entry file exists (e.g., main.rs)
- Entry function exists (fn main())
- Executable path resolvable

---

### F4: Build Validation (Rust)

- cargo build succeeds
- No compile errors
- No missing crates

Command:
- cargo check
- cargo build

---

### F5: Dependency Resolution

- Cargo.toml exists
- All dependencies resolvable
- No version conflicts

---

### F6: Runtime Execution

- Program executes without crash
- Exit code == 0
- No panic in runtime

Command:
- cargo run

---

### F7: Environment Consistency

- No hardcoded absolute paths
- No OS-specific path issues
- Relative paths only (where required)

---

### F8: Import / Module Resolution

- All modules correctly linked
- No missing mod/use statements

---

### F9: Side-Effect Boundary Check

- File writes only in allowed directory
- No unexpected external I/O
- DB access matches spec

---

### F10: Deterministic Replay (Basic)

- Same input → same output (single run check)
- No random or time drift

---

## 4. Failure Handling

If ANY check fails:

- Abort pipeline
- Trigger rollback
- Emit structured error report

---

## 5. Output Format

```json
{
  "status": "PASS | FAIL",
  "failed_checks": ["F3", "F6"],
  "logs": "Optional execution logs"
}
```

---

## 6. Execution Order

1. File checks (F1~F2)
2. Structure checks (F3~F5)
3. Build checks (F4)
4. Runtime checks (F6~F10)

---

## 7. Optional Advanced Checks

- Multi-run determinism (N=3)
- Memory leak detection
- File descriptor leak
- Thread safety (if async)

---

## 8. Summary

Physical Validator is:

- Fully automatable
- Deterministic
- Fast-fail oriented

Goal: Catch environment & runtime issues BEFORE human review.
