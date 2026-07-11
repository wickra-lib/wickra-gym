//! Determinism: a `(seed, policy)` pair yields a byte-identical trajectory, and
//! the feature tensor is byte-identical whether the `parallel` (rayon) feature is
//! on or off.
//!
//! The parallel-vs-sequential equivalence is enforced across the build: CI runs
//! this and `golden.rs` under both `--all-features` and `--no-default-features`
//! against the same committed golden fixtures, so a divergence in either path
//! fails. Within a single build, this test pins that repeated runs are identical.

use gym_core::{Candle, Env};

const SPEC: &str = r#"{
    "dataset_ref": "det", "symbol": "S",
    "observation": { "features": [
        { "kind": "price", "field": "close" },
        { "kind": "indicator", "name": "Sma", "params": [3] }
    ] },
    "action_space": { "type": "discrete", "n": 3 },
    "reward": "pnl",
    "episode": { "max_steps": 8, "warmup": 3 }
}"#;

fn candles() -> Vec<Candle> {
    (0..20)
        .map(|i| {
            let close = 100.0 + 5.0 * f64::from(i).sin() + 0.5 * f64::from(i);
            Candle {
                ts: i64::from(i),
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

fn rollout(seed: u64) -> Vec<String> {
    let mut env = Env::new(SPEC).unwrap();
    env.load(&candles()).unwrap();
    let mut trace = vec![serde_json::to_string(&env.reset(Some(seed)).unwrap()).unwrap()];
    for i in 0..8u32 {
        match env.step(f64::from(i % 3)) {
            Ok(step) => trace.push(serde_json::to_string(&step).unwrap()),
            Err(_) => break,
        }
    }
    trace
}

#[test]
fn same_seed_is_byte_identical() {
    assert_eq!(rollout(42), rollout(42));
}

#[test]
fn distinct_runs_are_reproducible() {
    // Two fresh environments, same seed and policy, must agree step for step.
    let a = rollout(7);
    let b = rollout(7);
    assert_eq!(a, b);
    assert!(a.len() > 1);
}
