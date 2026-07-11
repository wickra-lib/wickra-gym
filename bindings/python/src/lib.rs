//! Python bindings for wickra-gym — a thin wrapper over the `gym-core` command
//! surface. `RawEnv` forwards command JSONs to [`gym_core::Env::command_json`]
//! verbatim; the real `gymnasium.Env` subclass lives in Python
//! (`wickra_gym.WickraGymEnv`), keeping the byte-identical cross-language JSON
//! boundary as the single source of truth.

use pyo3::prelude::*;

/// A raw environment handle: create from a spec JSON, then drive it with command
/// JSONs (`load`, `reset`, `step`, `spec`, `version`).
#[pyclass(unsendable)]
struct RawEnv(gym_core::Env);

#[pymethods]
impl RawEnv {
    /// Construct from an [`EnvSpec`] JSON string.
    #[new]
    fn new(spec_json: &str) -> PyResult<Self> {
        gym_core::Env::new(spec_json)
            .map(RawEnv)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Apply a command JSON and return the response JSON string.
    fn command(&mut self, cmd_json: &str) -> PyResult<String> {
        self.0
            .command_json(cmd_json)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    /// The gym-core version.
    #[staticmethod]
    fn version() -> &'static str {
        gym_core::version()
    }
}

/// The native extension module (`wickra_gym._wickra_gym`).
#[pymodule]
fn _wickra_gym(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RawEnv>()?;
    m.add("__version__", gym_core::version())?;
    Ok(())
}
