cmake_minimum_required(VERSION 3.16)
project(hex_engine_cpp CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    /opt/homebrew/include
    /usr/local/include
)

link_directories(
    /opt/homebrew/lib
    /usr/local/lib
)

# Path to compiled Rust C-ABI library
set(RUST_LIB_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../target/release)
find_library(HEX_CORE_LIB hex_engine PATHS ${RUST_LIB_DIR} NO_DEFAULT_PATH)
find_library(SDL3_LIB SDL3 PATHS /opt/homebrew/lib /usr/local/lib)

# Standalone CLI binary
add_executable(hex_cpp_cli src/main.cpp)
if(HEX_CORE_LIB)
    target_link_libraries(hex_cpp_cli PRIVATE ${HEX_CORE_LIB})
endif()

# Unit test binary
add_executable(test_cpp_engine tests/test_cpp_engine.cpp)
if(HEX_CORE_LIB)
    target_link_libraries(test_cpp_engine PRIVATE ${HEX_CORE_LIB})
endif()

# Standalone Benchmark binary
add_executable(hex_benchmark src/benchmark.cpp)
if(HEX_CORE_LIB)
    target_link_libraries(hex_benchmark PRIVATE ${HEX_CORE_LIB})
endif()

# Native C++ GUI binary
if(SDL3_LIB)
    add_executable(hex_gui src/gui_main.cpp)
    if(HEX_CORE_LIB)
        target_link_libraries(hex_gui PRIVATE ${HEX_CORE_LIB} ${SDL3_LIB})
    else()
        target_link_libraries(hex_gui PRIVATE ${SDL3_LIB})
    endif()
endif()
