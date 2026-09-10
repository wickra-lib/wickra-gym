# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Three steps of the `examples` job ran a file that is not here.** Python,
  Node.js and R each invoked `examples/<lang>/scan.*` -- the screener's file
  name, left over from the port -- so they died on a missing file before
  reaching any assertion.

- **Every language step asserted `"symbol":"BBB"`**, a line from the screener's
  scan report that no example here prints. Each step now matches a string its
  own example emits, read off the format string rather than guessed: the
  assertions were checked against a real run of each example.

- **The C++ example now goes through the C++ hull.** It called the C functions
  directly and rebuilt the two-call length protocol by hand -- the very thing
  `wickra_gym.hpp` exists to remove -- which left the shipped C++ surface built
  by nothing. Verified by running both: the C and C++ examples print
  byte-identical output.

- **Every C++ hull used the include guard `WICKRA_SCREENER_HPP`.** The C headers
  beside them are guarded correctly; only the `.hpp` files shared one name, so
  including two of the family's headers in the same translation unit dropped the
  second silently. Proven by compiling a file that includes two of them and
  names a class from each: `'Env' is not a member of 'wickra'`. All seven now
  compile standalone and together.

- **The C++ hull did not compile at all.** It declared `WickraGym* handle_`
  where the C header names the opaque type `WickraGymEnv`. Nothing caught it,
  because this repository's only C++ example included the C header rather than
  the hull beside it, so nothing built the shipped C++ surface — and the
  shared include guard hid the error behind a skipped `#include`. (The
  screener builds its hull from both of its C++ examples; the feature store
  from one. This repository built it from none.)

- **The hull's usage example could not run.** It showed a spec shaped
  `{"universe":[...]}` and `{"cmd":"scan"}`, the screener's, which this core
  rejects twice over. It now shows this repository's own spec fields and one of
  its own commands.

- **The issue and pull-request templates asked for a `ScanSpec`**, a type this
  repository does not have, so a contributor was asked to attach something that
  does not exist. `GOVERNANCE.md`, `SUPPORT.md` and `CONTRIBUTING.md` carried
  the same substitution, along with the screener's "condition schema" for a core
  that has no conditions.

- **The R `configure` scripts still defined `wkscreen_download`**, the last
  trace of the screener's prefix — the CI-visible half of which already had to
  be fixed once.

- **The CLI was published without ever being run in CI.** `release.yml` builds
  and attaches `cli-binaries`, and five sibling repositories smoke-test their
  binary on every push; this one did not, so a CLI that failed to start would
  have been found by whoever downloaded it. The job runs a committed spec over
  committed data in both output formats, asserting on real output rather than on
  the exit code alone.

- **The CI Java example step compiled a file that is not there.** The `examples`
  job was ported from the screener, whose Java example is a single
  `examples/java/Scan.java` built with `javac`. This repository ships a Maven
  project instead, so the step compiled a missing file and then asserted on
  output the example never prints. It now builds the binding into the local
  repository and runs the example through `mvn exec:exec`, the way the example's
  own javadoc documents -- verified by running it.

- **Dependabot watched directories that do not exist**, so it reported nothing
  and the silence read as calm. `nuget` pointed at `WickraVerify.Tests`, a
  project name from another repository, and `pip` did not cover
  `/.github/requirements`.

- **The examples README claimed byte-identical output that three of them do not
  produce.** C, C++ and WASM printed a `wickra-gym <version>` banner; Python,
  Node.js, Rust and Java did not; and C, C++, Go, Java and R echo the raw command
  JSON where the others format it. All ten print the banner now, and the README
  says what is actually identical -- the numbers -- rather than the text.

- **`release.yml` overwrote the binding READMEs before packing.** Three steps
  copied the root README over `bindings/python/README.md` (wheel and sdist) and
  `bindings/node/README.md`. They date from when the bindings had no README of
  their own; they do now, one per registry, and `check_readme_links.py` exists to
  keep their links absolute because a relative link is dead on PyPI and npm. The
  copy threw that away and shipped the root README, whose links are relative by
  design. The remaining relative links in the C, C#, Go and WASM READMEs are
  absolute now.

- **The Python wheel would have shipped without its licence texts.**
  `bindings/python/` carried neither `LICENSE-MIT` nor `LICENSE-APACHE`, so
  maturin had nothing to include, while every crate and the release archive
  carry both.

- **`SECURITY.md` named a support policy for releases that do not exist yet.**
  It promised fixes for "the latest `0.x` release line" where there is no
  released line; it now says plainly that nothing is published and names `0.1.0`
  as the first version that will be.

- **The headline indicator count was the catalogue figure, not the reachable
  one.** 514 is what the `wickra-core` catalogue ships; what a spec can actually
  name is what the shared registry in `wickra-backtest-core` resolves, and that
  `build` match has 497 arms. The two are different sets — bar builders emit
  bars rather than a value per bar, and a handful of indicators the registry
  does not carry yet — so the observation was advertised over 514 indicators a
  spec cannot name.

- **The Ecosystem section repeated two claims their own repositories had already
  corrected**: DARWIN at "millions of backtests per second" across "the
  514-indicator space", where its benchmark says hundreds of thousands over the
  registry, and GENOME as "a 514-dim live vector", where the dimension is
  whatever the spec's feature list names.

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
- Observations over the 497 registry indicators plus price and optional
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
