//! # enkrypt-core
//!
//! 100% client-side OpenPGP crypto core for enKrypt, built on the pure-Rust
//! [`pgp`](https://crates.io/crates/pgp) (rpgp) implementation and compiled to
//! WebAssembly with `wasm-bindgen`. No network, no C dependencies.
//!
//! Every exported function is pure: it takes key material (armored strings) and
//! data, and returns results. Key storage/orchestration lives in the browser.

mod error;
mod keys;
mod model;
mod ops;
mod vault;

use wasm_bindgen::prelude::*;

pub use error::{ErrorCode, WasmError};
pub use model::*;

/// Install a panic hook that surfaces Rust panics in the browser console.
/// Safe to call multiple times.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Library + underlying rpgp version, for the About dialog.
#[wasm_bindgen]
pub fn version() -> String {
    format!("enkrypt-core {} (rpgp 0.20)", env!("CARGO_PKG_VERSION"))
}

// ---------------------------------------------------------------------------
// Key management
// ---------------------------------------------------------------------------

/// Generate a new passphrase-protected key pair.
#[wasm_bindgen]
pub fn generate_key(opts: GenerateOptions) -> Result<KeyBundle, WasmError> {
    keys::generate(opts)
}

/// Extract metadata from armored or binary key bytes (public or secret).
#[wasm_bindgen]
pub fn parse_key(bytes: &[u8]) -> Result<KeyInfo, WasmError> {
    keys::info_for_bytes(bytes)
}

/// Re-armor the public certificate from any key input.
#[wasm_bindgen]
pub fn export_public(bytes: &[u8]) -> Result<String, WasmError> {
    keys::export_public(bytes)
}

/// Re-armor a secret key (S2K-encrypted packets preserved; never plaintext).
#[wasm_bindgen]
pub fn export_secret(bytes: &[u8]) -> Result<String, WasmError> {
    keys::export_secret(bytes)
}

/// Generate an armored revocation certificate.
/// `reason_code`: 0 none, 1 superseded, 2 compromised, 3 retired.
#[wasm_bindgen]
pub fn generate_revocation(
    secret_key: &[u8],
    passphrase: &str,
    reason_code: u8,
    reason: &str,
) -> Result<String, WasmError> {
    keys::generate_revocation(secret_key, passphrase, reason_code, reason)
}

// ---------------------------------------------------------------------------
// Encrypt / decrypt
// ---------------------------------------------------------------------------

/// Encrypt `data` to one or more armored recipient public keys, optionally
/// signing with an armored secret key.
#[wasm_bindgen]
pub fn encrypt(
    data: &[u8],
    recipients: Vec<String>,
    sign_with: Option<String>,
    sign_passphrase: Option<String>,
    armor: bool,
) -> Result<Vec<u8>, WasmError> {
    ops::encrypt(data, recipients, sign_with, sign_passphrase, armor)
}

/// Decrypt `data` with the first matching secret key, verifying embedded
/// signatures against `verify_keys`.
#[wasm_bindgen]
pub fn decrypt(
    data: &[u8],
    secret_keys: Vec<String>,
    passphrase: String,
    verify_keys: Vec<String>,
) -> Result<DecryptResult, WasmError> {
    ops::decrypt(data, secret_keys, passphrase, verify_keys)
}

// ---------------------------------------------------------------------------
// Detached sign / verify
// ---------------------------------------------------------------------------

/// Produce a detached signature over `data`.
#[wasm_bindgen]
pub fn sign_detached(
    data: &[u8],
    secret_key: &str,
    passphrase: &str,
    armor: bool,
) -> Result<Vec<u8>, WasmError> {
    ops::sign_detached(data, secret_key, passphrase, armor)
}

/// Verify `data` against a detached `signature`.
#[wasm_bindgen]
pub fn verify_detached(
    data: &[u8],
    signature: &[u8],
    public_keys: Vec<String>,
) -> Result<VerifyResult, WasmError> {
    ops::verify_detached(data, signature, public_keys)
}

// ---------------------------------------------------------------------------
// Text (clipboard) mode
// ---------------------------------------------------------------------------

/// Clear-sign a UTF-8 text block.
#[wasm_bindgen]
pub fn sign_cleartext(
    text: &str,
    secret_key: &str,
    passphrase: &str,
) -> Result<String, WasmError> {
    ops::sign_cleartext(text, secret_key, passphrase)
}

/// Verify a clear-signed text block.
#[wasm_bindgen]
pub fn verify_cleartext(
    armored: &str,
    public_keys: Vec<String>,
) -> Result<CleartextVerifyResult, WasmError> {
    ops::verify_cleartext(armored, public_keys)
}

// ---------------------------------------------------------------------------
// Vault (Argon2id + AES-256-GCM) for the persistent keyring blob
// ---------------------------------------------------------------------------

/// Seal arbitrary bytes under a vault passphrase.
#[wasm_bindgen]
pub fn vault_seal(data: &[u8], passphrase: &str) -> Result<Vec<u8>, WasmError> {
    vault::seal(data, passphrase)
}

/// Open a vault blob produced by [`vault_seal`].
#[wasm_bindgen]
pub fn vault_open(blob: &[u8], passphrase: &str) -> Result<Vec<u8>, WasmError> {
    vault::open(blob, passphrase)
}
