"""Raw rollout over the native command surface (no Gymnasium required).

    python examples/python/rollout.py

Reads the momentum_discrete spec and its candle dataset, then drives a fixed
long policy through the environment via ``RawEnv.command`` — the same JSON-in /
JSON-out boundary every language binding forwards verbatim, so this trajectory
is byte-identical to the C, Node, Go, C#, Java and R examples on the same seed.
"""

import json
from pathlib import Path

from wickra_gym import RawEnv

DATA = Path(__file__).resolve().parent.parent / "data"


def main() -> None:
    spec = (DATA / "specs" / "momentum_discrete.json").read_text()
    candles = json.loads((DATA / "candles.json").read_text())

    env = RawEnv(spec)
    env.command(json.dumps({"cmd": "load", "candles": candles}))

    reset = json.loads(env.command(json.dumps({"cmd": "reset", "seed": 42})))
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
