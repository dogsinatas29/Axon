# Automated LLM Contract Pipeline Blueprint v0.0.16

## 1. Core Principle
Full automation is valid, but must be controlled.

Key rule:
Automation + Forced Structure + Fail Fast

---

## 2. Pipeline Overview

AUTO PIPELINE:
- Fragment
- Tag
- Extract
- Normalize
- Contract
- Validate

Rule:
If validation fails → STOP (no fallback)

---

## 3. System Philosophy

- Generation is automated
- Responsibility remains with human

---

## 4. Mandatory Constraints

### 4.1 Contract Schema (Strict)

All outputs MUST follow:

{
  "goal": "...",
  "inputs": [...],
  "outputs": [...],
  "components": [...],
  "flow": [...],
  "constraints": [...],
  "success": [...],
  "failure": [...]
}

Missing field → INVALID

---

### 4.2 System Qualification

A SYSTEM must have:
- Defined Inputs
- Defined Outputs
- Independent execution capability

Otherwise → downgrade to MODULE

---

### 4.3 Flow Safety

All flows must satisfy:
- Bounded loops
- No infinite execution paths
- Step limits defined

---

### 4.4 Resource Constraints

Each system must define:
- Time budget
- Memory constraints

Missing → FAIL

---

## 5. LLM Usage Strategy

### Incorrect Approach

"Structure this document"

→ unreliable

---

### Correct Approach (Stepwise)

1. Fragmentation (non-LLM or simple split)

2. Classification
- Assign type:
  SYSTEM / MODULE / FLOW / DATA / CONSTRAINT

3. Extraction
- Extract SYSTEM only
- Deduplicate
- Normalize names

4. Contract Generation
- Convert SYSTEM into strict schema

5. Validation
- Check execution validity
- Return YES / NO with reason

---

## 6. Critical Insight

LLM strength:
- Structuring
- Summarization
- Reasoning

LLM weakness:
- Rule enforcement

Solution:
Rules must be enforced by code, not prompts

---

## 7. Success Conditions

System succeeds only if:

1. Phases are strictly separated
2. Contract schema is enforced
3. Validation stops invalid outputs

---

## 8. Missing Component

Required:

LLM Executor

Responsible for:
- Step orchestration
- Constraint enforcement
- Failure handling

---

## 9. Next Step

Define LLM Task Contracts

Example:

Task: ClassifyFragments
Input: Fragment[]
Output: TaggedFragment[]

Constraints:
- Must assign one of:
  SYSTEM / MODULE / FLOW / DATA / CONSTRAINT / UNKNOWN

Failure:
- Unclassified fragment → FAIL

---

## 10. Final Conclusion

Automation is correct direction.

But the real system is:

"Controlled execution of LLM under strict constraints"
