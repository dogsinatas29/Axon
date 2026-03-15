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

## 🛠️ Key Features (v0.1.0 Ready)
- **Hierarchical Org Chart**: Real-time hiring/firing of agents with automatic succession policies.
- **Reactive Workflow**: Junior implementation -> Senior review -> Architect validation automated pipeline.
- **Event-Driven Architecture**: Fully traceable system signals and agent dialogs powered by a high-speed Event Bus.
- **Real-time Studio Dashboard**: Live streaming of factory signals, agent debates, and process logs via WebSockets.
- **SCADA Control**: Global Pause/Resume and strict project isolation for industrial-grade development management.

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
- [x] Core Orchestration Engine (v0.1.0 Framework POC)
- [x] Hierarchical Org Chart & Agent Succession (v0.1.0)
- [x] Real-time UI Streaming & Event Dashboard (v0.1.0)
- [ ] Adversarial Persona Mode (Forced Debates)
- [ ] Automated Code Export & Documentation (Final Spec)

## 📜 License
GPL-3.0 - See [LICENSE](LICENSE) for details.
