# AXON Rust Skeleton

This document provides the **initial Rust code skeleton** for the AXON
system. The goal is to produce a project that:

-   Compiles with `cargo build`
-   Represents the AXON architecture
-   Allows incremental implementation

This is **not the full implementation**.\
It is the **starting runtime kernel structure**.

------------------------------------------------------------------------

# 1. Create Project

``` bash
cargo new axon
cd axon
```

Recommended structure:

    axon/
     ├ Cargo.toml
     └ src/
         ├ main.rs
         ├ runtime/
         │   ├ mod.rs
         │   └ runtime.rs
         ├ board/
         │   ├ mod.rs
         │   └ board.rs
         ├ scheduler/
         │   ├ mod.rs
         │   └ scheduler.rs
         ├ event/
         │   ├ mod.rs
         │   └ event_bus.rs
         ├ agent/
         │   ├ mod.rs
         │   └ agent_engine.rs
         ├ workspace/
         │   ├ mod.rs
         │   └ workspace_manager.rs
         └ patch/
             ├ mod.rs
             └ patch_engine.rs

Create folders:

``` bash
mkdir -p src/{runtime,board,scheduler,event,agent,workspace,patch}
```

------------------------------------------------------------------------

# 2. main.rs

``` rust
mod runtime;
mod board;
mod scheduler;
mod event;
mod agent;
mod workspace;
mod patch;

use runtime::runtime::Runtime;

fn main() {
    let mut runtime = Runtime::new();
    runtime.run();
}
```

------------------------------------------------------------------------

# 3. runtime/mod.rs

``` rust
pub mod runtime;
```

------------------------------------------------------------------------

# 4. runtime/runtime.rs

``` rust
use crate::board::board::Board;
use crate::scheduler::scheduler::Scheduler;
use crate::event::event_bus::EventBus;
use crate::agent::agent_engine::AgentEngine;
use crate::workspace::workspace_manager::WorkspaceManager;
use crate::patch::patch_engine::PatchEngine;

pub struct Runtime {
    pub board: Board,
    pub scheduler: Scheduler,
    pub event_bus: EventBus,
    pub agent_engine: AgentEngine,
    pub workspace: WorkspaceManager,
    pub patch_engine: PatchEngine,
}

impl Runtime {

    pub fn new() -> Self {
        Self {
            board: Board::new(),
            scheduler: Scheduler::new(),
            event_bus: EventBus::new(),
            agent_engine: AgentEngine::new(),
            workspace: WorkspaceManager::new(),
            patch_engine: PatchEngine::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let events = self.event_bus.poll();
            self.board.apply_events(events);

            self.scheduler.tick(&mut self.board);
        }
    }
}
```

------------------------------------------------------------------------

# 5. board/mod.rs

``` rust
pub mod board;
```

------------------------------------------------------------------------

# 6. board/board.rs

``` rust
pub struct Board {}

impl Board {

    pub fn new() -> Self {
        Self {}
    }

    pub fn apply_events(&mut self, _events: Vec<String>) {}

}
```

------------------------------------------------------------------------

# 7. scheduler/mod.rs

``` rust
pub mod scheduler;
```

------------------------------------------------------------------------

# 8. scheduler/scheduler.rs

``` rust
use crate::board::board::Board;

pub struct Scheduler {}

impl Scheduler {

    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self, _board: &mut Board) {
        // scheduling logic will go here
    }

}
```

------------------------------------------------------------------------

# 9. event/mod.rs

``` rust
pub mod event_bus;
```

------------------------------------------------------------------------

# 10. event/event_bus.rs

``` rust
pub struct EventBus {}

impl EventBus {

    pub fn new() -> Self {
        Self {}
    }

    pub fn poll(&mut self) -> Vec<String> {
        vec![]
    }

}
```

------------------------------------------------------------------------

# 11. agent/mod.rs

``` rust
pub mod agent_engine;
```

------------------------------------------------------------------------

# 12. agent/agent_engine.rs

``` rust
pub struct AgentEngine {}

impl AgentEngine {

    pub fn new() -> Self {
        Self {}
    }

}
```

------------------------------------------------------------------------

# 13. workspace/mod.rs

``` rust
pub mod workspace_manager;
```

------------------------------------------------------------------------

# 14. workspace/workspace_manager.rs

``` rust
pub struct WorkspaceManager {}

impl WorkspaceManager {

    pub fn new() -> Self {
        Self {}
    }

}
```

------------------------------------------------------------------------

# 15. patch/mod.rs

``` rust
pub mod patch_engine;
```

------------------------------------------------------------------------

# 16. patch/patch_engine.rs

``` rust
pub struct PatchEngine {}

impl PatchEngine {

    pub fn new() -> Self {
        Self {}
    }

}
```

------------------------------------------------------------------------

# 17. First Build

Run:

``` bash
cargo build
```

If everything is correct, the project compiles.

------------------------------------------------------------------------

# 18. Next Implementation Steps

Implement modules in this order:

1.  Board data structures
2.  Thread lifecycle
3.  Scheduler runnable detection
4.  Event system
5.  Agent execution
6.  Workspace file management
7.  Patch application

------------------------------------------------------------------------

This skeleton represents the **minimal AXON runtime kernel**.
