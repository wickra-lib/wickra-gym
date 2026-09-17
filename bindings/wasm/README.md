<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Gym — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for fast, deterministic RL rollouts" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/ci.svg)](https://github.com/wickra-lib/wickra-gym/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-gym)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/npm.svg)](https://www.npmjs.com/package/wickra-gym-wasm)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/license.svg)](https://github.com/wickra-lib/wickra-gym#license)

# Wickra Gym — WASM

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for WASM. `npm install wickra-gym-wasm` — pure WebAssembly, runs anywhere a modern JS engine does.**

A deterministic, Gymnasium-compatible backtest environment compiled to
WebAssembly, for running rollouts directly in the browser (or any WASM host).
The feature-tensor precompute runs sequentially here — byte-identical to the
native parallel build — so a WASM rollout matches every other language binding.

## Install

```bash
npm install wickra-gym-wasm
```

### Building from this repository (contributors)

```sh
wasm-pack build --target web       # for bundlers / browsers
wasm-pack build --target nodejs    # for Node.js (used by the tests)
```

## Quick start

```js
import init, { Env } from "wickra-gym-wasm";

await init();

const spec = JSON.stringify({
  dataset_ref: "demo",
  symbol: "BTCUSDT",
  observation: { features: [{ kind: "price", field: "close" }] },
  action_space: { type: "discrete", n: 3 },
  reward: "pnl",
  episode: { max_steps: 256, warmup: 0 },
});

const candles = Array.from({ length: 300 }, (_, i) => ({
  ts: i, open: 100 + i, high: 100 + i, low: 100 + i, close: 100 + i,
}));

const env = new Env(spec);
env.command(JSON.stringify({ cmd: "load", candles }));
const reset = JSON.parse(env.command(JSON.stringify({ cmd: "reset", seed: 0 })));
const step = JSON.parse(env.command(JSON.stringify({ cmd: "step", action: 2 })));
```

The surface is a single JSON command boundary — the same one every language
binding forwards verbatim. Commands: `load`, `reset`, `step`, `spec`, `version`.
Domain errors come back as `{"ok": false, "error": ...}`; a bad spec throws at
construction.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of wasm-bindgen, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-gym/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-gym>
- **Docs** (guides, spec reference, cookbook): <https://gym.wickra.org>
- **Runnable example:** [`examples/wasm/`](https://github.com/wickra-lib/wickra-gym/tree/main/examples/wasm)

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
