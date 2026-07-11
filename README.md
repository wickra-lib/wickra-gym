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

## Status

**Pre-release — functionally complete, CI-verified, not yet published.** The
core, the CLI, all ten language bindings, the golden trajectory corpus, the
property + fuzz suites, the benchmarks and one runnable example per language are
built and green across Linux, macOS and Windows. Packages are not yet on the
registries. Track progress in [ROADMAP.md](ROADMAP.md) and
[CHANGELOG.md](CHANGELOG.md).

## Documentation

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — the workspace, the feature tensor, and
  the environment lifecycle.
- [`docs/OBSERVATIONS.md`](docs/OBSERVATIONS.md) — the `ObsSpec`, the feature
  families, and the exact observation-vector order.
- [`docs/ACTIONS_REWARDS.md`](docs/ACTIONS_REWARDS.md) — discrete/continuous
  action spaces, the position model, and the reward kinds.
- [`docs/GYMNASIUM.md`](docs/GYMNASIUM.md) — the `gymnasium.Env` subclass, how
  the `Box`/`Discrete` spaces are derived, and `make` / `register`.
- [`docs/MICROSTRUCTURE.md`](docs/MICROSTRUCTURE.md) — orderbook levels, funding
  and open-interest observations.
- [`THREAT_MODEL.md`](THREAT_MODEL.md) — assets, actors, and what determinism
  buys.

## Quickstart

Drive a fixed deterministic policy through an environment from the command line:

```bash
# A discrete "always long" rollout on the bundled example data, seed 42.
cargo run -p wickra-gym -- \
  --spec examples/data/specs/momentum_discrete.json \
  --data examples/data/series/BTCUSDT.csv \
  --policy always-2 --seed 42

# Golden-format JSON ({reset, trajectory}) instead of the human-readable table.
cargo run -p wickra-gym -- \
  --spec examples/data/specs/momentum_discrete.json \
  --data examples/data/series/BTCUSDT.csv \
  --policy always-2 --seed 42 --format json
```

A spec names the observation features, the action space, the reward kind and the
episode bounds; the CLI loads the candle CSV, resets with the seed and prints
each step's reward and equity. The same spec, data and seed produce the same
trajectory here as in every language binding.

## Gymnasium usage

The Python binding ships a real `gymnasium.Env` subclass — the environment plugs
straight into the RL ecosystem (Stable-Baselines3, CleanRL, RLlib, …):

```python
import json
import gymnasium as gym
from wickra_gym import register

register()  # registers "WickraGym-v0"

spec = json.dumps({
    "dataset_ref": "demo",
    "symbol": "BTCUSDT",
    "observation": {"features": [
        {"kind": "indicator", "name": "Rsi", "params": [14]},
        {"kind": "price", "field": "close"},
    ]},
    "action_space": {"type": "discrete", "n": 3},
    "reward": "pnl",
    "episode": {"max_steps": 256, "warmup": 15},
    "seed": 42,
})
candles = [{"ts": 0, "open": 100, "high": 101, "low": 99, "close": 100, "volume": 1000}, ...]

env = gym.make("WickraGym-v0", spec_json=spec, candles=candles)
obs, info = env.reset(seed=42)
obs, reward, terminated, truncated, info = env.step(env.action_space.sample())
```

`observation_space` is a `Box` derived from the spec (finite bounds where the
core knows them, `±inf` otherwise); `action_space` is `Discrete(n)` or a 1-D
`Box`. Without Gymnasium installed, the lower-level `RawEnv` (the JSON command
surface) still works — see [`docs/GYMNASIUM.md`](docs/GYMNASIUM.md) and
[`examples/python/`](examples/python).

## Observations, actions and rewards

- **Observation** — an `ObsSpec` lists `Feature`s in declaration order, and the
  observation vector concatenates them in exactly that order: the features, then
  `4 × include_book_levels` orderbook columns, then `2` funding/OI columns if
  requested. A feature is an indicator (`{"kind":"indicator","name":"Rsi",
  "params":[14]}`, optionally a sub-output `"field"`), a price field, or a
  microstructure field. Before `warmup` the indicators are not ready, so the
  first `warmup` bars are skipped rather than emitted as `NaN`.
