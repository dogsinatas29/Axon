# AXON v0.0.29.25 SOVEREIGN 빌드 스크립트 (PRUNING_ENABLED)
cmake_minimum_required(VERSION 3.10)
project(spec)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 17)

include_directories(include)

add_executable(spec src/input_handler.c src/main.c src/main_window.c src/text_buffer.c)
