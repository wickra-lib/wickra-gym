"""Type stubs for wickra_gym."""

from typing import Any

import numpy as np
import numpy.typing as npt

__version__: str
__all__ = ["RawEnv", "WickraGymEnv", "register", "__version__"]

class RawEnv:
    """A raw environment handle over the native command surface."""

    def __init__(self, spec_json: str) -> None: ...
    def command(self, cmd_json: str) -> str: ...
    @staticmethod
    def version() -> str: ...

class WickraGymEnv:
    """A Gymnasium environment backed by gym-core (requires ``gymnasium``)."""

    metadata: dict[str, list[str]]
    observation_space: Any
    action_space: Any

    def __init__(self, spec_json: str, candles: list[dict[str, Any]]) -> None: ...
    def reset(
        self, *, seed: int | None = ..., options: dict[str, Any] | None = ...
    ) -> tuple[npt.NDArray[np.float64], dict[str, float]]: ...
    def step(
        self, action: Any
    ) -> tuple[npt.NDArray[np.float64], float, bool, bool, dict[str, float]]: ...

def register() -> None: ...
