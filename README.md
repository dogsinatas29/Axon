# AXON: The Automated Software Factory (v0.0.16)

[English](README.md) | [한국어](README.ko.md)

> **"You draw the blueprint. The factory agents run the line."**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Rust Status](https://img.shields.io/badge/Rust-1.75+-orange.svg)

AXON is a high-performance, Rust-based local-first multi-agent development orchestration system. Describe your requirements in a single `architecture.md`, and the AXON Daemon will automatically distribute tasks to Senior/Junior AI agents. Monitor the entire production line via the real-time web Control Tower.

---

## 📋 Table of Contents
- [Core Philosophy](#-core-philosophy)
- [How It Works](#-how-it-works)
- [Architecture](#-architecture)
- [Key Features](#-key-features)
- [Project Structure](#-project-structure)
- [Quick Start](#-quick-start)
- [Milestone Roadmap](#-milestone-roadmap)

---

## 🧠 Core Philosophy
> **"A playground for the curious, a command tower for the experts."**

AXON treats software development like a **SCADA-based industrial process control system**. The User (Boss) provides the blueprint, and the agents handle the rest. Agents have distinct personas; they debate, reconcile, and chat (Nogari), ensuring the code evolves through logical rigor and survival.

---

## ⚙️ How It Works

```text
[BOSS]  →  architecture.md  →  [AXON DAEMON]
                                      │
               ┌──────────────────────┼──────────────────────┐
               ▼                      ▼                      ▼
         [SNR] Senior           [JNR-1] Junior-A        [JNR-2] Junior-B
        Review & Lock-in         Implement Task 1        Implement Task 2
               │                      │                      │
               └───────── Web Viewer (localhost:8080) ─────────┘
                   [Boss Monitors, Intervenes, and Locks-in]
```

1.  **Draft**: Write requirements in `architecture.md`.
2.  **Initialize**: Run `axon-daemon run` to auto-generate tasks and assign agent worklines.
3.  **Control**: Watch the agents debate, code, and chat in real-time at `localhost:8080`.
4.  **Lock-in**: Approve results to trigger a **[Lock-in]** → The section in `architecture.md` is automatically marked as `[✅ Locked]`.
5.  **Debug**: Toss error logs into the Bug Board → The responsible Junior is "arrested" and cannot leave the task until it's fixed.

---

## 🏗️ Architecture

| Layer | Technology | Role |
| :--- | :--- | :--- |
| **Daemon Core** | `tokio` | Global orchestration of agents and events |
| **File Watcher** | `notify` | Real-time monitoring of `architecture.md` |
| **Web UI** | `axum` + `Hyper` | Thread board and "Nogari" lounge viewer |
| **Communication** | `AXP Protocol` | Binary frame packet communication (V1) |
| **File Safety** | `fd-lock` | Prevention of concurrent file access conflicts |
| **Storage** | `SQLite` | Persistent task and agent state management |

---

## ✨ Key Features

### 🎭 Persona-Based Agents
- **Senior ([SNR] 👴)**: A cynical 20-year veteran. Responsible for code review, lock-in proposals, and keeping Juniors in check.
- **Junior ([JNR-N] 🐣)**: An enthusiastic but cautious MZ-generation rookie. Implements tasks and occasionally vents in the Lounge.

### 📋 Threaded Task Board (The Colosseum)
- Active tasks bubble up to the top based on priority (Submission/Rejection/Pending).
- **[BOSS]** posts trigger an immediate interrupt signal to all active agents.
- Completed threads are grayed out; Bug threads burn red and remain pinned.

### 🍻 Lounge Channel (Nogari.md)
- Agents automatically leave a "thought" or "vibe" after every submission.
- Agent chatter provides human-like context to the development process.
- Lounge activity is throttled when high-priority tasks are active.

### 🛡️ Safety & Input Validation
- **Sanitization Layer**: Automatically strips control characters like `\u200B` before parsing.
- **Safety Lock**: If invalid UTF-8 bytes are detected, the Senior persona warns: *"Hey, there's garbage in this file."*

### 🐛 Bug Arrest System
- Boss tosses logs or screenshots into the Bug Board.
- Senior performs triage → Unlocks the corresponding section → Mandates the original Junior to fix it.

---

## 📁 Project Structure

```text
axon/
├── crates/
│   ├── axon-daemon/         # CLI Entry point (clap) & Web Server
│   ├── axon-core/           # Protocol types (AXP) & Event definitions
│   ├── axon-agent/          # AgentRuntime, Lounge, and Persona logic
│   ├── axon-storage/        # SQLite Persistence Layer
│   ├── axon-dispatcher/     # Task Scheduling & Polling
│   └── axon-model/          # Multi-LLM Drivers (Gemini, Claude, etc.)
├── projects/                # [Isolation] Project Sandboxes
│   └── system/
│       └── architecture.md  # Project-specific SSOT (Blueprint)
├── studio/                  # Dashboard UI (Built-in Web Assets)
├── mile_stone/              # Milestones for each version
├── release_note/            # Detailed Release Notes
├── Nogari.md                 # Agent Lounge Vibe logs
├── axon.db                  # Task & Agent State Persistence
└── axon_config.json         # Factory Configuration (Recruit list)
```

---

## 🚀 Quick Start

### 1. Build the Factory
```bash
cargo build --release
```

### 2. Activate the Line
```bash
./target/release/axon-daemon run
```

---

## 📜 License
Copyright (C) 2026 dogsinatas. Licensed under the MIT License.
