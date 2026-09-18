<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Gym — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for fast, deterministic RL rollouts" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/ci.svg)](https://github.com/wickra-lib/wickra-gym/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-gym)
[![Maven Central](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/maven.svg)](https://central.sonatype.com/artifact/org.wickra/wickra-gym)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/license.svg)](https://github.com/wickra-lib/wickra-gym#license)

# Wickra Gym — Java

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for Java. `org.wickra:wickra-gym` — prebuilt native library inside the jar, no JNI, no system dependencies.**

Java bindings for [wickra-gym](https://github.com/wickra-lib/wickra-gym) — a
deterministic, Gymnasium-compatible backtest environment — over the C ABI using
the Foreign Function & Memory API (FFM / Project Panama). A rollout is
byte-identical to every other language binding.

## Requirements

- Java 22+ (FFM). Run with `--enable-native-access=ALL-UNNAMED`.
- The native `wickra_gym` library on the load path. The Maven build points to the
  workspace `target/debug` via the `native.lib.dir` system property; a release
  jar bundles the library under `resources/native/<os>-<arch>/`.

## Install

Maven:

```xml
<dependency>
  <groupId>org.wickra</groupId>
  <artifactId>wickra-gym</artifactId>
  <version>0.1.3</version>
</dependency>
```

Gradle:

```kotlin
implementation("org.wickra:wickra-gym:0.1.3")
```

The native library ships prebuilt per platform inside the jar and is
extracted automatically on first use. There is nothing to compile.

### Building from this repository (contributors)

```sh
cargo build -p wickra-gym-c          # produces target/debug/wickra_gym.<ext>
mvn -q test                          # -Dnative.lib.dir=... to override the path
```

## Quick start

```java
import org.wickra.gym.Env;

String spec = """
    {"dataset_ref":"demo","symbol":"BTCUSDT",
     "observation":{"features":[{"kind":"price","field":"close"}]},
     "action_space":{"type":"discrete","n":3},
     "reward":"pnl","episode":{"max_steps":256,"warmup":0}}
    """;

try (Env env = new Env(spec)) {
    // Build a candles JSON array and load it.
    env.command("{\"cmd\":\"load\",\"candles\":[/* ... */]}");
    String reset = env.command("{\"cmd\":\"reset\",\"seed\":0}");
    String step = env.command("{\"cmd\":\"step\",\"action\":2}");
    System.out.println(step);
}
```

Commands: `load`, `reset`, `step`, `spec`, `version`. Domain errors come back as
`{"ok":false,"error":...}`; a bad spec throws `IllegalArgumentException`.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the Java Foreign Function & Memory API over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-gym/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-gym>
- **Docs** (guides, spec reference, cookbook): <https://gym.wickra.org>
- **Runnable example:** [`examples/java/`](https://github.com/wickra-lib/wickra-gym/tree/main/examples/java)

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
