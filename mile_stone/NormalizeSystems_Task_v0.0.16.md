# LLM Task Contract: NormalizeSystems v0.0.16

---

## 1. Goal

Transform SystemCandidate into structured, contract-ready System by aggregating and organizing related fragments.

---

## 2. Input

```json
{
  "systems": [
    {
      "name": "string",
      "source_fragments": ["id"]
    }
  ],
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
  "normalized_systems": [
    {
      "name": "string",
      "inputs": [],
      "outputs": [],
      "components": [],
      "flows": [],
      "constraints": [],
      "related_fragments": ["id"]
    }
  ]
}
```

---

## 4. Processing Strategy

### 4.1 Anchor Initialization
- Start from system.source_fragments as anchor points

---

### 4.2 Fragment Expansion

Attach additional fragments based on:

- Keyword similarity (e.g., "render", "frame")
- Shared variables or data references
- Flow linkage mentioning the system

---

### 4.3 Classification Mapping

Map fragments into structure:

| Fragment Type | Target Field |
|--------------|-------------|
| DATA         | inputs / outputs |
| MODULE       | components |
| FLOW         | flows |
| CONSTRAINT   | constraints |

---

### 4.4 Structure Assembly

Each system becomes:

- inputs
- outputs
- components
- flows
- constraints

---

## 5. Constraints

- Unrelated fragments MUST NOT be included
- Under-inclusion is preferred over over-inclusion
- A fragment may belong to multiple systems (allowed)
- Systems MUST NOT collapse into a single large system
- Each system must include related_fragments

---

## 6. Success Criteria

- Every system has related_fragments
- At least one structural field is populated
- No obvious cross-system contamination

---

## 7. Failure Conditions

- System has empty structure
- Excessive unrelated fragments included
- All fragments collapse into one system

---

## 8. Notes

- This is the most complex stage in the pipeline
- Goal is NOT perfect structure
- Goal is minimal viable structure for contract generation

---

## 9. Execution Principle

Prefer conservative linking over aggressive aggregation.

---

## 10. Conclusion

This stage converts loose system candidates into structured architectural units.

All contract generation depends on the integrity of this step.
