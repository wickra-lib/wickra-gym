//! # gym-core
//!
//! The deterministic core of `wickra-gym`: a Gymnasium-compatible, O(1)-step
//! backtest environment. A JSON [`EnvSpec`] (observation features, action space,
//! reward kind, episode parameters) plus a candle dataset is precomputed once
//! into a fixed feature tensor, so each `step()` is a constant-time index into
//! that tensor. The only randomness is an explicitly-seeded episode RNG, so a
//! `(seed, policy)` pair fully determines a byte-identical trajectory across all
//! ten language bindings.
//!
//! Additional modules (`spec`, `indicator_set`, `tensor`, `reward`, `env`,
//! `config`) land phase by phase; this crate currently exposes the error type
//! and the observation [`Feature`] model.

mod error;
mod feature;

pub use error::{Error, Result};
pub use feature::{Feature, MicroField, PriceField};

/// The gym-core crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
