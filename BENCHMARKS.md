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

## Results

Indicative single-machine numbers (parallel precompute; criterion median of 100
samples). They measure rollout and precompute throughput, not a cross-framework
speed comparison — the product's value is deterministic, O(1) rollouts, not raw
speed.

### Rollout (`reset` + 100 × `step`)

| Observation dim | median (100 steps) | Throughput |
|----------------:|-------------------:|-----------:|
| 5   | ~43.1 µs | ~2.3M steps/s |
| 20  | ~42.6 µs | ~2.3M steps/s |
| 50  | ~43.3 µs | ~2.3M steps/s |

Step throughput is **flat across the observation dimension** — that is the whole
point of the fixed feature tensor: `step()` is a constant-time index that does
not depend on how many features the observation carries.

### Feature-tensor precompute (`build_tensor`)

| Dataset size | median | Throughput |
|-------------:|-------:|-----------:|
| 1,000 bars   | ~20.6 µs | ~48M bars/s |
| 10,000 bars  | ~204 µs  | ~49M bars/s |
| 100,000 bars | ~2.58 ms | ~39M bars/s |

Precompute is roughly linear in the number of bars — the one-time O(1)-per-bar
pass that turns every later `step()` into a pure index.

## Method

- Machine and OS vary; treat the absolute numbers as indicative and re-run
  locally for your hardware.
- The tensor is byte-identical between the parallel (rayon) and sequential
  (`--no-default-features`) precompute, so the feature flag changes scheduling,
  not results.
- The nightly `bench.yml` workflow re-runs this on a schedule and uploads the
  report as a CI artifact.
