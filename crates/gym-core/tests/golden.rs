//! Rust golden replay — the Rust core is the ninth language in the cross-language
//! determinism proof. Loads each committed golden case, replays the fixed policy
//! and asserts the reset / trajectory match value-for-value.

use std::fs;
use std::path::{Path, PathBuf};

use gym_core::{Candle, Env};
use serde_json::Value;

fn golden_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../golden")
}

#[test]
fn golden_rollouts_match() {
    let root = golden_root();
    let mut ran = 0;
    for entry in fs::read_dir(&root).expect("golden dir") {
        let dir = entry.unwrap().path();
        if !dir.is_dir() || !dir.join("spec.json").exists() {
            continue;
        }

        let spec = fs::read_to_string(dir.join("spec.json")).unwrap();
        let candles: Vec<Candle> =
            serde_json::from_str(&fs::read_to_string(dir.join("candles.json")).unwrap()).unwrap();
        let expected: Value =
            serde_json::from_str(&fs::read_to_string(dir.join("expected.json")).unwrap()).unwrap();

        let mut env = Env::new(&spec).unwrap();
        env.load(&candles).unwrap();

        let seed = expected.get("seed").and_then(Value::as_u64);
        let reset = env.reset(seed).unwrap();
        assert_eq!(
            serde_json::to_value(&reset).unwrap(),
            expected["reset"],
            "{}: reset",
            dir.display()
        );

        let actions = expected["actions"].as_array().unwrap();
        let trajectory = expected["trajectory"].as_array().unwrap();
        assert_eq!(actions.len(), trajectory.len());
        for (i, action) in actions.iter().enumerate() {
            let step = env.step(action.as_f64().unwrap()).unwrap();
            assert_eq!(
                serde_json::to_value(&step).unwrap(),
                trajectory[i],
                "{}: step {i}",
                dir.display()
            );
        }
        ran += 1;
    }
    assert!(ran > 0, "no golden cases found at {}", root.display());
}
