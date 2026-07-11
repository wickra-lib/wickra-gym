# Build and run the C and C++ examples against the wickra-gym C ABI.
#
#   cargo build --release -p wickra-gym-c
#   cmake -S examples/c -B examples/c/build
#   cmake --build examples/c/build --config Release
#   ctest --test-dir examples/c/build -C Release --output-on-failure
#
# WICKRA_GYM_LIB_DIR defaults to the workspace target/release directory and can
# be overridden for out-of-tree builds.

cmake_minimum_required(VERSION 3.15)
project(wickra_gym_examples C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(WICKRA_GYM_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release"
    CACHE PATH "Directory containing the built wickra-gym C ABI library")

set(WICKRA_GYM_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/../../bindings/c/include")
set(WICKRA_GYM_DATA "${CMAKE_CURRENT_SOURCE_DIR}/../data")

if(WIN32 AND CMAKE_C_COMPILER_ID MATCHES "GNU")
    # MinGW's ld cannot consume the MSVC-built static import archive
    # (wickra_gym.lib pulls in objects referencing __chkstk). GNU ld links a
    # cdylib directly from the DLL and synthesises the import stubs itself.
    set(WICKRA_GYM_LIB "${WICKRA_GYM_LIB_DIR}/wickra_gym.dll")
else()
    # Unix resolves the shared object; MSVC consumes the .lib import library.
    find_library(WICKRA_GYM_LIB
        NAMES wickra_gym libwickra_gym
        HINTS "${WICKRA_GYM_LIB_DIR}"
        REQUIRED)
endif()

add_executable(rollout rollout.c)
target_include_directories(rollout PRIVATE "${WICKRA_GYM_INCLUDE}")
target_link_libraries(rollout PRIVATE "${WICKRA_GYM_LIB}")
target_compile_definitions(rollout PRIVATE DATA_DIR="${WICKRA_GYM_DATA}")

add_executable(rolloutpp rollout.cpp)
target_include_directories(rolloutpp PRIVATE "${WICKRA_GYM_INCLUDE}")
target_link_libraries(rolloutpp PRIVATE "${WICKRA_GYM_LIB}")
target_compile_definitions(rolloutpp PRIVATE DATA_DIR="${WICKRA_GYM_DATA}")

# On Windows the runtime DLL must sit next to the executable (there is no rpath);
# without it the loader pops a modal "DLL not found" dialog that hangs ctest. Copy
# it next to each example after build. (Unix resolves the .so/.dylib via the
# linker's embedded path, so this is Windows-only.)
if(WIN32)
    foreach(example rollout rolloutpp)
        add_custom_command(TARGET ${example} POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy_if_different
                "${WICKRA_GYM_LIB_DIR}/wickra_gym.dll"
                "$<TARGET_FILE_DIR:${example}>"
            VERBATIM)
    endforeach()
endif()

enable_testing()
add_test(NAME rollout COMMAND rollout)
add_test(NAME rolloutpp COMMAND rolloutpp)
# A missing runtime dependency must fail fast, never block the runner for hours.
set_tests_properties(rollout rolloutpp PROPERTIES TIMEOUT 60)
