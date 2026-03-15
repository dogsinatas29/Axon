# AXON Rust Module Structure

This document defines the **Rust crate structure and module boundaries**
for implementing the AXON system.

It translates the architectural specifications into a structure that can
be implemented directly in Rust.

The goal of this document is to answer:

-   How the project is organized
-   Which modules exist
-   Who owns which data
-   How modules interact
-   Where the runtime starts

This is the **bridge between system design and Rust code**.

------------------------------------------------------------------------

# 1. Crate Layout

Recommended crate layout:

axon/ ├ Cargo.toml └ src/ ├ main.rs ├ runtime/ │ ├ mod.rs │ └ runtime.rs
│ ├ board/ │ ├ mod.rs │ ├ board.rs │ ├ thread.rs │ ├ milestone.rs │ └
project.rs │ ├ scheduler/ │ ├ mod.rs │ └ scheduler.rs │ ├ event/ │ ├
mod.rs │ ├ event.rs │ └ event_bus.rs │ ├ agent/ │ ├ mod.rs │ ├ agent.rs
│ └ agent_engine.rs │ ├ workspace/ │ ├ mod.rs │ └ workspace_manager.rs │
└ patch/ ├ mod.rs └ patch_engine.rs

------------------------------------------------------------------------

# 2. Runtime Entry Point

The system starts in main.rs.

Example:

main.rs

fn main() { let mut runtime = runtime::Runtime::new(); runtime.run(); }

The Runtime struct becomes the **kernel of the system**.

------------------------------------------------------------------------

# 3. Runtime Module

Runtime owns all core subsystems.

runtime/runtime.rs

pub struct Runtime { pub board: Board, pub scheduler: Scheduler, pub
event_bus: EventBus, pub agent_engine: AgentEngine, pub workspace:
WorkspaceManager, pub patch_engine: PatchEngine, }

Runtime drives the execution loop.

Responsibilities:

-   initialize subsystems
-   run scheduler loop
-   coordinate modules

------------------------------------------------------------------------

# 4. Ownership Graph

Ownership defines how data flows safely.

Runtime ├ owns Board ├ owns Scheduler ├ owns EventBus ├ owns AgentEngine
├ owns WorkspaceManager └ owns PatchEngine

Subsystems should communicate through function calls or events, not
shared mutable globals.

------------------------------------------------------------------------

# 5. Board Module

Board is the **single source of truth**.

board/ ├ board.rs ├ thread.rs ├ milestone.rs └ project.rs

Board responsibilities:

-   store project state
-   manage threads
-   manage milestones
-   track thread lifecycle
-   commit execution results

Example structure:

pub struct Board { pub projects: Vec`<Project>`{=html}, pub milestones:
Vec`<Milestone>`{=html}, pub threads: Vec`<Thread>`{=html}, }

------------------------------------------------------------------------

# 6. Scheduler Module

Scheduler decides what work happens next.

scheduler/scheduler.rs

Responsibilities:

-   detect runnable threads
-   determine next step
-   assign agent roles

Example interface:

pub struct Scheduler;

impl Scheduler { pub fn tick(&mut self, board: &mut Board) { //
scheduling logic } }

------------------------------------------------------------------------

# 7. Event System

Events decouple subsystems.

event/ ├ event.rs └ event_bus.rs

Event examples:

THREAD_CREATED THREAD_UPDATED THREAD_COMPLETED AGENT_EXECUTED

EventBus responsibilities:

-   queue events
-   dispatch events
-   notify subsystems

------------------------------------------------------------------------

# 8. Agent Engine

AgentEngine manages AI workers.

agent/ ├ agent.rs └ agent_engine.rs

Responsibilities:

-   select agent
-   execute prompts
-   produce results

Result output:

pub struct ExecutionResult { pub artifacts: Vec`<Artifact>`{=html}, pub
patch: Patch, pub events: Vec`<Event>`{=html}, }

------------------------------------------------------------------------

# 9. Workspace Manager

Workspace handles project files.

workspace/workspace_manager.rs

Responsibilities:

-   maintain project directory
-   write artifacts
-   provide context for agents

Workspace should mirror the project repository structure.

------------------------------------------------------------------------

# 10. Patch Engine

PatchEngine applies code modifications.

patch/patch_engine.rs

Responsibilities:

-   apply patches
-   modify files
-   track code changes

This module updates the workspace based on agent output.

------------------------------------------------------------------------

# 11. Runtime Loop

Runtime drives the system.

Example:

impl Runtime {

    pub fn run(&mut self) {
        loop {

            let events = self.event_bus.poll();

            self.board.apply_events(events);

            self.scheduler.tick(&mut self.board);

        }
    }

}

The scheduler triggers agent execution and board updates.

------------------------------------------------------------------------

# 12. Thread Lifecycle

Thread state progression:

draft ↓ junior_proposal ↓ senior_review ↓ architect_review ↓
boss_approval ↓ completed

The scheduler determines which agent role handles each stage.

------------------------------------------------------------------------

# 13. Implementation Order

Recommended order:

1.  Board module
2.  Scheduler
3.  Event system
4.  Runtime
5.  Agent engine
6.  Workspace manager
7.  Patch engine

This order allows incremental testing.

------------------------------------------------------------------------

# 14. Design Philosophy

AXON behaves like a simulated software company.

Human → Boss Scheduler → Production manager Agents → Workers Threads →
Tasks Milestones → Product goals

The runtime coordinates agents to advance development.
