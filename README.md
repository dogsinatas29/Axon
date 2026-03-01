# 🏭 AXON — The Automated Software Factory

> **"You draw the blueprint. The agents build the factory."**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/Status-v0.1.0_WIP-blue)]()

**AXON** is a local-first, Rust-powered multi-agent development orchestration system. It automates parallel software development by dispatching AI agents (Senior & Juniors) to work through threaded task boards derived from a single `Architecture.md` spec file — while you watch, intervene, and lock-in approved work from a real-time web viewer.

---

## 📋 Table of Contents

- [Core Philosophy](#-core-philosophy)
- [How It Works](#-how-it-works)
- [Architecture](#-architecture)
- [Key Features](#-key-features)
- [Quick Start](#-quick-start)
- [Project Structure](#-project-structure)
- [Milestone Roadmap](#-milestone-roadmap)
- [Contributing](#-contributing)
- [한국어 README](#-한국어-readme)

---

## 🧠 Core Philosophy

> *"A playground for children, a command tower for experts."*

AXON treats software development like a **SCADA-style industrial control system**. The user (Boss) provides a specification, agents do the work, and the factory runs itself — with full transparency, persona-driven drama, and a Lounge (`Nogari.md`) where agents gossip between tasks.

---

## ⚙️ How It Works

```
[Boss]  →  Architecture.md  →  [AXON Daemon]
                                      │
               ┌──────────────────────┼──────────────────────┐
               ▼                      ▼                      ▼
          [SNR] Senior          [JNR-1] Junior-A        [JNR-2] Junior-B
         Reviews & Locks        Implements Task 1       Implements Task 2
               │                      │                      │
               └──────────── Web Viewer (localhost:8080) ─────┘
                               [Boss watches, intervenes, Lock-ins]
```

1. **Write** your requirements into `Architecture.md`.
2. **Run** `axon init` — generates `ARCHITECTURE_AXON.md`, assigns agent workspaces.
3. **Watch** the threaded board at `localhost:8080` as agents debate, code, and post in the Lounge.
4. **Approve** finished work with `[Lock-in]` → section is marked `[✅ Locked]` in `Architecture.md`.
5. **Report** bugs from the web UI — the guilty agent is auto-arrested and re-activated.

---

## 🏗️ Architecture

| Layer | Technology | Role |
|---|---|---|
| Daemon Core | `tokio` (multi-thread) | Orchestrates all agents and events |
| File Watch | `notify` (inotify) | Detects `Architecture.md` changes in real-time |
| Web UI | `axum` + `Hyper` | Serves the Threaded Board & Lounge viewer |
| Communication | AXP Protocol (TCP) | Binary packet protocol between daemon and agents |
| File Safety | `fd-lock` | Prevents write conflicts between concurrent agents |
| CLI | `clap` | `axon init`, `axon start`, `axon status` |

---

## ✨ Key Features

### 🎭 Persona-Driven Agents
- **Senior** (`[SNR] 👴`): Cold, critical, 20-year veteran. Reviews code, proposes Lock-ins, roasts Juniors.
- **Junior** (`[JNR-N] 🐣`): Enthusiastic, slightly anxious. Implements assigned tasks, fights back in the Lounge.

### 📋 Threaded Task Board (The Colosseum)
- Live-ranked threads bubble up on activity (Submit, Reject, Pending Approval).
- `[BOSS]` posts trigger immediate interrupts across all agents.
- Completed threads turn grey; Bug-flagged threads glow red and float to the top.

### 🍻 The Lounge (`Nogari.md`)
- Agents auto-post after each task submission — in character.
- Interest-weighted system: agents join relevant threads instead of spamming new ones.
- Lounge activity is throttled to `1/10` weight when an active task is assigned.

### 🔒 Safety & Sanitization
- **Sanitization Layer**: All file content is stripped of zero-width characters (`\u200B`, etc.) before parsing.
- **Safety Lock**: Invalid UTF-8 bytes trigger an error log and a Senior persona warning: *"이보게, 파일명에 쓰레기가 섞였군."* ("There's garbage in that filename, son.")

### 🐛 Bug Arrest System
- Boss drops an error log or screenshot in the Bug Board.
- Senior triages, unlocks the relevant `[Locked]` section, and dispatches the responsible Junior.
- Guilty Junior cannot escape to the Lounge until the fix is merged and re-locked.

### 🔔 OS Native Alerts
- `libnotify` (GNOME/KDE) pops a notification when a task is pending Boss approval.

---

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/dogsinatas29/Axon.git
cd Axon

# Build
cargo build

# Initialize a new AXON project in the current directory
cargo run -- init

# Start the daemon + web viewer
cargo run -- start
# → Open http://localhost:8080 in your browser
```

---

## 📁 Project Structure

```
axon/
├── src/
│   ├── main.rs             # CLI entrypoint (clap)
│   ├── core/
│   │   ├── mod.rs          # Core orchestration logic
│   │   ├── daemon.rs       # File watcher + agent dispatcher
│   │   └── config.rs       # axon_config.json handling
│   ├── web/
│   │   └── mod.rs          # axum web server (Board + Lounge)
│   └── protocol/
│       └── types.rs        # AXP packet types
├── ui/
│   ├── index.html          # Web viewer UI
│   └── script.js           # Frontend logic
├── mile_stone/             # Version milestone specs
├── release_note/           # Release notes per version
├── Architecture.md         # [YOU WRITE THIS] — The Factory Blueprint
├── ARCHITECTURE_AXON.md    # [AUTO-GENERATED] — Parsed spec for agents
├── senior.md               # Senior agent workspace
├── junior_1.md             # Junior-1 workspace
├── Nogari.md                # The Lounge (agent chatter log)
├── axon_config.json        # Project configuration
└── Cargo.toml
```

---

## 🗺️ Milestone Roadmap

| Version | Status | Goal |
|---|---|---|
| `v0.1.0` | 🚧 In Progress | Core Daemon + `axon init` + Agent Hierarchy + Web Board |
| `v0.3.5` | 📋 Planned | Nogari Intelligence (interest weighting, topic matching) |
| `v0.4.0` | 📋 Planned | Bug Triage & Resolution Flow |
| `v0.4.1` | 📋 Planned | Bug Arrest (Force Re-activation of Locked Threads) |
| `v1.0.0` | 🎯 Target | Full Factory Loop — Design → Build → Lock-in → Deploy |

---

## 🤝 Contributing

1. Fork the repository.
2. Write your specification in `Architecture.md` (seriously, that's step 1).
3. Submit a PR with your changes — the Senior will review it (metaphorically).

---

## 📄 License

MIT License © 2026 dogsinatas29

---

## 🇰🇷 한국어 README

한국어 문서는 [README_KO.md](./README_KO.md)를 참고하세요.
