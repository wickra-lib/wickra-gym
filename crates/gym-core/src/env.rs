//! The environment handle: a spec, a precomputed [`FeatureTensor`], and the
//! rollout cursor. `command_json` is the single FFI entry point every binding
//! forwards verbatim, so cross-language byte-equality is automatic.

use std::collections::BTreeMap;

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use serde::Deserialize;
use serde_json::json;

use crate::error::{Error, Result};
use crate::reward::{running_sharpe, step_reward, RewardState};
use crate::spec::{round8, Candle, EnvSpec, ResetResult, SpecInfo, StepResult};
use crate::tensor::{self, FeatureTensor};

/// The pinned episode RNG. It is reserved for future random episode-start
/// offsets; v1 always draws `0`, so a trajectory depends only on policy + data.
type PinnedRng = StdRng;

/// The mutable rollout cursor for one episode.
struct RolloutState {
    bar: usize,
    steps: u32,
    pos: f64,
    equity: f64,
    reset_done: bool,
    reward: RewardState,
    rng: PinnedRng,
}

impl RolloutState {
    fn new() -> Self {
        Self {
            bar: 0,
            steps: 0,
            pos: 0.0,
            equity: 0.0,
            reset_done: false,
            reward: RewardState::default(),
            rng: StdRng::seed_from_u64(0),
        }
    }
}

/// A Gymnasium-style backtest environment.
pub struct Env {
    spec: EnvSpec,
    tensor: Option<FeatureTensor>,
    state: RolloutState,
}

/// The `command_json` request envelope. Only the fields relevant to the given
/// `cmd` are read.
#[derive(Deserialize)]
struct Command {
    cmd: String,
    #[serde(default)]
    candles: Vec<Candle>,
    #[serde(default)]
    seed: Option<u64>,
    #[serde(default)]
    action: Option<f64>,
}

impl Env {
    /// Construct an environment from a JSON [`EnvSpec`]. No data is loaded yet.
    pub fn new(spec_json: &str) -> Result<Self> {
        let spec = EnvSpec::from_json(spec_json)?;
        Ok(Self {
            spec,
            tensor: None,
            state: RolloutState::new(),
        })
    }

    /// The crate version.
    #[must_use]
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Load the episode candles and precompute the feature tensor. Resets the
    /// rollout cursor (a fresh `reset` is required before stepping).
    pub fn load(&mut self, candles: &[Candle]) -> Result<()> {
        let tensor = tensor::build(candles, &self.spec.observation)?;
        self.tensor = Some(tensor);
        self.state = RolloutState::new();
        Ok(())
    }

    /// Reset the episode to the warmup bar and return the first observation.
    pub fn reset(&mut self, seed: Option<u64>) -> Result<ResetResult> {
        let tensor = self.tensor.as_ref().ok_or(Error::NoData)?;
        let resolved_seed = seed.or(self.spec.seed).unwrap_or(0);
        self.state = RolloutState::new();
        self.state.rng = StdRng::seed_from_u64(resolved_seed);
        // v1: the start offset is always 0 (deterministic); the RNG draw keeps
        // the seed plumbing live for future randomized starts.
        let max_offset: usize = 0;
        let offset = self.state.rng.random_range(0..=max_offset);
        let bar = self.spec.episode.warmup as usize + offset;
        if bar >= tensor.n_bars {
            return Err(Error::Data("warmup exceeds the loaded dataset".to_string()));
        }
        self.state.bar = bar;
        self.state.reset_done = true;
        let observation = tensor.row(bar).to_vec();
        let info = self.info_map();
        Ok(ResetResult { observation, info })
    }

    /// Advance one step: apply the action as a target position, realize the
    /// reward over the next bar, and return the next observation.
    pub fn step(&mut self, action: f64) -> Result<StepResult> {
        let tensor = self.tensor.as_ref().ok_or(Error::NoData)?;
        if !self.state.reset_done {
            return Err(Error::NotReset);
        }
        let n = tensor.n_bars;
        // Already terminal from a previous step → no further stepping.
        if self.state.bar + 1 >= n || self.state.steps >= self.spec.episode.max_steps {
            return Err(Error::EpisodeDone);
        }
        let bar = self.state.bar;
        let target = self.spec.action_space.target(action)?;
        // The position change fills at this bar's close; the next bar realizes it.
        self.state.pos = target;
        let close_now = tensor.closes[bar];
        let close_next = tensor.closes[bar + 1];
        let reward = step_reward(
            self.spec.reward,
            self.state.pos,
            close_now,
            close_next,
            &mut self.state.reward,
        );
        self.state.equity += reward;
        self.state.bar = bar + 1;
        self.state.steps += 1;

        let terminated = self.state.bar + 1 >= n;
        let truncated = self.state.steps >= self.spec.episode.max_steps;
        let observation = tensor.row(self.state.bar).to_vec();
        let info = self.info_map();
        Ok(StepResult {
            observation,
            reward: round8(reward),
            terminated,
            truncated,
            info,
        })
    }

    /// The derived space description (§6.8).
    pub fn spec_info(&self) -> Result<SpecInfo> {
        Ok(self.spec.spec_info())
    }

    /// Dispatch one `command_json` envelope. Domain errors are returned in-band
    /// as `{"ok":false,"error":...}`; the outer `Result` is only for the API
    /// shape and, in practice, always `Ok` for a well-formed call.
    pub fn command_json(&mut self, cmd_json: &str) -> Result<String> {
        match self.dispatch(cmd_json) {
            Ok(s) => Ok(s),
            Err(e) => Ok(json!({ "ok": false, "error": e.to_string() }).to_string()),
        }
    }

