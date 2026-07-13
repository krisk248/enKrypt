//! Serde/Tsify structs forming the typed contract between Rust and TypeScript.
//! `tsify-next` generates `.d.ts` definitions for every struct below.

use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

/// Algorithm family choice for key generation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "snake_case")]
pub enum KeyAlgo {
    /// Modern default: Ed25519 (sign/certify) + Cv25519/X25519 (encrypt), v4 — GnuPG compatible.
    Ed25519,
    /// RSA 2048 — broad, decades-old compatibility.
    Rsa2048,
    /// RSA 3072 primary + RSA 3072 encryption subkey.
    Rsa3072,
    /// RSA 4096 primary + RSA 4096 encryption subkey.
    Rsa4096,
    /// NIST P-256: ECDSA (sign) + ECDH (encrypt), v4 — enterprise / FIPS-style compatibility.
    NistP256,
    /// NIST P-384: ECDSA (sign) + ECDH (encrypt), v4.
    NistP384,
    /// Ed448 (sign) + X448 (encrypt), v6 — modern high-security.
    Ed448,
    /// Post-quantum (experimental, draft RFC): ML-DSA-65+Ed25519 (sign) +
    /// ML-KEM-768+X25519 (encrypt), v6. Future-proof; only interoperates with
    /// very recent OpenPGP implementations.
    PostQuantum,
}

/// Inputs for [`crate::generate_key`].
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct GenerateOptions {
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub comment: Option<String>,
    pub passphrase: String,
    pub algo: KeyAlgo,
    /// Optional key expiry, in seconds from creation. Enforced at the app layer.
    #[serde(default)]
    pub expiry_secs: Option<u32>,
}

/// One sub-key's metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SubkeyInfo {
    pub fingerprint: String,
    pub key_id: String,
    pub algorithm: String,
    pub bits: u32,
    pub created_at: i64,
    pub can_encrypt: bool,
    pub can_sign: bool,
}

/// Full metadata extracted from a parsed key.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct KeyInfo {
    pub fingerprint: String,
    /// 16-hex-char long key id.
    pub key_id: String,
    /// 8-hex-char short key id.
    pub short_id: String,
    pub algorithm: String,
    pub bits: u32,
    pub created_at: i64,
    pub has_secret: bool,
    pub user_ids: Vec<String>,
    pub primary_user_id: Option<String>,
    pub can_encrypt: bool,
    pub can_sign: bool,
    pub subkeys: Vec<SubkeyInfo>,
}

/// Return value of [`crate::generate_key`].
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct KeyBundle {
    pub info: KeyInfo,
    pub public_key: String,
    pub secret_key: String,
}

/// Verdict for a single signature.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify, PartialEq, Eq)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "snake_case")]
pub enum SignatureStatus {
    Valid,
    Invalid,
    UnknownKey,
}

/// Details about a signature found on a message / detached signature.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SignatureInfo {
    pub status: SignatureStatus,
    pub key_id: Option<String>,
    pub fingerprint: Option<String>,
    pub signer_user_id: Option<String>,
    pub created_at: Option<i64>,
}

/// Result of decrypting a message.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct DecryptResult {
    #[tsify(type = "Uint8Array")]
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    /// Original filename embedded in the literal packet, if any.
    pub filename: Option<String>,
    pub signatures: Vec<SignatureInfo>,
}

/// Result of verifying a detached signature.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct VerifyResult {
    pub signatures: Vec<SignatureInfo>,
}

/// Result of verifying a cleartext-signed message.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CleartextVerifyResult {
    pub text: String,
    pub signatures: Vec<SignatureInfo>,
}
