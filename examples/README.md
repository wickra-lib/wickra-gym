# Examples

Runnable rollouts in every language wickra-gym binds to. Each one loads the
`momentum_discrete` spec and its candle dataset, resets with seed `42`, and
drives a fixed "always long" policy (discrete action `2`) until the episode
truncates.

Because every binding forwards the same JSON commands to the same Rust core, the
trajectory is **byte-identical** across all of them:

```
reset observation: [59.75522252, 102.70537863]
step 0: reward +1.227974  equity +1.227974  terminated=false truncated=false
step 1: reward +1.675656  equity +2.903630  terminated=false truncated=false
step 2: reward +1.993914  equity +4.897544  terminated=false truncated=false
step 3: reward +2.147713  equity +7.045256  terminated=false truncated=false
step 4: reward +2.120121  equity +9.165378  terminated=false truncated=true
```

These are the same numbers the golden fixtures pin (see `../golden/`), so the
examples double as a live cross-language conformance check.

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

## Notes

- **Rust** uses `gym-core` directly (the in-process API); every other example
  goes through the C ABI or a native binding via the JSON command surface.
- **`python/gymnasium_ppo.py`** builds a real `gymnasium.Env`
  (`WickraGym-v0`), runs one episode with a random policy, then trains a tiny
  PPO if Stable-Baselines3 is installed (skipped otherwise).
- The C example links the DLL directly under MinGW and via the import library
  under MSVC; the CMake `TIMEOUT` guards against a missing runtime dependency
  hanging the test.
