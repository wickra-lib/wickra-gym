<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Gym — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for fast, deterministic RL rollouts" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/ci.svg)](https://github.com/wickra-lib/wickra-gym/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-gym)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/npm.svg)](https://www.npmjs.com/package/wickra-gym)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/license.svg)](https://github.com/wickra-lib/wickra-gym#license)

# Wickra Gym — Node.js

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for Node.js. `npm install wickra-gym` — prebuilt native binary, no system dependencies.**

A deterministic, Gymnasium-compatible backtest environment, powered by Rust. The
whole candle dataset is precomputed once into a fixed feature tensor, so each
`step` is a constant-time array index — and the same spec, data, seed and actions
produce a byte-identical trajectory across every language binding.

## Install

```bash
npm install wickra-gym
```

The native addon ships as a prebuilt binary per platform (Linux, macOS,
Windows — x64 and arm64), selected automatically through optional
dependencies. There is nothing to compile.

The correct native binary is pulled in automatically as an optional dependency
for your platform (Linux / macOS / Windows, x64 / arm64).

## Quick start

```js
const { Env } = require('wickra-gym')

const spec = JSON.stringify({
  dataset_ref: 'demo',
  symbol: 'BTCUSDT',
  observation: { features: [
    { kind: 'price', field: 'close' },
    { kind: 'indicator', name: 'Rsi', params: [14] },
  ] },
  action_space: { type: 'discrete', n: 3 },
  reward: 'pnl',
  episode: { max_steps: 256, warmup: 14 },
})

const candles = Array.from({ length: 300 }, (_, i) => ({
  ts: i, open: 100 + i, high: 100 + i, low: 100 + i, close: 100 + i,
}))

const env = new Env(spec)
env.command(JSON.stringify({ cmd: 'load', candles }))
const reset = JSON.parse(env.command(JSON.stringify({ cmd: 'reset', seed: 0 })))
const step = JSON.parse(env.command(JSON.stringify({ cmd: 'step', action: 2 })))
console.log(step.reward, step.terminated)
```

The surface is a single JSON command boundary — the same one every language
binding forwards verbatim. Commands: `load`, `reset`, `step`, `spec`, `version`.
Domain errors come back as `{"ok": false, "error": ...}`; a bad spec throws at
construction.

### Determinism

`spec`, `data`, `seed` and the action sequence fully determine a byte-identical
`{reset, trajectory}`. See the [main repository](https://github.com/wickra-lib/wickra-gym)
for the observation and reward semantics.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of napi-rs, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-gym/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-gym>
- **Docs** (guides, spec reference, cookbook): <https://gym.wickra.org>
- **Runnable example:** [`examples/node/`](https://github.com/wickra-lib/wickra-gym/tree/main/examples/node)

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
