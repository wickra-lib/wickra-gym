# wickra-gym (C#)

.NET bindings for [`wickra-gym`](https://github.com/wickra-lib/wickra-gym) over
the C ABI hub, via source-generated P/Invoke. Build an `Env` from a spec JSON,
drive it with command JSON and read back observations and rewards — the same
protocol the CLI and every other binding speak, returning the same bytes.

```csharp
using Wickra.Gym;

const string spec = """
{"dataset_ref":"demo","symbol":"BTCUSDT",
 "observation":{"features":[{"kind":"indicator","name":"Rsi","params":[14]},
                            {"kind":"price","field":"close"}]},
 "action_space":{"type":"discrete","n":3},
 "reward":"pnl","episode":{"max_steps":256,"warmup":14}}
""";

using var env = new Env(spec);
env.Command("""{"cmd":"load","candles":[ … ]}""");
string reset = env.Command("""{"cmd":"reset","seed":7}""");
string step = env.Command("""{"cmd":"step","action":2.0}""");
```

The dataset is precomputed once into a fixed feature tensor, so every `step` is a
pure array index and a `(seed, policy)` pair fully determines the trajectory — in
this binding exactly as in the other nine.

`episode.warmup` must be at least the longest lookback the observation's
indicators declare: below it a column is `0.0` because nothing has been produced
yet, which an agent cannot tell from a market reading of zero. A spec below the
floor is refused rather than run. An indicator that reads an order book or a
funding print needs bars that carry them, and is likewise refused rather than
answered with a column of constant zeros.

Requires .NET 8+. The native library (`wickra_gym`) must be resolvable on the
loader path — `PATH` on Windows, `LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH`
on macOS — or beside the assembly, where the bundled resolver finds it.

Licensed under either of [MIT](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-APACHE) at your option.
