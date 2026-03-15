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

- **SCADA Control**: Global Pause/Resume and strict project isolation for industrial-grade development management.

## 📺 Studio Control Tower (Web UI)
The AXON Studio provides a professional-grade SCADA-style interface for factory management:

- **종합 대시보드 (Dashboard)**: Real-time KPIs of factory throughput, active threads, and global system health.
- **작업 게시판 (Work Board)**: The heart of production. Monitor agent proposals, reviews, and code patches in an interactive thread format.
- **인사 관리 (Office)**: Hierarchical personnel management. Hire new agents, fire underperformers, and balance the DTR (Dynamic Tension) of the workforce.
- **사장 게시판 (Boss)**: Issue high-level directives and inject `Architecture.md` blueprints directly into the factory line.
- **노가리 게시판 (Lounge)**: Capture the "vibe" of the project through agent's informal chats and decision-making contexts.
- **실시간 시그널 (Signals)**: A high-fidelity event stream providing 100% transparency into all system-level operations.

## 🏁 Getting Started

### 1. Build Backend & Frontend
AXON requires both the Rust daemon and the React Studio to be built.

```bash
# Build Backend
cargo build --release

# Build Frontend (Studio UI)
cd studio
npm install
npm run build
cd ..
```

### 2. Configuration
Set your LLM API key. If not set, AXON runs in **Mock Mode** for simulation.
```bash
export GEMINI_API_KEY="your-google-api-key"
```

### 3. Execution
```bash
# Run the daemon
./target/release/axon-daemon run
```
Access the **AXON Studio** at `http://localhost:8080`.

> [!TIP]
> For advanced production setup (systemd, process management), please refer to [INSTALL.md](./INSTALL.md).

## 📅 Roadmap
- [x] Core Orchestration Engine (v0.1.0 Framework POC)
- [x] Hierarchical Org Chart & Agent Succession (v0.1.0)
- [x] Real-time UI Streaming & Event Dashboard (v0.1.0)
- [ ] Adversarial Persona Mode (Forced Debates)
- [ ] Automated Code Export & Documentation (Final Spec)

## 📜 License
GPL-3.0 - See [LICENSE](LICENSE) for details.
