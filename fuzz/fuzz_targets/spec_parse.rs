#![no_main]
//! Fuzz spec parsing/validation. Arbitrary bytes handed to `EnvSpec::from_json`
//! must never panic — a malformed or invalid spec is an `Err`, never a crash.

use gym_core::EnvSpec;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = EnvSpec::from_json(text);
    }
});
