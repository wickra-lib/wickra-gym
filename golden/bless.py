#!/usr/bin/env python3
"""Regenerate the golden fixtures from the fixed specs + datasets.

Each golden case is a directory ``golden/<case>/`` holding:

  * ``spec.json``     — the EnvSpec.
  * ``candles.json``  — the input candle series (the fixed, deterministic
                        universe; see README for the formula).
  * ``expected.json`` — ``{seed, actions, reset, trajectory}``: the canonical
                        rollout, captured verbatim from the deterministic core.

The rollout is produced through the Python binding, but the response strings are
the core's canonical ``command_json`` output — byte-identical to what every other
language binding produces. Every binding's golden test replays the same
``spec.json`` + ``candles.json`` + ``actions`` and asserts value equality with
``reset`` / ``trajectory``.

Do NOT edit ``expected.json`` by hand — run ``python golden/bless.py``.
"""

import json
import math
from pathlib import Path

import wickra_gym

ROOT = Path(__file__).resolve().parent

# The fixed, deterministic universe: close(i) = 100 + 5*sin(i/3) + 0.5*i.
N_BARS = 30


def ohlcv(n=N_BARS):
    out = []
    for i in range(n):
        close = 100.0 + 5.0 * math.sin(i / 3.0) + 0.5 * i
        out.append(
            {
                "ts": 1_700_000_000 + i * 3600,
                "open": close,
                "high": close + 0.5,
                "low": close - 0.5,
                "close": close,
                "volume": 1000.0,
            }
        )
    return out


def ohlcv_book(n=N_BARS):
    # A microstructure dataset: a two-level book around the close.
    out = []
    for i in range(n):
        close = 100.0 + 5.0 * math.sin(i / 3.0) + 0.5 * i
        out.append(
            {
                "ts": 1_700_000_000 + i * 3600,
                "open": close,
                "high": close + 0.5,
                "low": close - 0.5,
                "close": close,
                "volume": 1000.0,
                "bid_px": [close - 0.1, close - 0.2],
                "bid_sz": [10.0 + i, 20.0 + i],
                "ask_px": [close + 0.1, close + 0.2],
                "ask_sz": [11.0 + i, 21.0 + i],
                "funding": 0.0001,
                "oi": 50000.0 + 100.0 * i,
            }
        )
    return out


RSI_CLOSE = {
    "features": [
        {"kind": "indicator", "name": "Rsi", "params": [14]},
        {"kind": "price", "field": "close"},
    ]
}


def spec(observation, action_space, reward, warmup=15, max_steps=5, seed=42):
    return {
        "dataset_ref": "golden",
        "symbol": "BTCUSDT",
        "observation": observation,
        "action_space": action_space,
        "reward": reward,
        "episode": {"max_steps": max_steps, "warmup": warmup},
        "seed": seed,
    }


def policy_action(name, step):
    if name == "always-0":
        return 0
    if name == "always-1":
        return 1
    if name == "always-2":
        return 2
    if name == "cycle":
        return step % 3
    raise ValueError(name)


CASES = {
    "momentum_discrete": {
        "spec": spec(RSI_CLOSE, {"type": "discrete", "n": 3}, "pnl"),
        "candles": ohlcv(),
        "policy": "always-2",
    },
    "momentum_continuous": {
        "spec": spec(RSI_CLOSE, {"type": "continuous", "low": -1.0, "high": 1.0}, "pnl"),
        "candles": ohlcv(),
        "policy": "always-2",
    },
    "micro_book": {
        "spec": spec(
            {
                "features": [{"kind": "price", "field": "close"}],
                "include_book_levels": 2,
                "include_funding_oi": True,
            },
            {"type": "discrete", "n": 3},
            "pnl",
            warmup=2,
            seed=7,
        ),
        "candles": ohlcv_book(),
        "policy": "cycle",
    },
    "sharpe": {
        "spec": spec(RSI_CLOSE, {"type": "discrete", "n": 3}, "sharpe"),
        "candles": ohlcv(),
        "policy": "always-2",
    },
    "logreturn": {
        "spec": spec(RSI_CLOSE, {"type": "discrete", "n": 3}, "log_return"),
        "candles": ohlcv(),
        "policy": "always-2",
    },
}


def bless(name, case):
    spec_json = json.dumps(case["spec"])
    env = wickra_gym.RawEnv(spec_json)
    env.command(json.dumps({"cmd": "load", "candles": case["candles"]}))

    seed = case["spec"].get("seed")
    reset_cmd = {"cmd": "reset"} | ({"seed": seed} if seed is not None else {})
    reset = json.loads(env.command(json.dumps(reset_cmd)))

    actions = []
    trajectory = []
    step = 0
    while True:
        action = policy_action(case["policy"], step)
        response = json.loads(env.command(json.dumps({"cmd": "step", "action": action})))
        if "error" in response:
            break
        actions.append(action)
        trajectory.append(response)
        step += 1
        if response["terminated"] or response["truncated"]:
            break

    expected = {"seed": seed, "actions": actions, "reset": reset, "trajectory": trajectory}

    out = ROOT / name
    out.mkdir(parents=True, exist_ok=True)
    (out / "spec.json").write_text(json.dumps(case["spec"], indent=2) + "\n", encoding="utf-8")
    (out / "candles.json").write_text(json.dumps(case["candles"], indent=2) + "\n", encoding="utf-8")
    (out / "expected.json").write_text(json.dumps(expected, indent=2) + "\n", encoding="utf-8")
    print(f"blessed {name}: {len(trajectory)} steps")


def main():
    for name, case in CASES.items():
        bless(name, case)


if __name__ == "__main__":
    main()
