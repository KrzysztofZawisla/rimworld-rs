//! Client side of the host protocol: one JSON request, one JSON response.

use std::sync::OnceLock;

use serde_json::{json, Value};

use crate::abi::{Buf, HostApi};

static HOST: OnceLock<&'static HostApi> = OnceLock::new();

/// Everything that can go wrong while talking to the game.
#[derive(Debug)]
pub enum Error {
    /// `rwrs_init` has not been called yet.
    NotInitialized,
    /// `rwrs_init` was called twice.
    AlreadyInitialized,
    /// The host callback itself failed.
    HostUnavailable(u32),
    /// The game reported an error for the request.
    Host(String),
    /// A payload could not be encoded or decoded.
    Json(serde_json::Error),
    /// The response did not have the expected shape.
    InvalidResponse(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotInitialized => write!(formatter, "the mod was not initialized by the game yet"),
            Error::AlreadyInitialized => write!(formatter, "rwrs_init was already called"),
            Error::HostUnavailable(code) => write!(formatter, "the host callback failed with code {code}"),
            Error::Host(message) => write!(formatter, "{message}"),
            Error::Json(error) => write!(formatter, "json error: {error}"),
            Error::InvalidResponse(detail) => write!(formatter, "unexpected host response: {detail}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Json(error)
    }
}

/// Stores the host table for the lifetime of the process.
pub(crate) fn install(api: &'static HostApi) -> Result<(), Error> {
    HOST.set(api).map_err(|_| Error::AlreadyInitialized)
}

/// Whether the game already handed us its API table.
pub fn is_installed() -> bool {
    HOST.get().is_some()
}

/// Performs one host call. `request` must be a JSON object.
pub(crate) fn call(op: &str, mut request: Value) -> Result<Value, Error> {
    let api = HOST.get().ok_or(Error::NotInitialized)?;
    let callback = api.call.ok_or(Error::NotInitialized)?;

    match request.as_object_mut() {
        Some(object) => {
            object.insert("op".to_string(), Value::String(op.to_string()));
        }
        None => {
            request = json!({ "op": op });
        }
    }

    let encoded = serde_json::to_vec(&request)?;
    let len = u32::try_from(encoded.len())
        .map_err(|_| Error::Host("the host request exceeds the ABI's u32 length".to_string()))?;
    if len > i32::MAX as u32 {
        return Err(Error::Host("the host request exceeds the bridge's int length".to_string()));
    }

    let mut response = Buf::EMPTY;
    let code = callback(api.user_data, encoded.as_ptr(), len, &mut response);
    if code != 0 {
        return Err(Error::HostUnavailable(code));
    }

    // The buffer stays valid until the next host call; copy it out immediately.
    let text = String::from_utf8(unsafe { response.as_slice() }.to_vec())
        .map_err(|error| Error::InvalidResponse(error.to_string()))?;
    let value: Value = serde_json::from_str(&text)?;
    if !value.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        let message = value.get("error").and_then(Value::as_str).unwrap_or("unknown host error");
        return Err(Error::Host(message.to_string()));
    }

    Ok(value.get("result").cloned().unwrap_or(Value::Null))
}

#[cfg(test)]
pub(crate) mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::abi::Buf;

    thread_local! {
        /// Host calls made on this thread. Tests in one binary run in parallel threads and the mock
        /// host is a process-wide function, so a global counter would count another test's dispatches
        /// and make "exactly one call" assertions fail at random.
        static CALLS: Cell<usize> = const { Cell::new(0) };
    }

    /// The host calls this thread has made so far.
    pub(crate) fn calls() -> usize {
        CALLS.with(Cell::get)
    }

    pub(crate) extern "C" fn mock_host(
        _user_data: *mut core::ffi::c_void,
        request_ptr: *const u8,
        request_len: u32,
        response_out: *mut Buf,
    ) -> u32 {
        let request = unsafe { core::slice::from_raw_parts(request_ptr, request_len as usize) };
        let value: Value = serde_json::from_slice(request).unwrap();
        CALLS.with(|made| made.set(made.get() + 1));
        let response = match value.get("op").and_then(Value::as_str).unwrap_or("") {
            "log" => json!({ "ok": true, "result": null }),
            "game_info" => json!({
                "ok": true,
                "result": {
                    "hasGame": true,
                    "tick": 1234,
                    "programState": "Playing",
                    "currentMapId": 7,
                    "maps": [
                        { "id": 7, "biome": "TemperateForest", "tile": 42, "sizeX": 250, "sizeZ": 250, "colonists": 3, "isPlayerHome": true }
                    ]
                }
            }),
            "def_info" => {
                json!({ "ok": true, "result": { "defName": "Steel", "defType": "ThingDef", "label": "steel", "description": "", "modName": "Core" } })
            }
            "unknown_op" => json!({ "ok": false, "error": "no such op" }),
            _ => json!({ "ok": true, "result": null }),
        };
        let bytes = response.to_string().into_bytes().into_boxed_slice();
        let len = bytes.len() as u32;
        let ptr = Box::into_raw(bytes) as *mut u8;
        unsafe {
            *response_out = Buf { ptr, len };
        }
        0
    }
}
