# Actions and rewards

## Action spaces

The action space is one of two kinds, tagged by `type` (snake_case):

```jsonc
{ "type": "discrete",   "n": 3 }                 // n buckets
{ "type": "continuous", "low": -1.0, "high": 1.0 } // target position, clamped
```

- **`Discrete{n}`** — the `n` action buckets map onto a target position. With
  `n = 3` the canonical mapping is `0 -> short (-1)`, `1 -> flat (0)`,
  `2 -> long (+1)`; larger `n` interpolates evenly across `[-1, +1]`.
- **`Continuous{low, high}`** — the action *is* the target position, clamped to
  `[low, high]`.

The chosen target position is the input to the fill/PnL model.

## The position model

`step(action)` sets the desired target position for the next bar, and the
[`wickra-backtest`](https://github.com/wickra-lib/wickra-backtest) fill/PnL model
realises the move (fees and slippage included). The realised position path — not
the raw action — is what the reward is computed from, so a position that cannot
be filled does not silently inflate the reward.

## Reward kinds

`reward` selects how each step's scalar reward is computed from the realised
position path:

| `reward` | Meaning |
| -------- | ------- |
| `pnl` | The step's realised profit and loss. |
| `sharpe` | A rolling Sharpe-style ratio of the return stream. |
| `log_return` | The log return of equity over the step. |

All three are deterministic functions of the position path and the price data,
so the reward stream is byte-identical across languages for a given
`(spec, data, seed, policy)`.

## Episode bounds

```jsonc
{ "max_steps": 256, "warmup": 15 }
```

- **`warmup`** — the first `warmup` bars are skipped so indicators are ready;
  `reset()` starts the episode at `bar = warmup`.
- **`max_steps`** — the truncation limit. When the step count reaches
  `max_steps`, or the dataset runs out, `step()` returns `truncated = true`
  (Gymnasium's time-limit signal) rather than `terminated`.

## `StepResult`

Each `step` returns the observation for the new bar, the scalar reward, the
`terminated` / `truncated` flags, and an `info` object (bar index, step index,
position and equity). See the [examples](../examples) for the exact fields, and
[GYMNASIUM.md](GYMNASIUM.md) for how these map onto the Gymnasium 5-tuple.
