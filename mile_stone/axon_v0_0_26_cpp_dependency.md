# AXON v0.0.26 — C++ Dependency Automation Addendum

## 1. Header-based Include Inference

### Goal
Generate minimal includes from header files automatically.

### Principles
- Include only directly used types
- Prefer forward declaration when possible
- Use standard type mapping
- Fail-safe: include if uncertain

### Pipeline
Header → Type Extraction → Classification → Dependency Resolution → Include Set

### Type Classification
- Builtin → ignore
- std::* → map to standard headers
- Project types → resolve via registry

### Forward Declaration Rules
- Value → include
- Pointer/Reference → forward declare

### Output Structure
```
#include <string>
#include "User.h"

class User;
```

---

## 2. CPP Include + Link Dependency Generator

### Goal
Generate .cpp includes and link dependencies from usage.

### Principles
- Always include own header
- Infer dependencies from usage
- Avoid transitive includes
- Link based on symbol usage

### Pipeline
Header → Implementation Scan → Symbol Map → Dependency Merge → Build Plan

### Include Rules
- Always include self header
- Include only used symbols
- No forward declaration (prefer include)

### Link Resolution
Symbol → Source → Object file

Example:
UserRepository → UserRepository.cpp → UserRepository.o

### Output
```
#include "UserService.h"
#include "UserRepository.h"
```

Link:
```
UserRepository.o
```

---

## 3. Build Plan Structure
```
BuildPlan {
  includes: [],
  link_deps: []
}
```

---

## 4. Validator Integration
- Missing include → retry implementation
- Missing link → retry implementation

---

## Summary
- Header decides structure
- CPP decides usage
- System resolves dependencies
- LLM focuses only on logic
