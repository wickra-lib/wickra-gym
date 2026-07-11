# Wickra Gym — C#

A deterministic, Gymnasium-compatible backtest environment for .NET, over the
Wickra C ABI. A rollout is byte-identical to every other language binding.

## Install

```sh
dotnet add package Wickra.Gym
```

The native library for your runtime is packed under `runtimes/<rid>/native/` and
resolved automatically (a sentinel-export check rejects a wrong or stale
library).

## Usage

```csharp
using System.Text.Json;
using Wickra.Gym;

const string spec = """
    {"dataset_ref":"demo","symbol":"BTCUSDT",
     "observation":{"features":[{"kind":"price","field":"close"}]},
     "action_space":{"type":"discrete","n":3},
     "reward":"pnl","episode":{"max_steps":256,"warmup":0}}
    """;

using var env = new Env(spec);

var candles = Enumerable.Range(0, 300).Select(i => new
{
    ts = i, open = 100.0 + i, high = 100.0 + i, low = 100.0 + i, close = 100.0 + i,
});
env.Command(JsonSerializer.Serialize(new { cmd = "load", candles }));

string reset = env.Command("""{"cmd":"reset","seed":0}""");
string step = env.Command("""{"cmd":"step","action":2}""");
Console.WriteLine(step);
```

Commands: `load`, `reset`, `step`, `spec`, `version`. Domain errors come back as
`{"ok":false,"error":...}`; a bad spec throws `ArgumentException` at construction.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-APACHE),
at your option.
