# Gymnasium integration

The Python binding ships a real [`gymnasium.Env`](https://gymnasium.farama.org)
subclass, `WickraGymEnv`, so a wickra-gym environment plugs straight into the RL
ecosystem (Stable-Baselines3, CleanRL, RLlib, …). It is a thin, deterministic
wrapper over the same `command_json` surface every other binding uses — the
numpy conversion happens on top, so trajectories stay byte-identical to the other
languages.

## `RawEnv` vs `WickraGymEnv`

- **`RawEnv`** — the low-level JSON command surface (`RawEnv(spec_json)` then
  `command(cmd_json) -> str`). Works with **no** Gymnasium installed.
- **`WickraGymEnv`** — the `gymnasium.Env` subclass. Only defined when
  `gymnasium` is importable; `pip install wickra-gym[gym]` pulls it in.

## Register and make

```python
import json
import gymnasium as gym
from wickra_gym import register

register()  # registers the id "WickraGym-v0"

env = gym.make("WickraGym-v0", spec_json=spec_json, candles=candles)
obs, info = env.reset(seed=42)
obs, reward, terminated, truncated, info = env.step(action)
```

`register()` binds `WickraGym-v0` to `wickra_gym:WickraGymEnv`. The env takes the
`spec_json` string and the `candles` list; it loads them once at construction.

## How the spaces are derived

At construction the env asks the core for its `spec` (the derived space
description) and builds:

- **`observation_space`** — a `Box`. The core reports finite `observation_low` /
  `observation_high` bounds where it knows them and a large finite sentinel
  (`1e30`) where a bound is unbounded (JSON cannot carry `inf`); the wrapper maps
  that sentinel back to `np.inf`, so the `Box` has `±inf` where appropriate.
- **`action_space`** — `Discrete(n)` for a discrete spec, or a 1-D
  `Box(low, high, shape=(1,))` for a continuous spec.

## The step/reset contract

- `reset(seed=None)` returns `(observation: np.ndarray, info: dict)`. Passing a
  `seed` reseeds the episode RNG (equivalent to the spec's `seed`).
- `step(action)` returns the Gymnasium 5-tuple
  `(observation, reward, terminated, truncated, info)`. A discrete action is
  taken as an `int`; a continuous action is read as a scalar `float`.
- `truncated` is the time-limit signal (episode reached `max_steps` or the data
  ran out); `terminated` is a genuine terminal state.

The `test_gymnasium.py` env-contract test asserts these shapes and dtypes, and
`examples/python/gymnasium_ppo.py` runs a full random-policy episode (and a tiny
PPO if Stable-Baselines3 is installed).
