/*
 * AXON - The Automated Software Factory
 * Copyright (C) 2026 dogsinatas
 */

# AXON: Agent Orchestration Framework

AXON is an autonomous software production system where AI agents (Architects, Seniors, and Juniors) collaborate to transform specifications into high-quality code.

## 🚀 Principles

1.  **Architecture-First**: No code is written without a validated `architecture.md` (SSOT).
2.  **Role Sanctity**: Strict hierarchical boundaries ensure quality and accountability.
3.  **Stateless Execution**: Agents operate as interchangeable factory workers.

## 🏭 Quick Start: Bootstrapping the Factory (v0.0.14+)

AXON now supports an interactive bootstrapping process. No more manual terminal tab management.

1.  **Preparation**: Set your API keys (e.g., `export GEMINI_API_KEY=your_key`).
2.  **Launch**: Run the daemon command:
    ```bash
    cargo run -- run
    ```
3.  **HR Assignment**: Select models for each role:
    *   **Architect (CTO)**: 1 core intelligence to design the SSOT (`Architecture.md`).
    *   **Seniors (Teams)**: Count and models for critical code review.
    *   **Juniors (Workers)**: Count and models for implementation.
4.  **Spec Ingestion**: Provide the path to your specification file (e.g., `mile_stone/v0.0.1.md`).
5.  **Watch the Factory**: The Architect will automatically generate the architecture and break down tasks, which are then distributed to the team.

## 📅 Roadmap

- [x] **v0.0.1** - Basic AXP Protocol & Daemon
- [x] **v0.0.13** - Role Sanctity & System Summary
- [x] **v0.0.14** - Intelligence Factory Bootstrapping (Interactive HR & Spec Breakdown)
- [ ] **v0.1.0** - Beta Release (Full Persistence & Studio Refinements)

## 📜 License
GPL-3.0 - See [LICENSE](LICENSE) for details.
