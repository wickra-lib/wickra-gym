# Documentation

These pages are the guides that live beside the code, because they describe how
this repository behaves and have to change in the same commit the behaviour does.

| Page | What it answers |
|------|-----------------|
| [OBSERVATIONS.md](OBSERVATIONS.md) | The shape of an `ObsSpec`: indicator, price and microstructure columns, and the order they appear in |
| [FEEDS.md](FEEDS.md) | Which indicator reads what beyond the candle, what the bar can supply, and why a spec is refused rather than answered with a column of zeros |
| [ACTIONS_REWARDS.md](ACTIONS_REWARDS.md) | The action spaces, the reward functions, and the episode fields |
| [MICROSTRUCTURE.md](MICROSTRUCTURE.md) | Order-book levels, funding and open interest as observation columns |
| [GYMNASIUM.md](GYMNASIUM.md) | The `gymnasium.Env` subclass, registration, and what it wraps |
| [Cookbook.md](Cookbook.md) | Worked environments |

The API reference for each language is generated from the source rather than
committed here — `cargo doc` for Rust, the `.d.ts` beside the Node binding, the
docstrings in the Python module, the C header. Keeping a second copy in this
repository would drift from the code that generates it, and a reader opening
`docs/` would have no way to tell which copy was current.

The indicator library the environment resolves names through documents itself at
<https://docs.wickra.org>.

What stays here is what a generator cannot produce: the meaning of a field, the
reason a case is refused rather than answered, and the worked examples.

Elsewhere in the repository:

- [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — the crate and binding layout
- [`../BENCHMARKS.md`](../BENCHMARKS.md) — what is measured and how
- [`../golden/README.md`](../golden/README.md) — the cross-language rollouts and how to re-bless them
- [`../CONTRIBUTING.md`](../CONTRIBUTING.md) — how to build, test and propose a change
- [`../THREAT_MODEL.md`](../THREAT_MODEL.md) — what the environment does and does not touch
