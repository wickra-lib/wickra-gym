//! The error type for gym-core. Domain errors travel in-band across the FFI
//! boundary as `{"ok":false,"error":...}` JSON; only unusable FFI arguments and
//! caught panics are signalled out of band (see the C ABI hub).

/// A gym-core error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A spec, candle payload or command envelope failed to parse.
    #[error("parse: {0}")]
    Parse(String),
    /// A feature references an indicator the registry does not know.
    #[error("unknown indicator: {0}")]
    UnknownIndicator(String),
    /// The spec is structurally invalid (bad field, out-of-range parameter).
    #[error("bad spec: {0}")]
    BadSpec(String),
    /// An action is outside the declared action space.
    #[error("bad action: {0}")]
    BadAction(String),
    /// The candle data is malformed or missing a required column.
    #[error("data: {0}")]
    Data(String),
    /// A `reset` or `step` was issued before any data was loaded.
    #[error("no data loaded")]
    NoData,
    /// A `step` was issued before the first `reset`.
    #[error("not reset")]
    NotReset,
    /// A `step` was issued after the episode terminated or truncated.
    #[error("episode done")]
    EpisodeDone,
}

/// The gym-core result type.
pub type Result<T> = core::result::Result<T, Error>;
