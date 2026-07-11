//! The environment data model: the [`Candle`] input, the [`EnvSpec`] tree
//! (observation / action space / reward / episode) and the [`ResetResult`] /
//! [`StepResult`] / [`SpecInfo`] outputs. Every type here is the cross-language
//! JSON wire contract, so field names and tags are load-bearing.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::feature::Feature;

/// Round to a fixed grid of `1e-8`; non-finite values collapse to `0.0`. Every
/// observation / reward / info value passes through this before serialization,
/// so the JSON is byte-identical across languages and never contains `NaN`/`inf`.
#[must_use]
pub(crate) fn round8(x: f64) -> f64 {
    if x.is_finite() {
        (x * 1e8).round() / 1e8
    } else {
        0.0
    }
}

/// One input bar. `ts` is epoch seconds (matching `wickra-data`); the optional
/// microstructure columns feed the [`Feature::Microstructure`] observation
/// fields and the `include_book_levels` / `include_funding_oi` blocks. Absent
/// columns yield `0.0`, never an error.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Candle {
    /// Bar open time (epoch seconds).
    pub ts: i64,
    /// Open price.
    pub open: f64,
    /// High price.
    pub high: f64,
    /// Low price.
    pub low: f64,
    /// Close price.
    pub close: f64,
    /// Bar volume (defaults to `0.0`).
    #[serde(default)]
    pub volume: f64,
    /// Bid prices per orderbook level (level 0 = best bid).
    #[serde(default)]
    pub bid_px: Vec<f64>,
    /// Bid sizes per orderbook level.
    #[serde(default)]
    pub bid_sz: Vec<f64>,
    /// Ask prices per orderbook level (level 0 = best ask).
    #[serde(default)]
    pub ask_px: Vec<f64>,
    /// Ask sizes per orderbook level.
    #[serde(default)]
    pub ask_sz: Vec<f64>,
    /// Funding rate for this bar (perpetuals).
    #[serde(default)]
    pub funding: Option<f64>,
    /// Open interest for this bar.
    #[serde(default)]
    pub oi: Option<f64>,
}

/// The observation definition: an ordered feature list plus optional raw
/// orderbook / funding-OI blocks. The order here *is* the observation-vector
/// order, and therefore the basis of the determinism contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct ObsSpec {
    /// Declared observation columns, in vector order.
    pub features: Vec<Feature>,
    /// `N` orderbook levels appended as `4*N` columns (`bid_px,bid_sz,ask_px,ask_sz`
    /// per level); `0` = no raw book block.
    #[serde(default)]
    pub include_book_levels: u32,
    /// When `true`, append `funding` and `oi` columns.
    #[serde(default)]
    pub include_funding_oi: bool,
}

impl ObsSpec {
    /// The fixed observation-vector length: features, then `4` per book level,
    /// then `2` if funding/OI is included.
    #[must_use]
    pub fn feature_dim(&self) -> usize {
        self.features.len()
            + 4 * self.include_book_levels as usize
            + if self.include_funding_oi { 2 } else { 0 }
    }

    /// The canonical per-column keys in vector order (§6.5): declared features,
    /// then the book-level columns, then `micro.funding` / `micro.oi`.
    #[must_use]
    pub fn feature_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.features.iter().map(Feature::key).collect();
        for level in 0..self.include_book_levels {
            keys.push(format!("micro.bid_px.{level}"));
            keys.push(format!("micro.bid_sz.{level}"));
            keys.push(format!("micro.ask_px.{level}"));
            keys.push(format!("micro.ask_sz.{level}"));
        }
        if self.include_funding_oi {
            keys.push("micro.funding".to_string());
            keys.push("micro.oi".to_string());
        }
        keys
    }
}

/// The action space. `Discrete{n}` maps an integer bucket to a target position
/// on `[-1,+1]`; `Continuous{low,high}` clamps a float to that band.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionSpace {
    /// `n` buckets mapped linearly onto `[-1,+1]`.
    Discrete { n: u32 },
    /// A continuous target position, clamped to `[low, high]`.
    Continuous { low: f64, high: f64 },
}

