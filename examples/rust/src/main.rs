//! A runnable Rust example: read a candle CSV, load a spec, then drive a fixed
//! long policy through the environment and print each step's reward.
//!
//! ```bash
//! cargo run -p wickra-gym-example
//! ```
//!
//! This mirrors the CLI's ingestion path — the CSV is read with the same
//! `wickra-data` reader — so the trajectory matches `wickra-gym run` on the same
//! spec, data and seed.

use std::path::Path;

use gym_core::{Candle, Env};
use wickra_data::csv::CandleReader;

/// Read a candle CSV via `wickra-data` and map into gym candles (OHLCV only;
/// microstructure columns are absent and default to `0.0`).
fn load_candles(path: &Path) -> Vec<Candle> {
    let mut reader = CandleReader::open(path).expect("open the candle CSV");
    reader
        .read_all()
        .expect("read the candle CSV")
        .into_iter()
        .map(|c| Candle {
            ts: c.timestamp,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
            bid_px: Vec::new(),
            bid_sz: Vec::new(),
            ask_px: Vec::new(),
            ask_sz: Vec::new(),
            funding: None,
            oi: None,
        })
        .collect()
}

fn main() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data");

    let spec =
        std::fs::read_to_string(data.join("specs/momentum_discrete.json")).expect("read the spec");
    let candles = load_candles(&data.join("series/BTCUSDT.csv"));

    let mut env = Env::new(&spec).expect("valid spec");
    env.load(&candles).expect("load candles");

    let reset = env.reset(Some(42)).expect("reset");
    println!("reset observation: {:?}", reset.observation);

    // A fixed "always long" policy (discrete action 2 -> target +1).
    let mut equity = 0.0;
    for step in 0.. {
        match env.step(2.0) {
            Ok(result) => {
                equity += result.reward;
                println!(
                    "step {step}: reward {:+.6}  equity {:+.6}  terminated={} truncated={}",
                    result.reward, equity, result.terminated, result.truncated
                );
                if result.terminated || result.truncated {
                    break;
                }
            }
            Err(err) => {
                eprintln!("step {step}: {err}");
                break;
            }
        }
    }
}
