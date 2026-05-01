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
  <b>Full pipeline demo</b>
</p>

<a name="when-to-use-axon"></a>
### 🚀 When to use Axon?

- **When you need to implement complex requirements structurally**: Ideal for moving beyond simple coding to ensure total alignment with system architecture.
- **When the reliability of code generation is critical**: Essential for eliminating AI hallucinations and ensuring logically verified outputs.
- **For systems requiring multi-stage validation**: Perfect for projects that need automated validation loops, covering physical file creation, compilation, and runtime testing.
- **When you need a human-reviewable AI pipeline**: Designed for those who want direct control over a transparent workflow—from Architect to Junior and Senior agents.

<p align="center">
  <a href="https://www.gnu.org/licenses/gpl-3.0"><img src="https://img.shields.io/badge/license-GPL--3.0-blue.svg" alt="License: GPL 3.0"></a>
  <br>
  <a href="README.ko.md">🇰🇷 한국어 버전 (Korean Version)</a>
</p>

**[Source Specification (spec.md)](./spec.md)**

## 📑 Index
- [🚀 When to use Axon?](#when-to-use-axon)
- [🏗️ Conceptual Workflow](#conceptual-workflow)
- [🏛️ System Architecture: The Physical Pipeline](#system-architecture-the-physical-pipeline)
- [🛠️ Getting Started](#getting-started)
- [🖥️ Studio UI & Monitoring](#studio-ui-monitoring)
- [🏗️ Agent Role Definitions](#agent-role-definitions)
  - [🔬 Error Diagnostics & Recovery](#error-diagnostics-recovery)
  - [🏛️ Senior Review Protocol](#senior-review-protocol)
- [🛡️ Safety & Reliability](#safety-reliability)
- [📋 Thread-based Board (The Colosseum)](#thread-based-board-the-colosseum)
- [🐛 Bug Arrest System](#bug-arrest-system)
- [📋 Planned Features](#planned-features)
  - [🍻 Lounge System](#lounge-system)
  - [🎭 Persona-based Agents](#persona-based-agents)
  - [🤝 HR Board](#hr-board)
- [💻 Test HW/SW SPEC](#test-hw-sw-spec)

<a name="conceptual-workflow"></a>
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

<a name="system-architecture-the-physical-pipeline"></a>
## 🏛️ System Architecture: The Physical Pipeline (v0.0.23+)

<p align="center">
  <img src="./asset/mermaid-diagram.png" alt="AXON Architecture Concept" width="800">
</p>
*Figure 1. The Deterministic Physical Pipeline: A 5-stage enforcement loop ensuring code integrity. It bridges the gap between logical LLM reasoning and physical filesystem reality with a mandatory Senior Gate and Auto-Rollback safety net.*

1. **Logical Approval (Axon Pass)**: Junior's proposal is validated for logical consistency.
2. **Materialization (Physical Commit)**: Code is written to the actual project filesystem.
3. **Physical Validation (Harness v0.1)**: Automated check for file integrity (F1/F2), entry points (F3), and side-effects (F9).
4. **Senior Gate (Final Lock-in)**: Senior Agent reviews the *actual* materialized code before final commit.
5. **Auto-Rollback**: Any failure in step 3 or 4 triggers an immediate revert to keep the factory clean.

### 👴 Senior Intervention Point
The Senior now acts as the **Final Gatekeeper**. They review the code *after* it has been proven to run in the physical environment. If any physical stage fails, the Senior is alerted for immediate intervention.

---

<a name="getting-started"></a>
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

<a name="studio-ui-monitoring"></a>
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

<a name="agent-role-definitions"></a>
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

<a name="error-diagnostics-recovery"></a>
### 🔬 Error Diagnostics & Recovery (Stage 5 & 8)
<p align="center">
  <img src="./asset/details.png" alt="Error Details" width="800">
</p>
*Figure 4. Physical Validation Deep-Dive: When a build or test fails, AXON captures the exact stack trace and file-system diff. This "Evidence-based Feedback" is automatically injected into the agent's context, triggering a self-correction cycle to resolve runtime bugs without human intervention.*

AXON uses a **Feedback-Driven Correction** mechanism to handle runtime and logic errors:
1. **Trace Collection**: Logs, stack traces, and compiler errors are captured by the harness.
2. **Context Injection**: The failure data is injected back into the Junior's prompt for the next iteration.
3. **Self-Correction**: The Junior attempts to fix the code based on the actual physical feedback, reducing token waste.

<a name="senior-review-protocol"></a>
### 🏛️ Senior Review Protocol (The 3 Pillars)

The Senior Agent applies a non-negotiable checklist before any [Lock-in]:
- **Architectural Drift**: Does the code match the `architecture.md` and `spec.md` exactly?
- **Logic Integrity**: Are there any `# AXON STUB` markers or "pending" comments? (Hard Rejection)
- **Side-Effect Isolation**: Does the code violate filesystem or network isolation rules?

---

<a name="safety-reliability"></a>
## 🛡️ Safety & Reliability
AXON employs a dual-defense layer to prevent data corruption and unexpected crashes.
- **Sanitization Layer**: Automatically strips invisible control characters like `\u200B` (Zero Width Space) before parsing.
- **Safety Lock**: If invalid UTF-8 bytes or corrupted paths are detected, the Senior agent intervenes immediately.
  > **SNR 👴**: "Look here, there's garbage in the filename. Clean it up now!"

<a name="thread-based-board-the-colosseum"></a>
## 📋 Thread-based Board (The Colosseum)
- **Real-time Bubbling**: Task threads move to the top when they are pending submission, rejection, or approval.
- **Boss Interrupt**: Posts with **[BOSS]** authority send an immediate interrupt signal to all agents, stopping current work.
- **State Visualization**: Completed threads fade into grayscale; bug-report threads glow red and stay pinned to the top until fixed.

<a name="bug-arrest-system"></a>
## 📋 Bug Arrest System
When a bug compromises factory integrity, an immediate 'Arrest' protocol begins.
1. **Issue Reporting**: Boss drops error logs or screenshots in the Bug Board.
2. **Triage**: Senior analyzes the report and unlocks the specific **[Locked]** section.
3. **Junior Summons**: The Junior who wrote the code is forcibly summoned to the bug thread.
4. **Grounded State**: The summoned Junior is **forbidden from Lounge access or starting new tasks** until the fix is approved by the Senior.


---

<a name="planned-features"></a>
## 📋 Planned Features

<a name="lounge-system"></a>
### 🍻 Lounge System (Lounge System / Nogari.md) ( Planned )
Agents don't just work like machines; they leave reflections on their work, forming the project's 'Vibe'.

- **Auto-Retrospective**: Agents automatically leave a line of their feelings in the Lounge channel after task submission.
- **Intelligent Participation**: Based on the agent's interest weight, they reply to existing conversations or create new gossip threads.
- **Workaholic Mode**: When there are tasks to be processed, the lounge activity weight is automatically reduced to **1/10** to focus on work.

<a name="persona-based-agents"></a>
### 🎭 Persona-based Agents ( Planned )
AXON's agents are not just LLM instances, but personas with unique personalities:
- **Senior ([SNR] 👴)**: A cynical 20-year veteran engineer. Responsible for ruthless code reviews, Lock-in proposals, and taming juniors for quality.
- **Junior ([JNR-N] 🐣)**: An enthusiastic but timid newcomer. Follows orders but occasionally reacts timidly in the Lounge channel.

<a name="hr-board"></a>
### 🤝 HR Board ( Planned )
A board showing the hierarchy of Axon's working agents. 
The boss can flexibly hire/fire seniors and juniors according to the workload here. 
Also, you can inject personas into the hired agents. 


---

<a name="test-hw-sw-spec"></a>
## 💻 Test HW/SW SPEC

### 🖥️ Hardware Information
- **CPU**: Intel(R) Core(TM) i7-4790 (8) @ 4.00 GHz
- **GPU**: NVIDIA GeForce GTX 1050 Ti [Discrete] (4GB VRAM)
- **RAM**: 16GB DDR3

### ⚙️ Software Information
- **OS**: Ubuntu 25.10 x86_64
- **Kernel**: Linux 6.18.6-061806-generic

### 🧠 Local LLM Models
- **Model**: `qwen2.5:7b-instruct (q4_K_M)` (4.7 GB)
- **Engines**: 
  - [Ollama](https://github.com/ollama/ollama): Main Inference Engine (Running)
  - [AirLLM](https://github.com/lyogavin/airllm): Memory-Optimized Layered Loading (Used)
- **Environment**: Venv Active

---

## 📋 Release Notes

### v0.0.23 - Physical Pipeline & Anti-Stub Hardening
- **COMMIT_PENDING Pipeline**: Split into Logical Approval → Physical Materialization → Physical Validation.
- **Auto-Rollback (Auto-Rollback)**: Immediate revert on physical validation failure.
- **Anti-Stub v2**: Global forbidden marker detection (even in hidden comments).
- **F8.1 Guardrail**: Full audit to ensure functions defined in architecture exist in files.

### v0.0.22 - Deterministic Factory Pipeline
- **IR Convergence Loop**: Auto-repair loop until fixed-point IR is reached.
- **Stage 3.5 Stub Generation**: Proactive skeleton code generation for dependency resolution.
- **High-Fidelity Feedback**: Stack trace reporting via `axon_property_tester.py`.

### v0.0.18 - Bug Arrest & Quota Management
- **3-Tier Parser**: Guaranteed code extraction even from corrupted LLM outputs.
- **0-byte Bug Fix**: Resolved daemon merge logic failures.
- **503 Mitigation**: Added wait logic for Gemini API quotas.

### v0.0.17 - Control & Isolation
- **Multi-Agent Orchestration**: JNR -> SNR -> ARCH command hierarchy.
- **Ollama Adapter**: Local model execution and performance tracking integration.

---
*Created by Antigravity AI Coding Assistant.*