impl ActionSpace {
    /// Map a raw action to a target position (§6.6). Out-of-range discrete
    /// actions are a [`Error::BadAction`].
    pub(crate) fn target(self, action: f64) -> Result<f64> {
        match self {
            ActionSpace::Discrete { n } => {
                if action.fract() != 0.0 || action < 0.0 || action as i64 >= i64::from(n) {
                    return Err(Error::BadAction(format!(
                        "discrete action {action} not in 0..{n}"
                    )));
                }
                if n <= 1 {
                    Ok(0.0)
                } else {
                    Ok(-1.0 + 2.0 * action / f64::from(n - 1))
                }
            }
            ActionSpace::Continuous { low, high } => Ok(action.clamp(low, high)),
        }
    }

    fn to_info(self) -> ActionSpaceInfo {
        match self {
            ActionSpace::Discrete { n } => ActionSpaceInfo::Discrete { n },
            ActionSpace::Continuous { low, high } => ActionSpaceInfo::Continuous { low, high },
        }
    }
}

/// The reward signal (§6.6).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RewardKind {
    /// Position times the next bar's price delta.
    Pnl,
    /// Dense increment of the running Sharpe ratio of the PnL series.
    Sharpe,
    /// Position times the next bar's log return.
    LogReturn,
}

/// Episode parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EpisodeSpec {
    /// The truncation limit: the number of `step()` calls per episode.
    pub max_steps: u32,
    /// Bars skipped so indicators are ready; `reset()` starts at `bar = warmup`.
    #[serde(default)]
    pub warmup: u32,
}

/// The full environment specification — the JSON that constructs an [`crate::Env`].
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EnvSpec {
    /// Logical dataset name (referenced by golden fixtures / the CLI).
    pub dataset_ref: String,
    /// The instrument symbol.
    pub symbol: String,
    /// The observation definition.
    pub observation: ObsSpec,
    /// The action space.
    pub action_space: ActionSpace,
    /// The reward signal.
    pub reward: RewardKind,
    /// Episode parameters.
    pub episode: EpisodeSpec,
    /// Default episode seed; a `reset(seed)` argument overrides it.
    #[serde(default)]
    pub seed: Option<u64>,
}

impl EnvSpec {
    /// Parse and validate an [`EnvSpec`] from JSON.
    pub fn from_json(s: &str) -> Result<Self> {
        let spec: EnvSpec = serde_json::from_str(s).map_err(|e| Error::Parse(e.to_string()))?;
        spec.validate()?;
        Ok(spec)
    }

    /// Parse and validate an [`EnvSpec`] from TOML.
    pub fn from_toml(s: &str) -> Result<Self> {
        let spec: EnvSpec = toml::from_str(s).map_err(|e| Error::Parse(e.to_string()))?;
        spec.validate()?;
        Ok(spec)
    }

    /// Structural validation: positive episode length, well-formed action space,
    /// and every referenced indicator resolvable in the registry.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.episode.max_steps == 0 {
            return Err(Error::BadSpec("episode.max_steps must be > 0".to_string()));
        }
        match self.action_space {
            ActionSpace::Discrete { n } => {
                if n == 0 {
                    return Err(Error::BadSpec(
                        "discrete action space needs n >= 1".to_string(),
                    ));
                }
            }
            ActionSpace::Continuous { low, high } => {
                if low.is_nan() || high.is_nan() || low >= high {
                    return Err(Error::BadSpec(
                        "continuous action space needs low < high".to_string(),
                    ));
                }
            }
        }
        // Constructing the set resolves every indicator by name+params, mapping
        // an unknown one to Error::UnknownIndicator.
        crate::indicator_set::IndicatorSet::from_obs(&self.observation)?;
        Ok(())
    }

    /// The derived space description used by the Gymnasium layer (§6.8).
    #[must_use]
    pub fn spec_info(&self) -> SpecInfo {
        let dim = self.observation.feature_dim();
        SpecInfo {
            observation_dim: dim,
            observation_low: vec![-1e30; dim],
            observation_high: vec![1e30; dim],
            action_space: self.action_space.to_info(),
            feature_keys: self.observation.feature_keys(),
        }
    }
}

/// The reset observation and its info block.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ResetResult {
    /// The observation at the warmup bar.
    pub observation: Vec<f64>,
    /// Deterministic info (`bar`, `pos`, `equity`, `step`).
    pub info: BTreeMap<String, f64>,
}

/// A single step's observation, reward, termination flags and info block.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepResult {
    /// The observation after the step.
    pub observation: Vec<f64>,
    /// The step reward.
    pub reward: f64,
    /// `true` once the dataset is exhausted.
    pub terminated: bool,
    /// `true` once `max_steps` is reached.
    pub truncated: bool,
    /// Deterministic info (`bar`, `pos`, `equity`, `step`, plus `sharpe` for
    /// [`RewardKind::Sharpe`]). A `BTreeMap` pins the JSON key order.
    pub info: BTreeMap<String, f64>,
}

