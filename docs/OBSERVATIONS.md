# Observations

An observation is a fixed-length `f64` vector. Its length and the meaning of
every slot are fully determined by the `ObsSpec` in the environment spec, so an
observation is comparable across languages bit for bit.

## `ObsSpec`

```jsonc
{
  "features": [ /* Feature, in declaration order */ ],
  "include_book_levels": 0,   // N orderbook levels -> 4*N columns
  "include_funding_oi": false  // true -> 2 columns (funding, oi)
}
```

The observation length is:

```
feature_dim = features.len() + 4 * include_book_levels + (include_funding_oi ? 2 : 0)
```

## Feature families

A `Feature` is one scalar observation column, internally tagged by `kind`
(snake_case):

| `kind` | Shape | Example |
| ------ | ----- | ------- |
| `indicator` | `{"name": <Registry name>, "params": [..], "field"?: <sub-output>}` | `{"kind":"indicator","name":"Rsi","params":[14]}` |
| `price` | `{"field": open\|high\|low\|close\|volume}` | `{"kind":"price","field":"close"}` |
| `microstructure` | `{"field": <MicroField>, "level"?: <u32>}` | `{"kind":"microstructure","field":"imbalance"}` |

- **Indicator names are the registry's PascalCase identifiers** (`Rsi`, `Sma`,
  `Ema`, `Macd`, …) — the same 514 indicators as
  [`wickra-core`](https://github.com/wickra-lib/wickra). `params` are the
  indicator's constructor arguments; `field` selects a sub-output on a
  multi-output indicator (e.g. `"hist"` on `Macd`) and defaults to the primary
  output.
- An unknown indicator name is an `UnknownIndicator` error; an unknown `field`
  or an unavailable microstructure column is a `BadSpec` error.

## Exact observation order

The observation vector concatenates, in this exact order:

1. **`features`** — in declaration order.
2. **Orderbook levels** — for each level `0 .. include_book_levels-1`, the four
   columns `bid_px, bid_sz, ask_px, ask_sz`.
3. **Funding / OI** — if `include_funding_oi`, then `funding, oi`.

This order is the determinism contract: the same `ObsSpec` produces the same
layout in every binding.

## Warmup

Indicators are not ready until they have seen enough bars. Rather than emit
`NaN`, the environment **skips the first `episode.warmup` bars** — `reset()`
starts the episode at `bar = warmup`, so every observation the agent sees is
fully warmed up. Before warmup a feature value is a defined `0.0`, never `NaN`.

## Cross-language equality

Every binding forwards the same `command_json` string verbatim, so `reset` and
`step` return byte-identical observation arrays. The `golden/` fixtures pin the
blessed arrays and the cross-language golden tests assert equality; see the
[examples](../examples) for one runnable rollout per language.
