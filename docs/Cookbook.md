# Cookbook

Task-oriented recipes. Each one is deterministic: the same spec, data and seed
reproduce the same result here and in every language binding.

## Train an agent with Stable-Baselines3

```python
import json
import gymnasium as gym
from stable_baselines3 import PPO
from wickra_gym import register

register()
env = gym.make("WickraGym-v0", spec_json=spec_json, candles=candles)
model = PPO("MlpPolicy", env, verbose=1)
model.learn(total_timesteps=100_000)
```

The `observation_space` (a `Box`) and `action_space` (`Discrete` or a 1-D `Box`)
are derived from the spec, so SB3 configures its policy automatically. See
[GYMNASIUM.md](GYMNASIUM.md).

## Observe a custom indicator

Add any of the 514 registry indicators to the observation by name and params:

```jsonc
"observation": { "features": [
  {"kind": "indicator", "name": "Macd", "params": [12, 26, 9], "field": "hist"},
  {"kind": "indicator", "name": "Rsi",  "params": [14]},
  {"kind": "price", "field": "close"}
]}
```

`field` selects a sub-output on a multi-output indicator. The observation vector
follows the declaration order (see [OBSERVATIONS.md](OBSERVATIONS.md)).

## Add microstructure to the observation

```jsonc
"observation": {
  "features": [{"kind": "price", "field": "close"}],
  "include_book_levels": 2,
  "include_funding_oi": true
}
```

This appends `bid_px, bid_sz, ask_px, ask_sz` for two levels, then `funding, oi`.
Missing columns are `0.0`, never `NaN` (see [MICROSTRUCTURE.md](MICROSTRUCTURE.md)).

## Roll out from the command line

```bash
cargo run -p wickra-gym -- \
  --spec examples/data/specs/momentum_discrete.json \
  --data examples/data/series/BTCUSDT.csv \
  --policy always-2 --seed 42 --format json
```

`--format json` prints the golden-format `{reset, trajectory}` — the same bytes
the golden fixtures pin.

## Confirm a trajectory reproduces across languages

Run the same spec, data and seed through two bindings and diff the JSON. Because
every binding forwards the core's `command_json` response verbatim, the two
outputs are byte-identical. The `examples/` directory has one runnable rollout
per language; the cross-language golden tests assert this in CI.

## Gate determinism in CI

The CLI exits non-zero on error, so a rollout can guard a pipeline:

```bash
cargo run -q -p wickra-gym -- --spec spec.json --data data.csv --seed 42 \
  --format json > got.json
diff -u expected.json got.json   # fails the job on any drift
```
