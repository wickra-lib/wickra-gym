<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Gym — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for fast, deterministic RL rollouts" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/ci.svg)](https://github.com/wickra-lib/wickra-gym/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-gym)
[![PyPI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/pypi.svg)](https://pypi.org/project/wickra-gym/)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-gym/license.svg)](https://github.com/wickra-lib/wickra-gym#license)

# Wickra Gym — Python

---

**Part of the [Wickra ecosystem](https://github.com/wickra-lib) — for Python. `pip install wickra-gym` — prebuilt wheels for Linux, macOS and Windows, nothing to compile.**

A deterministic, [Gymnasium](https://gymnasium.farama.org/)-compatible backtest
environment. The whole candle dataset is precomputed once into a fixed feature
tensor, so each `step()` is a constant-time array index — and the same spec,
data, seed and actions produce a byte-identical trajectory across every language
binding.

## Install

```bash
pip install wickra-gym
```

Pre-built wheels ship for Linux, macOS and Windows — there is nothing to
compile and no C library to track down.

## Quick start

[`examples/python/rollout.py`](https://github.com/wickra-lib/wickra-gym/blob/main/examples/python/rollout.py) is the runnable example the CI smoke job executes; in full:

```python
"""Raw rollout over the native command surface (no Gymnasium required).

    python examples/python/rollout.py

Reads the momentum_discrete spec and its candle dataset, then drives a fixed
long policy through the environment via ``RawEnv.command`` — the same JSON-in /
JSON-out boundary every language binding forwards verbatim, so this trajectory
is byte-identical to the C, Node, Go, C#, Java and R examples on the same seed.
"""

import json
from pathlib import Path

from wickra_gym import RawEnv, __version__

DATA = Path(__file__).resolve().parent.parent / "data"

def main() -> None:
    spec = (DATA / "specs" / "momentum_discrete.json").read_text()
    candles = json.loads((DATA / "candles.json").read_text())

    env = RawEnv(spec)
    env.command(json.dumps({"cmd": "load", "candles": candles}))

    reset = json.loads(env.command(json.dumps({"cmd": "reset", "seed": 42})))
    print(f"wickra-gym {__version__}")
    print("reset observation:", reset["observation"])

    equity = 0.0
    step = 0
    while True:
        result = json.loads(env.command(json.dumps({"cmd": "step", "action": 2})))
        equity += result["reward"]
        print(
            f"step {step}: reward {result['reward']:+.6f}  equity {equity:+.6f}  "
            f"terminated={result['terminated']} truncated={result['truncated']}"
        )
        if result["terminated"] or result["truncated"]:
            break
        step += 1

if __name__ == "__main__":
    main()
```

### Use as a Gymnasium environment

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

### The raw command surface

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

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of PyO3, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-gym/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-gym>
- **Docs** (guides, spec reference, cookbook): <https://gym.wickra.org>
- **Runnable example:** [`examples/python/`](https://github.com/wickra-lib/wickra-gym/tree/main/examples/python)

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
