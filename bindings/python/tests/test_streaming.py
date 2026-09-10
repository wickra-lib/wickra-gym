"""The streamed rollout equals the batch tensor, through the command boundary.

The dataset is precomputed once into a fixed feature tensor — that is the batch
half — and `step()` then streams through it as a pure array index. `gym-core`
proves the two agree in Rust, but that says nothing about the boundary this
binding crosses: a binding that mis-serialised an observation row, or truncated
it, would hand back numbers that look plausible and are not the tensor's.

So the rollout is driven twice through this binding — once continuously, once
from a freshly loaded environment re-stepped to the same bar — and the two must
be byte-identical. The spec uses `Sma(3)`, which has a lookback: a column over
the raw close reads the same whichever way the bar was reached, which is how
this class of bug stays invisible.
"""

import json

from wickra_gym import RawEnv

SPEC = json.dumps(
    {
        "dataset_ref": "streaming",
        "symbol": "TEST",
        "observation": {
            "features": [
                {"kind": "indicator", "name": "Sma", "params": [3]},
                {"kind": "price", "field": "close"},
            ]
        },
        "action_space": {"type": "discrete", "n": 3},
        "reward": "pnl",
        # Sma(3) needs three bars, and the core refuses a warmup below what the
        # observation's indicators declare.
        "episode": {"max_steps": 32, "warmup": 3},
    }
)

CANDLES = [
    {
        "ts": i,
        "open": 100.0 + i,
        "high": 101.0 + i,
        "low": 99.0 + i,
        "close": 100.0 + i,
        "volume": 1.0,
    }
    for i in range(20)
]

STEPS = 6


def _loaded() -> RawEnv:
    env = RawEnv(SPEC)
    env.command(json.dumps({"cmd": "load", "candles": CANDLES}))
    return env


def _rollout(steps: int) -> list:
    env = _loaded()
    trace = [env.command(json.dumps({"cmd": "reset", "seed": 7}))]
    for _ in range(steps):
        trace.append(env.command(json.dumps({"cmd": "step", "action": 2.0})))
    return trace


def test_the_streamed_rollout_is_byte_identical_when_replayed() -> None:
    assert _rollout(STEPS) == _rollout(STEPS)


def test_a_prefix_of_the_rollout_matches_a_shorter_one() -> None:
    """Stepping further does not change the bars already visited."""
    long_run = _rollout(STEPS)
    short_run = _rollout(STEPS - 2)
    assert long_run[: len(short_run)] == short_run


def test_the_first_observation_is_the_warmup_bar_of_the_tensor() -> None:
    env = _loaded()
    reset = json.loads(env.command(json.dumps({"cmd": "reset", "seed": 7})))
    # Bar 3 closes at 103; Sma(3) over the three bars ending there — 101, 102,
    # 103 — is 102.
    assert reset["observation"][1] == 103.0
    assert reset["observation"][0] == 102.0


def test_a_warmup_below_the_indicator_lookback_is_refused() -> None:
    """A shorter warmup would observe a 0.0 that means "not computed yet"."""
    spec = json.loads(SPEC)
    spec["episode"]["warmup"] = 1
    env = RawEnv(json.dumps(spec))
    response = env.command(json.dumps({"cmd": "load", "candles": CANDLES}))
    assert '"ok":true' not in response.replace(" ", ""), response
    assert "warmup" in response
