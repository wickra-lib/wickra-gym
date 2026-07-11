//! # gym-core
//!
//! The deterministic core of `wickra-gym`: a Gymnasium-compatible, O(1)-step
//! backtest environment. A JSON [`EnvSpec`] (observation features, action space,
//! reward kind, episode parameters) plus a candle dataset is precomputed once
//! into a fixed [`FeatureTensor`], so each [`Env::step`] is a constant-time index
//! into that tensor. The only randomness is an explicitly-seeded episode RNG, so
//! a `(seed, policy)` pair fully determines a byte-identical trajectory across
//! all ten language bindings.
//!
//! The whole surface is reachable through one FFI entry point,
//! [`Env::command_json`], which every binding forwards verbatim — so
//! cross-language byte-equality is automatic.

mod config;
mod env;
mod error;
mod feature;
mod indicator_set;
mod reward;
mod spec;
mod tensor;

pub use config::Config;
pub use env::{build_tensor, Env};
pub use error::{Error, Result};
pub use feature::{Feature, MicroField, PriceField};
pub use indicator_set::IndicatorSet;
pub use reward::{running_sharpe, step_reward, RewardState};
pub use spec::{
    ActionSpace, ActionSpaceInfo, Candle, EnvSpec, EpisodeSpec, ObsSpec, ResetResult, RewardKind,
    SpecInfo, StepResult,
};
pub use tensor::FeatureTensor;

/// The gym-core crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_non_empty() {
        assert!(!version().is_empty());
    }

    #[test]
    fn end_to_end_pnl_rollout_is_deterministic() {
        let spec = r#"{
            "dataset_ref": "d",
            "symbol": "S",
            "observation": { "features": [
                { "kind": "price", "field": "close" },
                { "kind": "indicator", "name": "Sma", "params": [2] }
            ] },
            "action_space": { "type": "discrete", "n": 3 },
            "reward": "pnl",
            "episode": { "max_steps": 100, "warmup": 1 }
        }"#;
        let candles: Vec<Candle> = (0..8)
            .map(|i| Candle {
                ts: i,
                open: 100.0 + i as f64,
                high: 100.0 + i as f64,
                low: 100.0 + i as f64,
                close: 100.0 + i as f64,
                volume: 0.0,
                bid_px: vec![],
                bid_sz: vec![],
                ask_px: vec![],
                ask_sz: vec![],
                funding: None,
                oi: None,
            })
            .collect();

        let run = |seed: u64| {
            let mut env = Env::new(spec).unwrap();
            env.load(&candles).unwrap();
            let mut trace = Vec::new();
            trace.push(serde_json::to_string(&env.reset(Some(seed)).unwrap()).unwrap());
            for _ in 0..6 {
                trace.push(serde_json::to_string(&env.step(2.0).unwrap()).unwrap());
            }
            trace
        };
        // Same seed → byte-identical trajectory.
        assert_eq!(run(7), run(7));
    }
}
