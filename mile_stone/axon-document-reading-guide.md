# AXON Document Reading Guide

This document explains how the AXON specification documents should be
read and which parts are important for implementation.

The AXON project intentionally separates design into multiple documents.
Each document serves a different purpose in the system design.

This guide clarifies what developers must read in each document.

------------------------------------------------------------------------

# 1. v0.0.3.md (AXON Constitution)

Purpose: Philosophical and conceptual foundation of the AXON system.

Important sections to read:

-   Board-driven architecture
-   Thread lifecycle concept
-   Milestone system
-   Agent hierarchy (Junior → Senior → Architect → Boss)
-   Persona concept

Key ideas:

Board is the single source of truth. Development happens through threads
and milestones. Agents behave like workers inside a software company.

This document explains why AXON exists.

------------------------------------------------------------------------

# 2. axon-architecture-spec.md

Purpose: Defines the overall system structure.

Important sections:

-   System component overview
-   Module relationships
-   Core subsystems

Main modules described:

Scheduler Board Engine Agent Engine Event Bus Workspace Manager Patch
Engine

This document answers:

"What major components make up AXON?"

------------------------------------------------------------------------

# 3. axon-runtime-spec.md

Purpose: Defines how the system runs in real time.

Important sections:

-   Runtime execution graph
-   Main runtime loop
-   Module boundaries
-   Data ownership

Key runtime loop concept:

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

This document explains how AXON actually runs.

------------------------------------------------------------------------

# 4. axon-scheduler-spec.md

Purpose: Defines the scheduling system that progresses development.

Important sections:

-   Runnable thread detection
-   Thread state machine
-   Agent assignment
-   Multi-project scheduling
-   Idle detection

Key concept:

The scheduler is a milestone progress engine.

It decides which development step happens next.

------------------------------------------------------------------------

# 5. axon-event-spec.md

Purpose: Defines communication between modules.

Important sections:

-   Event types
-   Event propagation
-   Event lifecycle

Example events:

THREAD_CREATED THREAD_UPDATED THREAD_COMPLETED AGENT_EXECUTED

Events allow subsystems to react without tight coupling.

------------------------------------------------------------------------

# 6. axon-db-schema-plan.md

Purpose: Defines persistent data storage.

Important sections:

-   Thread tables
-   Milestone tables
-   Project tables
-   Event logs

Key principle:

Database mirrors board state.

This allows persistence and replay.

------------------------------------------------------------------------

# 7. axon-idle-conversation-system.md

Purpose: Defines the Nogari idle conversation system.

Important sections:

-   Persona injection
-   Anonymous conversation model
-   Idle scheduler triggers

Key rule:

Task threads → identity visible Nogari → persona-based anonymity

------------------------------------------------------------------------

# 8. Recommended Reading Order

Developers should read in this order:

1.  v0.0.3.md
2.  axon-architecture-spec.md
3.  axon-runtime-spec.md
4.  axon-scheduler-spec.md
5.  axon-event-spec.md
6.  axon-db-schema-plan.md
7.  axon-idle-conversation-system.md

This moves from philosophy to implementation.

------------------------------------------------------------------------

# 9. Minimum Knowledge Before Coding

Before implementation developers must understand:

-   Board-driven architecture
-   Runtime loop
-   Scheduler logic
-   Thread lifecycle
-   Event propagation

Once these are understood, development can begin.

------------------------------------------------------------------------

# 10. Conceptual Model

AXON behaves like an automated software company.

Human → Boss Scheduler → Production manager Agents → Workers Threads →
Tasks Milestones → Product goals
