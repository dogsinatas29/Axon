# GEMINI Reverse Architecture Strategy v0.0.16

## 1. Core Problem
Transform unstructured GEMINI.md into structured contract units.

This is not authoring — it is reverse architecture.

---

## 2. Decomposition Rules

### (1) "What it does" → SYSTEM
Verb-driven blocks:
- Rendering
- Physics
- Input

### (2) "How it does" → MODULE
Implementation details:
- Raycasting
- Shading
- Buffer

### (3) "Flow" → Execution Flow
Step-by-step logic

---

## 3. Mechanical Process

### STEP 1 — Fragmentation
Split by:
- Paragraph
- Code block
- List

### STEP 2 — Tagging

Each fragment must be classified:

- SYSTEM
- MODULE
- FLOW
- DATA
- CONSTRAINT
- UNKNOWN

---

### STEP 3 — System Grouping

SYSTEM = externally callable unit

Examples:

Rendering System
- RaycastingCore
- Shading
- Buffer

Physics System
- Gravity
- Collision

---

## 4. System Identification Rule

Question:

"Can this run independently and produce meaningful output?"

YES → SYSTEM  
NO → MODULE

---

## 5. Contract Transformation

Each SYSTEM converts to:

- Goal
- Inputs
- Outputs
- Components
- Execution Flow
- Constraints
- Success / Failure

---

## 6. Strategy

### Phase 1 — Brutal Split
Ignore meaning, split everything

### Phase 2 — Tagging
Force classification

### Phase 3 — Extract SYSTEM only

### Phase 4 — Contract Generation

---

## 7. Key Insight

Do NOT preserve structure  
Preserve meaning  
Rebuild structure

---

## 8. Immediate Goal

Extract SYSTEM list from GEMINI.md

Do NOT build contracts yet
