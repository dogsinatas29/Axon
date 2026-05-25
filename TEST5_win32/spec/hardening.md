# encoding: utf-8
# Hardening & Defensive Engineering Guidelines

This document details failure cases, edge scenarios, and defense guidelines to avoid rendering failures, window leaks, and process hanging.

## 1. Window Destruction & Loop Termination
- Always call `PostQuitMessage(0)` inside the `WM_DESTROY` message block of `WndProc`. Failure to do so will result in the application window closing while the process continues to hang in the background.
- Cleanly intercept `WM_CLOSE` to allow the application to prompt save states before calling `DestroyWindow`.

## 2. Win32 Object & Handle Leak Prevention
- Always release or delete custom drawing objects (brushes, pens, fonts) generated via `CreateSolidBrush` or `CreateFont` using `DeleteObject`.
- Ensure device contexts obtained via `GetDC` are released with `ReleaseDC`, and device contexts obtained via `BeginPaint` are released with `EndPaint`.

## 3. HDC & HWND Null Checking
- Validate the window handle (`HWND`) returned by `CreateWindowEx` before entering the message pump. If `HWND` is null, log the error and terminate the process immediately.
- Guard the device context handle (`HDC`) returned by `BeginPaint`. Do not perform any GDI calls (`TextOut`, etc.) if `HDC` is null.
