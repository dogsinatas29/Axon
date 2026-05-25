# MyLightIDE v0.0.1 — WIN32 / DIRECT2D COMPLIANT SPEC

Identity: Native Win32 Editor + Direct2D System Framework Transition Specification.

---

# 0. PURPOSE

This document exists ONLY to help AXON, IR generators, validators, low-end LLMs, and code synthesis systems correctly generate a native Win32 editor application.

The editor architecture, Lua ownership model, renderer lifecycle, extension model, and buffer semantics MUST remain compatible with the Linux GTK4 specification.

Only the platform runtime backend is replaced.

---

# 1. PRIMARY RULE

IMPORTANT.
The generated application MUST be:

```text
A native Win32 desktop application.
```

Required:

```text
Win32 API
Direct2D
DirectWrite
IMM32
COM
```

STRICTLY FORBIDDEN:

```text
GTK
Qt
SDL
Electron
Chromium embedding
Cross-platform abstraction wrappers
```

---

# 2. REQUIRED RUNTIME TARGET

```text
Target OS   : Windows 10+
Target ABI  : Win32 / PE
Target Type : Native desktop application
Target Arch : x86_64
Output      : PE executable (.exe)
```

---

# 3. REQUIRED ENTRY POINT

The application MUST use:

```cpp
int WINAPI WinMain(HINSTANCE hInstance,
                   HINSTANCE hPrevInstance,
                   LPSTR lpCmdLine,
                   int nCmdShow)
```

---

# 4. FORBIDDEN ENTRY POINTS

Forbidden:

```cpp
main()
wmain()
```

Reason:

```text
The Win32 subsystem requires WinMain.
```

---

# 5. REQUIRED SYSTEM DEPENDENCIES

These are OS/runtime dependencies.

DO NOT generate local replacements.

```text
user32
kernel32
gdi32
imm32
d2d1
dwrite
ole32
shell32
comdlg32
```

STRICTLY FORBIDDEN:

```text
src/user32.cpp
src/kernel32.cpp
src/direct2d.cpp
src/dwrite.cpp
src/imm32.cpp
```

---

# 6. REQUIRED WINDOW LOOP

The editor MUST use the standard Win32 message loop.

```cpp
MSG msg;
while (GetMessage(&msg, NULL, 0, 0)) {
    TranslateMessage(&msg);
    DispatchMessage(&msg);
}
```

Window callback:

```cpp
LRESULT CALLBACK WndProc(HWND hwnd,
                         UINT msg,
                         WPARAM wParam,
                         LPARAM lParam)
```

---

# 7. REQUIRED RENDERING STACK

Allowed:

```text
Direct2D
DirectWrite
IMM32
```

Forbidden:

```text
OpenGL
Vulkan
SDL renderer
Skia
Chromium renderer
GDI text rendering
```

---

# 8. REQUIRED BUILD TOOLCHAIN

```text
Compiler : MSVC 2022+
Standard : C++17
Target   : Win32 x64
```

---

# 9. REQUIRED CMAKE STRUCTURE

```cmake
cmake_minimum_required(VERSION 3.16)
project(MyLightIDE)

set(CMAKE_CXX_STANDARD 17)

add_executable(MyLightIDE WIN32
    src/main.cpp
    src/win32_app.cpp
    src/text_buffer.cpp
    src/lua_runtime.cpp
    src/input.cpp
    src/renderer.cpp
)

target_include_directories(MyLightIDE PRIVATE include)

target_link_libraries(MyLightIDE
    d2d1
    dwrite
    imm32
    ole32
    shell32
    comdlg32
)
```

---

# 10. REQUIRED CORE COMPONENTS

The application MUST contain:

```text
1. Direct2D renderer
2. Custom text buffer
3. Embedded Lua 5.4 runtime
4. Win32 input handler
5. Viewport logic
6. UTF-8 rendering pipeline
```

---

# 11. REQUIRED COMPONENT COUNT POLICY

```text
5 ~ 10 core modules maximum
```

Avoid:

```text
placeholder explosion
generic architecture inflation
```

---

# 12. TEXT BUFFER MODEL

Preferred structure:

```cpp
std::vector<std::string> lines;
```

