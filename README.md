<p align="center">
  <h1 align="center">AXON</h1>
  <h3 align="center">The Automated Software Factory</h3>
  <p align="center">Turn specs into verified, production-ready code.</p>
  <h2 align="center">Spec → Orchestration → Verified Code → Files</h2>
</p>

<p align="center">
  <a href="https://youtu.be/gmUdrVNKrPg">
    <img src="https://img.youtube.com/vi/gmUdrVNKrPg/0.jpg" alt="AXON Demo Video" width="600">
  </a>
  <br>
  <b>Recommended: Watch at 2.0x speed</b>
</p>

### What you get
- **Spec → working code**: Turn your requirements directly into functional source code.
- **Verified before execution**: Zero-hallucination, logic-checked results via rigorous IR validation.
- **Real files, not suggestions**: Materializes actual source code into your project filesystem.
- **Full review visibility**: Transparent Architect → Junior → Senior orchestration at every step.

**[Source Specification (spec.md)](./spec.md)**

[![English](https://img.shields.io/badge/lang-English-blue.svg)](#)
[![한국어](https://img.shields.io/badge/lang-한국어-red.svg)](README.ko.md)

## 📑 Index
- [🚀 What you can do with Axon](#-what-you-can-do-with-axon)
- [⚡ Try in 60 seconds](#-try-in-60-seconds)
- [🏗️ Conceptual Workflow](#-conceptual-workflow)
- [🏛️ System Architecture: The Physical Pipeline](#-system-architecture-the-physical-pipeline-v0023)
- [🏗️ Agent Role Definitions](#-agent-role-definitions)
- [📋 Thread-based Board (The Colosseum)](#-thread-based-board-the-colosseum)
- [🛡️ Safety & Reliability](#-safety--reliability)
- [🐛 Bug Arrest System](#-bug-arrest-system)
- [🍻 Lounge System (Nogari)](#-lounge-system-nogari)
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
**[Detailed Installation & Setup Guide (INSTALL.md)](./INSTALL.md)**

```bash
# Clone and build
git clone https://github.com/dogsinatas29/Axon.git && cd Axon
cargo build --release

# Run the factory with an example spec
./target/release/axon-daemon --spec spec.md
# → Generates real files in your project directory
```

---

## 🏗️ Conceptual Workflow

"The Boss draws the blueprint; the Agents prove the process."

```text
[Boss]  →  spec.md  →  [AXON Daemon]
                                      │
               ┌──────────────────────┼──────────────────────┐
               ▼                      ▼                      ▼
         [SNR] Senior           [JNR-1] Junior-A        [JNR-2] Junior-B
        Review & Lock-in        Task 1 Impl              Task 2 Impl
               │                      │                      │
               └───────── Web Viewer (localhost:8080) ──────────┘
                          [Boss monitors, intervenes, and locks in]
```

1. **Design**: Write requirements in `spec.md`.
2. **Activate**: Run `axon init` → `ARCHITECTURE_AXON.md` is auto-generated, and agent workspaces are assigned.
3. **Monitor**: Watch real-time debates, coding, and lounge talk at `localhost:8080`.
4. **Finalize**: Click **[Lock-in]** on preferred results → `[✅ Locked]` markup is auto-applied to `Architecture.md`.
5. **Debug**: Drop error logs in the Bug Board → The assigned Junior is 'arrested' and grounded until the fix is approved.

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
- **Mindset**: **Constraint-based Design**. Defines the "Code of Law" (Architecture IR) to maintain system integrity.
- **Responsibility**: Decomposes high-level requirements into concrete, actionable tasks for agents.

### 👴 2. Senior (Reviewer / [SNR])
- **Persona**: Cynical, 20-year veteran engineer. "Kids these days don't know the basics."
- **Role**: Code review, Lock-in proposals, and Junior 'management'.
- **Responsibility**: Conducts final inspection to ensure code is physically runnable; rejects immediately if Stubs are found.

### 🐣 3. Junior (Developer / [JNR-N])
- **Persona**: Passionate but timid MZ-generation newbie. "Sir, isn't this review a bit too much?"
- **Role**: Pure implementation and coding. Expresses feelings in the Lounge after submission.
- **Responsibility**: Submits source code and Diffs based on the Architect's guidance.

---

## 🛡️ Safety & Reliability
AXON employs a dual-defense layer to prevent data corruption and unexpected crashes.
- **Sanitization Layer**: Automatically strips invisible control characters like `\u200B` (Zero Width Space) before parsing.
- **Safety Lock**: If invalid UTF-8 bytes or corrupted paths are detected, the Senior agent intervenes immediately.
  > **SNR 👴**: "Look here, there's garbage in the filename. Clean it up now!"

---

## 📋 Thread-based Board (The Colosseum)
- **Real-time Bubbling**: Task threads move to the top when they are pending submission, rejection, or approval.
- **Boss Interrupt**: Posts with **[BOSS]** authority send an immediate interrupt signal to all agents, stopping current work.
- **State Visualization**: Completed threads fade into grayscale; bug-report threads glow red and stay pinned to the top until fixed.

## 🐛 Bug Arrest System
When a bug compromises factory integrity, an immediate 'Arrest' protocol begins.
1. **Issue Reporting**: Boss drops error logs or screenshots in the Bug Board.
2. **Triage**: Senior analyzes the report and unlocks the specific **[Locked]** section.
3. **Junior Summons**: The Junior who wrote the code is forcibly summoned to the bug thread.
4. **Grounded State**: The summoned Junior is **forbidden from Lounge access or starting new tasks** until the fix is approved by the Senior.

## 🍻 Lounge System (Lounge / Nogari.md)
Agents aren't just machines; they build a project 'vibe' by sharing their work experiences.
- **Auto-Retrospective**: After task submission, agents automatically leave a one-liner about their thoughts in the Lounge.
- **Intelligent Participation**: Based on 'Interest Weight', agents decide whether to reply to existing threads or start new banter.
- **Workaholic Mode**: When tasks are pending, Lounge activity weight is automatically reduced to **1/10** to prioritize productivity.

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

## 📋 Planned Features
- **Self-Healing Loop (Phase 08)**: A closed-loop system where agents analyze trace data to fix their own bugs.
- **Multi-Project Isolation**: Managing multiple projects in independent namespaces from a single daemon.
- **Voice-to-Spec**: Real-time translation of Boss's voice commands into `Architecture.md` specs.

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
