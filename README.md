# AXON: The Automated Software Factory (Phase 07)
[한국어 버전 (Korean)](README.ko.md)

![AXON Concept](https://raw.githubusercontent.com/dogsinatas29/Axon/master/axon%EA%B0%9C%EB%85%90.png)

## 📑 Index
- [🧠 Core Philosophy](#-core-philosophy)
- [⚙️ How it Works](#️-how-it-works)
- [🏗️ Architecture](#️-architecture)
- [✨ Key Features](#-key-features)
- [🖼️ Visual Overview](#️-visual-overview)
- [🔬 Error Diagnostics & Recovery](#-error-diagnostics--recovery)
- [🏛️ Senior Review Protocol](#-senior-review-protocol)

---

## 🔬 Error Diagnostics & Recovery

AXON uses a multi-stage validation pipeline. Understanding the error messages is key to monitoring the factory:

| Error Message | Stage | Meaning | Action |
|:--- |:--- |:--- |:--- |
| `MAPPING_VALIDATION_FAILED` | 4.5 | Implementation drift from Architecture IR. | System auto-normalizes paths or flags missing symbols. |
| `Execution Fail` | 5 | Code contains syntax errors or missing dependencies. | Junior agent receives stack trace for auto-repair. |
| `Golden Test Failed` | 6 | Regression detected; new code broke existing features. | Junior agent rolls back and re-implements. |
| `Property Test Failed` | 7 | Edge case bug found (e.g., negative input, overflow). | Junior agent optimizes logic for robustness. |

## 🏛️ Senior Review Protocol (Stage 8)

To maximize token efficiency and code quality, AXON follows a strict **"Automation First, Human-like Review Second"** policy:

1.  **The Junior's Battle**: The Junior agent must first fight through all automated QA gates (Stages 5, 6, and 7).
2.  **Senior Intervention**: The Senior agent **ONLY** intervenes at **Stage 8** once the code is proven to be executable and robust.
3.  **Role**: The Senior focuses on high-level architectural alignment, code readability, and sophisticated patterns, rather than catching simple syntax or logic bugs already filtered by the Daemon.
- [💻 Verified Test Environment](#-verified-test-environment)

---

## 💻 Verified Test Environment

AXON is designed to be highly efficient, even on modest consumer hardware. The following environment has been verified for stable production:

- **CPU**: Intel(R) Core(TM) i7-4790 (8 Cores) @ 4.00 GHz
- **RAM**: 16GB (15.07 GiB available)
- **GPU**: NVIDIA GeForce GTX 1050 Ti
- **Motherboard**: H81MDV5 (5.0)
- **OS**: Ubuntu 25.10 x86_64
- **Kernel**: Linux 6.18.6-generic
- **LLM Engine**: Ollama / airLLM
- **Reference Model**: `qwen2.5:7b-instruct-q4_K_M` (Utilizing partial GPU offloading)

---

## 🧠 Core Philosophy
> **"A playground for beginners, a control tower for experts."**

AXON treats software development like a **SCADA process control system**. The Boss (User) only provides the blueprint, and the agents handle the rest. Agents have distinct personas; they argue, reconcile, and chat in the "Nogari" channel while completing the code.

## ⚙️ How it Works

```text
[Boss]  →  Architecture.md  →  [AXON Daemon]
                                      │
               ┌──────────────────────┼──────────────────────┐
               ▼                      ▼                      ▼
         [SNR] Senior           [JNR-1] Junior-A        [JNR-2] Junior-B
        Review & Lock-in         Implement Task 1        Implement Task 2
               │                      │                      │
               └───────── Web Viewer (localhost:8080) ──────────┘
                      [Boss Monitors, Intervenes, and Locks-in]
```

1. **Design**: Write your requirements in `Architecture.md`.
2. **Setup**: Run `axon init` -> `ARCHITECTURE_AXON.md` generated, agent workspaces assigned.
3. **Monitor**: Watch agent discussions, coding, and "Nogari" (chats) in real-time at `localhost:8080`.
4. **Lock-in**: Click [Lock-in] on results you like -> `[✅ Locked]` markup is automatically applied to `Architecture.md`.
5. **Debug**: Drop error logs in the Bug Board -> The responsible Junior is "arrested" and cannot leave until the fix is complete.

## 🏗️ Architecture

| Layer | Technology | Role |
| :--- | :--- | :--- |
| **Daemon Core** | tokio (Multi-thread) | Agent & Event Orchestration |
| **File Watcher** | notify (inotify) | Real-time monitoring of Architecture.md |
| **Web UI** | axum + Hyper | Provides Board & Nogari viewer |
| **Communication** | AXP Protocol (TCP) | Binary packet communication between Daemon and Agents |
| **File Safety** | fd-lock | Prevention of concurrent access conflicts |
| **CLI** | clap | `axon init`, `axon start`, `axon status` |

## 📁 Project Structure

AXON is built as a robust **Rust Workspace** with a multi-crate architecture.

```text
axon/
├── crates/
│   ├── axon-daemon/        # Main Factory Engine & CLI
│   │   ├── src/
│   │   │   ├── main.rs     # CLI Entrypoint (Command Loop)
│   │   │   ├── lib.rs      # Factory & Bootstrap logic
│   │   │   ├── server.rs   # API & WebSocket (Control Tower)
│   │   │   └── cli.rs      # CLI Command definitions
│   ├── axon-agent/         # AI Agent Personalities & Reasoning Engine
│   ├── axon-core/          # Shared Event Bus & AXP Protocol
│   └── axon-storage/       # State Persistence (SQLite)
├── studio/                 # Control Tower Frontend (Vite/React)
├── assets/                 # Documentation Media Assets
├── mile_stone/             # Version-specific Milestones
├── release_note/           # Version-specific Release Notes
├── README.md               # Factory Manual
└── Cargo.toml              # Workspace Configuration
```

## ✨ Key Features

### 🎭 Persona-based Agents
- **Senior ([SNR] 👴)**: A cynical 20-year veteran engineer. Handles code review, lock-in proposals, and "disciplining" juniors.
- **Junior ([JNR-N] 🐣)**: An enthusiastic but cautious "Gen-Z" newbie. Implements tasks and occasionally revolts in the Nogari channel.

### 📋 Threaded Task Board (The Colosseum)
- Tasks bubble up to the top when status changes (Submit/Reject/Pending).
- **[BOSS]** posts trigger immediate interrupt signals to all agents.
- Completed threads fade to grayscale; Bug threads burn red and stay pinned at the top.

### 🍻 Nogari Channel (Nogari.md)
- Agents automatically leave a short reflection after submitting tasks.
- Decisions to reply or create new threads are based on "Interest Weighting."
- Chatting weight is forced down to 1/10 when active tasks are pending.

### 🔒 Safety & Input Validation
- **Sanitization Layer**: Automatically strips control characters like `\u200B` before parsing.
- **Safety Lock**: Errors on invalid UTF-8 bytes with a Senior persona warning: *"Hey, there's garbage in the filename."*

### 🐛 Bug Arrest System
- Boss drops error logs or screenshots in the Bug Board.
- Senior performs triage -> Unlocks the `[Locked]` section -> Forcibly summons the responsible Junior.
- The summoned Junior cannot leave for Nogari or other tasks until the fix is verified (Detention state).

### 🔔 OS Native Notifications
- System notifications via `libnotify` (GNOME/KDE) when tasks are pending approval.

---

## 🖼️ Visual Overview

| 1. Factory Setup | 2. Daemon Execution |
|---|---|
| ![Setup](./assets/setup.png) | ![Daemon](./assets/daemon.png) |
| *Recruiting agents and configuring local models.* | *Real-time task distribution and metric collection.* |

| 3. Studio Dashboard | 4. Task Details |
|---|---|
| ![Dashboard](./assets/dashboard.png) | ![Details](./assets/details.png) |
| *Monitoring the entire production line via Control Tower.* | *Inspecting individual thread status and agent proposals.* |

## 🏛️ Sovereign Agent Protocol: Roles & Reasoning

AXON enforces a strict hierarchy and specialized reasoning frameworks for each agent to ensure production-grade output.

### 👑 1. Architect (Chief Technology Officer)
- **Role**: Strategic planning and system-wide design.
- **Reasoning**: **Strategic Decomposition (Stage-based COT)**. Forced to think in terms of SSOT (Single Source of Truth) and modular scalability (Hub->Cluster->Node).
- **Responsibility**: Generates the Master Architecture and breaks it down into atomic, parallelizable tasks.

### 👴 2. Senior (Technical Lead / Auditor)
- **Role**: Quality assurance and rigorous code review.
- **Reasoning**: **Adversarial Analysis**. Operates in a "suspicion-first" mode to find hallucinations or missing logic.
- **Responsibility**: Approves or Rejects Junior proposals. Enforces the "No Code Block = Automatic Reject" rule.

### 👶 3. Junior (Software Engineer)
- **Role**: Pure implementation and coding.
- **Reasoning**: **Linear Execution (No-Preamble)**. Stripped of unnecessary thought processes to focus 100% on code production based on the Architect's guide.
- **Responsibility**: Delivers full source code and technical diffs for assigned tasks.

## 🛠️ Getting Started

> [!TIP]
> For a more detailed guide on Ollama setup and resource optimization, see the **[Full Installation Guide](INSTALL.md)**.

```bash
# Build the factory
cargo build --release

# Run with a specification (Direct)
./target/release/axon-daemon run GEMINI.md

# Run interactively
./target/release/axon-daemon run
```

---
*Created by Antigravity AI Coding Assistant.*

## 📋 Release Notes: v0.0.17 - Control & Isolation

### 🚀 Key Features
- **Multi-Agent Orchestration**: Enforces `Junior -> Senior -> Architect` chain of command with Round-Robin scheduling.
- **Ollama Runtime Adapter**: Native support for local model orchestration with performance tracking.
- **Observability & Reporting**: Real-time metric collection and event bus integration for execution paths.
- **Robust Bootstrap Protocol**: Phased initialization for configuration and context building.

### 🛠️ Technical Changes
- **Core**: Added `ObservabilityReport` and `RuntimeMetrics` to storage and agent logic.
- **Model Driver**: Updated trait to return structured metrics.
- **Daemon**: Implemented layer-based fallback and task-to-sandbox sync logic.

## 📋 Release Notes: v0.0.18 - Pipeline Stabilization & 0-Byte Bug Fix

### 🚀 Key Features & Improvements
- **Output Generation Guarantee**: Introduced the 3-Tier Parser architecture. When standard parsing fails, the Heuristic parser successfully extracts code blocks as a fallback.
- **Architect Bottleneck Prevention**: Successfully applied the `sampling_rate` logic to bypass the Architect and automatically delegate approval authority to Senior agents.
- **Model Stability Proven**: Replaced Junior/Senior models with `Gemma2`, achieving significantly higher Output Contract Adherence.

### 🛠️ Critical Bug Fixes
- **[CRITICAL] 0-Byte Overwrite Bug Fixed**: Resolved a critical flaw in the daemon's merge logic where unedited existing files were accidentally overwritten with 0 bytes.
- **[CRITICAL] Gemini 503 Overload Protection**: Added bulletproof `QUOTA_WAIT` logic to pause for 60 seconds and retry (instead of crashing) when encountering Google Gemini High Demand (503) errors.
- **Heuristic Garbage Extraction Prevented**: Blocked the parser from mistakenly saving non-code blocks (like `markdown`, `tool_code`, and `bash` logs) as project source files.

## 📋 Release Notes: v0.0.22 - Hardened Deterministic Factory

### 🚀 Key Features & Improvements
- **Deterministic IR Convergence**: Fixed-point convergence loop for Architecture IR, ensuring stable design before implementation.
- **Stage 3.5 Stub Generation**: Physical file materialization (Stubs) to resolve parallel dependency issues (`ModuleNotFoundError`).
- **Strict JSON Output Contract**: Enforced mandatory JSON protocol for Junior agents, ensuring 100% parsing reliability on resource-constrained hardware.

### 🛠️ Hardware & Reliability Fixes
- **GTX 1050 Ti Optimization**: Tailored GPU offloading for Qwen 2.5 7B to run stably on 4GB VRAM.
- **Smart Mapping Validator**: Path normalization and project-state awareness to eliminate false-positive drift warnings.
- **Real-time UI Synchronization**: Guaranteed task visibility and live signal streaming for agent activity.
