//! WebAssembly bindings for wickra-gym — a thin wrapper over the `gym-core`
//! command surface. `Env` forwards command JSONs to
//! [`gym_core::Env::command_json`] verbatim. The feature-tensor precompute runs
//! sequentially here (no rayon in the browser sandbox), which is byte-identical
//! to the native parallel run, so a wasm rollout matches every other language.

use wasm_bindgen::prelude::*;

/// A gym environment handle: construct from a spec JSON, then drive it with
/// command JSONs (`load`, `reset`, `step`, `spec`, `version`).
#[wasm_bindgen]
pub struct Env(gym_core::Env);

#[wasm_bindgen]
impl Env {
    /// Construct from an `EnvSpec` JSON string.
    #[wasm_bindgen(constructor)]
    pub fn new(spec_json: &str) -> Result<Env, JsValue> {
        gym_core::Env::new(spec_json)
            .map(Env)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Apply a command JSON and return the response JSON string.
    pub fn command(&mut self, cmd_json: &str) -> Result<String, JsValue> {
        self.0
            .command_json(cmd_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// The gym-core version.
    pub fn version() -> String {
        gym_core::version().into()
    }
}
