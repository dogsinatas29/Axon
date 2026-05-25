# encoding: utf-8
# Constitution

This document defines the absolute, immutable rules and constraints of the MyWin32App system. All synthesis systems, engines, validators, and generators must enforce these rules strictly.

## 1. System Platform & Subsystem Constraints
- **Platform Target**: Native Windows GUI Subsystem (Subsystem 2). Console allocation or stdio-driven console layouts are forbidden.
- **Entry Point**: Must begin with `wWinMain` standard signature. Standard console `main` is strictly prohibited.
- **Language Standard**: Modern C++17.
- **Build System**: CMake 3.10+ targeting Windows SDK structures.

## 2. Forbidden Dependencies & Wrappers
- **Forbidden GUI Wrappers**: Cross-platform wrapper libraries like `SDL2`, `GLFW`, `GTK`, `Qt`, or `WXWidgets` are strictly forbidden. The system must interact directly with the Win32 API.
- **Forbidden Database Engines**: Relational database engines (`sqlite3`, `libpq`) or ORMs are strictly prohibited.
- **Forbidden Text UI (TUI) Engines**: `ncurses`, `termbox` are forbidden.

## 3. Rendering & Event Loop Contract
- **Rendering Restriction**: GDI rendering via `WM_PAINT` messages using `BeginPaint`/`EndPaint` and `TextOut` is mandatory. Legacy OpenGL or Direct3D wrappers are forbidden.
- **Message Loop Restriction**: Standard non-blocking message pump using `GetMessage`, `TranslateMessage`, and `DispatchMessage` is mandatory. Do not use busy-renderer or continuous busy loop structures to prevent CPU starvation.
