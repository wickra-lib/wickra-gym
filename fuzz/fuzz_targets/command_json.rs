#![no_main]
//! Fuzz the FFI command boundary — the surface every language binding forwards
//! verbatim. Arbitrary bytes are handed to `command_json` against a fresh env. It
//! must never panic: a bad envelope, an unknown command, a step before reset or
//! malformed data all come back in-band as `{"ok":false,...}` (or a valid
//! result), never a crash. Any returned response is re-parseable JSON.

use gym_core::Env;
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

const SPEC: &str = r#"{
    "dataset_ref": "fuzz", "symbol": "S",
    "observation": { "features": [{ "kind": "price", "field": "close" }] },
    "action_space": { "type": "discrete", "n": 3 },
    "reward": "pnl",
    "episode": { "max_steps": 8, "warmup": 0 }
}"#;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(mut env) = Env::new(SPEC) else {
        return;
    };
    if let Ok(response) = env.command_json(text) {
        let _: Value = serde_json::from_str(&response).expect("response is valid JSON");
    }
});