/// The action-space shape echoed in [`SpecInfo`].
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionSpaceInfo {
    /// A discrete space of `n` buckets.
    Discrete { n: u32 },
    /// A continuous `[low, high]` band.
    Continuous { low: f64, high: f64 },
}

/// The derived space description (§6.8). Bounds are finite sentinels
/// (`±1e30`) because `serde_json` cannot serialize `±inf`; the Python layer maps
/// them back to `±np.inf` when building the Gymnasium `Box`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SpecInfo {
    /// The observation-vector length.
    pub observation_dim: usize,
    /// Per-column lower bounds (sentinel `-1e30`).
    pub observation_low: Vec<f64>,
    /// Per-column upper bounds (sentinel `+1e30`).
    pub observation_high: Vec<f64>,
    /// The action-space shape.
    pub action_space: ActionSpaceInfo,
    /// The canonical per-column keys in vector order.
    pub feature_keys: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::PriceField;

    /// Grid-exact equality within a tolerance well below the `1e-8` round grid.
    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn obs(features: Vec<Feature>, levels: u32, funding_oi: bool) -> ObsSpec {
        ObsSpec {
            features,
            include_book_levels: levels,
            include_funding_oi: funding_oi,
        }
    }

    #[test]
    fn feature_dim_and_keys_follow_canonical_order() {
        let o = obs(
            vec![
                Feature::Price {
                    field: PriceField::Close,
                },
                Feature::Indicator {
                    name: "rsi".into(),
                    params: vec![14.0],
                    field: None,
                },
            ],
            2,
            true,
        );
        // 2 features + 4*2 book columns + 2 funding/oi = 12.
        assert_eq!(o.feature_dim(), 12);
        let keys = o.feature_keys();
        assert_eq!(keys.len(), 12);
        assert_eq!(keys[0], "price.close");
        assert_eq!(keys[1], "rsi(14)");
        assert_eq!(keys[2], "micro.bid_px.0");
        assert_eq!(keys[5], "micro.ask_sz.0");
        assert_eq!(keys[6], "micro.bid_px.1");
        assert_eq!(keys[10], "micro.funding");
        assert_eq!(keys[11], "micro.oi");
    }

    #[test]
    fn discrete_action_maps_to_band() {
        let a = ActionSpace::Discrete { n: 3 };
        assert!(approx(a.target(0.0).unwrap(), -1.0));
        assert!(approx(a.target(1.0).unwrap(), 0.0));
        assert!(approx(a.target(2.0).unwrap(), 1.0));
        assert!(a.target(3.0).is_err());
        assert!(a.target(-1.0).is_err());
        assert!(a.target(1.5).is_err());
        // n == 1 always yields flat.
        assert!(approx(
            ActionSpace::Discrete { n: 1 }.target(0.0).unwrap(),
            0.0
        ));
    }

    #[test]
    fn continuous_action_clamps() {
        let a = ActionSpace::Continuous {
            low: -0.5,
            high: 0.5,
        };
        assert!(approx(a.target(0.2).unwrap(), 0.2));
        assert!(approx(a.target(9.0).unwrap(), 0.5));
        assert!(approx(a.target(-9.0).unwrap(), -0.5));
    }

    #[test]
    fn validate_rejects_bad_specs() {
        let mut spec = EnvSpec {
            dataset_ref: "d".into(),
            symbol: "S".into(),
            observation: obs(vec![], 0, false),
            action_space: ActionSpace::Discrete { n: 3 },
            reward: RewardKind::Pnl,
            episode: EpisodeSpec {
                max_steps: 0,
                warmup: 0,
            },
            seed: None,
        };
        assert!(spec.validate().is_err()); // max_steps == 0
        spec.episode.max_steps = 10;
        assert!(spec.validate().is_ok());
        spec.action_space = ActionSpace::Continuous {
            low: 1.0,
            high: 1.0,
        };
        assert!(spec.validate().is_err()); // low !< high
    }

    #[test]
    fn round8_collapses_non_finite() {
        assert!(approx(round8(f64::NAN), 0.0));
        assert!(approx(round8(f64::INFINITY), 0.0));
        assert!(approx(round8(1.234_567_894_9), 1.234_567_89));
    }
}
