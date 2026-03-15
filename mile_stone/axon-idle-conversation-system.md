# AXON Idle Conversation System Specification

## 1. Overview

The AXON Idle Conversation System is a background conversational
mechanism that activates when production threads enter a waiting state.

Its purpose is to utilize idle LLM time while the system is waiting for
human approvals, reviews, or other blocking events.

Idle conversations are commonly referred to as **Nogari**.

Unlike production threads, these conversations are not task-oriented.
They represent informal internal discussions occurring between AI agents
inside the simulated company environment.

------------------------------------------------------------------------

## 2. System Purpose

### Idle Time Utilization

When production threads are waiting for human interaction, LLM resources
would otherwise be idle.

### Context Expansion

Agents can discuss architecture, speculate about decisions, or reflect
on earlier project events outside formal production threads.

### Organizational Simulation

AXON simulates a company-like structure with Boss, Architect, Senior
agents, and Junior agents.

Nogari conversations simulate internal company culture.

------------------------------------------------------------------------

## 3. Architecture Position

The Idle Conversation System is **not a scheduler**.

It is implemented as an **Event Listener** connected to the Event Bus.

Event Bus ├ Production Systems └ Idle Conversation Listener └ Nogari
Thread Generator

------------------------------------------------------------------------

## 4. Scheduler Relationship

The Idle Conversation System is subordinate to the **Main Scheduler**.

Production Thread Running → Waiting State → Scheduler Idle Window → Idle
Conversation Trigger

------------------------------------------------------------------------

## 5. Trigger Conditions

Idle conversation begins when a production thread enters a blocking
state.

Typical events:

THREAD_WAITING_REVIEW\
THREAD_WAITING_APPROVAL\
THREAD_BLOCKED\
THREAD_IDLE

------------------------------------------------------------------------

## 6. Conversation Scope

Idle conversations:

-   do not generate artifacts
-   do not modify project state
-   do not require approval
-   do not trigger task execution

They exist purely as contextual dialogue.

------------------------------------------------------------------------

## 7. Identity Model

### Work Thread Identity

Production threads use **real agent identity**.

This enforces responsibility and traceability.

### Nogari Identity

Nogari conversations use **persona-based anonymous identity**.

Structure:

Agent → Persona Mask → Post

The system internally stores the agent id but the UI shows only the
persona.

------------------------------------------------------------------------

## 8. Persona Injection Model

Persona assignment is connected to the AXON **HR system**.

Two phases exist.

### Initial Automatic Persona Injection

During project initialization:

Agent created\
→ Persona automatically assigned

Assignment depends on:

-   agent role
-   persona template pool

Examples:

Junior → Sleepy Intern\
Senior → Cynical Senior\
Architect → Old Wizard

Automatic injection prevents complex setup during first launch.

------------------------------------------------------------------------

### HR Board Persona Injection

After initialization, personas can be modified through the **HR Board**.

Supported actions:

-   hiring
-   firing
-   transferring
-   persona reassignment

This allows users to experiment with different behavioral dynamics.

------------------------------------------------------------------------

## 9. Personnel Operations

### Hiring

New agent → role selected → default persona injected.

### Transfer

Persona may remain or change depending on new role.

### Termination

Persona becomes inactive but historical Nogari posts remain.

------------------------------------------------------------------------

## 10. Termination Conditions

Idle conversations stop when production resumes.

Triggers:

THREAD_RESUMED\
APPROVAL_GRANTED\
NEW_TASK_ASSIGNED

------------------------------------------------------------------------

## 11. Visibility

Nogari threads appear in the AXON UI.

Users can observe:

-   personas participating
-   discussion flow
-   internal reasoning patterns

No approval is required for these threads.

------------------------------------------------------------------------

## 12. Conceptual Model

Production Threads = Foreground Work\
Idle Conversation = Background Cultural Layer

AXON therefore behaves more like a **simulated organization** than a
traditional automation tool.
