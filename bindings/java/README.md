# Wickra Gym — Java

Java bindings for [wickra-gym](https://github.com/wickra-lib/wickra-gym) — a
deterministic, Gymnasium-compatible backtest environment — over the C ABI using
the Foreign Function & Memory API (FFM / Project Panama). A rollout is
byte-identical to every other language binding.

## Requirements

- Java 22+ (FFM). Run with `--enable-native-access=ALL-UNNAMED`.
- The native `wickra_gym` library on the load path. The Maven build points to the
  workspace `target/debug` via the `native.lib.dir` system property; a release
  jar bundles the library under `resources/native/<os>-<arch>/`.

## Usage

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

## Build & test

```sh
cargo build -p wickra-gym-c          # produces target/debug/wickra_gym.<ext>
mvn -q test                          # -Dnative.lib.dir=... to override the path
```

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-APACHE),
at your option.
