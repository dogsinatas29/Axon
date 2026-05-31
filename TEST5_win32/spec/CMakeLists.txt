# AXON v0.0.31 SOVEREIGN 빌드 스크립트 (SPEC-DRIVEN)
cmake_minimum_required(VERSION 3.16)
project(spec)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 17)

include_directories(include)

add_executable(spec src/input.cpp src/lua_runtime.cpp src/main.cpp src/renderer.cpp src/text_buffer.cpp src/win32_app.cpp)
