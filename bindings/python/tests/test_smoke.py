"""Smoke test: the raw command surface loads, resets and steps."""

import json

from wickra_gym import RawEnv, __version__

SPEC = json.dumps(
    {
        "dataset_ref": "smoke",
        "symbol": "TEST",
        "observation": {"features": [{"kind": "price", "field": "close"}]},
        "action_space": {"type": "discrete", "n": 3},
        "reward": "pnl",
        "episode": {"max_steps": 100, "warmup": 0},
    }
)

CANDLES = [
    {"ts": i, "open": 100.0 + i, "high": 100.0 + i, "low": 100.0 + i, "close": 100.0 + i}
    for i in range(5)
]


def test_version_is_a_string():
    assert isinstance(__version__, str)
    assert RawEnv.version() == __version__


def test_load_reset_step():
    env = RawEnv(SPEC)
    assert json.loads(env.command(json.dumps({"cmd": "load", "candles": CANDLES}))) == {
        "ok": True
    }
    reset = json.loads(env.command(json.dumps({"cmd": "reset"})))
    assert reset["observation"] == [100.0]
    step = json.loads(env.command(json.dumps({"cmd": "step", "action": 2})))
    assert step["reward"] == 1.0  # long over a +1 close move
    assert step["terminated"] is False
    assert step["info"]["step"] == 1.0


def test_bad_spec_raises():
    import pytest

    with pytest.raises(ValueError):
        RawEnv(json.dumps({"not": "a spec"}))
