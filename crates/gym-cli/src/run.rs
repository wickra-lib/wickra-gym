//! The rollout: load a spec + candles, drive the policy through the environment,
//! and render the trajectory.

use std::fmt::Write as _;
use std::path::Path;

use gym_core::{Candle, EnvSpec, ResetResult, StepResult};
use serde::Serialize;
use wickra_data::csv::CandleReader;

use crate::args::{Args, Format};

/// The golden-format rollout output (`{reset, trajectory}`); field order is
/// fixed so the JSON matches the golden fixtures byte for byte.
#[derive(Serialize)]
struct Rollout<'a> {
    reset: &'a ResetResult,
    trajectory: &'a [StepResult],
}

/// Load the candle CSV via `wickra-data` and map into gym candles. Only OHLCV is
/// carried; microstructure columns are absent (and default to `0.0`).
fn load_candles(path: &Path) -> Result<Vec<Candle>, String> {
    let mut reader = CandleReader::open(path).map_err(|e| e.to_string())?;
    let raw = reader.read_all().map_err(|e| e.to_string())?;
    Ok(raw
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
        .collect())
}

/// Load the spec from JSON or TOML, applying the optional `--steps` override.
fn load_spec(args: &Args) -> Result<EnvSpec, String> {
    let text = std::fs::read_to_string(&args.spec).map_err(|e| e.to_string())?;
    let is_toml = args
        .spec
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("toml"));
    let mut spec = if is_toml {
        EnvSpec::from_toml(&text)
    } else {
        EnvSpec::from_json(&text)
    }
    .map_err(|e| e.to_string())?;
    if let Some(steps) = args.steps {
        spec.episode.max_steps = steps;
    }
    Ok(spec)
}

/// Run the rollout and return the rendered output.
pub fn run(args: &Args) -> Result<String, String> {
    let spec = load_spec(args)?;
    let spec_json = serde_json::to_string(&spec).map_err(|e| e.to_string())?;
    let candles = load_candles(&args.data)?;

    let mut env = gym_core::Env::new(&spec_json).map_err(|e| e.to_string())?;
    env.load(&candles).map_err(|e| e.to_string())?;
    let reset = env.reset(args.seed).map_err(|e| e.to_string())?;

    let mut trajectory = Vec::new();
    let mut step = 0u32;
    loop {
        let action = args.policy.action(step);
        match env.step(action) {
            Ok(result) => {
                let done = result.terminated || result.truncated;
                trajectory.push(result);
                if done {
                    break;
                }
            }
            // EpisodeDone / any domain error ends the rollout cleanly.
            Err(_) => break,
        }
        step += 1;
    }

    match args.format {
        Format::Json => {
            let out = Rollout {
                reset: &reset,
                trajectory: &trajectory,
            };
            serde_json::to_string(&out).map_err(|e| e.to_string())
        }
        Format::Text => Ok(render_text(args, &trajectory)),
    }
}

/// Render the trajectory as an aligned table.
fn render_text(args: &Args, trajectory: &[StepResult]) -> String {
    let mut out = String::new();
    out.push_str(
        "step | action |       reward |          pos |       equity | terminated | truncated\n",
    );
    out.push_str(
        "-----+--------+--------------+--------------+--------------+------------+----------\n",
    );
    for (i, s) in trajectory.iter().enumerate() {
        let action = args.policy.action(i as u32);
        let pos = s.info.get("pos").copied().unwrap_or(0.0);
        let equity = s.info.get("equity").copied().unwrap_or(0.0);
        let _ = writeln!(
            out,
            "{i:>4} | {action:>6} | {:>12.8} | {pos:>12.8} | {equity:>12.8} | {:>10} | {:>9}",
            s.reward, s.terminated, s.truncated
        );
    }
    out
}
