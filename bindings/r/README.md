<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Gym — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for fast, deterministic RL rollouts" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/ci.svg)](https://github.com/wickra-lib/wickra-gym/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-gym)
[![r-universe](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/r-universe.svg)](https://wickra-lib.r-universe.dev)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/license.svg)](https://github.com/wickra-lib/wickra-gym#license)

# Wickra Gym — R

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for R. `install.packages("wickragym", repos = "https://wickra-lib.r-universe.dev")` — over the C ABI via `.Call`, prebuilt library fetched on install.**

R bindings for [wickra-gym](https://github.com/wickra-lib/wickra-gym) — a
deterministic, Gymnasium-compatible backtest environment — over the C ABI, via
R's `.Call` interface. A rollout is byte-identical to every other language
binding.

## Install

From r-universe:

```r
install.packages("wickragym", repos = "https://wickra-lib.r-universe.dev")
```

The package's `configure` downloads the prebuilt C ABI library for this exact
version from the GitHub release and bundles it, so an ordinary install needs
nothing but a C toolchain (Rtools on Windows) for the thin `.Call` glue layer. To
build against a local checkout instead, point it at the header and library with
the environment variables below.

The package links the native `wickra_gym` C ABI library. Point the build at its
header and library directories:

```sh
export WKGYM_INC=/path/to/wickra-gym/bindings/c/include
export WKGYM_LIB=/path/to/wickra-gym/target/release   # holds libwickra_gym.*
R CMD INSTALL bindings/r
```

At run time the loader must find the shared library (via `PATH` on Windows,
`LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH` on macOS).

## Quick start

```r
library(wickragym)

spec <- paste0(
  '{"dataset_ref":"demo","symbol":"BTCUSDT",',
  '"observation":{"features":[{"kind":"price","field":"close"}]},',
  '"action_space":{"type":"discrete","n":3},',
  '"reward":"pnl","episode":{"max_steps":256,"warmup":0}}'
)

env <- wkgym_new(spec)

candles <- paste0(
  '[', paste(vapply(0:299, function(i) {
    p <- 100 + i
    sprintf('{"ts":%d,"open":%f,"high":%f,"low":%f,"close":%f}', i, p, p, p, p)
  }, character(1)), collapse = ","), ']'
)
wkgym_command(env, paste0('{"cmd":"load","candles":', candles, '}'))

reset <- wkgym_command(env, '{"cmd":"reset","seed":0}')
step  <- wkgym_command(env, '{"cmd":"step","action":2}')
cat(step, "\n")
```

Commands: `load`, `reset`, `step`, `spec`, `version`. Domain errors come back as
`{"ok":false,"error":...}`; a bad spec raises an R error.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of R's native `.Call` interface over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-gym/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-gym>
- **Docs** (guides, spec reference, cookbook): <https://gym.wickra.org>
- **Runnable example:** [`examples/r/`](https://github.com/wickra-lib/wickra-gym/tree/main/examples/r)

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
