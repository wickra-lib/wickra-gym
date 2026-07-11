#![no_main]
//! Fuzz a step sequence. A loaded, reset env is driven by an arbitrary action
//! stream (one action per input byte). No action ever panics; stepping past the
//! end of the episode returns an error, never a crash.

use gym_core::{Candle, Env};
use libfuzzer_sys::fuzz_target;

const SPEC: &str = r#"{
    "dataset_ref": "fuzz", "symbol": "S",
    "observation": { "features": [{ "kind": "price", "field": "close" }] },
    "action_space": { "type": "discrete", "n": 3 },
    "reward": "pnl",
    "episode": { "max_steps": 32, "warmup": 0 }
}"#;

fn candles() -> Vec<Candle> {
    (0..40)
        .map(|i| {
            let close = 100.0 + f64::from(i);
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

fuzz_target!(|data: &[u8]| {
    let Ok(mut env) = Env::new(SPEC) else {
        return;
    };
    if env.load(&candles()).is_err() || env.reset(Some(1)).is_err() {
        return;
    }
    for &byte in data {
        // A discrete action in 0..3, plus occasional out-of-range values that
        // must be rejected as a BadAction error rather than panic.
        let action = f64::from(byte % 5);
        if env.step(action).is_err() {
            break;
        }
    }
});
