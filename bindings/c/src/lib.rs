//! The wickra-gym C ABI — the hub every C-capable language links against.
//!
//! The surface is deliberately tiny and JSON-shaped, exactly like
//! [`gym_core::Env::command_json`]: create an environment handle from a spec
//! JSON, drive it with command JSONs (`load`, `reset`, `step`, `spec`,
//! `version`) and read back response JSONs, then free the handle. No result type
//! crosses the boundary by value — the handle is opaque and payloads are always
//! UTF-8 JSON strings.
//!
//! Responses use a caller-owned buffer with a length-out protocol (the classic
//! C two-call idiom):
//!
//! 1. Call [`wickra_gym_command`] with `out = NULL`, `cap = 0` to learn the
//!    response length `len` (excluding the terminating NUL).
//! 2. Allocate `len + 1` bytes and call again; the response plus a NUL is
//!    written into `out`.
//!
//! **Mutating commands and the response cache.** Unlike a read-only surface,
//! `step` mutates the environment, so the two-call idiom must not execute it
//! twice. Each handle therefore caches the response of the command it last
//! *computed but not yet delivered* (`pending`). A repeated call with the same
//! command bytes reuses that cached response instead of re-executing; once the
//! response is successfully written to a buffer, the cache is cleared so the
//! next identical command executes freshly. A logical command is thus executed
//! exactly once, no matter how many buffer-sizing retries it takes.
//!
//! Negative returns are reserved for unusable arguments
//! ([`WICKRA_GYM_ERR_NULL`], [`WICKRA_GYM_ERR_UTF8`]) and caught panics
//! ([`WICKRA_GYM_ERR_PANIC`]); a non-negative return is always the response
//! length. Domain errors (a bad spec, an out-of-range action) are *not*
//! negative — they come back in-band as `{"ok":false,"error":...}` JSON.

use core::ffi::{c_char, CStr};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use gym_core::Env;

/// A required pointer argument (`handle` or `cmd_json`) was null.
pub const WICKRA_GYM_ERR_NULL: i32 = -1;
/// `cmd_json` was not valid UTF-8.
pub const WICKRA_GYM_ERR_UTF8: i32 = -2;
/// A panic was caught at the FFI boundary.
pub const WICKRA_GYM_ERR_PANIC: i32 = -3;

/// An opaque handle to a gym environment. Created by [`wickra_gym_new`] and
/// destroyed by [`wickra_gym_free`]; never dereferenced by the caller.
pub struct WickraGymEnv {
    inner: Env,
    /// The last command computed but not yet delivered: `(cmd_bytes, response)`.
    /// See the module docs for the mutating-command cache contract.
    pending: Option<(Vec<u8>, String)>,
}

/// Read a NUL-terminated C string as `&str`, or `None` on null / bad UTF-8.
///
/// # Safety
/// `ptr` must be null or a valid NUL-terminated C string.
unsafe fn opt_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

/// Construct an environment handle from a spec JSON. Returns null if the spec
/// pointer is null / not UTF-8 or the spec fails to parse or validate.
///
/// # Safety
/// `spec_json` must be null or a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn wickra_gym_new(spec_json: *const c_char) -> *mut WickraGymEnv {
    let Some(json) = (unsafe { opt_str(spec_json) }) else {
        return ptr::null_mut();
    };
    match catch_unwind(AssertUnwindSafe(|| Env::new(json))) {
        Ok(Ok(inner)) => Box::into_raw(Box::new(WickraGymEnv {
            inner,
            pending: None,
        })),
        _ => ptr::null_mut(),
    }
}

/// Destroy an environment handle. Null is a no-op.
///
/// # Safety
/// `handle` must be null or a handle previously returned by [`wickra_gym_new`]
/// and not already freed.
#[no_mangle]
pub unsafe extern "C" fn wickra_gym_free(handle: *mut WickraGymEnv) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

