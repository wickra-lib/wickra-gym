"""Cross-language golden: replay the committed golden rollouts through RawEnv and
confirm byte-identical output. Skipped until the golden fixtures land (§4)."""

import json
from pathlib import Path

import pytest

# bindings/python/tests -> repo root -> golden/
GOLDEN = Path(__file__).resolve().parents[3] / "golden"


def _cases():
    if not GOLDEN.is_dir():
        return []
    return sorted(p for p in GOLDEN.glob("*/spec.json"))


CASES = _cases()


@pytest.mark.skipif(not CASES, reason="golden fixtures not present yet")
@pytest.mark.parametrize("spec_path", CASES, ids=lambda p: p.parent.name)
def test_golden_rollout(spec_path):
    from wickra_gym import RawEnv

    case = spec_path.parent
    spec = spec_path.read_text(encoding="utf-8")
    candles = json.loads((case / "candles.json").read_text(encoding="utf-8"))
    expected = json.loads((case / "expected.json").read_text(encoding="utf-8"))
    actions = expected["actions"]

    env = RawEnv(spec)
    env.command(json.dumps({"cmd": "load", "candles": candles}))
    seed = expected.get("seed")
    reset_cmd = {"cmd": "reset"} | ({"seed": seed} if seed is not None else {})
    reset = json.loads(env.command(json.dumps(reset_cmd)))
    assert reset == expected["reset"]

    trajectory = []
    for action in actions:
        trajectory.append(
            json.loads(env.command(json.dumps({"cmd": "step", "action": action})))
        )
    assert trajectory == expected["trajectory"]
