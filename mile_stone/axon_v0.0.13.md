# Axon v0.0.13

## Core Principle
Code generation guarantees structural consistency.
Execution and validation remain the responsibility of humans.

---

## LLM Limitation Definition

LLMs optimize for:
- "Code that runs"

They do NOT inherently optimize for:
- Maintainability
- Correctness under constraints
- Architectural integrity

Therefore, external constraints and system design must enforce quality.

---

## LLM Coding Constraints

- Think Before Coding
- Simplicity First
- Minimal Changes
- Goal-Oriented Execution
- No Hallucinated APIs
- Stable Code Protection
- Context Confirmation

---

## Additional Constraint (Critical)

LLM must NOT:
- Interpret beyond given specification
- Invent missing intent
- Optimize unless explicitly instructed

---

## Role Definition

### Junior (LLM Worker)

Responsibilities:
- Execute task
- Generate code
- Perform mechanical constraint checks

Output (minimal, non-interpretive):
- task_id
- changed_files
- diff
- full code

❗ No summaries  
❗ No explanations  
❗ No risk analysis  

---

### Senior (LLM or Human)

Responsibilities:
- Interpret changes
- Summarize
- Make decision (approve / reject)

Process:
1. Read diff
2. Understand intent
3. Decide

---

### System (Critical Layer)

Responsibilities:
- Auto-generate summary from diff
- Highlight key changes
- Surface potential risks (optional)

---

## Review Flow

Junior → System → Senior

Code → Auto Summary → Decision

---

## Review SLA

- Target decision time: **5–10 seconds**

If exceeded:
- System design is incorrect

---

## Decision Rules

APPROVE if:
- Matches task intent
- No constraint violations
- Risk acceptable

REJECT if:
- Any violation exists

---

## Rejection Format

- reason: one line
- fix_hint: one line

---

## System Design Philosophy

- Do not over-engineer
- Do not introduce custom rendering engines
- Operate on standard technologies
- Keep verification scope minimal

---

## Architecture Insight

Responsibility separation is mandatory

- Junior = Production
- Senior = Judgment
- System = Compression

---

## Key Insight

Never let the code producer explain the code.

Explanation introduces bias and reduces review quality.

---

## Performance & UX Constraint

- Human must follow orchestration flow
- Too fast = cognitive overload
- Too slow = workflow stall

Target cycle:
- Generation: 2–5s
- Review: 5–10s

---

## Final Principle

Do not review code.  
Review decision-ready information.
