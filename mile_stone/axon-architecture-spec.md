# AXON Architecture Specification

## axon-architecture-spec.md

Version: v0.1

------------------------------------------------------------------------

# 1. Problem Space

Modern AI coding tools mostly operate using a **single-agent interaction
model**.

Typical workflow:

User Prompt ↓ LLM generates code ↓ User approves

While effective for small tasks, this approach creates major problems in
larger projects:

-   weak task structure
-   poor architectural oversight
-   limited code review visibility
-   unclear responsibility boundaries
-   difficulty managing multiple AI agents

As AI-assisted development becomes more complex, a more structured
system is required.

------------------------------------------------------------------------

# 2. Existing Approaches

Current AI development tools generally fall into three categories.

## 2.1 AI Coding Assistants

Examples:

-   GitHub Copilot
-   Cursor
-   Codeium

Characteristics:

-   single LLM interaction
-   prompt-driven generation
-   minimal workflow structure

Limitations:

-   no multi-agent collaboration
-   no project-level orchestration
-   limited architecture visibility

------------------------------------------------------------------------

## 2.2 Agent Frameworks

Examples:

-   AutoGPT
-   BabyAGI
-   CrewAI

Characteristics:

-   multi-agent execution
-   automated task loops

Limitations:

-   script-based orchestration
-   weak persistence models
-   limited UI and monitoring

------------------------------------------------------------------------

## 2.3 LLM Orchestration Libraries

Examples:

-   LangChain
-   LlamaIndex

Characteristics:

-   pipeline composition
-   tool integration
-   model abstraction

Limitations:

-   library-focused
-   not a runtime system
-   no project-level orchestration

------------------------------------------------------------------------

# 3. AXON Approach

AXON introduces a fundamentally different model.

Instead of treating AI as a **code generator**, AXON treats AI agents as
**members of a development organization**.

Conceptual mapping:

User → Company Owner\
Architect → System Architect\
Senior → Team Lead\
Junior → Engineer

Tasks are executed through **structured collaboration between agents**.

AXON therefore behaves more like a **software development operating
system**.

------------------------------------------------------------------------

# 4. System Definition

AXON can be defined as:

AI Agent Real-Time Orchestration System (RTOS)

It manages:

-   AI agents
-   project threads
-   development workflows
-   event-driven communication
-   persistent project state

------------------------------------------------------------------------

# 5. High-Level Architecture

System layers:

User │ AXON CLI / Studio │ AXON Core Runtime │ Model Driver Layer │
External LLM Providers

The Core Runtime is responsible for orchestration.

------------------------------------------------------------------------

# 6. Core Runtime Components

The Rust daemon runtime consists of the following modules.

axon-core

daemon\
dispatcher\
scheduler\
agent_runtime\
event_bus\
model_driver\
state_store

Component roles:

daemon\
Controls system lifecycle.

dispatcher\
Assigns tasks to agents.

scheduler\
Determines thread execution order.

agent_runtime\
Executes AI agents and manages their lifecycle.

event_bus\
Handles communication between system components.

model_driver\
Provides abstraction for AI providers.

state_store\
Persists project state using SQLite.

------------------------------------------------------------------------

# 7. Thread-Based Execution Model

AXON uses a **Thread-per-Task execution model**.

Project ├ Thread ├ Thread └ Thread

Thread structure:

Thread ├ Messages ├ Assigned Agents ├ Task State └ Approval Status

Threads represent the smallest executable unit.

------------------------------------------------------------------------

# 8. Event-Driven Architecture

AXON runtime operates using an event-driven model.

Agent ↓ Event ↓ Event Bus ↓ Subscribers

Subscribers include:

-   Scheduler
-   UI streaming
-   logging systems
-   thread managers

------------------------------------------------------------------------

# 9. Agent Organization Model

Agents are structured as a hierarchical organization.

User (Boss) │ Architect │ Senior │ Junior

Responsibilities:

Architect

-   maintains system architecture
-   validates structural changes

Senior

-   supervises development threads
-   reviews junior work

Junior

-   implements tasks
-   generates code

------------------------------------------------------------------------

# 10. Multi-Project Architecture

AXON supports multi-project orchestration through a shared event bus.

Event Bus ├ Project A ├ Project B └ Project C

Free users:

maximum concurrent projects = 1

Paid users:

multiple concurrent projects supported.

------------------------------------------------------------------------

# 11. Nogari Threads

AXON includes a special thread type called **Nogari**.

Nogari threads allow agents to freely discuss ideas and problems.

Purpose:

-   idea exploration
-   context expansion
-   creative problem solving

Unlike production threads, Nogari threads do not generate direct
development tasks.

------------------------------------------------------------------------

# 12. AXON Studio

AXON Studio provides a local dashboard interface.

Features:

-   thread board
-   agent organization view
-   approval workflow
-   runtime monitoring
-   event logs

Users can:

-   hire agents
-   fire agents
-   change models
-   monitor project execution

------------------------------------------------------------------------

# 13. Runtime Execution Flow

Typical system execution:

axon read blueprint.md ↓ LLM parses specification ↓ Thread creation ↓
Scheduler assigns agents ↓ Agent execution ↓ Review and approval ↓
Project state update

------------------------------------------------------------------------

# 14. Architectural Summary

AXON combines multiple architectural ideas:

-   event-driven runtime
-   thread-based execution
-   multi-agent orchestration
-   hierarchical organization model
-   persistent project state

The resulting system can be described as:

AI Software Development Operating System
