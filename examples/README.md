# Wickra Gym examples

Runnable rollouts in every language wickra-gym binds to. Each one loads the
`momentum_discrete` spec and its candle dataset, resets with seed `42`, and
drives a fixed "always long" policy (discrete action `2`) until the episode
truncates.

## What every example prints

Because every binding forwards the same JSON commands to the same Rust core, the
**numbers are identical** across all of them; the printing is each language's
own. Python, Node.js and Rust format the observation and each step:

```
wickra-gym 0.1.3
reset observation: [59.75522252, 102.70537863]
step 0: reward +1.227974  equity +1.227974  terminated=false truncated=false
step 1: reward +1.675656  equity +2.903630  terminated=false truncated=false
step 2: reward +1.993914  equity +4.897544  terminated=false truncated=false
step 3: reward +2.147713  equity +7.045256  terminated=false truncated=false
step 4: reward +2.120121  equity +9.165378  terminated=false truncated=true
```

## Rust — `examples/rust/`

As the CI examples job runs it, from the repository root:

```bash
cargo run -q --manifest-path examples/rust/Cargo.toml
```

| Example | What it does |
| --- | --- |
| `src/main.rs` | A runnable Rust example: read a candle CSV, load a spec, then drive a fixed long policy through the environment and print each step's reward. |

## C / C++ — `examples/c/`

Build the library first (`cargo build -p wickra-gym-c --release`), then build and run
the examples via CMake, as the CI C ABI job does:

```bash
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

| Example | What it does |
| --- | --- |
| `rollout.c` | A minimal C example: load a spec and its candle dataset over the wickra-gym C |
| `rollout.cpp` | A minimal C++ example: load a spec and its candle dataset, then drive a fixed long policy and print each step. |

## C# — `examples/csharp/`

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Rollout
```

| Example | What it does |
| --- | --- |
| `Rollout/Program.cs` | A runnable C# example: load the momentum_discrete spec and its candle dataset over the wickra-gym C ABI binding, then drive a fixed long policy and print each step. |

## Go — `examples/go/`

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

| Example | What it does |
| --- | --- |
| `rollout.go` | A runnable Go example: load the momentum_discrete spec and its candle dataset over the wickra-gym C ABI binding, then drive a fixed long policy and print each step. |

## R — `examples/r/`

As the CI examples job runs it, from the repository root:

```bash
R CMD INSTALL bindings/r
Rscript examples/r/rollout.R
```

| Example | What it does |
| --- | --- |
| `rollout.R` | A runnable R example: load the momentum_discrete spec and its candle dataset over the wickra-gym C ABI binding, then drive a fixed long policy and print each step. |

## Java — `examples/java/`

As the CI examples job runs it, from the repository root:

```bash
mvn -f bindings/java/pom.xml -q install -DskipTests
mvn -f examples/java/pom.xml -q compile exec:exec  -Dnative.lib.dir="$PWD/target/release"
```

| Example | What it does |
| --- | --- |
| `src/main/java/org/wickra/gym/examples/Rollout.java` | A runnable Java example: load the momentum_discrete spec and its candle dataset over the wickra-gym C ABI binding, then drive a fixed long policy and print each step. |

## Python — `examples/python/`

As the CI examples job runs it, from the repository root:

```bash
python -m pip install --require-hashes -r .github/requirements/ci-dev-py3.txt
( cd bindings/python && maturin build --release --out dist )
python -m pip install --no-index --find-links bindings/python/dist wickra-gym
python examples/python/rollout.py
```

| Example | What it does |
| --- | --- |
| `gymnasium_ppo.py` | The main consumer: a real ``gymnasium.Env`` driven by a random agent. |
| `rollout.py` | Raw rollout over the native command surface (no Gymnasium required). |

## Node.js — `examples/node/`

As the CI examples job runs it, from the repository root:

```bash
( cd bindings/node && npm install --no-audit --no-fund && npx napi build --platform --release )
( cd examples/node && npm install --no-audit --no-fund )
node examples/node/rollout.js
```

| Example | What it does |
| --- | --- |
| `rollout.js` | A minimal Node.js rollout over the wickra-gym command surface. |

## WASM — `examples/wasm/`

Build the WASM package, serve the repository root, and open the page in a browser;
the module script inside it is what runs (CI parses it with `node --check`):

```bash
wasm-pack build bindings/wasm --target web
python -m http.server 8000     # then open http://localhost:8000/examples/wasm/
```

| Example | What it does |
| --- | --- |
| `rollout.html` | A runnable example against this binding. |

## Example datasets

The examples read from [`examples/data/`](data/): `candles.json`, `candles_micro.json`. The
cross-language golden fixtures, which every binding is checked against byte for
byte, live in [`../golden/`](../golden).

## Data (`data/`)

| Path | What |
|------|------|
| `data/series/BTCUSDT.csv` | The candle series, human-readable (`timestamp,open,high,low,close,volume`). The Rust example reads this via the same `wickra-data` CSV reader the CLI uses. |
| `data/candles.json` | The same 30 bars serialized as the JSON array the `load` command takes. Every binding that goes through the command surface feeds this. |
| `data/candles_micro.json` | A microstructure-bearing dataset (order book / funding / OI columns) for the `micro_book` spec. |
| `data/specs/*.json` | The five example specs: `momentum_discrete`, `momentum_continuous`, `micro_book`, `sharpe`, `logreturn`. |

The universe is deterministic — `close(i) = 100 + 5·sin(i/3) + 0.5·i` over 30
bars — so the whole rollout is reproducible with no external data.

## Running

Each example is self-contained. Build the core artifact it needs first.

| Language | Build | Run |
|----------|-------|-----|
| Rust | — | `cargo run -p wickra-gym-example` |
| C / C++ | `cargo build --release -p wickra-gym-c` | `cmake -S c -B c/build && cmake --build c/build && ctest --test-dir c/build --output-on-failure` |
| Python (raw) | `pip install wickra-gym` | `python python/rollout.py` |
| Python (Gymnasium) | `pip install wickra-gym[gym]` | `python python/gymnasium_ppo.py` |
| Node.js | `(cd node && npm install)` | `node node/rollout.js` |
| Go | `cargo build --release -p wickra-gym-c` then stage the library under `../bindings/go/lib/<goos>_<goarch>/` | `(cd go && go run .)` |
| C# | `cargo build --release -p wickra-gym-c` | `dotnet run --project csharp/Rollout` |
| Java | `cargo build --release -p wickra-gym-c` then `(cd ../bindings/java && mvn -q install -DskipTests)` | `mvn -q -f java compile exec:exec` |
| R | `R CMD INSTALL ../bindings/r` | `Rscript r/rollout.R` |
| WASM | `wasm-pack build bindings/wasm --target web` | serve the repository root, then open `examples/wasm/rollout.html` |

## Notes

- **Rust** uses `wickra-gym-core` directly (the in-process API); every other example
  goes through the C ABI or a native binding via the JSON command surface.
- **`python/gymnasium_ppo.py`** builds a real `gymnasium.Env`
  (`WickraGym-v0`), runs one episode with a random policy, then trains a tiny
  PPO if Stable-Baselines3 is installed (skipped otherwise).
- The C example links the DLL directly under MinGW and via the import library
  under MSVC; the CMake `TIMEOUT` guards against a missing runtime dependency
  hanging the test.
