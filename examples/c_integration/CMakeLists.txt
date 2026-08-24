cmake_minimum_required(VERSION 3.16)
project(oxidize_c_embedded_example C)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

include_directories(../../poc/output)

add_executable(c_embedded_demo main.c)

# Link against the compiled static Rust archive (liboxidize_core.a or similar)
# target_link_libraries(c_embedded_demo PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../../target/release/liboxidize_core.a)
