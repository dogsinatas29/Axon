# Master Hub Architecture (Sovereign Protocol v0.2.21+)

This document outlines the high-level architecture of "DooM for AntigravitY" following the Sovereign Protocol (v0.2.21+). It demotes existing detailed systems (e.g., ECS, legacy architectures) to 'Node' level components and establishes a new 'Hub' layer that governs them, ensuring strict adherence to the specified domain logic and `SYNAPSE Performance Constraints v0.0.3`.

## 1. Core Principles

*   **Hierarchical Governance (Hub -> Cluster -> Node):** The `MasterHub` orchestrates `Clusters`, and `Clusters` orchestrate their respective `Nodes`. This provides clear lines of authority and responsibility.
*   **Single Source of Truth (SSOT):** Explicit boundaries are defined for data ownership and control flow at each architectural layer.
*   **Performance-Driven Design:** All components are designed with strict adherence to `SYNAPSE Performance Constraints v0.0.3` (Bounded Cost, Frame Budget, System Budget, etc.), ensuring predictable and stable performance.
*   **Zero-Dependency Enforcement:** The entire architecture and all its components strictly adhere to the Python 3.8+ Standard Library Only constraint.

## 2. Global Sovereign Protocol Enforcement

### 2.1. System Bootstrap Protocol

The `MasterHub` serves as the primary entry point and orchestrator for the entire application lifecycle. Its responsibilities include:
*   Initializing and configuring all defined `Clusters`.
*   Loading global application-level settings and assets.
*   Establishing and managing the main game loop (`engine.py`'s core loop logic).
*   Ensuring system stability by managing `atexit` hooks for critical cleanup (e.g., terminal state restoration).

### 2.2. SYNAPSE Performance Constraints Enforcement

Performance constraints are enforced hierarchically to maintain predictable frame rates in a CPU-bound terminal environment.

*   **MasterHub (`Frame Budget`):** The `MasterHub` is responsible for enforcing the overall `Frame Budget` (target: 30-60 FPS, budget: 16ms-33ms). It measures total frame time and can signal `Clusters` to initiate soft degradation policies if the budget is exceeded.
*   **Clusters (`System Budget`):** Each `Cluster` receives an allocated `System Budget` from the `MasterHub`. It is responsible for profiling and distributing this budget among its constituent `Nodes`, ensuring that its domain-specific operations collectively remain within their time limits.
*   **Nodes (`Internal Constraints`):** Each `Node` must internally adhere to all granular `SYNAPSE` rules:
    *   `Loop Constraints`: All iterations must be explicitly bounded (O(n) where n is fixed, limited O(n log n), rare O(n²) for small fixed n).
    *   `Recalculation Control`: Implement caching and dirty flags to avoid redundant computations.
    *   `State-Driven Updates`: Logic executes only upon relevant state changes.
    *   `Allocation Constraints`: No cumulative memory allocation in hot paths; utilize object pooling/reuse.
    *   `CPU Budget Protection`: Minimize control logic, prioritize calculation -> storage -> reuse.
    *   `LLM Forbidden Patterns`: Strictly avoid unbounded loops, uncontrolled O(n²) operations, hot-path allocations, and state-unaware recalculations.

## 3. Architecture Layers

### 3.1. Master Hub

**Authority:** Supreme Orchestrator. The global SSOT for high-level game state, application lifecycle, and overall performance governance.
**Responsibilities:**
*   Main game loop management and `Frame Budget` enforcement.
*   Management of global game state variables (e.g., `current_physics_mode`, `game_running`, `paused_state`).
*   Configuration distribution to `Clusters` during initialization.
*   Lifecycle management (initialization, update, shutdown) of all `Clusters`.
*   Centralized logging and high-level error reporting.
**Components:**
*   `main.py` (Project entry point, instantiates and runs the `MasterHub`).
*   `master_hub.py` (Contains the `MasterHub` class definition).

### 3.2. Clusters

**Authority:** Domain-specific Orchestrator. Each `Cluster` is the SSOT for its functional domain, dictating interaction and resource allocation among its `Nodes`.
**Responsibilities:**
*   Manage and update a specific set of related `Nodes`.
*   Enforce the `System Budget` allocated by the `MasterHub` among its `Nodes`.
*   Facilitate controlled data exchange and communication patterns between its `Nodes`.
*   Aggregate performance metrics and status from its `Nodes` for reporting to the `MasterHub`.

**Defined Clusters:**

#### 3.2.1. `CoreSimulationCluster`
*   **Purpose:** Manages the game world's fundamental state, entities, and core physics/gameplay logic.
*   **SSOT:** Game World (entities, components), current physics modes, collision states, combat encounters.
*   **Nodes Managed:**
    *   `ECSWorldNode`: Entity creation/deletion, component registration and management.
    *   `ComponentsNode`: Centralized definition and registry for game components (`Transform`, `Motion`, `Stats`, `PhysicsMode`, etc.).
    *   `PhysicsNode`: Position/velocity integration (`P = P + V`), friction application, wall/floor/ceiling collision detection and resolution.
    *   `GravityNode`: Applies gravitational forces based on the active `PhysicsMode` (Normal, Zero-G, Inverted).
    *   `CombatNode`: Manages projectile movement, hit detection, damage application, and strategic destruction logic.
    *   `SaveLoadNode`: Handles serialization and deserialization of the ECS world state for game persistence.

#### 3.2.2. `GraphicsRenderingCluster`
*   **Purpose:** Handles all visual output to the terminal, including 3D rendering, UI, and visual effects.
*   **SSOT:** Screen buffer state, rendering pipeline configuration, visual effect states.
*   **Nodes Managed:**
    *   `RenderNode`: Core raycasting (DDA algorithm, Z-Shearing), 3D viewport generation, double buffering, ANSI character mapping for geometry. Handles terminal cursor control.
    *   `UINode`: Renders the Hardcore Big-Face HUD, player stats, weapon overlay (including animations), and menu systems.
    *   `PostProcessingNode`: Applies visual enhancements and effects like distance-based shading, noise dithering, view flipping (for Inverted Mode), fog/mist systems, muzzle flashes, and damage flashes.

#### 3.2.3. `InputControlCluster`
*   **Purpose:** Manages player input from the keyboard and translates it into game actions.
*   **SSOT:** Raw and processed input state (key presses, buffered actions, input mappings).
*   **Nodes Managed:**
    *   `InputNode`: Manages `termios` raw mode, performs non-blocking `sys.stdin.read()`, maintains a `KeyBuffer` for simultaneous input, and translates key events into velocity/acceleration vectors for `Motion` components.

#### 3.2.4. `AudioSystemCluster`
*   **Purpose:** Manages all in-game sound effects and background audio.
*   **SSOT:** Current sound queue, active `aplay` subprocesses, sound asset mappings.
*   **Nodes Managed:**
    *   `SoundNode`: Utilizes `subprocess.Popen` for non-blocking playback of sound files via `aplay` (ALSA), manages a queue of sound events, and maps game events to audio assets.

#### 3.2.5. `ResourceManagementCluster`
*   **Purpose:** Handles the loading, parsing, and caching of all game assets and configuration files.
*   **SSOT:** Loaded WAD data, parsed configuration settings, cached game assets.
*   **Nodes Managed:**
    *   `WADLoaderNode`: Performs binary parsing of the original DOOM.WAD file, applies auto-scaling (x2.5 vertical), and extracts texture data.
    *   `ConfigLoaderNode`: Loads and parses `config.json` (keybindings, sensitivity) and `color_map.csv` (ANSI color definitions).
    *   `AssetCacheNode`: Manages in-memory caching of frequently accessed assets (e.g., textures, sprites) to optimize retrieval and minimize I/O.

### 3.3. Nodes

**Authority:** Worker component. Each `Node` is the SSOT for its specific task's internal logic and data, operating under the directive of its `Cluster`.
**Responsibilities:**
*   Execute a single, well-defined, and granular task (e.g., process input, apply gravity, render a frame segment).
*   Strictly adhere to its allocated `System Budget` and all `SYNAPSE Performance Constraints` within its implementation.
*   Operate on data provided by, or registered with, its `Cluster` or the `ECSWorldNode`.
*   Report results, status, or computed data back to its `Cluster` or to the `ECSWorldNode`.
*   Directly leverage shared utility modules (`utils/math_core.py`, `utils/terminal_utils.py`) where appropriate for common, low-level functionalities.

## 4. Proposed File Structure (Sovereign Protocol)

```
DooM-AntigravitY/
├── assets/                                 # External game resources
│   ├── DOOM.WAD                            # Original Doom WAD file
│   ├── config.json                         # User settings (keybindings, sensitivity)
│   ├── color_map.csv                       # ANSI color definitions for rendering
│   └── visual_assets.py                    # ASCII art for HUD faces, weapon sprites
├── saves/                                  # Game save data
│   └── save_slot_1.json                    # Serialized ECS state dump
├── src/                                    # Source code
│   ├── main.py                             # Project entry point, instantiates and runs MasterHub
│   ├── master_hub.py                       # MasterHub Class: Global orchestration, Frame Budget enforcement
│   ├── core/                               # Core architectural base classes
│   │   ├── cluster_base.py                 # Abstract base class for all Clusters
│   │   └── node_base.py                    # Abstract base class for all Nodes
│   ├── clusters/                           # All defined Cluster implementations
│   │   ├── core_simulation_cluster.py
│   │   ├── graphics_rendering_cluster.py
│   │   ├── input_control_cluster.py
│   │   ├── audio_system_cluster.py
│   │   └── resource_management_cluster.py
│   ├── nodes/                              # All individual Node implementations (demoted systems)
│   │   ├── ecs_world_node.py               # Manages entities and component associations
│   │   ├── components_node.py              # Defines all game components (Transform, Motion, etc.)
│   │   ├── physics_node.py                 # Handles position/velocity, friction, collisions
│   │   ├── gravity_node.py                 # Applies gravity based on PhysicsMode
│   │   ├── combat_node.py                  # Manages projectiles, damage, destruction
│   │   ├── save_load_node.py               # Handles game state serialization
│   │   ├── render_node.py                  # Raycasting, 3D viewport, terminal output
│   │   ├── ui_node.py                      # Renders HUD, weapon overlay, menus
│   │   ├── post_processing_node.py         # Shading, dithering, view effects, fog
│   │   ├── input_node.py                   # termios raw input, key buffering
│   │   ├── sound_node.py                   # aplay subprocess for audio
│   │   ├── wad_loader_node.py              # Parses DOOM.WAD, auto-scaling
│   │   ├── config_loader_node.py           # Loads config.json and color_map.csv
│   │   └── asset_cache_node.py             # Caches loaded assets
│   └── utils/                              # Shared utility libraries (NOT nodes, directly imported)
│       ├── math_core.py                    # Trigonometric LUTs, vector math
│       └── terminal_utils.py               # Generic terminal control functions (resize, cursor, raw mode)
└── docs/
    └── architecture.md                     # This architectural blueprint.
```

```json
[
  {
    "task_id": "bootstrap-task-001-01",
    "title": "MasterHub Initialization & Main Loop Setup",
    "description": "Implement the `MasterHub` class (`master_hub.py`) to serve as the application's entry point. It will orchestrate the main game loop, manage global state (e.g., `game_running`, `current_physics_mode`), and enforce the overall `SYNAPSE Frame Budget`. Ensure robust terminal state restoration on program exit.",
    "target_cluster": "MasterHub",
    "target_node": null,
    "engineering_requirements": [
      "Implement the `MasterHub` class with `__init__`, `run_game_loop`, and `shutdown` methods.",
      "Calculate delta time (`dt`) for frame-rate independent updates.",
      "Integrate global game state variables to control game flow.",
      "Set up `atexit` hook within `MasterHub` to call `terminal_utils.restore_terminal_settings`.",
      "Monitor overall frame execution time to ensure adherence to `SYNAPSE Frame Budget` (16ms-33ms)."
    ],
    "priority": "High"
  },
  {
    "task_id": "bootstrap-task-001-02",
    "title": "Utility: `terminal_utils.py` for Core Terminal Control",
    "description": "Create a `terminal_utils.py` utility module to encapsulate generic terminal manipulation functions. This module will be used by various Nodes (Input, Render) but managed globally for setup and teardown by the `MasterHub`.",
    "target_cluster": "UtilityServicesCluster (shared utils)",
    "target_node": "terminal_utils.py (utility module)",
    "engineering_requirements": [
      "Implement `set_raw_mode()` and `restore_cooked_mode()` using `termios` and `tty` for non-blocking input.",
      "Implement `hide_cursor()` and `show_cursor()` using standard ANSI escape codes (`\\033[?25l`, `\\033[?25h`).",
      "Implement `resize_terminal(width, height)` using ANSI codes (`\\033[8;{height};{width}t`) to enforce a 100x40 grid, with fallback/warning if unsupported.",
      "Ensure these functions are robust against errors and maintain terminal integrity."
    ],
    "priority": "High"
  },
  {
    "task_id": "bootstrap-task-001-03",
    "title": "InputNode: Raw Mode Input & Key Buffer",
    "description": "Implement the `InputNode` (`nodes/input_node.py`) to handle non-blocking keyboard input using `termios` and `tty` in raw mode. It should maintain a `KeyBuffer` to track simultaneous key presses (`WASD`, `Space`, `Shift`, `Ctrl`).",
    "target_cluster": "InputControlCluster",
    "target_node": "InputNode",
    "engineering_requirements": [
      "Initialize `termios` raw mode via `terminal_utils.set_raw_mode()`.",
      "Implement a non-blocking `read_input()` method using `sys.stdin.read(1)` to poll for keys without blocking the game loop.",
      "Maintain a `KeyBuffer` (e.g., a dictionary or set) to store currently pressed keys.",
      "Process `WASD`, `Space`, `Shift`, `Ctrl` inputs, converting them into actionable states for other Nodes.",
      "Adhere to `SYNAPSE System Budget` for input processing."
    ],
    "priority": "High"
  },
  {
    "task_id": "bootstrap-task-001-04",
    "title": "ECS Core Nodes: `ECSWorldNode` & `ComponentsNode`",
    "description": "Implement the foundational ECS components: `ECSWorldNode` (`nodes/ecs_world_node.py`) for entity creation/deletion and component management, and `ComponentsNode` (`nodes/components_node.py`) to define initial data classes (`Transform`, `Motion`, `Stats`).",
    "target_cluster": "CoreSimulationCluster",
    "target_node": "ECSWorldNode, ComponentsNode",
    "engineering_requirements": [
      "Implement `ECSWorldNode` with methods for `create_entity()`, `destroy_entity()`, `add_component(entity_id, component)`, `get_component(entity_id, component_type)`.",
      "Utilize a dictionary-of-dictionaries (or similar standard library structure) for efficient component storage.",
      "Define `Transform`, `Motion`, `Stats` as `dataclasses` (or simple classes) in `ComponentsNode` with attributes like `x, y, z, angle`, `vx, vy, vz, friction`, `hp, armor, ammo, fuel` respectively.",
      "Ensure `ComponentsNode` primarily defines data structures, with logic isolated to other Nodes.",
      "Minimize allocations and follow `SYNAPSE Allocation Constraints` for component management."
    ],
    "priority": "High"
  },
  {
    "task_id": "bootstrap-task-001-05",
    "title": "RenderNode: Double Buffering & Dummy Map Raycasting",
    "description": "Implement the core `RenderNode` (`nodes/render_node.py`) to manage the `ScreenBuffer` (100x40 character grid) and perform basic DDA raycasting on a dummy map. Optimize terminal output using double buffering and a single `sys.stdout.write` call per frame.",
    "target_cluster": "GraphicsRenderingCluster",
    "target_node": "RenderNode",
    "engineering_requirements": [
      "Create a `ScreenBuffer` (e.g., a 2D list of characters) of `100x40` dimensions.",
      "Implement the DDA raycasting algorithm for a hardcoded `DUMMY_MAP` (e.g., a 10x10 Python list representing walls).",
      "Apply the `wall_height * 2.5` vertical scaling logic during raycasting calculations as per spec.",
      "Optimize screen update: join the `ScreenBuffer` into a single string, prepend `\\033[H` (cursor home), and use `sys.stdout.write()` to prevent flickering.",
      "Adhere to `SYNAPSE System Budget` for rendering (e.g., 8ms or less).",
      "Ensure `SYNAPSE Loop Constraints` are met, particularly for raycasting iterations."
    ],
    "priority": "High"
  },
  {
    "task_id": "bootstrap-task-001-06",
    "title": "PostProcessingNode: Initial Hybrid Shading & Noise Dithering",
    "description": "Implement initial hybrid shading within the `PostProcessingNode` (`nodes/post_processing_node.py`). This involves blending `TEXTURE_CHARS` and `BLOCK_CHARS` based on raycasting distance and introducing basic noise dithering at color/texture transition boundaries to achieve a 'gritty' Doom-like feel.",
    "target_cluster": "GraphicsRenderingCluster",
    "target_node": "PostProcessingNode",
    "engineering_requirements": [
      "Define `TEXTURE_CHARS` (e.g., `#`, `|`, `H`) and `BLOCK_CHARS` (e.g., `█`, `▓`, `▒`) lookup tables in `assets/visual_assets.py`.",
      "Implement logic to select appropriate characters based on raycast `distance` (close, mid, far) and `texture_id` (if available from `WADLoaderNode` later).",
      "Introduce `random.random()` checks at distance thresholds to apply noise dithering, preventing sharp transitions between shading levels.",
      "Ensure this processing is efficient and adheres to its allocated `SYNAPSE System Budget`, minimizing redundant calculations (`SYNAPSE Recalculation Control`)."
    ],
    "priority": "Medium"
  }
]
```