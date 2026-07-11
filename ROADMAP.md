# Roadmap

`wickra-gym` is pre-1.0 and under active construction. This roadmap sketches
direction, not commitments or dates.

## Now (0.1.x) — foundation

- The deterministic `gym-core`: feature extraction, the O(1) `FeatureTensor`,
  the observation/action/reward specs, and the seeded environment lifecycle.
- The reference CLI (rollout runner) and the ten-language binding surface, with
  Python's `gymnasium.Env` subclass.
- Byte-exact golden trajectories, conformance / determinism / property tests,
  fuzz targets, per-language examples, and the full cross-OS CI matrix.

## Next

- **Vectorized / batched rollouts** — a `VecEnv`-style API that steps many
  independent episodes at once for higher training throughput.
- **More reward kinds** — additional shaped rewards (drawdown-aware, Sharpe,
  transaction-cost-sensitive) beyond log-return and PnL.
- **Multi-symbol environments** — observations and actions spanning several
  instruments in one episode.
- **Richer microstructure** — deeper orderbook levels and additional funding /
  open-interest features under the optional `live` universe.

## Guiding constraints

- **Determinism first** — every new feature preserves the byte-identical
  `(seed, policy)` trajectory guarantee across all ten languages.
- **O(1) steps** — new observation features must be precomputable into the fixed
  feature tensor; `step()` stays a constant-time index.
- **Ten-language parity** — the core stays a small `command_json` data API so
  every binding returns the same bytes.
