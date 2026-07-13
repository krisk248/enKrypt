//! Optional app-level vault: wrap the whole keyring blob with AES-256-GCM
//! under a key derived from a passphrase via Argon2id. This is an extra layer
//! *on top of* the per-key OpenPGP S2K encryption, for the persistent keyring.
//!
//! Blob layout (all concatenated):
//!   magic[4] = b"WPTV" | version[1]=1 | salt[16] | nonce[12] | ciphertext(+16 tag)

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use zeroize::Zeroize;

use crate::error::{ErrorCode, WResult, WasmError};

const MAGIC: &[u8; 4] = b"WPTV";
const VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

fn derive_key(passphrase: &str, salt: &[u8]) -> WResult<[u8; 32]> {
    // Argon2id, 64 MiB, 3 passes, 4 lanes — RFC 9106 "parameter choice 2".
    let params = Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|e| WasmError::new(ErrorCode::VaultError, e.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; 32];
    argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| WasmError::new(ErrorCode::VaultError, e.to_string()))?;
    Ok(out)
}

/// Encrypt `data` under `passphrase`.
pub fn seal(data: &[u8], passphrase: &str) -> WResult<Vec<u8>> {
    if passphrase.is_empty() {
        return Err(WasmError::new(
            ErrorCode::VaultError,
            "Vault passphrase must not be empty",
        ));
    }
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    rand::rngs::OsRng.fill_bytes(&mut nonce);

    let mut key_bytes = derive_key(passphrase, &salt)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), data)
        .map_err(|_| WasmError::new(ErrorCode::VaultError, "AES-GCM sealing failed"))?;
    key_bytes.zeroize();

    let mut out = Vec::with_capacity(4 + 1 + SALT_LEN + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt a blob produced by [`seal`].
pub fn open(blob: &[u8], passphrase: &str) -> WResult<Vec<u8>> {
    let header = 4 + 1 + SALT_LEN + NONCE_LEN;
    if blob.len() < header + 16 || &blob[0..4] != MAGIC {
        return Err(WasmError::new(
            ErrorCode::VaultError,
            "Not a valid vault blob",
        ));
    }
    if blob[4] != VERSION {
        return Err(WasmError::new(
            ErrorCode::VaultError,
            format!("Unsupported vault version: {}", blob[4]),
        ));
    }
    let salt = &blob[5..5 + SALT_LEN];
    let nonce = &blob[5 + SALT_LEN..header];
    let ciphertext = &blob[header..];

    let mut key_bytes = derive_key(passphrase, salt)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| {
            WasmError::new(
                ErrorCode::WrongPassphrase,
                "Wrong vault passphrase or corrupt vault",
            )
        });
    key_bytes.zeroize();
    plaintext
}
