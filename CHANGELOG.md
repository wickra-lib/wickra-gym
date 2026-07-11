# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
