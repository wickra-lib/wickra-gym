#![no_main]
//! Fuzz the feature-tensor precompute. Arbitrary candle payloads (bounded in
//! length) with a fixed price/microstructure observation must build without a
//! panic — non-finite prices collapse to `0.0`, missing columns to `0.0`.

use gym_core::{build_tensor, Candle, Feature, MicroField, ObsSpec, PriceField};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(candles) = serde_json::from_str::<Vec<Candle>>(text) else {
        return;
    };
    if candles.len() > 20_000 {
        return; // keep the corpus bounded
    }
    let obs = ObsSpec {
        features: vec![
            Feature::Price {
                field: PriceField::Close,
            },
            Feature::Microstructure {
                field: MicroField::Imbalance,
                level: None,
            },
        ],
        include_book_levels: 1,
        include_funding_oi: true,
    };
    let _ = build_tensor(&candles, &obs);
});
