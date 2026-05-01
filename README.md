# AXON: The Automated Software Factory (Phase 07)

[![English](https://img.shields.io/badge/lang-English-blue.svg)](#)
[![한국어](https://img.shields.io/badge/lang-한국어-red.svg)](README.ko.md)

![AXON Concept](https://raw.githubusercontent.com/dogsinatas29/Axon/master/axon%EA%B0%9C%EB%85%90.png)

AXON is a high-performance, deterministic AI agent factory designed to transform architectural specifications into production-ready code with 100% physical integrity.

## 📑 Index
- [🧠 Core Philosophy](#-core-philosophy)
- [🏛️ System Architecture: The Physical Pipeline](#-system-architecture-the-physical-pipeline-v0023)
- [🏗️ Role Definitions](#-role-definitions)
- [🛠️ Getting Started](#-getting-started)
- [📋 Release Notes](#-release-notes)

## 🧠 Core Philosophy: "Code as a Result of Architecture"
AXON treats coding not as a creative writing task, but as a **Deterministic Materialization** process.
- **SSOT (Single Source of Truth)**: The Architecture IR is the law.
- **Physical Integrity**: Code must not only be logical but must also survive in the physical environment (Filesystem, Runtime).
- **Adversarial Governance**: Agents must fight (Debate) to produce the most optimized logic.

## 🏛️ System Architecture: The Physical Pipeline (v0.0.23+)

![AXON Architecture Concept](asset/mermaid-diagram.png)

AXON Phase 07 implements the **"Optimistic Automation, Pessimistic Intervention"** strategy:

1. **Logical Approval (Axon Pass)**: Junior's proposal is validated for logical consistency.
2. **Materialization (Physical Commit)**: Code is written to the actual project filesystem.
3. **Physical Validation (Harness v0.1)**: Automated check for file integrity (F1/F2), entry points (F3), and side-effects (F9).
4. **Senior Gate (Final Lock-in)**: Senior Agent reviews the *actual* materialized code before final commit.
5. **Auto-Rollback**: Any failure in step 3 or 4 triggers an immediate revert to keep the factory clean.

### 👴 Senior Intervention Point
The Senior now acts as the **Final Gatekeeper**. They review the code *after* it has been proven to run in the physical environment. If any physical stage fails, the Senior is alerted for immediate intervention.

## 🏗️ Role Definitions

### 👑 1. Architect (CTO)
- **Role**: Strategic planning and system-wide design.
- **Responsibility**: Generates the Master Architecture and breaks it down into atomic tasks.

### 👴 2. Senior (Tech Lead)
- **Role**: Quality assurance and rigorous code review.
- **Responsibility**: Approves or rejects Junior's proposals. Enforces the "Final Gate" rule.

### 👶 3. Junior (Developer)
- **Role**: Pure implementation and coding.
- **Responsibility**: Submits source code and diffs based on the Architect's guide.

## 🛠️ Getting Started

```bash
# Build the factory
cargo build --release

# Run with a specification
./target/release/axon-daemon run GEMINI.md
```

---
*Created by Antigravity AI Coding Assistant.*
