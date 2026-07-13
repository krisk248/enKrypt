//! Encrypt / decrypt / sign / verify operations, plus text (clipboard) modes.

use pgp::composed::{
    CleartextSignedMessage, DecryptionOptions, Deserializable, DetachedSignature, Message,
    MessageBuilder, SignedPublicKey, SignedPublicSubKey, TheRing, VerificationResult,
};
use pgp::crypto::hash::HashAlgorithm;
use pgp::crypto::sym::SymmetricKeyAlgorithm;
use pgp::packet::Signature;
use pgp::ser::Serialize as _;
use pgp::types::{KeyDetails as _, KeyId, Password, SigningKey, VerifyingKey};

use crate::error::{ErrorCode, WResult, WasmError};
use crate::keys::{armor_opts, parse_public, parse_secret};
use crate::model::{DecryptResult, SignatureInfo, SignatureStatus, VerifyResult};

/// Map an error from a passphrase-gated signing/unlock operation. rpgp often
/// surfaces a wrong passphrase as a generic "invalid input" (the wrong key
/// material simply fails downstream), so in this context we bias to
/// [`ErrorCode::WrongPassphrase`].
fn map_unlock_err(err: pgp::errors::Error) -> WasmError {
    let we: WasmError = err.into();
    match we.code {
        ErrorCode::CorruptData | ErrorCode::Internal => {
            WasmError::new(ErrorCode::WrongPassphrase, we.message)
        }
        _ => we,
    }
}

fn is_armored(data: &[u8]) -> bool {
    data.windows(5).take(64).any(|w| w == b"-----")
        || data.starts_with(b"-----BEGIN")
}

/// Pick the best encryption-capable subkey from a recipient certificate.
fn encryption_subkey(cert: &SignedPublicKey) -> WResult<&SignedPublicSubKey> {
    cert.public_subkeys
        .iter()
        .find(|s| s.algorithm().can_encrypt())
        .ok_or_else(|| {
            WasmError::new(
                ErrorCode::InvalidKey,
                "Recipient key has no encryption-capable subkey",
            )
        })
}

/// Encrypt `data` to one or more recipients, optionally signing.
///
/// Note: rpgp 0.20's high-level `MessageBuilder` does not embed a literal
/// filename, so the caller derives the output name (by stripping the `.gpg` /
/// `.asc` suffix on decrypt) — the standard behaviour of GnuPG front-ends.
pub fn encrypt(
    data: &[u8],
    recipients: Vec<String>,
    sign_with: Option<String>,
    sign_passphrase: Option<String>,
    armor: bool,
) -> WResult<Vec<u8>> {
    if recipients.is_empty() {
        return Err(WasmError::new(
            ErrorCode::BadParams,
            "At least one recipient is required",
        ));
    }

    let certs: Vec<SignedPublicKey> = recipients
        .iter()
        .map(|r| parse_public(r.as_bytes()))
        .collect::<WResult<Vec<_>>>()?;

    // The signing key must outlive the builder that borrows it.
    let signer = match &sign_with {
        Some(armored) => Some(parse_secret(armored.as_bytes())?),
        None => None,
    };

    let mut builder = MessageBuilder::from_bytes("", data.to_vec())
        .seipd_v1(rand::rngs::OsRng, SymmetricKeyAlgorithm::AES256);

    if let Some(ssk) = &signer {
        let pw = Password::from(sign_passphrase.unwrap_or_default());
        builder.sign(&ssk.primary_key as &dyn SigningKey, pw, HashAlgorithm::Sha512);
    }

    for cert in &certs {
        let subkey = encryption_subkey(cert)?;
        builder
            .encrypt_to_key(rand::rngs::OsRng, subkey)
            .map_err(WasmError::from)?;
    }

    // If a signer is involved, a failure here is most likely a wrong signing
    // passphrase (the actual sign happens lazily during output).
    let map_out = |e: pgp::errors::Error| {
        if signer.is_some() {
            map_unlock_err(e)
        } else {
            WasmError::from(e)
        }
    };

    if armor {
        let s = builder
            .to_armored_string(rand::rngs::OsRng, armor_opts())
            .map_err(map_out)?;
        Ok(s.into_bytes())
    } else {
        builder.to_vec(rand::rngs::OsRng).map_err(map_out)
    }
}