/// Apply a command JSON and write the response JSON into the caller's buffer.
///
/// Returns the response length in bytes (excluding the terminating NUL), or a
/// negative error code. When the return value `len` satisfies `len < cap`, the
/// response and a trailing NUL have been written to `out`; otherwise `out` is
/// left untouched and the caller should re-call with a `cap` of at least
/// `len + 1`. Pass `out = NULL`, `cap = 0` to query the length without writing.
/// A mutating command is executed exactly once across all such retries.
///
/// # Safety
/// `handle` must be a valid handle; `cmd_json` a valid NUL-terminated C string;
/// `out` either null or a writable buffer of at least `cap` bytes.
#[no_mangle]
pub unsafe extern "C" fn wickra_gym_command(
    handle: *mut WickraGymEnv,
    cmd_json: *const c_char,
    out: *mut c_char,
    cap: usize,
) -> i32 {
    if handle.is_null() || cmd_json.is_null() {
        return WICKRA_GYM_ERR_NULL;
    }
    let Some(cmd) = (unsafe { opt_str(cmd_json) }) else {
        return WICKRA_GYM_ERR_UTF8;
    };
    let env = unsafe { &mut *handle };

    // Reuse the cached response for an identical, not-yet-delivered command;
    // otherwise execute once and cache the result.
    let is_retry = matches!(&env.pending, Some((bytes, _)) if bytes.as_slice() == cmd.as_bytes());
    if !is_retry {
        let response = match catch_unwind(AssertUnwindSafe(|| env.inner.command_json(cmd))) {
            // `command_json` folds domain errors into `{"ok":false,...}` JSON, so
            // a top-level `Err` should not occur; surface it in-band all the same.
            Ok(result) => result.unwrap_or_else(|err| {
                format!(
                    "{{\"ok\":false,\"error\":{}}}",
                    json_string(&err.to_string())
                )
            }),
            Err(_) => return WICKRA_GYM_ERR_PANIC,
        };
        env.pending = Some((cmd.as_bytes().to_vec(), response));
    }

    let (len, delivered) = {
        let response = &env.pending.as_ref().expect("pending set above").1;
        let bytes = response.as_bytes();
        let len = bytes.len();
        let delivered = len < cap && !out.is_null();
        if delivered {
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), out.cast::<u8>(), len);
                *out.add(len) = 0;
            }
        }
        (len, delivered)
    };
    // The response has been delivered: clear the cache so the next identical
    // command executes freshly.
    if delivered {
        env.pending = None;
    }
    i32::try_from(len).unwrap_or(i32::MAX)
}

/// The library version as a static NUL-terminated string (do not free).
#[no_mangle]
pub extern "C" fn wickra_gym_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0")
        .as_ptr()
        .cast::<c_char>()
}

