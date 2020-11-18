cmake_minimum_required(VERSION 3.10)

project(Kaleidescope)

set(SOURCES src/main.cpp src/lexer.cpp)
set(CMAKE_EXPORT_COMPILE_COMMANDS TRUE)

find_package(LLVM REQUIRED CONFIG)
include_directories(${LLVM_INCLUDE_DIRS})
add_definitions(${LLVM_DEFINITIONS})

add_executable(kaleidoscope ${SOURCES})
set_target_properties(kaleidoscope PROPERTIES
  CXX_STANDARD 17
  CXX_EXTENSIONS OFF)

llvm_map_components_to_libnames(llvm_libs support core irreader)
target_link_libraries(kaleidoscope ${llvm_libs})
