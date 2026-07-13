//! Rich, machine-readable error type that crosses the wasm boundary as a
//! structured JS object `{ code, message }` so the UI can show precise messages
//! (e.g. distinguish a wrong passphrase from a missing key or corrupt data).

use serde::Serialize;
use tsify_next::Tsify;

/// Discriminant the frontend can switch on.
#[derive(Debug, Clone, Copy, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// The supplied passphrase did not unlock the secret key / message.
    WrongPassphrase,
    /// No secret key in the provided set could decrypt the message.
    NoMatchingKey,
    /// Input bytes were not valid OpenPGP data (or wrong armor type).
    CorruptData,
    /// A signature was present but could not be verified as valid.
    BadSignature,
    /// The requested key material was not usable (e.g. no encryption subkey).
    InvalidKey,
    /// Key generation parameters were rejected.
    BadParams,
    /// Vault (Argon2id + AES-GCM) sealing/opening failed.
    VaultError,
    /// Anything else.
    Internal,
}

#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct WasmError {
    pub code: ErrorCode,
    pub message: String,
}

impl WasmError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        WasmError {
            code,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }
}

impl std::fmt::Display for WasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

// Note: `#[tsify(into_wasm_abi)]` on `WasmError` generates the
// `From<WasmError> for JsValue` conversion, so a `Result<T, WasmError>` returned
// from a `#[wasm_bindgen]` fn rejects with a structured `{ code, message }`.

/// Classify a raw rpgp error string into a precise [`ErrorCode`]. rpgp does not
/// expose a stable typed error taxonomy for every case, so we pattern-match the
/// most important, user-facing failure modes.
pub fn classify(err: &pgp::errors::Error) -> WasmError {
    let msg = err.to_string();
    let lower = msg.to_lowercase();

    let code = if lower.contains("missing key") || lower.contains("no matching") {
        ErrorCode::NoMatchingKey
    } else if lower.contains("passphrase")
        || lower.contains("password")
        || lower.contains("mac")
        || lower.contains("checksum mismatch")
        || lower.contains("unable to unlock")
        || lower.contains("decrypt")
    {
        // rpgp surfaces a bad passphrase as an unlock / MAC / decrypt failure.
        ErrorCode::WrongPassphrase
    } else if lower.contains("signature") {
        ErrorCode::BadSignature
    } else if lower.contains("parse") || lower.contains("invalid") || lower.contains("armor") {
        ErrorCode::CorruptData
    } else {
        ErrorCode::Internal
    };

    WasmError::new(code, msg)
}

pub type WResult<T> = Result<T, WasmError>;

/// Ergonomic conversion so `?` works on `pgp::errors::Error`.
impl From<pgp::errors::Error> for WasmError {
    fn from(err: pgp::errors::Error) -> Self {
        classify(&err)
    }
}
