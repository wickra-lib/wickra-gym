# Golden fixtures

The cross-language determinism harness. Each case is a directory holding a fixed
spec, a fixed candle dataset, and the canonical rollout the deterministic core
produces from them. Every language binding (Rust, C, Python, Node, WASM, Go, C#,
Java, R) replays the same inputs and asserts value-equality with the recorded
`reset` / `trajectory` — so the same `(spec, data, seed, actions)` yields a
byte-identical trajectory in all ten languages.

## Layout

```
golden/<case>/
├── spec.json       # the EnvSpec
├── candles.json    # the input candle series
└── expected.json   # { seed, actions, reset, trajectory }
```

`expected.json` records the fixed policy (`actions`), the reset observation, and
one `StepResult` per step. The binding tests load `spec.json` + `candles.json`,
`reset(seed)`, replay `actions`, and compare against `reset` / `trajectory`.

## The dataset formula

The base OHLCV universe is deterministic:

```
close(i) = 100 + 5 * sin(i / 3) + 0.5 * i        for i in 0 .. 29
open(i)  = close(i)
high(i)  = close(i) + 0.5
low(i)   = close(i) - 0.5
volume   = 1000
ts(i)    = 1_700_000_000 + i * 3600
```

The `micro_book` dataset adds a two-level order book around the close
(`bid_px/bid_sz/ask_px/ask_sz`) plus `funding` and `oi`.

## Cases and fixed policies

| Case                  | Observation             | Action space       | Reward      | Policy       |
|-----------------------|-------------------------|--------------------|-------------|--------------|
| `momentum_discrete`   | `Rsi(14)`, `price.close`| Discrete(3)        | `pnl`       | always long (2) |
| `momentum_continuous` | `Rsi(14)`, `price.close`| Continuous(-1, 1)  | `pnl`       | always 2 → clamp |
| `micro_book`          | `price.close` + 2 book levels + funding/oi | Discrete(3) | `pnl` | cycle 0,1,2,… |
| `sharpe`              | `Rsi(14)`, `price.close`| Discrete(3)        | `sharpe`    | always long (2) |
| `logreturn`          | `Rsi(14)`, `price.close`| Discrete(3)        | `log_return`| always long (2) |

All cases run `max_steps = 5`; the momentum/sharpe/logreturn cases use
`warmup = 15` (so the RSI is ready) and `seed = 42`; `micro_book` uses
`warmup = 2` and `seed = 7`.

## Blessing

The fixtures are generated, never hand-written. Regenerate after any change to
the core semantics:

```sh
python golden/bless.py
```

**Never edit `expected.json` by hand.** A change there that is not reproduced by
`bless.py` will fail every binding's golden test.
