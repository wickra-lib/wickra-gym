"""Cross-language golden: replay the committed golden rollouts through RawEnv and
confirm byte-identical output."""

import json
from pathlib import Path

# bindings/python/tests -> repo root -> golden/
GOLDEN = Path(__file__).resolve().parents[3] / "golden"


def _cases():
    if not GOLDEN.is_dir():
        return []
    return sorted(p for p in GOLDEN.glob("*/spec.json"))


CASES = _cases()


def test_golden_rollouts():
    # One function over every case rather than pytest.mark.parametrize: this
    # module also runs on the Python 3.9 row, which has no pytest (see
    # run_without_pytest.py). A missing corpus is a failure, not a skip.
    assert CASES, "golden corpus not found"
    for spec_path in CASES:
        _check_case(spec_path)


def _check_case(spec_path):
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
    assert reset == expected["reset"], spec_path.parent.name

    trajectory = []
    for action in actions:
        trajectory.append(
            json.loads(env.command(json.dumps({"cmd": "step", "action": action})))
        )
    assert trajectory == expected["trajectory"], spec_path.parent.name
