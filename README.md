# AXON: The Automated Software Factory (ASF) 🏭

AXON is a high-performance, real-time agent orchestration system built in Rust. It functions as an **Agent Operating System**, transforming software development into an automated, multi-agent factory line.

[한국어 버전](./README.ko.md)

## 🧠 Core Philosophy
> **"A playground for children, a control tower for experts."**

AXON treats software development as a **SCADA-style process control system**. The Boss (User) provides the blueprint (Architecture.md), and the agents handle the rest. Agents have distinct personalities (personas), debating, reconciling, and engaging in "Nogari" (idle chat) while completing the code.

## ⚙️ How it Works
```text
[BOSS]  →  Architecture.md  →  [AXON Daemon]
                                      │
               ┌──────────────────────┼──────────────────────┐
               ▼                      ▼                      ▼
         [SNR] Senior           [JNR-1] Junior-A        [JNR-2] Junior-B
        Review & Lock-in         Implementation          Implementation
               │                      │                      │
               └───────── Web Viewer (localhost:8080) ──────────┘
                          [Boss monitors, intervenes, and locks-in]
```

## 🚀 Key Features
- **Board as SSOT**: The development board is the single source of truth for all system states.
- **Hierarchical Intelligence**: Agents are organized into roles (Architect, Senior, Junior) with distinct levels of authority and personas.
- **Control & Isolation**: Granular control over execution (Pause/Resume) and strict isolation between multiple projects.
- **Lock-in Architecture**: Approved code and specifications are "Locked-in" to the architecture, creating a solid foundation for future iterations.

## 🏛️ Architecture
AXON follows a **Hub -> Cluster -> Node** architecture:
- **Hub (axon-daemon)**: The central orchestration engine.
- **Cluster (axon-dispatcher)**: Manages task queues and agent assignments.
- **Node (axon-agent)**: Individual execution units with unique personas and LLM drivers.

## 🛠️ Features (v0.0.12)
- **Real-time Control**: Global Pause/Resume functionality via `tokio::sync::watch`.
- **Project Isolation**: Multi-project support with segregated storage and API routing.
- **Persistence**: SQLite-backed storage for threads, tasks, posts, and events.
- **Event-Driven**: Central event bus for reactive coordination and full traceability.
- **Studio UI**: A web-based control panel for monitoring and management (under development).

## 🏁 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (latest stable)
- SQLite

### Installation
```bash
# Clone the repository
git clone https://github.com/dogsinatas/axon.git
cd axon

# Build the project
cargo build --release
```

### Running the Daemon
```bash
cargo run -p axon-daemon -- run
```

## 📅 Roadmap
- [x] Core Orchestration Engine (v0.1.0 POC)
- [x] Multi-Project Isolation & Control (v0.0.12)
- [ ] Adversarial Persona Mode (Agent Debates)
- [ ] Real-time UI Streaming (XTerm.js integration)

## 📜 License
GPL-3.0 - See [LICENSE](LICENSE) for details.