Encoding:

```text
UTF-8
```

---

# 13. FILE SIZE LIMIT

Maximum:

```text
10,000 lines
```

If exceeded:

```text
abort loading immediately
close file handle
show warning message
```

---

# 14. LUA VERSION CONTRACT

Required:

```text
Lua 5.4 ONLY
Static linking ONLY
Single lua_State ONLY
```

Forbidden:

```text
LuaJIT
LuaRocks
runtime package installs
multiple lua_State instances
```

---

# 15. LUA OWNERSHIP

Lua owns:

```text
cursor state
viewport state
editor logic
motion logic
render ordering
```

---

# 16. C++ OWNERSHIP

C++ owns:

```text
Win32 APIs
IMM32 input handling
Direct2D rendering
file I/O
process launch
thread launch
clipboard APIs
```

---

# 17. WINDOW ARCHITECTURE MAP

```text
UI Component               Win32 Mapping
────────────────────────────────────────────
Main Window             → HWND
Sidebar                 → Child HWND
Editor Surface          → Custom Render HWND
Status Bar              → Child HWND
Menu Bar                → HMENU
File Explorer           → Owner-drawn list
Extension Manager       → Owner-drawn list
```

---

# 18. WINDOW HIERARCHY

```text
Main HWND
 ├─ MenuBar
 ├─ Activity Sidebar
 ├─ Explorer Panel
 ├─ Extension Panel
 ├─ Editor Surface HWND
 └─ Status Bar HWND
```

---

# 19. INPUT EVENT CONTRACT

```text
WM_KEYDOWN   → navigation / control keys
WM_CHAR      → printable UTF-16 input
WM_IME_*     → IME composition handling
```

Routing:

```text
Win32 Event         → C++ Handler        → Lua Callback
────────────────────────────────────────────────────────
WM_KEYDOWN          → on_key()           → lua_call("on_key")
WM_PAINT            → draw_frame()       → lua_call("on_render")
WM_LBUTTONDOWN      → on_click()         → lua_call("on_click")
```

---

# 20. PRINTABLE CHARACTER INPUT FLOW

Rules:

```text
WM_CHAR handles printable character insertion.
WM_KEYDOWN handles navigation and shortcuts.
```

C++ directly owns:

```text
character insertion
Backspace
Delete
Enter
clipboard shortcuts
```

Lua owns:

```text
cursor movement
viewport scrolling
motion semantics
```

---

# 21. UTF-16 → UTF-8 CONVERSION RULE

Win32 input arrives as UTF-16.

The editor buffer MUST remain UTF-8 internally.

Required conversion:

```cpp
WideCharToMultiByte(CP_UTF8, ...)
```

Forbidden:

```text
ASCII-only insertion
char cast conversion
```

---

# 22. IME SPECIFICATION

IME handling MUST use IMM32.

Required messages:

```text
WM_IME_STARTCOMPOSITION
WM_IME_COMPOSITION
WM_IME_ENDCOMPOSITION
```

Required APIs:

```text
ImmGetContext
ImmReleaseContext
ImmGetCompositionStringW
```

---

# 23. IME PREEDIT RENDERING

Composition text MUST NOT immediately commit into the buffer.

Required behavior:

```text
preedit text is rendered separately
preedit disappears immediately after commit
redraw invalidation required after composition change
```

---

# 24. CTRL SHORTCUT PRIORITY RULE

Control shortcuts MUST execute BEFORE IME handling.

Examples:

```text
Ctrl+C
Ctrl+V
Ctrl+X
Ctrl+A
Ctrl+S
Ctrl+O
```

Reason:

```text
IME interception can consume control combinations.
```

---

# 25. REQUIRED RENDER LOOP

```cpp
BeginPaint()
Direct2D BeginDraw()
lua_call("on_render")
Direct2D EndDraw()
EndPaint()
```

Redraw requests:

```cpp
InvalidateRect(hwnd, NULL, FALSE);
```

---

# 26. VIEWPORT CONTRACT

Only visible lines may render.

Rules:

```text
visible_start = scroll_offset
visible_end   = scroll_offset + visible_line_count
```

Forbidden:

```text
full-buffer rendering every frame
```

---

