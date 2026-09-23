# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4] - 2026-09-23

A maintenance release: the trading environment and its bindings are unchanged.
It publishes the refreshed dependency tree and toolchain pins.

### Added

- **The Node binding reports which artifact it loaded.** The loader generated
  by `@napi-rs/cli` 3.10.4 exports `__napiBindingTarget` -- `'native'` for the
  native addon, otherwise the WASI flavor it resolved -- and follows a
  `NAPI_RS_NATIVE_LIBRARY_PATH` override to a WASI loader instead of
  misreporting it as native. Typed in `index.d.ts`.

### Changed

- **Built on wickra-core 1.0.5.** The lock takes the indicator core's latest
  release; the `1.0` requirement already admitted it.
- **Third-party dependencies refreshed.** `Cargo.lock` takes 107 crates to their
  newest semver-compatible versions, run across the family in one pass so every
  repository resolves the same day's versions. No manifest changed.
- **`@napi-rs/cli` 3.10.4** for the Node binding, the family's line.
- **uv 0.12.18** for the lockfile bootstrap in `scripts/update-lockfiles.sh`,
  with all four platform checksums moved together.
- **The README's static badges are served by the organization** rather than
  hot-linked from shields.io, so they no longer break when shields is down.

### Fixed

- **The Go install line names the published module.** The badge and `go get`
  in `bindings/go/README.md` pointed at the in-repo path
  `github.com/wickra-lib//bindings/go`, which the Go proxy never
  serves; they now name the mirror `github.com/wickra-lib/-go`, as the
  other bindings' READMEs do.

## [0.1.3] - 2026-09-18

### Fixed

- **The Java binding loads the library it ships.** The jar carries the native
  library under `native/<os>-<arch>/` -- the release pipeline stages every
  platform there -- but the loader only ever looked at `-Dnative.lib.dir` and
  the working directory, so a Maven Central consumer got a jar it could not
  load without pointing the JVM at a library it had to build itself. The loader
  now resolves in wickra's order: `-Dnative.lib.dir` when set, the bundled copy
  extracted to a temporary file, every `target/release` or `target/debug` up
  the tree from the working directory and the class's own location, then the
  bare name.

### Changed

- **Family pins follow the owners' releases:** wickra-backtest =0.1.6 -> =0.1.7, wickra-exchange =0.1.5 -> =0.1.6. No code of this repository changes; the engine it links is the one those releases ship.
- **Every README follows wickra's shape.** A cross-repo scan compared the
  heading skeleton of each README against wickra's and this repository's
  differed throughout. The root README opens as wickra's does (banner, badges,
  the one-liner, the live-demo and ecosystem lines, no separate H1), the
  License section carries wickra's wording and its `### Contribution` clause,
  and the shared sections run in wickra's order. Each binding README is
  `Install`, `Quick start`, `Benchmark`, `Documentation`, `Security`,
  `Disclaimer`, `License` with the product's own surface and protocol notes
  as subsections; the registry pages that render them now say how to report a
  vulnerability and under which licence the package ships.
  `examples/README.md` lists every language the way wickra's does, with the
  commands the CI examples job runs; the per-language example READMEs,
  `fuzz/README.md` and the `## Editing the docs` section of
  `docs/README.md` exist as they do in wickra.

### Changed

- **wickra-backtest 0.1.6 and wickra-exchange 0.1.5.** The pins move to the releases the family is on; the lock follows.
  A cross-repo scan lined the 24 wickra-lib repositories up, and the rest is
  what this one spelled differently: the fuzz job runs the family's pinned
  `nightly-2026-07-01` rather than a rolling nightly, and the example job's
  `dotnet-version` reads `8.0.x`.

### Changed

- **wickra-backtest 0.1.6 and wickra-exchange 0.1.5.** The pins move to the releases the family is on; the lock follows.
  A cross-repo scan lined the 24 wickra-lib repositories up, and the rest is
  what this one spelled differently: the fuzz job runs the family's pinned
  `nightly-2026-07-01` rather than a rolling nightly, and the example job's
  `dotnet-version` reads `8.0.x`.

### Changed

- **uv 0.12.15 for the lockfile script.** `scripts/update-lockfiles.sh`
  bootstraps 0.12.15 (was 0.12.13); the pin and all four release
  checksums move together, taken from the release's `.sha256` files.

## [0.1.2] - 2026-09-14

### Security

- **rustls 0.23.45.** RUSTSEC-2026-0285: rustls accepted TLS 1.3 handshake
  messages sent at the wrong encryption level. The lock moves to the
  patched release; nothing in the code changes.

### Fixed

- **The R package installs on macOS and Windows.** r-universe built the
  first release on every platform and failed on ten of thirteen: the package
  object linked the C ABI library but nothing bundled it, so macOS could not
  load `@rpath/libwickra_*.dylib`, and `Makevars.win` still expected the
  header and library through environment variables that r-universe never
  sets, so the Windows link found no symbols at all. The package is in the
  family's form now: `configure` / `configure.win` stage the library into
  `src/`, `install.libs.R` bundles it beside the package object (the DLL under
  its `_abi` name, the dylib and the `.so`), `Makevars.win` links the import
  library `configure.win` builds, a shipped `tests/smoke.R` runs inside the
  tarball, and `DESCRIPTION` states the R floor.

## [0.1.1] - 2026-09-14

### Fixed

- crates.io publish: the `reinforcement-learning` keyword is 22 characters and
  crates.io caps keywords at 20, so the v0.1.0 run published PyPI, npm, NuGet
  and Maven Central but no crate. The Cargo keyword is `reinforcement`; the
  npm and PyPI keyword lists are unchanged.
- The release workflow uploads the Java jar the provenance job attests: the
  Maven job never uploaded a `java-jar` artifact, so the attestation job would
  have had nothing to sign for Java.

