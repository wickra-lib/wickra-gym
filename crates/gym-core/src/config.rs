//! A thin configuration wrapper around [`EnvSpec`] for CLI spec files, in either
//! JSON or TOML form.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::spec::EnvSpec;

/// A loaded environment configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    /// The environment specification.
    pub spec: EnvSpec,
}

impl Config {
    /// Load a config whose top level *is* an [`EnvSpec`] from JSON.
    pub fn from_json(s: &str) -> Result<Self> {
        Ok(Self {
            spec: EnvSpec::from_json(s)?,
        })
    }

    /// Load a config whose top level *is* an [`EnvSpec`] from TOML.
    pub fn from_toml(s: &str) -> Result<Self> {
        Ok(Self {
            spec: EnvSpec::from_toml(s)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = r#"{
        "dataset_ref": "d",
        "symbol": "S",
        "observation": { "features": [{ "kind": "price", "field": "close" }] },
        "action_space": { "type": "discrete", "n": 3 },
        "reward": "pnl",
        "episode": { "max_steps": 5 }
    }"#;

    #[test]
    fn loads_from_json() {
        let cfg = Config::from_json(JSON).unwrap();
        assert_eq!(cfg.spec.symbol, "S");
        assert_eq!(cfg.spec.observation.feature_dim(), 1);
    }

    #[test]
    fn loads_from_toml() {
        let toml = r#"
            dataset_ref = "d"
            symbol = "S"
            reward = "pnl"

            [observation]
            features = [{ kind = "price", field = "close" }]

            [action_space]
            type = "discrete"
            n = 3

            [episode]
            max_steps = 5
        "#;
        let cfg = Config::from_toml(toml).unwrap();
        assert_eq!(
            cfg.spec.action_space,
            crate::spec::ActionSpace::Discrete { n: 3 }
        );
    }
}