fn parse_message(data: &[u8]) -> WResult<Message<'_>> {
    if is_armored(data) {
        let s = std::str::from_utf8(data)
            .map_err(|_| WasmError::new(ErrorCode::CorruptData, "Armored input is not UTF-8"))?;
        let (msg, _) = Message::from_string(s).map_err(WasmError::from)?;
        Ok(msg)
    } else {
        Message::from_bytes(data).map_err(WasmError::from)
    }
}

fn sig_info_from(
    status: SignatureStatus,
    sig: &Signature,
    matched_key: Option<&SignedPublicKey>,
) -> SignatureInfo {
    let key_id = sig
        .issuer_key_id()
        .first()
        .map(|k| hex::encode_upper(k.as_ref()));
    SignatureInfo {
        status,
        key_id,
        fingerprint: matched_key.map(|k| hex::encode_upper(k.fingerprint().as_bytes())),
        signer_user_id: matched_key.and_then(|k| {
            k.details
                .users
                .first()
                .map(|u| String::from_utf8_lossy(u.id.id()).into_owned())
        }),
        created_at: sig.created().map(|t| t.as_secs() as i64),
    }
}

/// Decrypt a message; auto-detects the matching secret key and verifies any
/// embedded signatures against the provided public keys.
pub fn decrypt(
    data: &[u8],
    secret_keys: Vec<String>,
    passphrase: String,
    verify_keys: Vec<String>,
) -> WResult<DecryptResult> {
    let secrets = secret_keys
        .iter()
        .map(|s| parse_secret(s.as_bytes()))
        .collect::<WResult<Vec<_>>>()?;
    if secrets.is_empty() {
        return Err(WasmError::new(
            ErrorCode::NoMatchingKey,
            "No secret keys available to attempt decryption",
        ));
    }
    let pubs = verify_keys
        .iter()
        .map(|s| parse_public(s.as_bytes()))
        .collect::<WResult<Vec<_>>>()?;

    let pw = Password::from(passphrase);
    let passwords: Vec<&Password> = secrets.iter().map(|_| &pw).collect();
    let secret_refs: Vec<&_> = secrets.iter().collect();

    let msg = parse_message(data)?;
    // Opt into the widest set of real-world containers so we can read what other
    // tools actually produce:
    //  - `enable_gnupg_aead`: GnuPG's proprietary OCB/AEAD packet (type 20),
    //    which modern GnuPG emits by default for AEAD-capable keys.
    //  - `enable_legacy`: historical (malleable) SED packets (type 9), for
    //    decades-old archives.
    let ring = TheRing {
        secret_keys: secret_refs,
        key_passwords: passwords,
        decrypt_options: DecryptionOptions::new().enable_gnupg_aead().enable_legacy(),
        ..Default::default()
    };
    let (mut msg, _) = msg
        .decrypt_the_ring(ring, true)
        .map_err(WasmError::from)?;

    while msg.is_compressed() {
        msg = msg.decompress().map_err(WasmError::from)?;
    }

    let signed = msg.is_signed();
    let plaintext = msg
        .as_data_vec()
        .map_err(|e| WasmError::new(ErrorCode::CorruptData, e.to_string()))?;

    // The literal-data header (which carries the original filename) is only
    // populated once the packet body has been read.
    let filename = msg.literal_data_header().and_then(|h| {
        let raw = h.file_name();
        if raw.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(raw).to_string())
        }
    });

    let mut signatures = Vec::new();
    if signed {
        let vkeys: Vec<&dyn VerifyingKey> =
            pubs.iter().map(|k| k as &dyn VerifyingKey).collect();
        let mut matched = false;
        if !vkeys.is_empty() {
            if let Ok(results) = msg.verify_nested(&vkeys) {
                for (i, res) in results.iter().enumerate() {
                    if let VerificationResult::Valid(sig) = res {
                        signatures.push(sig_info_from(
                            SignatureStatus::Valid,
                            sig,
                            Some(&pubs[i]),
                        ));
                        matched = true;
                    }
                }
            }
        }
        if !matched {
            signatures.push(SignatureInfo {
                status: SignatureStatus::UnknownKey,
                key_id: None,
                fingerprint: None,
                signer_user_id: None,
                created_at: None,
            });
        }
    }

    Ok(DecryptResult {
        data: plaintext,
        filename,
        signatures,
    })
}

