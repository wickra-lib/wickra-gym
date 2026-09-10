# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Indicators that read a side feed produced a column of constant zeros.**
  `IndicatorSet::update` hardcoded the reference series, derivatives tick,
  order book, trades and cross-section to absent, so an indicator needing any
  of them ticked and returned nothing every bar — and the tensor collapses a
  missing value to `0.0`, because it cannot carry `NaN`. The column was
  therefore indistinguishable from an indicator that is merely warming up, and
  an agent trained on it as if it meant something. The bar already carried the
  order-book levels and the funding print; they were simply never given to the
  indicators.

- **`episode.warmup` was a free parameter.** An episode could start before any
  indicator had produced a value, handing the agent a `0.0` that means "not
  computed yet". The floor now comes from the indicators the spec names. The
  doc example in `lib.rs` was itself below it — `warmup: 1` against `Sma(2)` —
  which is the first thing the check caught.

- **`CMAKE_CXX_STANDARD` asked for C++14** while the C++ hull requires C++17.
  Nothing compiled it, so nothing found out.

- **`chacha20` was pinned at the yanked 0.10.1**, which had been failing
  `cargo-deny` on `main`.

### Added

- **Side feeds derived from the bar.** `feeds::bar_feeds` builds the order book
  and the derivatives tick from the same `Candle` the tensor reads its raw
  microstructure columns from, so those indicator families are reachable.

- **A feed check that refuses rather than zeroes.** `ObsSpec::check_feeds`
  rejects a spec whose column needs a feed these bars cannot supply, naming the
  indicator, the feed and why — including the three families a single-symbol
  environment can never have.

- **Streaming-equals-batch tests in every binding.** Python, Node, Go, Java,
  C#, R, WASM and C each drive the rollout through the JSON boundary twice and
  compare, and check the first observation against the tensor the core built.

- **A golden test for the C binding**, which had none: all five committed
  rollouts, replayed from the blessed seed.

- The blueprint scaffold: `LICENSES/`, `docs/README.md`, `docs/FEEDS.md`, the
  five long-form issue templates, the CodeQL config, the actionlint and
  CodSpeed workflows, the five check scripts, a C++ hull, licence copies in
  every published crate and npm package, a WASM example, hash-locked pip
  requirements for both Python rows, and dependabot coverage for the fuzz
  workspace and the Go, Java and Node examples.

- CI gains `osv`, `links`, `binding-surface`, `semver`, `examples` and
  `python-wheel-container-smoke`; the release pipeline gains the `gate` and
  `guard` jobs, provenance over the nupkg, jar and C ABI archives, a Maven
  artifact on the release page, and a Go mirror that builds before it
  publishes.

### Changed

- **The family pins move to the published releases.** `wickra-backtest` and
  `wickra-exchange` come from crates.io rather than git revs, and
  `wickra-core` / `wickra-data` rise from 0.9 to 1.0, so the tree carries one
  set of indicator types rather than two that share none.

- The R golden block moves to its own `tests/golden.R`, excluded from the built
  tarball: a test that ships must not reason about the repository above it,
  because in a tarball there is none.

### Added

- `gym-core`: the deterministic environment — `EnvSpec`/`ObsSpec`/`Feature`, the
  O(1)-per-bar `FeatureTensor` precompute (parallel via rayon, sequential on
  WASM, byte-identical either way), the O(1) `step()`, the `Pnl`/`Sharpe`/
  `LogReturn` rewards over the `wickra-backtest` fill/PnL model, and the
  `command_json` boundary (`load` / `reset` / `step` / `spec` / `version`).
- Observations over the 514 `wickra-core` indicators plus price and optional
  order-book / funding / open-interest microstructure, in a fixed canonical
  order.
- Reference CLI (`wickra-gym`): drive a fixed deterministic policy through an
  environment, text or golden-format JSON output.
- Ten-language bindings — native Rust, Python (a real `gymnasium.Env` subclass),
  Node.js and WASM, plus a C ABI hub for C/C++, C#, Go, Java and R — each
  returning the core's canonical JSON verbatim, so a trajectory is byte-identical
  across all of them.
- Byte-exact golden trajectory corpus, conformance / determinism / property
  tests, fuzz targets, criterion benchmarks, one runnable example per language,
  and the full cross-OS CI matrix (CodeQL, Scorecard, zizmor, link and metadata
  checks).
- Workspace scaffolding: dual `MIT OR Apache-2.0` license, supply-chain / lint
  config (`deny.toml`, `clippy.toml`, `lychee.toml`, `osv-scanner.toml`,
  `repo-metadata.toml`), and project governance and community docs.

[Unreleased]: https://github.com/wickra-lib/wickra-gym/commits/main