## [0.1.0] - 2026-09-14

### Fixed

- **CI is green again.** The R streaming test matched `"observation":[102,103]`
  against an envelope that prints floats with their fraction
  (`[102.0,103.0]`, as the Java test already expects), so the R job failed on
  every platform. The napi glue (`bindings/node/index.js`) was stale against
  the locked CLI, so the in-sync check failed on every Node job; it is
  regenerated. The Examples job pointed `dotnet run` at a project directory
  that does not exist (`Rollout` is the project).
- **The Maven Central publish is idempotent, and waits as long as Central
  takes.** A sibling's first release deployed successfully and still went red:
  Central published after the plugin's default 30-minute wait had expired,
  and a rerun could only fail on the duplicate. The release workflow now skips
  a version already on Central, the plugin waits up to two hours
  (`waitMaxTime`), and the job has the budget for it.
- **The engine pins are exact** (`wickra-backtest = "=0.1.4"`, and the
  exchange client where it is used), as the released siblings pin them, so a
  newer patch on one side cannot leave two copies of the engine in one graph.
- zizmor's `self-repository` note is a documented policy (`.github/zizmor.yml`)
  rather than an open alert per workflow; uv 0.12.13 for the lockfile script.
- **The Python 3.9 CI row runs without pytest.** pytest 9.x requires 3.10,
  so that row could only pin 8.4.2, below the fix for GHSA-6w46-j5rx-g56g
  with no backport. The 3.9 lock carries maturin only, and the row runs the
  same test modules through `bindings/python/tests/run_without_pytest.py`
  (plain functions, plain asserts; the golden and smoke tests drop their
  pytest-only constructs); 3.10 and up run them under pytest as before. The
  gymnasium step installs from a hash-locked `ci-gymnasium.txt` instead of an
  unpinned `pip install gymnasium`.
- **The R golden parity test runs instead of skipping.** It needed jsonlite
  and skipped itself silently when the package was absent, which it always
  was on CI; both R jobs install jsonlite now (the example needs it too) and
  the test requires it. osv-scanner runs with `--no-resolve`, since the Java
  example's dependency on the unpublished org.wickra artefact cannot be
  resolved from Maven Central until the release exists.
- **The R package builds for WebAssembly on r-universe.** `configure`
  refused the wasm target outright, which would have left the `wasm-release`
  job red on every build. The r-universe wasm image ships cargo and
  emscripten, so `configure` now builds the C ABI staticlib from the release
  tag's source for `wasm32-unknown-emscripten` right there and links it into
  the package object, the way the released siblings do.
- **The exported R functions are documented.** `wkgym_new`, `wkgym_command`
  and `wkgym_version` carried roxygen comments but no generated `man/` pages,
  which `R CMD check` reports as a WARNING on every platform.
- **The core crate carried a name the release could not upload.** `gym-core`
  is outside the org's crates.io token scope, which creates new crates under
  the `wickra-` prefix only, and it is taken besides: `gym-core` 0.1.0 belongs
  to gym-rs, an unrelated project. `cargo publish` on it returns 403 at upload
  while `--dry-run` passes, and because the publish jobs run in parallel the
  release would have landed on PyPI, npm, NuGet, Maven Central and the Go
  mirror without ever reaching crates.io. The core is now `wickra-gym-core`,
  the shape of every released sibling. The directory keeps its name; only the
  package and the `wickra_gym_core` path moved. The same audit ran across the
  family (xray paid for this with its first tag).

- **`release.yml` copied the CLI's SBOM from a directory that does not
  exist.** It read `crates/wickra-gym/wickra-gym.cdx.json`; the crate lives in
  `crates/gym-cli/`. The `cp` sits after both uploads, so the job would have
  failed with the crates already published and no `.crate` or SBOM attached to
  the GitHub Release.

- **The napi bump split a crate in two and the build stopped.**
  `napi-derive-backend` 6.1.3 pulls `convert_case` 0.12 while `napi-derive`
  3.6.3 still uses 0.11, and two versions of a crate are two unrelated types --
  so `napi-derive` itself failed to compile, taking the node binding, the clippy
  job and every `cargo build --workspace` row down with it. Held at 6.1.2, which
  is what the screener runs.

- **`cargo-deny` was set to warn about duplicated crates, so it noted that split
  and moved on.** It is an error now. Only four duplicates exist across this
  workspace and each is a crate part-way through a major release reached through
  two ecosystems; they are skipped by name with the reason recorded, so a fifth
  still fails. Verified by putting 6.1.3 back and watching the check fail on
  `convert_case` before the compiler ever ran.

- **`actionlint` failed on five shell constructs the screener had already
  fixed.** `a && b || c` is not if-then-else -- when the publish succeeded but
  the echo failed, the fallback branch ran and reported "already published";
  `local pkg=$(basename …)` and `export PATH="$(cygpath …)"` hide the command's
  exit status behind `local`/`export`; and an asset count taken from `ls` breaks
  on a filename containing a newline. The runner-label config the linter needs
  for `windows-11-arm` was missing too.

- **A yanked crate was in the lockfile.** `wnaf` 0.14.0, reached through `p256`
  -> `wickra-exchange-core`, was yanked from crates.io; 0.14.1 is not.

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

- `wickra-gym-core`: the deterministic environment — `EnvSpec`/`ObsSpec`/`Feature`, the
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

[Unreleased]: https://github.com/wickra-lib/wickra-gym/compare/v0.1.4...HEAD
[0.1.4]: https://github.com/wickra-lib/wickra-gym/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/wickra-lib/wickra-gym/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/wickra-lib/wickra-gym/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wickra-lib/wickra-gym/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wickra-lib/wickra-gym/releases/tag/v0.1.0