/// Produce a detached signature (binary or armored).
pub fn sign_detached(
    data: &[u8],
    secret_key: &str,
    passphrase: &str,
    armor: bool,
) -> WResult<Vec<u8>> {
    let ssk = parse_secret(secret_key.as_bytes())?;
    let pw = Password::from(passphrase.to_string());
    let sig = DetachedSignature::sign_binary_data(
        rand::rngs::OsRng,
        &ssk.primary_key,
        &pw,
        HashAlgorithm::Sha512,
        data,
    )
    .map_err(map_unlock_err)?;

    if armor {
        Ok(sig
            .to_armored_string(armor_opts())
            .map_err(WasmError::from)?
            .into_bytes())
    } else {
        let mut buf = Vec::new();
        sig.to_writer(&mut buf).map_err(WasmError::from)?;
        Ok(buf)
    }
}

fn key_id_matches(sig_ids: &[&KeyId], key: &SignedPublicKey) -> bool {
    let primary = key.legacy_key_id();
    let sub_ids: Vec<_> = key
        .public_subkeys
        .iter()
        .map(|s| s.legacy_key_id())
        .collect();
    sig_ids.iter().any(|id| {
        id.as_ref() == primary.as_ref() || sub_ids.iter().any(|s| s.as_ref() == id.as_ref())
    })
}

/// Verify `data` against a detached signature, choosing the signer from the
/// provided public keys.
pub fn verify_detached(
    data: &[u8],
    signature: &[u8],
    public_keys: Vec<String>,
) -> WResult<VerifyResult> {
    let (det, _) =
        DetachedSignature::from_reader_single(signature).map_err(WasmError::from)?;
    let pubs = public_keys
        .iter()
        .map(|s| parse_public(s.as_bytes()))
        .collect::<WResult<Vec<_>>>()?;

    let info = verify_signature_against(&det.signature, &pubs, data);
    Ok(VerifyResult {
        signatures: vec![info],
    })
}

/// Shared logic: given a signature, work out a precise
/// VALID / INVALID / UNKNOWN_KEY verdict against a set of public keys.
fn verify_signature_against(
    sig: &Signature,
    pubs: &[SignedPublicKey],
    data: &[u8],
) -> SignatureInfo {
    let issuer_ids = sig.issuer_key_id();
    for key in pubs {
        if sig.verify(key, data).is_ok() {
            return sig_info_from(SignatureStatus::Valid, sig, Some(key));
        }
    }
    // No key verified. Distinguish "wrong/no signature" from "we don't have the key".
    let have_issuer = pubs.iter().any(|k| key_id_matches(&issuer_ids, k));
    let status = if have_issuer {
        SignatureStatus::Invalid
    } else {
        SignatureStatus::UnknownKey
    };
    sig_info_from(status, sig, None)
}

/// Clear-sign a text block (WinPT clipboard "sign" behaviour).
pub fn sign_cleartext(text: &str, secret_key: &str, passphrase: &str) -> WResult<String> {
    let ssk = parse_secret(secret_key.as_bytes())?;
    let pw = Password::from(passphrase.to_string());
    let msg = CleartextSignedMessage::sign(rand::rngs::OsRng, text, &ssk.primary_key, &pw)
        .map_err(map_unlock_err)?;
    msg.to_armored_string(armor_opts()).map_err(WasmError::from)
}

/// Verify a clear-signed text block.
pub fn verify_cleartext(
    armored: &str,
    public_keys: Vec<String>,
) -> WResult<crate::model::CleartextVerifyResult> {
    let (msg, _) = CleartextSignedMessage::from_string(armored).map_err(WasmError::from)?;
    let pubs = public_keys
        .iter()
        .map(|s| parse_public(s.as_bytes()))
        .collect::<WResult<Vec<_>>>()?;

    let text = msg.text().to_string();
    let mut signatures = Vec::new();
    if let Some(sig) = msg.signatures().first() {
        let mut matched = false;
        for key in &pubs {
            if msg.verify(key).is_ok() {
                signatures.push(sig_info_from(SignatureStatus::Valid, sig, Some(key)));
                matched = true;
                break;
            }
        }
        if !matched {
            let issuer = sig.issuer_key_id();
            let have_issuer = pubs.iter().any(|k| key_id_matches(&issuer, k));
            let status = if have_issuer {
                SignatureStatus::Invalid
            } else {
                SignatureStatus::UnknownKey
            };
            signatures.push(sig_info_from(status, sig, None));
        }
    }

    Ok(crate::model::CleartextVerifyResult { text, signatures })
}
