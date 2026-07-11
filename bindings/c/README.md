# Wickra Gym — C ABI

The C ABI is the hub every C-capable language (C, C++, C#, Go, Java, R) links
against. It exposes `gym-core` — a deterministic, Gymnasium-compatible backtest
environment — as a tiny, JSON-shaped surface built as both a `cdylib` (dynamic
library) and a `staticlib`.

## Surface

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

## Command / response protocol

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

## Header generation

`include/wickra_gym.h` is generated with [cbindgen] and committed; CI fails if it
drifts from the source. Regenerate after changing the ABI:

```sh
cbindgen --config cbindgen.toml --output include/wickra_gym.h
```

## Building

```sh
cargo build -p wickra-gym-c --release
```

This produces `libwickra_gym` as both a shared and a static library under
`target/release/`, plus the committed header at `include/wickra_gym.h`.

## Determinism

The environment is fully deterministic: the same spec, dataset, seed and action
sequence produce a byte-identical `{reset, trajectory}` across every language
that links this ABI. See the [main repository][repo] for the observation and
reward semantics.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-APACHE),
at your option.

[cbindgen]: https://github.com/mozilla/cbindgen
[repo]: https://github.com/wickra-lib/wickra-gym
[`EnvSpec`]: https://github.com/wickra-lib/wickra-gym