# 27. LUA BINDING TABLE

Required Lua native functions:

| Lua Function          | Native Backend          |
| --------------------- | ----------------------- |
| draw_text             | DirectWrite DrawText    |
| set_color             | Direct2D brush color    |
| set_font_size         | DirectWrite text format |
| get_canvas_width      | HWND client rect        |
| get_canvas_height     | HWND client rect        |
| request_redraw        | InvalidateRect          |
| get_buffer_line       | text buffer             |
| get_buffer_line_count | text buffer             |

---

# 28. FILE DIALOG CONTRACT

Required:

```text
IFileDialog
```

Forbidden:

```text
GTK dialogs
cross-platform wrappers
```

---

# 29. CLIPBOARD CONTRACT

Required APIs:

```text
OpenClipboard
GetClipboardData
SetClipboardData
CF_UNICODETEXT
```

Clipboard format MUST be:

```text
UTF-16 on OS boundary
UTF-8 internally
```

---

# 30. FILE EXPLORER CONTRACT

Required backend:

```cpp
std::filesystem::directory_iterator
```

Sorting:

```text
1. .. parent directory
2. directories
3. files
4. case-insensitive alphabetical sort
```

Hidden files:

```text
visible
```

---

# 31. EXTENSION SYSTEM CONTRACT

Extensions are OPTIONAL.

Rules:

```text
failure-tolerant
removable
non-fatal
```

Editor MUST survive extension failure.

---

# 32. EXTENSION DOWNLOAD PIPELINE

Required behavior:

```text
background thread launch
curl download
zip extraction
runtime reload safe
```

Required APIs:

```text
CreateProcessW
WaitForSingleObject
ShellExecuteW
```

---

# 33. EXTENSION ISOLATION

Lua extension loading MUST use:

```text
lua_pcall()
```

Forbidden:

```text
unchecked luaL_dofile crash propagation
```

---

# 34. CONFIG ROOT POLICY

Required root:

```text
%APPDATA%/MyLightIDE/
```

Structure:

```text
config/
ext/
```

---

# 35. PROJECT DIRECTORY STRUCTURE

```text
MyLightIDE/
├── CMakeLists.txt
├── build/
├── src/
│   ├── main.cpp
│   ├── win32_app.cpp
│   ├── text_buffer.cpp
│   ├── lua_runtime.cpp
│   ├── input.cpp
│   └── renderer.cpp
├── include/
│   ├── win32_app.h
│   ├── text_buffer.h
│   ├── lua_runtime.h
│   ├── input.h
│   └── renderer.h
└── lua/
    ├── core/
    │   ├── editor.lua
    │   ├── renderer.lua
    │   └── motion.lua
    └── init.lua
```

---

# 36. REQUIRED VALIDATOR RULES

Validator MUST reject:

```text
GTK includes
GtkWidget
GtkApplication
g_signal_connect
GtkDrawingArea
GtkEventController
```

Validator MUST require:

```text
WinMain
WndProc
Direct2D
DirectWrite
IMM32
```

---

# 37. SUCCESS CONDITIONS

Project success means:

```text
Direct2D rendering success
Lua runtime success
Custom text buffer success
IME input success
Clipboard success
External process success
```

NOT:

```text
feature completeness
```

---

# 38. PORTING COMPATIBILITY CONTRACT

The following semantics MUST remain identical to the Linux GTK4 backend:

```text
Lua ownership model
Viewport behavior
Text buffer structure
Extension lifecycle
Renderer callback flow
Cursor semantics
Scroll semantics
Clipboard semantics
```

Only the platform backend implementation changes.

---

# 39. RENDERER FLOW

```text
WM_PAINT
    ↓
BeginDraw()
    ↓
lua_call("on_render")
    ↓
draw_text()
    ↓
DirectWrite rendering
    ↓
EndDraw()
```

---

# 40. FINAL ARCHITECTURE RULE

The editor MUST behave as:

```text
Lua-driven editor semantics
+ Native Win32 runtime shell
+ Direct2D renderer
+ IMM32 IME integration
```

The project MUST NOT evolve into:

```text
cross-platform abstraction framework
Electron architecture
widget toolkit wrapper
browser renderer
```

