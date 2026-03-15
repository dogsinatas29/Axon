# AXON Scheduler Specification

## 1. Overview

The AXON Scheduler is the core runtime engine responsible for
progressing software development inside the AXON system.

Unlike traditional job schedulers, the AXON scheduler operates as a
**Milestone Progress Engine** that advances project development through
structured AI collaboration.

The scheduler coordinates AI agents, threads, and milestones using a
board‑driven execution model.

Core responsibilities:

-   Observe Board state
-   Detect runnable threads
-   Assign agents
-   Execute agent steps
-   Commit results to the Board
-   Emit system events

The scheduler is therefore the **execution kernel of AXON**.

------------------------------------------------------------------------

## 2. Design Philosophy

The scheduler follows principles defined in the AXON constitution
(v0.0.3.md).

### Board Driven Execution

All runtime state must be stored on the Board.

The scheduler must never depend on hidden internal state.

Board state is the **single source of truth**.

### Sequential Execution

Agent steps execute sequentially to avoid conflicts.

Only one agent step may be executed at a time.

This guarantees deterministic system behavior.

### Event Driven Runtime

Subsystem reactions occur through the Event Bus.

Scheduler actions may emit events that trigger reactions from other
systems.

### Deterministic Behavior

Given the same board state and inputs, the scheduler must produce
identical execution results.

This enables replay, debugging, and reproducibility.

------------------------------------------------------------------------

## 3. Core Runtime Loop

The scheduler operates as a continuous event loop.

Pseudo‑code:

while true:

    events = read_event_bus()

    update_thread_states(events)

    runnable_thread = find_runnable_thread()

    if runnable_thread:

        step = determine_next_step(runnable_thread)

        agent = assign_agent(step)

        result = execute_agent(agent, step)

        commit_result_to_board(result)

        emit_event(result)

    else:

        wait_for_event()

------------------------------------------------------------------------

## 4. Runnable Thread Detection

A thread is runnable when the following conditions are met:

-   thread status is active
-   dependencies are satisfied
-   thread is not blocked
-   required approvals are not pending

Example conditions:

status = junior_proposal\
status = senior_review\
status = architect_review

Waiting states are not runnable:

WAITING_APPROVAL\
WAITING_REVIEW\
BLOCKED

------------------------------------------------------------------------

## 5. Thread State Machine

Thread progression follows a structured workflow.

draft ↓ junior_proposal ↓ senior_review ↓ architect_review ↓
boss_approval ↓ completed

The scheduler determines the next action based on the current state.

Example:

status = junior_proposal → assign junior agent

status = senior_review → assign senior agent

------------------------------------------------------------------------

## 6. Agent Assignment

Agents are selected according to role requirements.

Agent pools:

Junior Pool\
Senior Pool\
Architect Pool

Selection factors:

-   availability
-   quota limits
-   recent usage

The scheduler must avoid overusing a single agent when alternatives
exist.

------------------------------------------------------------------------

## 7. Milestone Driven Execution

Projects are organized around milestones.

Structure:

Project └ Milestone └ Threads

The scheduler prioritizes threads within the **active milestone**.

Milestones represent coherent development objectives, such as
implementing a subsystem or completing a major feature.

------------------------------------------------------------------------

## 8. Multi‑Project Orchestration

AXON supports multiple concurrent projects.

The scheduler must distribute execution fairly across projects.

Recommended strategy:

Project fairness ↓ Milestone priority ↓ Thread selection

Possible scheduling policy:

round‑robin across active projects.

------------------------------------------------------------------------

## 9. Idle Detection

When no runnable threads exist, the scheduler enters an idle state.

Common reasons:

WAITING_APPROVAL\
WAITING_REVIEW\
BLOCKED_DEPENDENCY

Idle windows may trigger background systems such as:

-   Idle conversation system (Nogari)
-   analytical reflection
-   planning discussions

Idle compute time should not be wasted.

------------------------------------------------------------------------

## 10. Cost and Resource Control

LLM execution has real costs.

The scheduler must enforce resource constraints.

Control levels:

Agent quota\
Milestone budget\
Project budget

Example:

project_budget = \$10

When a budget limit is reached, the scheduler pauses execution until the
user intervenes.

------------------------------------------------------------------------

## 11. Integration with AXON Runtime

The scheduler interacts with multiple subsystems.

Runtime architecture:

Board Engine\
Scheduler\
Event Bus\
Agent Engine\
Persona Engine\
Workspace Manager\
Patch Engine\
Model Adapter

Execution flow:

Board → Scheduler → Agent → Board Update → Event

------------------------------------------------------------------------

## 12. Deterministic Replay

Because all state is stored on the Board, scheduler execution can be
replayed.

Benefits:

-   debugging
-   auditing
-   reproducible development sessions

Replay is possible by re‑executing scheduler steps from a recorded board
state.

------------------------------------------------------------------------

## 13. Conceptual Model

AXON behaves more like a **software factory** than a typical tool.

Human user → CEO / Boss

Scheduler → Production Manager

Agents → Workers

Threads → Tasks

Milestones → Production Goals

The scheduler coordinates AI workers to advance development toward
milestone completion.
