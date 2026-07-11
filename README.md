<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra Gym — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for fast, deterministic RL rollouts" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-gym)
[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/ci.svg)](https://github.com/wickra-lib/wickra-gym/actions/workflows/ci.yml)
[![CodeQL](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/codeql.svg)](https://github.com/wickra-lib/wickra-gym/actions/workflows/codeql.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-gym)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/license.svg)](#license)
[![OpenSSF Scorecard](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/scorecard.svg)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra-gym)
[![OpenSSF Best Practices](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/best-practices.svg)](https://www.bestpractices.dev/)
[![Build provenance](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/provenance.svg)](https://github.com/wickra-lib/wickra-gym/attestations)
[![Deterministic across 10 languages](https://img.shields.io/badge/deterministic%20across-10%20languages-3b82f6)](#use-in-any-language)
[![Docs](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/docs.svg)](https://wickra.org)

---

# Wickra Gym

**A Gymnasium-compatible, microstructure-aware backtest environment with O(1)
steps for fast RL rollouts over 514 streaming indicators.** The whole dataset is
precomputed once into a fixed feature tensor, so `step()` is a pure array index —
and the same seed produces a byte-identical trajectory in every language.

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib).** Built on the
> same deterministic engine and ten-language binding surface as
> [wickra-backtest](https://github.com/wickra-lib/wickra-backtest),
> [wickra-benchmark](https://github.com/wickra-lib/wickra-benchmark) and the rest.

`wickra-gym` turns a candle dataset plus an observation/action/reward
specification into a reinforcement-learning environment. It precomputes every
bar's features (indicators, and optionally orderbook / funding / open-interest
microstructure) in one O(1)-per-bar pass into a fixed `FeatureTensor`; from then
on each `step()` is a constant-time index into that tensor, so rollouts are fast.
The reward draws on the [`wickra-backtest`](https://github.com/wickra-lib/wickra-backtest)
fill/PnL model, and the only source of randomness is an explicitly-seeded episode
RNG — so a `(seed, policy)` pair fully determines a trajectory.

The core (`gym-core`) is a JSON-over-C-ABI data API usable in ten languages, and
Python additionally ships a real **`gymnasium.Env` subclass** as its primary
consumer.

## Status

Pre-1.0, under active construction. The workspace, core, bindings, golden
trajectories, tests and CI land phase by phase; see [ROADMAP.md](ROADMAP.md) and
[CHANGELOG.md](CHANGELOG.md).

## Determinism is the product

- **O(1) steps** — the dataset is precomputed once to a fixed feature tensor;
  `step()` is a pure array index, never a recompute.
- **Seed-determined trajectories** — the only RNG is the seeded episode RNG
  (never `thread_rng`); the same `(seed, policy)` yields a byte-identical
  trajectory in every language and between the parallel (rayon) and sequential
  (WASM) tensor precompute.
- **Fixed observation layout** — an observation is an `f64` vector of fixed
  length and canonical order (the `ObsSpec` order), so it is comparable across
  languages bit for bit.

## Use in any language

The core drives a single `command_json` envelope (`load`, `reset`, `step`,
`spec`, `version`); every binding passes the same string through verbatim, so a
trajectory is byte-identical everywhere. Python additionally wraps the core in a
`gymnasium.Env` subclass.

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) — workspace layout, the feature tensor, and
  the environment lifecycle.
- [THREAT_MODEL.md](THREAT_MODEL.md) — assets, actors, and what determinism buys.
- Deep dives (`docs/`): the observation spec, the action spaces and reward kinds,
  and the gymnasium integration land alongside the core.

## Building

Rust `1.86+`. `cargo build` builds the workspace; `cargo test --workspace
--all-features` runs the suite. Per-binding build steps live in each
`bindings/<lang>/README.md`.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option.

## Disclaimer

Research and engineering tooling, not financial advice. A trained agent's
backtested performance says nothing about future returns; markets carry risk and
you are responsible for your own decisions. `wickra-gym` is free software you run
yourself: no hosted service, no data collection, no warranty.
