# Microstructure observations

Beyond price and indicators, an observation can include exchange microstructure —
the order book, the funding rate and open interest. These are opt-in on the
`ObsSpec` and, like every other feature, occupy fixed slots in a defined order.

## Order book levels

Set `include_book_levels: N` to append `N` order book levels. Each level adds
four columns, in this order:

```
bid_px, bid_sz, ask_px, ask_sz
```

so `N` levels add `4 * N` columns, appended after the declared `features` and
before the funding/OI columns (see [OBSERVATIONS.md](OBSERVATIONS.md) for the
full order). A specific level can also be referenced as a single feature with
`{"kind":"microstructure","field":"bid_sz","level":0}`.

Derived book fields are available as level-less microstructure features:

- `spread` — best ask minus best bid.
- `imbalance` — order-book imbalance at the top of book.

## Funding and open interest

Set `include_funding_oi: true` to append two columns, in this order:

```
funding, oi
```

or reference them individually with `{"kind":"microstructure","field":"funding"}`
and `{"kind":"microstructure","field":"oi"}`. The `level` field is ignored for
non-level microstructure fields (`spread`, `imbalance`, `funding`, `oi`).

## Missing columns are `0.0`, never `NaN`

If a dataset does not carry a requested microstructure column, the value is a
**defined `0.0`** rather than `NaN`. This keeps the observation length fixed and
the array finite, so the observation stays comparable across languages and the
`Box` bounds stay meaningful. The candle JSON carries the optional
`bid_px / bid_sz / ask_px / ask_sz` arrays and the `funding` / `oi` scalars; when
they are absent the corresponding observation slots are `0.0`.

## Data source

Live microstructure feeds (order book / funding / OI) are provided by
[`wickra-exchange`](https://github.com/wickra-lib/wickra-exchange) behind the
`live` feature; for offline rollouts the columns come from the candle dataset
itself.
