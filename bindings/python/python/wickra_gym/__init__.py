"""Wickra Gym — a deterministic, Gymnasium-compatible backtest environment.

``RawEnv`` is a thin wrapper over the native command surface and works without
Gymnasium. ``WickraGymEnv`` is a real :class:`gymnasium.Env` subclass (the main
consumer); it is only defined when ``gymnasium`` is importable.
"""

import json

import numpy as np

from ._wickra_gym import RawEnv, __version__

__all__ = ["RawEnv", "WickraGymEnv", "register", "__version__"]

# The core encodes unbounded observation bounds as this finite sentinel (JSON
# cannot carry ``inf``); the Box space maps it back to ``np.inf``.
_INF = 1e30

try:
    import gymnasium as gym
    from gymnasium import spaces
except ImportError:  # RawEnv works without gymnasium; the Env subclass needs it.
    gym = None


if gym is not None:

    class WickraGymEnv(gym.Env):
        """A Gymnasium environment backed by the deterministic gym-core."""

        metadata = {"render_modes": []}

        def __init__(self, spec_json: str, candles: list):
            self._raw = RawEnv(spec_json)
            self._raw.command(json.dumps({"cmd": "load", "candles": candles}))
            info = json.loads(self._raw.command(json.dumps({"cmd": "spec"})))
            low = np.array(
                [-np.inf if v <= -_INF else v for v in info["observation_low"]],
                dtype=np.float64,
            )
            high = np.array(
                [np.inf if v >= _INF else v for v in info["observation_high"]],
                dtype=np.float64,
            )
            self.observation_space = spaces.Box(low=low, high=high, dtype=np.float64)
            action = info["action_space"]
            if action["type"] == "discrete":
                self.action_space = spaces.Discrete(action["n"])
            else:
                self.action_space = spaces.Box(
                    low=action["low"],
                    high=action["high"],
                    shape=(1,),
                    dtype=np.float64,
                )

        def reset(self, *, seed=None, options=None):
            super().reset(seed=seed)
            cmd = {"cmd": "reset"}
            if seed is not None:
                cmd["seed"] = int(seed)
            result = json.loads(self._raw.command(json.dumps(cmd)))
            return np.asarray(result["observation"], dtype=np.float64), result["info"]

        def step(self, action):
            if self.action_space.__class__.__name__ == "Discrete":
                encoded = int(action)
            else:
                encoded = float(np.asarray(action).item())
            result = json.loads(
                self._raw.command(json.dumps({"cmd": "step", "action": encoded}))
            )
            return (
                np.asarray(result["observation"], dtype=np.float64),
                float(result["reward"]),
                bool(result["terminated"]),
                bool(result["truncated"]),
                result["info"],
            )

    def register():
        """Register ``WickraGym-v0`` with the Gymnasium registry."""
        gym.register(id="WickraGym-v0", entry_point="wickra_gym:WickraGymEnv")

else:  # pragma: no cover - exercised only when gymnasium is absent

    class WickraGymEnv:  # type: ignore[no-redef]
        """Placeholder raised when Gymnasium is not installed."""

        def __init__(self, *args, **kwargs):
            raise ImportError(
                "WickraGymEnv requires gymnasium; install with "
                "'pip install wickra-gym[gym]'"
            )

    def register():
        """Raise: Gymnasium is required to register the environment."""
        raise ImportError(
            "register() requires gymnasium; install with 'pip install wickra-gym[gym]'"
        )