    fn dispatch(&mut self, cmd_json: &str) -> Result<String> {
        let cmd: Command =
            serde_json::from_str(cmd_json).map_err(|e| Error::Parse(e.to_string()))?;
        match cmd.cmd.as_str() {
            "load" => {
                self.load(&cmd.candles)?;
                Ok(json!({ "ok": true }).to_string())
            }
            "reset" => {
                let result = self.reset(cmd.seed)?;
                to_json(&result)
            }
            "step" => {
                let action = cmd
                    .action
                    .ok_or_else(|| Error::BadAction("missing action".to_string()))?;
                let result = self.step(action)?;
                to_json(&result)
            }
            "spec" => {
                let info = self.spec_info()?;
                to_json(&info)
            }
            "version" => Ok(json!({ "version": Env::version() }).to_string()),
            other => Err(Error::BadSpec(format!("unknown cmd: {other}"))),
        }
    }

    /// The deterministic info block (`bar`, `pos`, `equity`, `step`, plus
    /// `sharpe` for the Sharpe reward). All values pass through `round8`.
    fn info_map(&self) -> BTreeMap<String, f64> {
        let mut info = BTreeMap::new();
        info.insert("bar".to_string(), round8(self.state.bar as f64));
        info.insert("pos".to_string(), round8(self.state.pos));
        info.insert("equity".to_string(), round8(self.state.equity));
        info.insert("step".to_string(), round8(f64::from(self.state.steps)));
        if matches!(self.spec.reward, crate::spec::RewardKind::Sharpe) {
            info.insert(
                "sharpe".to_string(),
                round8(running_sharpe(&self.state.reward)),
            );
        }
        info
    }
}

/// Precompute a dataset into a [`FeatureTensor`] (the free-function form of the
/// per-episode build, for the CLI / tests / bindings).
pub fn build_tensor(candles: &[Candle], obs: &crate::spec::ObsSpec) -> Result<FeatureTensor> {
    tensor::build(candles, obs)
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|e| Error::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = r#"{
        "dataset_ref": "test",
        "symbol": "TEST",
        "observation": { "features": [{ "kind": "price", "field": "close" }] },
        "action_space": { "type": "discrete", "n": 3 },
        "reward": "pnl",
        "episode": { "max_steps": 10, "warmup": 0 }
    }"#;

    fn candles_json(n: usize) -> String {
        let items: Vec<String> = (0..n)
            .map(|i| {
                let close = 100.0 + i as f64;
                format!(
                    r#"{{"ts":{i},"open":{close},"high":{close},"low":{close},"close":{close}}}"#
                )
            })
            .collect();
        format!(r#"{{"cmd":"load","candles":[{}]}}"#, items.join(","))
    }

    #[test]
    fn version_command() {
        let mut env = Env::new(SPEC).unwrap();
        let out = env.command_json(r#"{"cmd":"version"}"#).unwrap();
        assert!(out.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn step_before_load_is_no_data() {
        let mut env = Env::new(SPEC).unwrap();
        let out = env.command_json(r#"{"cmd":"reset"}"#).unwrap();
        assert!(out.contains("no data"));
    }

    #[test]
    fn step_before_reset_is_not_reset() {
        let mut env = Env::new(SPEC).unwrap();
        env.command_json(&candles_json(5)).unwrap();
        let out = env.command_json(r#"{"cmd":"step","action":2}"#).unwrap();
        assert!(out.contains("not reset"));
    }

    #[test]
    fn full_rollout_and_episode_done() {
        let mut env = Env::new(SPEC).unwrap();
        assert_eq!(
            env.command_json(&candles_json(4)).unwrap(),
            r#"{"ok":true}"#
        );
        env.command_json(r#"{"cmd":"reset"}"#).unwrap();
        // 4 bars → 3 possible steps (bar+1 must exist). Long the whole way.
        let s1 = env.step(2.0).unwrap();
        assert!((s1.reward - 1.0).abs() < 1e-9); // pos +1 * (101-100)
        assert!(!s1.terminated);
        let _ = env.step(2.0).unwrap();
        let s3 = env.step(2.0).unwrap();
        assert!(s3.terminated); // bar advanced to 3 == n-1
                                // Any further step errors, not panics.
        let out = env.command_json(r#"{"cmd":"step","action":2}"#).unwrap();
        assert!(out.contains("episode done"));
    }

    #[test]
    fn unknown_cmd_and_bad_action() {
        let mut env = Env::new(SPEC).unwrap();
        env.command_json(&candles_json(4)).unwrap();
        env.command_json(r#"{"cmd":"reset"}"#).unwrap();
        assert!(env
            .command_json(r#"{"cmd":"frobnicate"}"#)
            .unwrap()
            .contains("unknown cmd"));
        assert!(env
            .command_json(r#"{"cmd":"step","action":9}"#)
            .unwrap()
            .contains("bad action"));
    }

    #[test]
    fn deterministic_reset_is_identical() {
        let mut env = Env::new(SPEC).unwrap();
        env.command_json(&candles_json(6)).unwrap();
        let a = env.command_json(r#"{"cmd":"reset","seed":42}"#).unwrap();
        let b = env.command_json(r#"{"cmd":"reset","seed":42}"#).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn spec_info_has_finite_bounds() {
        let env = Env::new(SPEC).unwrap();
        let info = env.spec_info().unwrap();
        assert_eq!(info.observation_dim, 1);
        assert_eq!(info.observation_low, vec![-1e30]);
        assert_eq!(info.feature_keys, vec!["price.close".to_string()]);
    }
}
