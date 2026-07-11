# Benchmarks

Micro-benchmarks for `gym-core`, measured with
[criterion](https://github.com/bheisler/criterion.rs) via the `gym-bench` crate:

```bash
cargo bench -p gym-bench                         # parallel (rayon) tensor precompute
cargo bench -p gym-bench --no-default-features   # sequential precompute
```

The benches cover the two hot paths: the one-time **feature-tensor precompute**
(walk the dataset, compute every observation feature per bar, pack into the fixed
tensor) and the **rollout** (`reset` + N `step` calls — the O(1)-per-step index
into that tensor plus the fill/PnL reward).

Numbers land here once the core is built and the golden trajectories are blessed
(from P-GYM-5.6): `step()` throughput in steps/s at observation dimensions
{5, 20, 50}, and tensor-build time at {1k, 10k, 100k} bars. They are indicative
single-machine micro-benchmarks, not a cross-framework comparison — the product's
value is deterministic, O(1) rollouts, not raw speed.
