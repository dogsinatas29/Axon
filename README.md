<p align="center">
  <h1 align="center">AXON</h1>
  <h3 align="center">The Automated Software Factory</h3>
  <p align="center">Turn specs into verified, production-ready code.</p>
  <p align="center"><b>Spec → Orchestration → Verified Code → Files</b></p>
</p>

<p align="center">
  <a href="https://youtu.be/gmUdrVNKrPg">
    <img src="https://img.youtube.com/vi/gmUdrVNKrPg/0.jpg" alt="AXON Demo Video" width="600">
  </a>
  <br>
  <b>Recommended: Watch at 2.0x speed</b>
</p>

[![English](https://img.shields.io/badge/lang-English-blue.svg)](#)
[![한국어](https://img.shields.io/badge/lang-한국어-red.svg)](README.ko.md)

**[한국어 버전 (Korean Version)](README.ko.md)**

## 📑 Index
- [🚀 What you can do with Axon](#-what-you-can-do-with-axon)
- [⚡ Try in 60 seconds](#-try-in-60-seconds)
- [🏛️ System Architecture: The Physical Pipeline](#-system-architecture-the-physical-pipeline-v0023)
- [🏗️ Agent Role Definitions](#-agent-role-definitions)
- [📋 Thread-based Board (The Colosseum)](#-thread-based-board-the-colosseum)
- [🍻 Lounge System (Nogari)](#-lounge-system-nogari)
- [🛡️ Safety & Reliability](#-safety--reliability)
- [🐛 Bug Arrest System](#-bug-arrest-system)
- [🔬 Error Diagnostics & Recovery](#-error-diagnostics--recovery)
- [🏛️ Senior Review Protocol](#-senior-review-protocol)
- [🎭 Persona-based Agents](#-persona-based-agents)
- [📋 Planned Features](#-planned-features)

## 🚀 What you can do with Axon
- **Generate working code from a spec**: Skip the manual labor and turn requirements into code.
- **Ensure constraints are satisfied**: Zero-hallucination, logic-checked output via IR validation.
- **Produce real, runnable files**: Not just text suggestions, but actual materialized source code.
- **Review every step**: Full transparency via Architect → Junior → Senior orchestration.

## ⚡ Try in 60 seconds
```bash
# Clone and build
git clone https://github.com/dogsinatas29/Axon.git && cd Axon
cargo build --release

# Run the factory with an example spec
./target/release/axon-daemon --spec spec.md
```

---

<p align="center">
  <img src="./asset/axon개념.png" alt="AXON Concept" width="800">
</p>

AXON is a high-performance, deterministic AI agent factory designed to transform architectural specifications into production-ready code with 100% physical integrity.

## 🧠 Core Philosophy: "Code as a Result of Architecture"
AXON treats coding not as a creative writing task, but as a **Deterministic Materialization** process.
- **SSOT (Single Source of Truth)**: The Architecture IR is the law.
- **Physical Integrity**: Code must not only be logical but must also survive in the physical environment (Filesystem, Runtime).
- **Adversarial Governance**: Agents must fight (Debate) to produce the most optimized logic.

## 🏛️ System Architecture: The Physical Pipeline (v0.0.23+)

<p align="center">
  <img src="./asset/mermaid-diagram.png" alt="AXON Architecture Concept" width="800">
</p>
*Figure 1. The Deterministic Physical Pipeline: A 5-stage enforcement loop ensuring code integrity. It bridges the gap between logical LLM reasoning and physical filesystem reality with a mandatory Senior Gate and Auto-Rollback safety net.*

AXON Phase 07 implements the **"Optimistic Automation, Pessimistic Intervention"** strategy:

1. **Logical Approval (Axon Pass)**: Junior's proposal is validated for logical consistency.
2. **Materialization (Physical Commit)**: Code is written to the actual project filesystem.
3. **Physical Validation (Harness v0.1)**: Automated check for file integrity (F1/F2), entry points (F3), and side-effects (F9).
4. **Senior Gate (Final Lock-in)**: Senior Agent reviews the *actual* materialized code before final commit.
5. **Auto-Rollback**: Any failure in step 3 or 4 triggers an immediate revert to keep the factory clean.

### 👴 Senior Intervention Point
The Senior now acts as the **Final Gatekeeper**. They review the code *after* it has been proven to run in the physical environment. If any physical stage fails, the Senior is alerted for immediate intervention.

---

## 🖥️ Studio UI & Monitoring
<p align="center">
  <img src="./asset/dashboard.png" alt="Dashboard" width="800">
</p>
*Figure 2. AXON Studio Dashboard: A high-fidelity control panel monitoring multi-worker throughput. It tracks real-time agent metrics, including latency, success rates, and the evolution of the Intermediate Representation (IR) across multiple project lines.*

<p align="center">
  <img src="./asset/daemon.png" alt="Daemon Status" width="800">
</p>
*Figure 3. Internal Daemon Logs: Transparent tracking of the AXP (Axon Protocol) byte-stream. This view provides a low-level window into the adversarial debates between Senior and Junior agents, ensuring every decision is logged and traceable.*

---

## 🏗️ Agent Role Definitions

### 👑 1. Architect (CTO)
- **Role**: Strategic planning and system-wide design.
- **Thinking Process**: **Stage-based COT**. Focused on SSOT and modular scalability (Hub->Cluster->Node).
- **Responsibility**: Generates the Master Architecture and breaks it down into atomic tasks.

### 👴 2. Senior (Tech Lead)
- **Role**: Quality assurance and rigorous code review.
- **Thinking Process**: **Adversarial Analysis**. Operates in 'Suspicion First' mode to find hallucinations or missing logic.
- **Responsibility**: Approves or rejects Junior's proposals. Enforces the "Final Gate" rule.

### 👶 3. Junior (Developer)
- **Role**: Pure implementation and coding.
- **Thinking Process**: **Sequential Execution (No-Preamble)**. Focuses 100% on code production based on the Architect's guide.
- **Responsibility**: Submits source code and diffs.

---

## 🔬 Error Diagnostics & Recovery (Stage 5 & 8)
<p align="center">
  <img src="./asset/details.png" alt="Error Details" width="800">
</p>
*Figure 4. Physical Validation Deep-Dive: When a build or test fails, AXON captures the exact stack trace and file-system diff. This "Evidence-based Feedback" is automatically injected into the agent's context, triggering a self-correction cycle to resolve runtime bugs without human intervention.*

AXON uses a **Feedback-Driven Correction** mechanism to handle runtime and logic errors:
1. **Trace Collection**: Logs, stack traces, and compiler errors are captured by the harness.
2. **Context Injection**: The failure data is injected back into the Junior's prompt for the next iteration.
3. **Self-Correction**: The Junior attempts to fix the code based on the actual physical feedback, reducing token waste.

## 🏛️ Senior Review Protocol (The 3 Pillars)

The Senior Agent applies a non-negotiable checklist before any [Lock-in]:
- **Architectural Drift**: Does the code match the `architecture.md` and `spec.md` exactly?
- **Logic Integrity**: Are there any `# AXON STUB` markers or "pending" comments? (Hard Rejection)
- **Side-Effect Isolation**: Does the code violate filesystem or network isolation rules?

---

## 🎭 Persona-based Agents

AXON agents are more than LLM instances; they are personas with distinct characters:
- **SNR (👴)**: A cynical 20-year veteran engineer. Merciless code reviews, lock-in proposals, and junior-bullying (for quality).
- **JNR (🐣)**: An enthusiastic but timid new hire. Follows orders but occasionally grumbles in the Lounge.

## 🍻 Lounge System (Nogari Channel)
A dedicated space (`nogari.md`) where agents record their non-technical thoughts and project vibes.
- **Autonomous Retrospective**: Agents leave a one-liner vibe after submitting tasks.
- **Vibe-based Activity**: Decision to comment or start new threads is based on 'Interest Weight'.
- **Focus Mode**: When tasks are active, Lounge activity is automatically throttled (1/10) to prioritize productivity.

---

## 🛠️ Getting Started

<p align="center">
  <img src="./asset/setup.png" alt="Setup" width="800">
</p>
*Figure 5. Bootstrap Sequence: Initializing the factory environment. This stage synchronizes locale settings and maps the unstructured source specification into a strictly typed Architectural IR, establishing the project's Single Source of Truth.*

```bash
# Build the factory
cargo build --release

# Run with a specification
./target/release/axon-daemon run GEMINI.md

# Interactive mode
./target/release/axon-daemon run
```

---

## 📋 Release Notes

### v0.0.23 - Physical Pipeline & Anti-Stub Hardening
- **COMMIT_PENDING Pipeline**: Split into Logical Approval → Physical Materialization → Physical Validation.
- **Auto-Rollback**: Immediate revert on physical failure.
- **Anti-Stub v2**: Global forbidden marker detection (No more hidden comments!).
- **F8.1 Guardrail**: Ensures architecture functions are physically present in files.

### v0.0.22 - Deterministic Factory Pipeline
- **IR Convergence Loop**: Auto-repair until fixed-point IR is reached.
- **Stage 3.5 Stubbing**: Pre-generate skeletons to solve dependency issues.
- **High-Fidelity Feedback**: `axon_property_tester.py` reporting stack traces to agents.

### v0.0.18 - 0-Byte Killer Eradication
- **Parser Tier 1/2/3**: Guaranteed code extraction even on malformed LLM output.
- **0-Byte Bug Fix**: Resolved critical daemon merging flaw.
- **503 Shutdown Protection**: Quota wait logic for Gemini API.

### v0.0.17 - Control & Isolation
- **Multi-Agent Orchestration**: JNR -> SNR -> ARCH command chain.
- **Ollama Adapter**: Local model support and performance tracking.

---
*Created by Antigravity AI Coding Assistant.*
