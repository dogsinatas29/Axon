# AXON Event System Specification

## axon-event-spec.md

Version: v0.1

------------------------------------------------------------------------

# 1. Purpose

This document defines the **formal event specification** used by the
AXON runtime.

AXON is an **event-driven orchestration system**. All major runtime
components communicate through events.

Events represent **state transitions inside the system**.

Threads store the state.\
Events describe how the state changes.

------------------------------------------------------------------------

# 2. Event Architecture

The runtime follows a central event bus model.

Agent / System ↓ Event Created ↓ Event Bus ↓ Subscribers

Subscribers may include:

Scheduler\
Agent Runtime\
Thread Engine\
Nogari System\
UI Streamer\
Event Persistence

This architecture decouples runtime components and allows independent
reactions to system changes.

------------------------------------------------------------------------

# 3. Event Schema

All events follow a common structure.

Event

id\
type\
project_id\
thread_id\
source\
payload\
timestamp

Field descriptions:

id\
Unique event identifier.

type\
Event type identifier.

project_id\
Project associated with the event.

thread_id\
Thread associated with the event.

source\
Agent or subsystem generating the event.

payload\
Event-specific data.

timestamp\
Event creation time.

Example event:

{ "id": "evt_20491", "type": "MESSAGE_POSTED", "project_id": 1,
"thread_id": 42, "source": "junior_agent", "payload": { "message_id":
991, "role": "junior" }, "timestamp": "2026-03-15T12:20:11Z" }

------------------------------------------------------------------------

# 4. Event Type Registry

The following event groups exist in AXON.

Thread Events

THREAD_CREATED\
THREAD_ASSIGNED\
THREAD_STARTED\
THREAD_COMPLETED\
THREAD_ARCHIVED

Message Events

MESSAGE_POSTED\
MESSAGE_EDITED

Artifact Events

ARTIFACT_CREATED\
ARTIFACT_UPDATED

Approval Events

APPROVAL_REQUESTED\
APPROVAL_GRANTED\
APPROVAL_REJECTED

Agent Events

AGENT_ASSIGNED\
AGENT_RESPONSE

System Events

QUOTA_EXCEEDED\
SYSTEM_WARNING

These events describe the lifecycle of development tasks.

------------------------------------------------------------------------

# 5. Event Lifecycle

Each event moves through the following lifecycle.

Created\
Published\
Delivered\
Processed\
Archived

Lifecycle flow:

Subsystem ↓ Create Event ↓ Publish to Event Bus ↓ Subscribers Receive ↓
Subscribers Process ↓ Event Archived

Event persistence allows debugging and replay.

------------------------------------------------------------------------

# 6. Agent Subscription Model

Agents react to events through subscriptions.

Example:

Junior Agents listen for:

THREAD_ASSIGNED

Senior Agents listen for:

JUNIOR_MESSAGE_POSTED

Architect Agents listen for:

SENIOR_REVIEW_COMPLETED

Execution pattern:

Event ↓ Agent Listener ↓ Agent Action ↓ New Event Generated

This creates a reactive multi-agent workflow.

------------------------------------------------------------------------

# 7. Thread Lifecycle Events

Thread state transitions are represented through events.

THREAD_CREATED\
THREAD_ASSIGNED\
THREAD_STARTED\
MESSAGE_POSTED\
ARTIFACT_CREATED\
APPROVAL_REQUESTED\
THREAD_COMPLETED

Threads store state while events represent transitions.

------------------------------------------------------------------------

# 8. Nogari Event Integration

Nogari threads are driven by the same event system.

Example flow:

Thread Event Occurs ↓ Event Bus ↓ Nogari Listener ↓ Persona Reaction ↓
Nogari Message Posted

Nogari therefore acts as a specialized event subscriber.

------------------------------------------------------------------------

# 9. Event Persistence

Events may be stored for replay and auditing.

Example storage location:

.axon/events/

Example files:

event_0001.json\
event_0002.json

Persisted events allow:

runtime debugging\
full project replay\
audit trails

------------------------------------------------------------------------

# 10. Runtime Relationship

AXON runtime components interact through events.

Event Bus ├ Scheduler ├ Agent Runtime ├ Thread Engine ├ Nogari System ├
UI Stream └ Event Storage

The event bus acts as the **central nervous system** of AXON.

------------------------------------------------------------------------

# 11. Summary

AXON uses a unified event-driven architecture.

Threads represent work units.\
Events represent state transitions.

Event ↓ Event Bus ↓ Subscribers

This design enables:

decoupled runtime components\
multi-agent orchestration\
traceable development workflows\
full system replay capability
