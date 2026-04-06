# AXON: The Automated Software Factory (v0.0.16)

> **"A playground for the curious, a command tower for the experts."**

AXON is a high-performance, Rust-based automated software factory designed to manage multiple AI agents across isolated project sandboxes. It coordinates complex development workflows through the AXP (Axon eXchange Protocol) and maintains a single source of truth (SSOT) via `architecture.md`.

## 🏗️ Core Architecture (v0.0.16)

1.  **High-Performance Rust Daemon**: A low-memory, zero-copy binary engine managing task scheduling and agent coordination.
2.  **Namespace Isolation (Sandboxing)**: Each project is strictly isolated in `./projects/[project_id]/`. Work artifacts, architecture specifications, and code never mix.
3.  **AXP Protocol V1**: A binary frame protocol for high-speed, secure communication between the Boss (User), the Daemon, and the Agents.
4.  **The Lounge (Nogari System)**: Real-time agent "vibe" logging in `Nogari.md`. AI team members share thoughts, frustrations, and successes, providing human-like context to the development process.
5.  **Control System & Admin**: Real-time Pause/Resume, Task Lock-in (Architecture solidification), and Boss intervention via the Studio UI.

## 🚀 Quick Start

### 1. Build the Factory
```bash
cargo build --release
```

### 2. Bootstrapping a Project
Provide a specification (e.g., `test_task.md`) and choose your AI team (Junior, Senior, Architect).
```bash
./target/release/axon-daemon run
```

### 3. Studio UI (Control Tower)
Monitor your factory line in real-time at [http://localhost:8080](http://localhost:8080).
- **Active Threads**: Current work lines.
- **Lounge**: Agent "Nogari" chat log.
- **Boss Board**: Manual intervention and approval.

## 🛡️ Robustness & Reliability
- **Quota Self-Healing**: Automatically waits and retries on API rate limits.
- **Empty Response Recovery**: Detects and retries when models produce reasoning without content.
- **Atomic Persistence**: Task state is persisted via SQLite (`axon.db`).

## 📜 License
Copyright (C) 2026 dogsinatas. Licensed under the GNU General Public License v3.0.
