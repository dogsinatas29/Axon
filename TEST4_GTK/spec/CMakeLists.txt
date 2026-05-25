# AXON v0.0.29.25 SOVEREIGN BUILD SCRIPT (PRUNING_ENABLED)
cmake_minimum_required(VERSION 3.10)
project(spec)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 17)

include_directories(include)

add_executable(spec src/gtk_app.cpp src/input.cpp src/lua_runtime.cpp src/main.cpp src/terminal_renderer.cpp src/text_buffer.cpp)
