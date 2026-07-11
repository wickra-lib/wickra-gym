//! Node.js bindings for wickra-gym — a thin wrapper over the `gym-core` command
//! surface. `Env` forwards command JSONs to [`gym_core::Env::command_json`]
//! verbatim, keeping the byte-identical cross-language JSON boundary as the
//! single source of truth.

use napi_derive::napi;

/// A gym environment handle: construct from a spec JSON, then drive it with
/// command JSONs (`load`, `reset`, `step`, `spec`, `version`).
#[napi]
pub struct Env(gym_core::Env);

#[napi]
impl Env {
    /// Construct from an `EnvSpec` JSON string.
    #[napi(constructor)]
    pub fn new(spec_json: String) -> napi::Result<Self> {
        gym_core::Env::new(&spec_json)
            .map(Env)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Apply a command JSON and return the response JSON string.
    #[napi]
    pub fn command(&mut self, cmd_json: String) -> napi::Result<String> {
        self.0
            .command_json(&cmd_json)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// The gym-core version.
    #[napi]
    pub fn version(&self) -> &'static str {
        gym_core::version()
    }
}
