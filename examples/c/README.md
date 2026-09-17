# Wickra Gym — C / C++ examples

The Wickra Gym C ABI is a single shared/static library plus a generated header
([`bindings/c/include/wickra_gym.h`](../../bindings/c/include/wickra_gym.h)). Any C-capable
language links against the same artifact; these examples show the plain-C path
and, through [`wickra_gym.hpp`](../../bindings/c/include/wickra_gym.hpp), the C++ one.

## Build the library

From the workspace root:

```sh
cargo build -p wickra-gym-c --release
```

This produces, in `target/release/`:

| Platform | Shared library | Link target |
|----------|----------------|-------------|
| Linux    | `libwickra_gym.so`     | `-lwickra_gym` |
| macOS    | `libwickra_gym.dylib`  | `-lwickra_gym` |
| Windows (MSVC) | `wickra_gym.dll` | `wickra_gym.dll.lib` (import lib) |

A static library (`libwickra_gym.a` / `wickra_gym.lib`) is emitted alongside.

## Build and run the examples

### With CMake (portable, used by CI)

```sh
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

### Directly with a compiler

```sh
# Linux / macOS
cc examples/c/rollout.c -DDATA_DIR=\"examples/data\" -I bindings/c/include -L target/release -lwickra_gym -lm -o rollout
LD_LIBRARY_PATH=target/release ./rollout        # macOS: DYLD_LIBRARY_PATH

# Windows (MinGW gcc, linking the DLL directly)
gcc examples/c/rollout.c -DDATA_DIR=\"examples/data\" -I bindings/c/include target/release/wickra_gym.dll -lm -o rollout.exe
```

## The examples

| Example | What it does |
|---------|--------------|
| `rollout.c` | A minimal C example: load a spec and its candle dataset over the wickra-gym C |
| `rollout.cpp` | A minimal C++ example: load a spec and its candle dataset, then drive a fixed long policy and print each step. |

## Usage shape

Every call follows the same handle discipline: construct from a spec JSON, drive
with command JSON, read the response, free the handle exactly once. `wickra_gym.h` is
the whole contract; the C++ header, where one ships, wraps the handle in a
move-only RAII type. See [`bindings/c/README.md`](../../bindings/c/README.md).
