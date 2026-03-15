# encoding: utf-8
# AXON Architecture SSOT (Master Hub Specification)

## 🏛️ System Overview
AXON is an Automated Software Factory (ASF) built as an "Agent Operating System".
It follows a Hub -> Cluster -> Node architecture.

## 🔗 Core Modules
- **Hub**: [axon-daemon](file:///home/dogsinatas/rust_project/axon/crates/axon-daemon) - Orchestrates all tasks.
- **Cluster**: [axon-dispatcher](file:///home/dogsinatas/rust_project/axon/crates/axon-dispatcher) - Manage worker pools.
- **Node**: [axon-agent](file:///home/dogsinatas/rust_project/axon/crates/axon-agent) - Executes units of work.

## 📜 Sovereign Protocol
1. All changes must be reflected in the [Colosseum](http://localhost:8080).
2. [✅ Locked] sections in specifications represent immutable code.
3. Every task must have a corresponding [Post](file:///home/dogsinatas/rust_project/axon/crates/axon-core/src/lib.rs) in the storage.

## 📅 Roadmap (v0.0.12 - v0.0.3+)
- [x] Framework POC (Core/Model/Storage/Dispatcher)
- [x] Multi-Project Isolation & Control (v0.0.12)
- [ ] Adversarial Persona Mode
- [ ] Real-time UI Streaming
