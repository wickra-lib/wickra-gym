//! Command-line arguments for the reference rollout runner.

use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// Drive a fixed deterministic policy through a gym-core environment.
#[derive(Parser, Debug)]
#[command(name = "wickra-gym", version, about)]
pub struct Args {
    /// Path to the environment spec (`.json` or `.toml`, chosen by extension).
    #[arg(long)]
    pub spec: PathBuf,
    /// Path to the candle dataset CSV.
    #[arg(long)]
    pub data: PathBuf,
    /// The deterministic policy to run.
    #[arg(long, value_enum, default_value_t = Policy::Always0)]
    pub policy: Policy,
    /// Override the spec's `episode.max_steps`.
    #[arg(long)]
    pub steps: Option<u32>,
    /// Episode seed (overrides the spec's default seed).
    #[arg(long)]
    pub seed: Option<u64>,
    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    pub format: Format,
}

/// A fixed, deterministic action policy.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Policy {
    /// Always take action `0`.
    #[value(name = "always-0")]
    Always0,
    /// Always take action `1`.
    #[value(name = "always-1")]
    Always1,
    /// Always take action `2`.
    #[value(name = "always-2")]
    Always2,
    /// Cycle `0, 1, 2, 0, …` by step index.
    Cycle,
}

impl Policy {
    /// The action this policy takes at the given step index.
    #[must_use]
    pub fn action(self, step: u32) -> f64 {
        match self {
            Policy::Always0 => 0.0,
            Policy::Always1 => 1.0,
            Policy::Always2 => 2.0,
            Policy::Cycle => f64::from(step % 3),
        }
    }
}

/// The output rendering.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Format {
    /// A human-readable aligned table.
    Text,
    /// Golden-format JSON (`{reset, trajectory}`).
    Json,
}
