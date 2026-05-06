# AXON v0.0.26 — Decision Density Distributed Pipeline (L-DDP)

## Overview
This document consolidates the pipeline design for stable C/C++ generation using LLMs, focusing on reducing cognitive load and enabling recovery.

---

## Core Philosophy
- Do not push the model harder
- Reduce decision density
- Enforce order, not complexity

---

## Pipeline Stages

### Stage 1 — Skeleton
- Pure logical structure
- No syntax
- Defines functions and responsibilities only

Example:
```
UserService
 - createUser(name) -> bool
 - deleteUser(id) -> bool
```

---

### Stage 2 — Header Generation

#### Input
- Skeleton JSON
- Optional dependency headers

#### Prompt Structure

Role:
- Generate C++ header file
- Public interfaces only

Rules:
- One file only
- Use include guards
- No implementation
- Simple types only

Output:
```
[FILE: module.h]
<code>
```

---

### Stage 3 — Freeze
- Header becomes immutable
- No modification allowed

---

### Stage 4 — Implementation
- Input: Target.h + minimal dependencies
- No access to full system

---

### Stage 5 — Validation (Retry Scope)

#### Flow
Error → Cause → Scope → Retry

#### Causes
- MissingHeader
- MissingSymbol
- SignatureMismatch
- DependencyMissing
- LogicError

#### Scopes
- Skeleton
- HeaderOnly
- ImplementationOnly
- Full

#### Example
```
MissingSymbol → HeaderOnly
LogicError → ImplementationOnly
```

---

## Retry Plan Structure
```
RetryPlan {
  scope: HeaderOnly,
  target_files: ["user_service.h"]
}
```

---

## Hint System
Convert errors into actionable instructions.

Example:
- "Declare missing symbol in header"
- "Do not modify implementation"

---

## Key Rules

- Header = translation, not design
- Minimize context
- Restrict visibility
- Enforce immutability after generation

---

## Summary

1. Skeleton reduces thinking load
2. Header translates structure
3. Validator enables recovery

System becomes resilient instead of fragile.
