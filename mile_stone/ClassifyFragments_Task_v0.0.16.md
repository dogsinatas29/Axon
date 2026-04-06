# LLM Task Contract: ClassifyFragments v0.0.16

---

## 1. Goal

Classify every fragment into exactly one type.

---

## 2. Input

```json
{
  "fragments": [
    {
      "id": "string",
      "content": "string"
    }
  ]
}
```

---

## 3. Output

```json
{
  "tagged_fragments": [
    {
      "id": "string",
      "type": "SYSTEM | MODULE | FLOW | DATA | CONSTRAINT | UNKNOWN"
    }
  ]
}
```

---

## 4. Constraints

- Every fragment MUST have exactly one type
- Multiple types are NOT allowed
- New types are NOT allowed
- UNKNOWN is allowed

---

## 5. Classification Rules

### SYSTEM
- Externally callable unit
- Has input/output behavior

### MODULE
- Internal implementation
- Cannot operate independently

### FLOW
- Execution logic (sequence, loop, condition)

### DATA
- Structure, state, or value definition

### CONSTRAINT
- Limits, performance rules, restrictions

### UNKNOWN
- Does not fit above categories

---

## 6. Success Criteria

- tagged_fragments length == fragments length
- Every fragment has a valid type
- Types are within allowed enum

---

## 7. Failure Conditions

- Missing type in any fragment
- Invalid type value
- Missing fragment in output

---

## 8. Notes

- This is a foundational task
- All downstream tasks depend on correctness here
- Failure must STOP pipeline