/// Encode a string as a JSON string literal (quotes + minimal escaping).
fn json_string(s: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    const SPEC: &str = r#"{
        "dataset_ref":"t","symbol":"T",
        "observation":{"features":[{"kind":"price","field":"close"}]},
        "action_space":{"type":"discrete","n":3},
        "reward":"pnl","episode":{"max_steps":100,"warmup":0}
    }"#;

    /// Run a command with a generous buffer and return the response string.
    fn command(handle: *mut WickraGymEnv, cmd: &str) -> String {
        let c = CString::new(cmd).unwrap();
        let mut buf = vec![0u8; 8192];
        let len = unsafe {
            wickra_gym_command(
                handle,
                c.as_ptr(),
                buf.as_mut_ptr().cast::<c_char>(),
                buf.len(),
            )
        };
        assert!(len >= 0);
        CStr::from_bytes_until_nul(&buf)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn new_handle() -> *mut WickraGymEnv {
        let spec = CString::new(SPEC).unwrap();
        let handle = unsafe { wickra_gym_new(spec.as_ptr()) };
        assert!(!handle.is_null());
        handle
    }

    #[test]
    fn version_round_trips() {
        let handle = new_handle();
        let response = command(handle, r#"{"cmd":"version"}"#);
        assert!(response.contains(env!("CARGO_PKG_VERSION")));
        unsafe { wickra_gym_free(handle) };
    }

    #[test]
    fn bad_spec_returns_null() {
        let bad = CString::new(r#"{"not":"a spec"}"#).unwrap();
        let handle = unsafe { wickra_gym_new(bad.as_ptr()) };
        assert!(handle.is_null());
    }

    #[test]
    fn too_small_buffer_leaves_out_untouched() {
        let handle = new_handle();
        let cmd = CString::new(r#"{"cmd":"version"}"#).unwrap();
        let mut buf = vec![0xAAu8; 4]; // deliberately too small
        let len = unsafe {
            wickra_gym_command(
                handle,
                cmd.as_ptr(),
                buf.as_mut_ptr().cast::<c_char>(),
                buf.len(),
            )
        };
        assert!(usize::try_from(len).unwrap() >= buf.len());
        assert!(buf.iter().all(|&b| b == 0xAA)); // untouched
        unsafe { wickra_gym_free(handle) };
    }

    #[test]
    fn bad_action_reports_error_in_band() {
        let handle = new_handle();
        command(
            handle,
            r#"{"cmd":"load","candles":[{"ts":0,"open":1,"high":1,"low":1,"close":1},{"ts":1,"open":2,"high":2,"low":2,"close":2}]}"#,
        );
        command(handle, r#"{"cmd":"reset"}"#);
        let response = command(handle, r#"{"cmd":"step","action":9}"#);
        assert!(response.contains("\"ok\":false"));
        unsafe { wickra_gym_free(handle) };
    }

    #[test]
    fn null_guards_on_command() {
        let cmd = CString::new(r#"{"cmd":"version"}"#).unwrap();
        let code = unsafe { wickra_gym_command(ptr::null_mut(), cmd.as_ptr(), ptr::null_mut(), 0) };
        assert_eq!(code, WICKRA_GYM_ERR_NULL);
        let handle = new_handle();
        let code = unsafe { wickra_gym_command(handle, ptr::null(), ptr::null_mut(), 0) };
        assert_eq!(code, WICKRA_GYM_ERR_NULL);
        unsafe { wickra_gym_free(handle) };
    }

    #[test]
    fn free_null_is_a_noop() {
        unsafe { wickra_gym_free(ptr::null_mut()) };
    }

    #[test]
    fn version_is_nul_terminated() {
        let p = wickra_gym_version();
        let v = unsafe { CStr::from_ptr(p) }.to_str().unwrap();
        assert_eq!(v, env!("CARGO_PKG_VERSION"));
    }

    /// The critical mutating-command test: buffer-sizing retries of a `step`
    /// execute the step exactly once, and a fresh identical `step` advances.
    #[test]
    fn buffer_retry_does_not_double_step() {
        let handle = new_handle();
        command(
            handle,
            r#"{"cmd":"load","candles":[{"ts":0,"open":100,"high":100,"low":100,"close":100},{"ts":1,"open":101,"high":101,"low":101,"close":101},{"ts":2,"open":102,"high":102,"low":102,"close":102},{"ts":3,"open":103,"high":103,"low":103,"close":103}]}"#,
        );
        command(handle, r#"{"cmd":"reset"}"#);

        let step = CString::new(r#"{"cmd":"step","action":2}"#).unwrap();
        // Two length-only calls (cap = 0) then one delivering call — all one step.
        let a = unsafe { wickra_gym_command(handle, step.as_ptr(), ptr::null_mut(), 0) };
        let b = unsafe { wickra_gym_command(handle, step.as_ptr(), ptr::null_mut(), 0) };
        assert_eq!(a, b);
        let response1 = command(handle, r#"{"cmd":"step","action":2}"#);
        assert!(response1.contains("\"step\":1.0"));

        // A fresh identical step is logical step 2, not 3 (no double-stepping).
        let response2 = command(handle, r#"{"cmd":"step","action":2}"#);
        assert!(response2.contains("\"step\":2.0"));
        unsafe { wickra_gym_free(handle) };
    }
}
