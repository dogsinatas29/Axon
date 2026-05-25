# encoding: utf-8
# Implementation Rules & API Cookbook

This document governs the coding styles, patterns, API signatures, and compilation rules of the MyWin32App source code.

## 1. C++ Compiler Rules & Standards
- All source files must compile without errors using standard Windows-compatible C++17 compilers.
- Enforce clean header inclusions (`windows.h`, `windowsx.h`).

## 2. Win32 Window Registration & Creation
- Define standard class name and register using `RegisterClass` or `RegisterClassEx`.
- Instantiate the window frame using `CreateWindow` or `CreateWindowEx` with parameters:
  - Subsystem target: `WS_OVERLAPPEDWINDOW | WS_VISIBLE`
  - Link callback: Address of the `WndProc` function.

## 3. Message Pump & WndProc Implementation
- The message loop must be structured precisely as:
  ```cpp
  MSG msg = { 0 };
  while (GetMessage(&msg, NULL, 0, 0)) {
      TranslateMessage(&msg);
      DispatchMessage(&msg);
  }
  ```
- The window procedure must follow this signature:
  `LRESULT CALLBACK WndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam)`
- Delegate unhandled messages to `DefWindowProc`.

## 4. GDI Rendering Cookbook
- Handle the `WM_PAINT` message inside the `WndProc` callback.
- Execute paint routines precisely matching:
  ```cpp
  PAINTSTRUCT ps;
  HDC hdc = BeginPaint(hWnd, &ps);
  TextOut(hdc, x, y, TEXT("Hello Axon"), len);
  EndPaint(hWnd, &ps);
  ```
