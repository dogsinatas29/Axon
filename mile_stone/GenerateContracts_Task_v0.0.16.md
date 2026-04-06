# LLM Task Contract: GenerateContracts v0.0.16

---

## 1. Goal

Convert NormalizedSystem into a complete, executable contract specification.

---

## 2. Input

```json
{
  "normalized_systems": [
    {
      "name": "string",
      "inputs": [],
      "outputs": [],
      "components": [],
      "flows": [],
      "constraints": [],
      "related_fragments": []
    }
  ]
}
```

---

## 3. Output

```json
{
  "contracts": [
    {
      "name": "string",
      "goal": "string",
      "inputs": [],
      "outputs": [],
      "components": [],
      "execution_flow": [],
      "constraints": [],
      "success_criteria": [],
      "failure_conditions": []
    }
  ]
}
```

---

## 4. Processing Rules

### 4.1 Goal Generation
- Derive from system name + inputs + outputs
- Must be concise and measurable

---

### 4.2 Inputs / Outputs
- Use normalized data as primary source
- Allow minimal inference if missing

---

### 4.3 Components
- Directly map from normalized components
- Normalize naming if required

---

### 4.4 Execution Flow
- Construct ordered steps from flow fragments
- Minimum 3 steps required
- Generate minimal flow if missing

---

### 4.5 Constraints
- Include inherited constraints
- Enforce:
  - bounded loops
  - resource limits

---

### 4.6 Success Criteria
- Must be measurable
- Must include:
  - output validation
  - size/type checks
  - time constraints

---

### 4.7 Failure Conditions
- Derived from constraint violations
- Include:
  - runtime errors
  - output generation failure

---

## 5. Constraints

- ALL fields MUST be filled
- No new systems may be created
- Structure MUST NOT be modified
- Minimal inference only

---

## 6. Success Criteria

- All contract fields present
- Goal is measurable
- Execution flow has ≥ 3 steps
- Success and failure conditions exist

---

## 7. Failure Conditions

- Missing fields
- Vague or non-measurable goal
- Missing execution flow
- Missing constraints

---

## 8. Notes

- This is NOT a creative writing task
- This is a structure completion task
- Avoid over-generation

---

## 9. Execution Principle

Compress + Complete + Formalize

---

## 10. Conclusion

This stage transforms structured system data into enforceable contracts.

Validator stage is required to ensure correctness.
