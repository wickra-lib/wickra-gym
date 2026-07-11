# Wickra Gym — Python

A deterministic, [Gymnasium](https://gymnasium.farama.org/)-compatible backtest
environment. The whole candle dataset is precomputed once into a fixed feature
tensor, so each `step()` is a constant-time array index — and the same spec,
data, seed and actions produce a byte-identical trajectory across every language
binding.

## Install

```sh
pip install wickra-gym            # RawEnv (no Gymnasium dependency)
pip install "wickra-gym[gym]"     # + the gymnasium.Env subclass
```

## Use as a Gymnasium environment

```python
import numpy as np
from wickra_gym import WickraGymEnv

spec = """{
  "dataset_ref": "demo", "symbol": "BTCUSDT",
  "observation": {"features": [
    {"kind": "price", "field": "close"},
    {"kind": "indicator", "name": "Rsi", "params": [14]}
  ]},
  "action_space": {"type": "discrete", "n": 3},
  "reward": "pnl",
  "episode": {"max_steps": 256, "warmup": 14}
}"""

candles = [
    {"ts": i, "open": 100 + i, "high": 100 + i, "low": 100 + i, "close": 100 + i}
    for i in range(300)
]

env = WickraGymEnv(spec, candles)
obs, info = env.reset(seed=0)
done = False
while not done:
    action = env.action_space.sample()
    obs, reward, terminated, truncated, info = env.step(action)
    done = terminated or truncated
```

The observation and action spaces are derived from the spec: unbounded
observation columns use `±np.inf`; a discrete action space becomes
`spaces.Discrete(n)` and a continuous one becomes a 1-D `spaces.Box`.

Register it under a Gymnasium id:

```python
from wickra_gym import register
register()  # WickraGym-v0
```

## The raw command surface

`RawEnv` is the thin, dependency-free wrapper over the native command JSON
surface — the same boundary every language binding forwards verbatim:

```python
import json
from wickra_gym import RawEnv

env = RawEnv(spec)
env.command(json.dumps({"cmd": "load", "candles": candles}))
reset = json.loads(env.command(json.dumps({"cmd": "reset", "seed": 0})))
step = json.loads(env.command(json.dumps({"cmd": "step", "action": 2})))
```

Commands: `load`, `reset`, `step`, `spec`, `version`. Domain errors come back as
`{"ok": false, "error": ...}`; a bad spec raises `ValueError` at construction.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-gym/blob/main/LICENSE-APACHE),
at your option.
