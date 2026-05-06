# AXON v0.0.26 — CMake Auto Generator (Target / Dependency Graph Based)

## Goal
Automatically generate CMakeLists.txt from dependency graph and registry.

---

## Inputs
- File Registry
- Dependency Graph
- Symbol Ownership Map

---

## Core Principles
- Target-based (not file-based)
- Minimal linking
- No circular dependencies
- Deterministic output

---

## Pipeline
Registry → Dependency Graph → Target Classification → Target Graph → CMake Generation

---

## Target Classification
- main() present → Executable
- otherwise → Library

---

## Example Output
```cmake
cmake_minimum_required(VERSION 3.10)
project(AxonProject)

set(CMAKE_CXX_STANDARD 17)

add_library(UserRepository UserRepository.cpp)
add_library(User User.cpp)

add_executable(UserService UserService.cpp)

target_link_libraries(UserService PRIVATE UserRepository User)
target_link_libraries(UserRepository PRIVATE User)
```

---

## Key Rules
- One target per module
- No global linking
- Use PRIVATE linkage
- Deduplicate dependencies

---

## Validation
- Detect circular dependencies
- Ensure all linked targets exist
- Remove unused dependencies

---

## Summary
CMake is a serialized dependency graph.
