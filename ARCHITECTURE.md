# Architecture

`wickra-gym` is a small deterministic Rust core (`gym-core`) exposed as a
JSON-over-C-ABI data API in ten languages, plus a reference CLI (a rollout
runner) and — for Python — a real `gymnasium.Env` subclass.

## Workspace layout

```
crates/gym-core        the library: feature extraction, feature tensor, env, reward
crates/gym-cli         reference CLI (rollout runner), binary `wickra-gym`
crates/gym-bench       Criterion benchmarks (steps/s, tensor build)
bindings/{c,python,node,wasm}   native + C ABI surface
  (the C ABI hub serves c/c++/c#/go/java/r)
golden/                fixed (spec, data, policy) -> expected trajectory (byte-exact)
examples/              runnable per-language demos + a gymnasium PPO example
fuzz/                  cargo-fuzz targets (spec parse, command_json, tensor build, step)
```

## The data-driven boundary

The core carries no configuration of its own: an environment is fully described
by a JSON **spec** (observation features, action space, reward kind, episode
parameters), a candle dataset, and a seed. `gym-core` exposes a single
`command_json` entry point that every binding forwards verbatim:

| `cmd`     | Payload                       | Response |
|-----------|-------------------------------|----------|
| `load`    | `{spec, data}`                | env handle metadata (feature dim, length) |
| `reset`   | `{seed}`                      | the initial observation |
| `step`    | `{action}`                    | `{observation, reward, terminated, truncated, info}` |
| `spec`    | —                             | the resolved, canonicalized spec |
| `version` | —                             | the core and engine versions |

Because every binding returns the core's canonical response verbatim, a
trajectory is byte-identical across all ten languages — the cross-language golden
tests assert it.

## The feature tensor (O(1) steps)

On `load`, the whole dataset is walked once. For each bar, every feature named in
the `ObsSpec` is computed with the O(1)-per-bar streaming indicators from
`wickra-core` (and, under the optional `live` feature, the microstructure feeds
from `wickra-exchange`). The results are packed, in canonical `ObsSpec` order,
into a fixed **`FeatureTensor`**: a dense `f64` matrix of shape
`(n_bars, feature_dim)`. From then on, `step()` is a constant-time row index into
that tensor — no recompute, so rollouts are fast. The precompute may run in
parallel (rayon), but the per-feature reductions stay serial and the tensor is
byte-identical to the sequential (WASM) path.

## The environment lifecycle

`reset(seed)` seeds the single episode RNG (never `thread_rng`), positions the
cursor at the start of the (optionally randomized) episode window, and returns
the first observation row. `step(action)` advances the cursor by one bar, applies
the action through the `wickra-backtest` fill/PnL model to compute the reward,
and returns the next observation together with `terminated` / `truncated` flags.
Because the only randomness is the seeded RNG and the observations come straight
from the fixed tensor, a `(seed, policy)` pair fully determines the trajectory.

## Determinism invariants

- `BTreeMap` (never `HashMap`) in every output path.
- Observations are fixed-length, canonical-order `f64` vectors.
- The only RNG is the explicitly-seeded episode RNG.
- Feature reductions run in a fixed operation order, independent of rayon
  scheduling, so parallel and sequential precompute agree bit for bit.
