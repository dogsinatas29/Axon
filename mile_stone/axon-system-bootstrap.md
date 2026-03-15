# AXON System Bootstrap

This document is the **entry point for any AI or developer** attempting
to understand or implement the AXON system.

AXON consists of many design documents. Reading them in the wrong order
can lead to misunderstanding the architecture.

This document defines:

-   The purpose of the system
-   The correct document loading order
-   The expected implementation goal
-   The mental model required to understand AXON

This file should be provided **first** when initializing an AI with the
AXON project.

------------------------------------------------------------------------

# 1. System Identity

AXON is not a simple coding tool.

AXON is best understood as:

An **AI-driven software organization runtime**.

Conceptually:

Human → Boss\
Scheduler → Production Manager\
Agents → Workers\
Threads → Tasks\
Milestones → Product Goals

The system coordinates AI workers to advance software development.

------------------------------------------------------------------------

# 2. System Core Principle

AXON follows one central rule:

**The Board is the single source of truth.**

All system state must exist on the Board.

Subsystems may:

-   read board state
-   propose modifications
-   commit results

But they must not maintain hidden state outside the board.

This enables:

-   deterministic behavior
-   replay capability
-   transparent debugging

------------------------------------------------------------------------

# 3. Document Loading Order

The documents must be read in the following order.

Step 1 -- Constitution

v0.0.3.md

Defines the philosophy and core mental model of AXON.

------------------------------------------------------------------------

Step 2 -- Scenario

v0.0.3-scenario.md

Demonstrates how the system behaves in practice.

------------------------------------------------------------------------

Step 3 -- Evolution History

v0.0.4.md\
v0.0.5.md\
v0.0.6.md\
v0.0.7.md\
v0.0.8.md\
v0.0.9.md\
v0.0.10.md\
v0.0.11.md

These documents describe the progressive refinement of the design.

------------------------------------------------------------------------

Step 4 -- System Specifications

axon-architecture-spec.md\
axon-runtime-spec.md\
axon-scheduler-spec.md\
axon-event-spec.md\
axon-db-schema-plan.md

These define the architecture and runtime mechanics.

------------------------------------------------------------------------

Step 5 -- Behavioral Systems

axon-idle-conversation-system.md

Defines the internal social behavior system (Nogari).

------------------------------------------------------------------------

Step 6 -- Implementation Guides

axon-rust-module-structure.md\
axon-rust-skeleton.md

These documents describe how the architecture maps to Rust code.

------------------------------------------------------------------------

# 4. Final Implementation Goal

The objective after reading all documents is:

Implement the **AXON runtime system in Rust**.

The implementation must:

-   compile successfully
-   follow the architecture specifications
-   implement the runtime loop
-   maintain board-driven execution

The result should be a working AXON runtime kernel.

------------------------------------------------------------------------

# 5. Core Runtime Model

The system operates through a continuous runtime loop.

Pseudo execution:

while true:

    events = event_bus.poll()

    board.apply_events(events)

    thread = scheduler.find_runnable_thread(board)

    if thread:

        step = scheduler.determine_step(thread)

        agent = agent_engine.assign(step)

        result = agent_engine.execute(agent, step)

        workspace.apply(result.artifacts)

        patch_engine.apply(result.patch)

        board.commit(result)

        event_bus.emit(result.events)

------------------------------------------------------------------------

# 6. Key System Modules

Runtime

Board Engine\
Scheduler\
Event Bus\
Agent Engine\
Workspace Manager\
Patch Engine

Runtime owns all modules and coordinates execution.

------------------------------------------------------------------------

# 7. Thread Lifecycle

Threads represent units of development work.

Lifecycle:

draft ↓ junior_proposal ↓ senior_review ↓ architect_review ↓
boss_approval ↓ completed

The scheduler moves threads through this lifecycle.

------------------------------------------------------------------------

# 8. Mental Model

AXON behaves like a simulated software company.

Human user → Boss\
Scheduler → Production manager\
Agents → Workers\
Threads → Tasks\
Milestones → Product goals

Understanding this model is essential.

------------------------------------------------------------------------

# 9. Implementation Strategy

Recommended order:

1.  Implement Board Engine
2.  Implement Scheduler
3.  Implement Event Bus
4.  Implement Runtime Loop
5.  Implement Agent Engine
6.  Implement Workspace Manager
7.  Implement Patch Engine

Each module should compile and run before moving to the next.

------------------------------------------------------------------------

# 10. Bootstrap Complete

After reading this document and the referenced files, the system context
should be fully established.

The next step is **implementation of the AXON runtime**.
