//! Criterion benchmarks: rollout step throughput by observation dimension, and
//! feature-tensor precompute by dataset size. Run the sequential path with
//! `--no-default-features` — the output is byte-identical, so the delta is pure
//! scheduling overhead.

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use gym_core::{
    build_tensor, ActionSpace, Candle, Env, EnvSpec, EpisodeSpec, Feature, ObsSpec, PriceField,
    RewardKind,
};
use std::hint::black_box;

fn make_spec(dim: usize, max_steps: u32) -> String {
    let features = (0..dim)
        .map(|_| Feature::Price {
            field: PriceField::Close,
        })
        .collect();
    let spec = EnvSpec {
        dataset_ref: "bench".into(),
        symbol: "S".into(),
        observation: ObsSpec {
            features,
            include_book_levels: 0,
            include_funding_oi: false,
        },
        action_space: ActionSpace::Discrete { n: 3 },
        reward: RewardKind::Pnl,
        episode: EpisodeSpec {
            max_steps,
            warmup: 0,
        },
        seed: Some(1),
    };
    serde_json::to_string(&spec).unwrap()
}

fn make_candles(n: usize) -> Vec<Candle> {
    (0..n)
        .map(|i| {
            let close = 100.0 + (i as f64) * 0.01;
            Candle {
                ts: i64::try_from(i).unwrap(),
                open: close,
                high: close + 0.5,
                low: close - 0.5,
                close,
                volume: 1000.0,
                bid_px: Vec::new(),
                bid_sz: Vec::new(),
                ask_px: Vec::new(),
                ask_sz: Vec::new(),
                funding: None,
                oi: None,
            }
        })
        .collect()
}

const STEPS: u64 = 100;

fn bench_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("step");
    let candles = make_candles(500);
    for dim in [5usize, 20, 50] {
        let spec = make_spec(dim, 100_000);
        group.throughput(Throughput::Elements(STEPS));
        group.bench_with_input(BenchmarkId::from_parameter(dim), &dim, |b, _| {
            b.iter_batched(
                || {
                    let mut env = Env::new(&spec).unwrap();
                    env.load(&candles).unwrap();
                    env.reset(Some(1)).unwrap();
                    env
                },
                |mut env| {
                    for _ in 0..STEPS {
                        let _ = black_box(env.step(2.0));
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_build_tensor(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_tensor");
    let obs = ObsSpec {
        features: vec![Feature::Price {
            field: PriceField::Close,
        }],
        include_book_levels: 0,
        include_funding_oi: false,
    };
    for n in [1_000usize, 10_000, 100_000] {
        let candles = make_candles(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(build_tensor(&candles, &obs).unwrap()));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_step, bench_build_tensor);
criterion_main!(benches);
