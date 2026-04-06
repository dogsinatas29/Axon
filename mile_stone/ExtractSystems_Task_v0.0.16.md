# LLM Task Contract: ExtractSystems v0.0.16

---

## 1. Goal

Extract SYSTEM entities from tagged fragments, normalize their names, and merge duplicates.

---

## 2. Input

```json
{
  "tagged_fragments": [
    {
      "id": "string",
      "type": "SYSTEM | MODULE | FLOW | DATA | CONSTRAINT | UNKNOWN",
      "content": "string"
    }
  ]
}
```

---

## 3. Output

```json
{
  "systems": [
    {
      "name": "string",
      "source_fragments": ["fragment_id"]
    }
  ]
}
```

---

## 4. Processing Rules

### 4.1 Filtering
- Only fragments with type == SYSTEM are considered

---

### 4.2 Name Extraction
- Convert action phrases into noun-based system names

Examples:
- "render frame" → RenderingSystem
- "handle input" → InputSystem
- "update physics" → PhysicsSystem

---

### 4.3 Normalization
- Use PascalCase
- Must end with "System"

---

### 4.4 Deduplication
- Merge semantically identical systems

Examples:
- render / rendering / draw → RenderingSystem

---

## 5. Constraints

- Non-SYSTEM fragments MUST NOT be included
- System names MUST be unique
- Each system MUST reference at least one fragment
- All names MUST end with "System"

---

## 6. Success Criteria

- systems.length ≥ 1 (if SYSTEM fragments exist)
- All system names are unique
- Every system has source_fragments

---

## 7. Failure Conditions

- SYSTEM fragments exist but no systems extracted
- Duplicate system names remain
- System without source_fragments exists

---

## 8. Notes

- This step defines architectural boundaries
- Over-splitting creates system explosion
- Over-merging creates monolithic systems
- Maintain “independently executable unit” granularity

---

## 9. Conclusion

This stage establishes the architectural skeleton.

All downstream contract generation depends on the correctness of this step.