- **Action** — `Discrete{n}` maps `n` buckets onto a target position;
  `Continuous{low,high}` sets the target position directly (clamped). The
  position feeds the `wickra-backtest` fill/PnL model.
- **Reward** — `Pnl`, `Sharpe` or `LogReturn`, computed from the realised
  position path.

Full semantics (with the exact ordering and formulas) are in
[`docs/OBSERVATIONS.md`](docs/OBSERVATIONS.md) and
[`docs/ACTIONS_REWARDS.md`](docs/ACTIONS_REWARDS.md).

## Microstructure observations

Beyond price and indicators, the observation can include exchange
microstructure: set `include_book_levels: N` to append `bid_px, bid_sz, ask_px,
ask_sz` for each of `N` orderbook levels, and `include_funding_oi: true` to
append funding rate and open interest. When a dataset lacks a requested column
the value is a defined `0.0` (never `NaN`), so the observation length stays
fixed. See [`docs/MICROSTRUCTURE.md`](docs/MICROSTRUCTURE.md).

## Use in any language

The core drives a single `command_json` envelope (`load`, `reset`, `step`,
`spec`, `version`); every binding passes the same string through verbatim, so a
trajectory is byte-identical everywhere. The [`golden/`](golden) fixtures pin one
blessed response per case and the cross-language golden tests assert byte-for-byte
equality — the same observations, rewards and termination flags in all ten
bindings and between the parallel (rayon) and sequential (WASM) tensor
precompute. One runnable example per language lives under
[`examples/`](examples); per-binding quickstarts are in each
`bindings/<lang>/README.md`.

| Language | Binding | Package |
| -------- | ------- | ------- |
| Rust | `gym-core` (native) | crates.io |
| Python | PyO3 (native) + `gymnasium.Env` | PyPI |
| Node.js | napi (native) | npm |
| WASM | wasm-bindgen (native) | npm |
| C / C++ | C ABI | header + library |
| C# | C ABI (P/Invoke) | NuGet |
| Go | C ABI (cgo) | Go module |
| Java | C ABI (FFM/Panama) | Maven |
| R | C ABI (`.Call`) | R-universe |

## Project layout

```
crates/gym-core            the library: feature tensor + env + reward + command
crates/gym-cli             reference CLI, binary `wickra-gym`
crates/gym-bench           Criterion benchmarks
bindings/{c,python,node,wasm,go,csharp,java,r}   ten-language surface
golden/                    spec + policy + seed -> byte-exact trajectories
examples/                  runnable per-language rollouts + shared data
fuzz/                      cargo-fuzz targets (spec/command/tensor/step)
docs/                      observations, actions/rewards, gymnasium, microstructure
```

## Building from source

```bash
cargo build --workspace
cargo test --workspace --all-features
```

Each binding builds with its own toolchain; see `bindings/<lang>/README.md`. The
C-ABI consumers (C/C++, C#, Go, Java, R) need the C ABI library first:
`cargo build --release -p wickra-gym-c`.

## Requirements

Rust **1.86** (workspace) / **1.88** (Node binding). Per-binding toolchains:
Python 3.9+ (Gymnasium needs 3.10+), Node.js 22+, .NET 8, JDK 22+, Go 1.23+, R
release, and a C11/C++14 compiler with CMake for the C example.

## Benchmarks

Criterion benchmarks for `reset + N × step` throughput (at observation
dimensions 5/20/50) and `build_tensor` (at 1k/10k/100k bars, parallel vs
sequential) live in `crates/gym-bench`; numbers and methodology are in
[BENCHMARKS.md](BENCHMARKS.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Every change runs the full CI matrix (all
ten languages × three OSes) plus CodeQL, Scorecard, zizmor and the metadata
audit.

## Security

Report vulnerabilities per [SECURITY.md](SECURITY.md). The threat model is in
[THREAT_MODEL.md](THREAT_MODEL.md).

## License

Dual-licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.

## Disclaimer

`wickra-gym` is research and engineering tooling, not financial advice. A trained
agent's backtested performance says nothing about future returns; markets carry
risk and you are responsible for your own decisions. `wickra-gym` is free
software you run yourself: no hosted service, no data collection, no warranty.
