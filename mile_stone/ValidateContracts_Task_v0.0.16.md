# LLM Task Contract: ValidateContracts v0.0.16

---

## 1. Goal

Validate generated contracts for executability, correctness, and completeness.
Prevent infinite retry loops by enforcing strict termination conditions.

---

## 2. Input

```json
{
  "contract": {
    "name": "string",
    "goal": "string",
    "inputs": [],
    "outputs": [],
    "components": [],
    "execution_flow": [],
    "constraints": [],
    "success_criteria": [],
    "failure_conditions": []
  },
  "retry_count": "number",
  "previous_errors": []
}
```

---

## 3. Output

```json
{
  "status": "PASS | FIX | FAIL",
  "reason": "string",
  "fix_instructions": []
}
```

---

## 4. Validation Rules

### 4.1 Goal
- Must be measurable
- No vague expressions allowed

---

### 4.2 Execution Flow
- Minimum 3 steps
- Must be ordered and explicit

---

### 4.3 Success Criteria
- Must be testable
- Must include output validation

---

### 4.4 Inputs / Outputs
- Must define structure or type

---

### 4.5 Constraints
- At least one required

---

## 5. Decision Logic

### PASS
- All validation rules satisfied

---

### FIX
- Minor issues detected
- Fixable via explicit instructions

---

### FAIL
- Structural issues
- Non-recoverable
- Retry limit exceeded

---

## 6. Loop Breaker Rules

### 6.1 Retry Limit

```yaml
max_retry: 2
```

- If retry_count >= max_retry → FAIL

---

### 6.2 Repeated Error Detection

- If same error appears multiple times → FAIL

---

### 6.3 No Improvement Detection

- If contract changes are insignificant → FAIL

---

## 7. Fix Instructions

- Must be actionable
- Must be specific
- No vague guidance

Example:

```json
{
  "fix_instructions": [
    "Define measurable success criteria",
    "Expand execution steps to at least 3",
    "Specify output schema"
  ]
}
```

---

## 8. Constraints

- Structure MUST NOT change
- No new systems allowed
- Only correction permitted

---

## 9. Success Criteria

- Validator returns correct status
- Retry loop is bounded
- No infinite loop possible

---

## 10. Failure Conditions

- Missing required fields
- Vague definitions
- Repeated failures
- Retry overflow

---

## 11. Execution Principle

Validate + Constrain + Terminate

---

## 12. Conclusion

This stage ensures contracts are enforceable and prevents infinite correction loops.

It is a critical control layer in the LLM pipeline.
