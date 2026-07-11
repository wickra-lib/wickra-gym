"""The real Gymnasium environment contract (skipped without gymnasium)."""

import json

import pytest

gymnasium = pytest.importorskip("gymnasium")

from wickra_gym import WickraGymEnv  # noqa: E402

SPEC = json.dumps(
    {
        "dataset_ref": "gym",
        "symbol": "TEST",
        "observation": {
            "features": [
                {"kind": "price", "field": "close"},
                {"kind": "indicator", "name": "Sma", "params": [3]},
            ]
        },
        "action_space": {"type": "discrete", "n": 3},
        "reward": "pnl",
        "episode": {"max_steps": 32, "warmup": 3},
    }
)

CANDLES = [
    {
        "ts": i,
        "open": 100.0 + (i % 7),
        "high": 100.0 + (i % 7) + 0.5,
        "low": 100.0 + (i % 7) - 0.5,
        "close": 100.0 + (i % 7),
    }
    for i in range(64)
]


def make_env():
    return WickraGymEnv(SPEC, CANDLES)


def test_spaces_derived_correctly():
    env = make_env()
    assert env.observation_space.shape == (2,)
    assert env.action_space.n == 3


def test_reset_and_step_signatures():
    env = make_env()
    obs, info = env.reset(seed=0)
    assert obs.shape == (2,)
    assert isinstance(info, dict)
    obs, reward, terminated, truncated, info = env.step(2)
    assert obs.shape == (2,)
    assert isinstance(reward, float)
    assert isinstance(terminated, bool)
    assert isinstance(truncated, bool)


def test_seeded_reset_is_deterministic():
    a = make_env()
    b = make_env()
    obs_a, _ = a.reset(seed=42)
    obs_b, _ = b.reset(seed=42)
    assert (obs_a == obs_b).all()


def test_env_checker_passes():
    from gymnasium.utils.env_checker import check_env

    check_env(make_env(), skip_render_check=True)
