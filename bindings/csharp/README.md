<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Gym — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for fast, deterministic RL rollouts" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/ci.svg)](https://github.com/wickra-lib/wickra-gym/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-gym)
[![NuGet](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/nuget.svg)](https://www.nuget.org/packages/Wickra.Gym)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/license.svg)](https://github.com/wickra-lib/wickra-gym#license)

# Wickra Gym — C#

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for C#. `dotnet add package Wickra.Gym` — prebuilt native library, no system dependencies.**

.NET bindings for [`wickra-gym`](https://github.com/wickra-lib/wickra-gym) over
the C ABI hub, via source-generated P/Invoke. Build an `Env` from a spec JSON,
drive it with command JSON and read back observations and rewards — the same
protocol the CLI and every other binding speak, returning the same bytes.

## Install

```bash
dotnet add package Wickra.Gym
```

The native library ships prebuilt per platform under `runtimes/<rid>/native/`,
selected automatically. There is nothing to compile. Targets .NET 8 and later.

Requires .NET 8+. The native library (`wickra_gym`) must be resolvable on the
loader path — `PATH` on Windows, `LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH`
on macOS — or beside the assembly, where the bundled resolver finds it.

## Quick start

```csharp
using Wickra.Gym;

const string spec = """
{"dataset_ref":"demo","symbol":"BTCUSDT",
 "observation":{"features":[{"kind":"indicator","name":"Rsi","params":[14]},
                            {"kind":"price","field":"close"}]},
 "action_space":{"type":"discrete","n":3},
 "reward":"pnl","episode":{"max_steps":256,"warmup":14}}
""";

using var env = new Env(spec);
env.Command("""{"cmd":"load","candles":[ … ]}""");
string reset = env.Command("""{"cmd":"reset","seed":7}""");
string step = env.Command("""{"cmd":"step","action":2.0}""");
```

The dataset is precomputed once into a fixed feature tensor, so every `step` is a
pure array index and a `(seed, policy)` pair fully determines the trajectory — in
this binding exactly as in the other nine.

`episode.warmup` must be at least the longest lookback the observation's
indicators declare: below it a column is `0.0` because nothing has been produced
yet, which an agent cannot tell from a market reading of zero. A spec below the
floor is refused rather than run. An indicator that reads an order book or a
funding print needs bars that carry them, and is likewise refused rather than
answered with a column of constant zeros.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of `[LibraryImport]` P/Invoke over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-gym/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-gym>
- **Docs** (guides, spec reference, cookbook): <https://gym.wickra.org>
- **Runnable example:** [`examples/csharp/`](https://github.com/wickra-lib/wickra-gym/tree/main/examples/csharp)

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
