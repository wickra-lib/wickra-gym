# Feeds

Most indicators need only a candle. A large part of the registry needs something
else as well — a reference series, a derivatives tick, an order-book snapshot,
the trades that printed in the bar, or the market cross-section. This page is
which of those an environment can supply, which it cannot, and what happens when
a spec asks for one it cannot.

Related: [OBSERVATIONS.md](OBSERVATIONS.md) names an observation column;
[MICROSTRUCTURE.md](MICROSTRUCTURE.md) covers the raw book/funding columns.

## Why this matters more here than elsewhere

An indicator whose feed is absent resolves, ticks and returns nothing — every
bar, without complaint. In a table that shows as a `NaN` column. In an
observation tensor it cannot: a tensor has to hand the agent a number, so a
missing value is collapsed to `0.0`.

A permanently unfed indicator is therefore a column of **constant zeros**,
indistinguishable from an indicator that is merely warming up, and an agent
trains on it as if it meant something. Nothing anywhere would report it.

## What the bar already carries

`Candle` holds more than OHLCV: `bid_px` / `bid_sz` / `ask_px` / `ask_sz`,
`funding` and `oi`. The tensor already exposes those as raw observation columns.
The same data is now handed to the indicators as well, so:

| Family | Supplied from | Example |
| --- | --- | --- |
| `Candle` | the bar | `Sma`, `Rsi`, `Atr` |
| `OrderBook` | the bar's `bid_*` / `ask_*` levels | `Microprice` |
| `Derivatives` | the bar's `funding` and `oi` | `FundingRate` |

Availability is derived from the bars, not declared. A feed counts as present
only when **every** bar carries it: a book on some bars and not others would
leave the indicator ticking on a hole, which is the same silent case in a
smaller form, so it is refused too.

## What an environment cannot supply

Three families have no source in a single-symbol OHLCV environment, and a spec
naming one is refused with the reason rather than zeroed:

| Family | Why not | Example |
| --- | --- | --- |
| `Pair` | an environment steps one symbol, so there is no second series to read | `Beta`, `PearsonCorrelation` |
| `Trades` / `TradeQuote` | a bar carries aggregate volume, not the individual prints | `CumulativeVolumeDelta`, `EffectiveSpread` |
| `CrossSection` | an environment steps one symbol, so there is no market panel | `AdvanceDecline` |

The refusal names all three parts:

```
Microprice needs the orderbook feed: no bar in this dataset carries order-book levels
```

## Warmup is part of the same problem

A column is also `0.0` before its indicator has produced anything — and that
zero is just as unreadable to an agent. `episode.warmup` used to be a free
parameter, so an episode could start at bar 1 with an `Sma(3)` column reading
zero.

The floor now comes from the indicators the spec names: `episode.warmup` must be
at least the longest lookback among them, and a spec below it is refused:

```
episode.warmup is 1 but the observation needs 3: below that an indicator column
is 0.0 because nothing has been produced yet, which an agent cannot tell from a
market reading of zero
```

Raising `warmup` costs bars at the head of the dataset, not accuracy. It is the
difference between an agent that sees three fewer steps and one that trains on
three fabricated ones.
