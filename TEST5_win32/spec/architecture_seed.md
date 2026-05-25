# encoding: utf-8
# Runtime Topology & Component Specification

This specification defines the structural topology and module boundaries of MyWin32App. It serves as the single source of truth for Architecture extraction. All components specified here must be parsed with status "Core" to satisfy the immutable constraints.

> **[!NOTE]**
> LLM Parser Guide: Ensure every component below is mapped directly as a Core component (`status: "Core"`). Do not assign Optional or Experimental statuses.

## 1. Project Directory Structure
All C++ source files must follow the paths specified inside the component blocks.

## 2. 3대 물리 컴포넌트 토폴로지 (3 Core Components)

### MainWindow [Status: Core]
- **File Path**: `src/main_window.cpp` (Header: `src/main_window.h`)
- **Description**: Registers the WNDCLASS, instantiates the main application window via CreateWindowEx, and runs the standard GetMessage-based non-blocking message loop.
- **Dependencies**: WndProcHandler.

### WndProcHandler [Status: Core]
- **File Path**: `src/wndproc_handler.cpp` (Header: `src/wndproc_handler.h`)
- **Description**: Encapsulates the WndProc callback routine. Handles basic windows events like WM_DESTROY, window resizing, and delegates draw routines to the GDIEngine.
- **Dependencies**: GDIEngine.

### GDIEngine [Status: Core]
- **File Path**: `src/gdi_engine.cpp` (Header: `src/gdi_engine.h`)
- **Description**: Handles the WM_PAINT message by capturing the device context (HDC) with BeginPaint and drawing content using TextOut, and releases with EndPaint.
- **Dependencies**: WndProcHandler.
