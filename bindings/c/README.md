<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Gym — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for fast, deterministic RL rollouts" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/ci.svg)](https://github.com/wickra-lib/wickra-gym/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-gym)
[![GitHub release](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/release.svg)](https://github.com/wickra-lib/wickra-gym/releases/latest)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/license.svg)](https://github.com/wickra-lib/wickra-gym#license)

# Wickra Gym — C / C++

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for C / C++. `cargo build -p wickra-gym-c --release` — a prebuilt shared/static library plus a generated `wickra_gym.h`, no system dependencies.**

The C ABI is the hub every C-capable language (C, C++, C#, Go, Java, R) links
against. It exposes `wickra-gym-core` — a deterministic, Gymnasium-compatible backtest
environment — as a tiny, JSON-shaped surface built as both a `cdylib` (dynamic
library) and a `staticlib`.

## Install

Grab the prebuilt header + library for your platform from the
[GitHub releases](https://github.com/wickra-lib/wickra-gym/releases) — each archive
has `wickra_gym.h`, the C++ wrapper where the binding ships one, and the shared/static
library — or build from source:

```bash
cargo build -p wickra-gym-c --release
# -> target/release/libwickra_gym.{so,dylib} or wickra_gym.dll (+ import lib) + a staticlib
```

Then compile against the header and link the library.

### Building from this repository (contributors)

```sh
cargo build -p wickra-gym-c --release
```

This produces `libwickra_gym` as both a shared and a static library under
`target/release/`, plus the committed header at `include/wickra_gym.h`.

## Quick start

[`examples/c/rollout.c`](https://github.com/wickra-lib/wickra-gym/blob/main/examples/c/rollout.c) is the runnable example the CI smoke job executes; in full:

```c
/* A minimal C example: load a spec and its candle dataset over the wickra-gym C
 * ABI, then drive a fixed long policy through the environment and print each
 * step. No JSON parser is needed — the spec and candle JSON are read verbatim,
 * the load command is assembled by hand, and each response is printed as-is.
 * DATA_DIR is injected by CMake. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_gym.h"

/* Read an entire text file into a freshly malloc'd, NUL-terminated buffer. */
static char *slurp(const char *path) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        fprintf(stderr, "cannot open %s\n", path);
        return NULL;
    }
    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *buf = (char *)malloc((size_t)n + 1);
    if (buf) {
        size_t got = fread(buf, 1, (size_t)n, f);
        buf[got] = '\0';
    }
    fclose(f);
    return buf;
}

/* Run a command and return its response using the length-out protocol. The core
 * caches the not-yet-delivered response, so the second (delivering) call reuses
 * it — a mutating step runs exactly once. */
static char *run(WickraGymEnv *env, const char *cmd) {
    int len = wickra_gym_command(env, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", len);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (buf) {
        wickra_gym_command(env, cmd, buf, (size_t)len + 1);
    }
    return buf;
}

int main(void) {
    char *spec = slurp(DATA_DIR "/specs/momentum_discrete.json");
    char *candles = slurp(DATA_DIR "/candles.json");
    if (!spec || !candles) {
        return 1;
    }

    WickraGymEnv *env = wickra_gym_new(spec);
    if (!env) {
        fprintf(stderr, "invalid spec\n");
        return 1;
    }

    size_t cap = strlen(candles) + 64;
    char *load = (char *)malloc(cap);
    snprintf(load, cap, "{\"cmd\":\"load\",\"candles\":%s}", candles);
    char *loaded = run(env, load);
    free(loaded);
    free(load);

    char *reset = run(env, "{\"cmd\":\"reset\",\"seed\":42}");
    printf("wickra-gym %s\n", wickra_gym_version());
    printf("reset: %s\n", reset ? reset : "(null)");
    free(reset);

    int steps = 0, ok = 1;
    for (;;) {
        char *step = run(env, "{\"cmd\":\"step\",\"action\":2}");
        if (!step || strstr(step, "\"ok\":false")) {
            ok = 0;
            free(step);
            break;
        }
        printf("step %d: %s\n", steps, step);
        int done = strstr(step, "\"terminated\":true") || strstr(step, "\"truncated\":true");
        free(step);
        steps++;
        if (done) {
            break;
        }
    }

    free(spec);
    free(candles);
    wickra_gym_free(env);
    if (!ok || steps == 0) {
        fprintf(stderr, "rollout failed\n");
        return 1;
    }
    return 0;
}
```

### Surface

```c
#include "wickra_gym.h"

WickraGymEnv *wickra_gym_new(const char *spec_json);           /* NULL on parse error */
void          wickra_gym_free(WickraGymEnv *handle);           /* null-safe */
int32_t       wickra_gym_command(WickraGymEnv *handle,
                                 const char *cmd_json,
                                 char *out, size_t cap);
const char   *wickra_gym_version(void);                        /* static, do not free */
```

- **`wickra_gym_new`** builds an environment from an [`EnvSpec`] JSON. Returns
  `NULL` if the spec is null, not UTF-8, or fails to parse / validate.
- **`wickra_gym_free`** destroys a handle (null is a no-op).
- **`wickra_gym_command`** applies a command JSON and writes the response JSON
  into the caller's buffer using a length-out protocol (below).
- **`wickra_gym_version`** returns a static, NUL-terminated version string.

### Command / response protocol

Everything goes through `wickra_gym_command`. Commands are JSON objects with a
`"cmd"` field:

| `cmd`     | Request fields             | Response                          |
|-----------|----------------------------|-----------------------------------|
| `load`    | `"candles": [<Candle>, …]` | `{"ok":true}` (builds the tensor) |
| `reset`   | `"seed": <u64>` (optional) | `ResetResult`                     |
| `step`    | `"action": <number>`       | `StepResult`                      |
| `spec`    | —                          | `SpecInfo`                        |
| `version` | —                          | `{"version":"…"}`                 |

The response is returned via a caller-owned buffer with a length-out protocol —
the callee never allocates memory the caller must free:

1. Call with `out = NULL`, `cap = 0` to learn the response length `len`
   (excluding the terminating NUL).
2. Allocate `len + 1` bytes and call again; the response plus a NUL is written.

Whenever `len < cap`, the response is written on that call, so a
sufficiently-large buffer needs only one call.

**Mutating commands.** `step` mutates the environment, so the two-call idiom must
not execute it twice. Each handle caches the response of the command it last
computed but has not yet delivered; a repeated call with the same command bytes
reuses that cached response, and the cache is cleared once the response is
delivered. A logical command therefore runs exactly once regardless of how many
buffer-sizing retries it takes.

Return codes:

| Return | Meaning                                              |
|--------|------------------------------------------------------|
| `>= 0` | Response length in bytes (excluding the NUL).        |
| `-1`   | A required pointer (`handle` or `cmd_json`) is null. |
| `-2`   | `cmd_json` is not valid UTF-8.                        |
| `-3`   | A panic was caught at the boundary.                  |

Domain errors (a bad spec, an out-of-range action) are **not** negative — they
come back in-band as `{"ok":false,"error":...}` JSON in the buffer.

### Header generation

`include/wickra_gym.h` is generated with [cbindgen] and committed; CI fails if it
drifts from the source. Regenerate after changing the ABI:

```sh
cbindgen --config cbindgen.toml --output include/wickra_gym.h
```

### Determinism

The environment is fully deterministic: the same spec, dataset, seed and action
sequence produce a byte-identical `{reset, trajectory}` across every language
that links this ABI. See the [main repository][repo] for the observation and
reward semantics.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the C ABI itself, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-gym/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-gym>
- **Docs** (guides, spec reference, cookbook): <https://gym.wickra.org>
- **Runnable example:** [`examples/c/`](https://github.com/wickra-lib/wickra-gym/tree/main/examples/c)

Wickra Gym ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-gym/blob/main/SECURITY.md>.

## Disclaimer

`wickra-gym` is research and engineering tooling, not financial advice. A trained
agent's backtested performance says nothing about future returns; markets carry
risk and you are responsible for your own decisions. `wickra-gym` is free
software you run yourself: no hosted service, no data collection, no warranty.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-MIT) at your option.
