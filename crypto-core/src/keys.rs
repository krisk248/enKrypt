//! Key generation, parsing, metadata extraction, export and revocation.

use pgp::composed::{
    Deserializable, EncryptionCaps, KeyType, SecretKeyParamsBuilder, SignedPublicKey,
    SignedSecretKey, SubkeyParamsBuilder,
};
use pgp::crypto::ecc_curve::ECCCurve;
use pgp::crypto::hash::HashAlgorithm;
use pgp::crypto::public_key::PublicKeyAlgorithm;
use pgp::crypto::sym::SymmetricKeyAlgorithm;
use pgp::packet::{
    RevocationCode, Signature, SignatureConfig, SignatureType, Subpacket, SubpacketData,
};
use pgp::types::{KeyDetails as _, KeyVersion, Password, PublicParams, Timestamp};
use rsa::traits::PublicKeyParts;
use smallvec::smallvec;

use crate::error::{ErrorCode, WResult, WasmError};
use crate::model::{GenerateOptions, KeyAlgo, KeyBundle, KeyInfo, SubkeyInfo};

/// Standard armor options (with CRC checksum).
pub(crate) fn armor_opts() -> pgp::composed::ArmorOptions<'static> {
    pgp::composed::ArmorOptions::default()
}

fn hex_upper(bytes: &[u8]) -> String {
    hex::encode_upper(bytes)
}

/// Build the RFC-4880 User ID string "Name (comment) <email>".
fn build_uid(name: &str, email: &str, comment: Option<&str>) -> String {
    match comment {
        Some(c) if !c.trim().is_empty() => format!("{name} ({c}) <{email}>"),
        _ => format!("{name} <{email}>"),
    }
}

/// Human label + bit size for a public-key component.
fn algo_label(alg: PublicKeyAlgorithm, params: &PublicParams) -> (String, u32) {
    match params {
        PublicParams::RSA(p) => ("RSA".to_string(), p.key.n().bits() as u32),
        PublicParams::ECDSA(_) => ("ECDSA (NIST)".to_string(), 0),
        PublicParams::ECDH(_) => ("ECDH".to_string(), 0),
        PublicParams::EdDSALegacy(_) => ("EdDSA (Ed25519)".to_string(), 255),
        PublicParams::Ed25519(_) => ("Ed25519".to_string(), 255),
        PublicParams::X25519(_) => ("X25519".to_string(), 255),
        PublicParams::DSA(_) => ("DSA".to_string(), 0),
        PublicParams::Elgamal(_) => ("ElGamal".to_string(), 0),
        // Modern / post-quantum families (Ed448, X448, ML-KEM, ML-DSA, SLH-DSA)
        // are labelled from their algorithm id, with the composite PQC schemes
        // written in their readable RFC form.
        _ => {
            let label = match alg {
                PublicKeyAlgorithm::Ed448 => "Ed448".to_string(),
                PublicKeyAlgorithm::X448 => "X448".to_string(),
                other => format!("{other:?}")
                    .replace("MlKem768X25519", "ML-KEM-768 + X25519")
                    .replace("MlKem1024X448", "ML-KEM-1024 + X448")
                    .replace("MlDsa65Ed25519", "ML-DSA-65 + Ed25519")
                    .replace("MlDsa87Ed448", "ML-DSA-87 + Ed448")
                    .replace("SlhDsaShake", "SLH-DSA-SHAKE-"),
            };
            (label, 0)
        }
    }
}

/// Parse a key from bytes that may be armored (.asc) or binary (.gpg/.pgp),
/// public or secret. Returns whichever variant succeeds.
pub enum AnyKey {
    Secret(Box<SignedSecretKey>),
    Public(Box<SignedPublicKey>),
}

pub fn parse_any_key(bytes: &[u8]) -> WResult<AnyKey> {
    // A secret key parses as secret; a public key does not. Try secret first,
    // then public. `from_reader_single` auto-detects armored vs binary input.
    if let Ok((sk, _)) = SignedSecretKey::from_reader_single(bytes) {
        // Validate self-signatures; ignore binding errors to stay lenient on import.
        let _ = sk.verify_bindings();
        return Ok(AnyKey::Secret(Box::new(sk)));
    }
    match SignedPublicKey::from_reader_single(bytes) {
        Ok((pk, _)) => {
            let _ = pk.verify_bindings();
            Ok(AnyKey::Public(Box::new(pk)))
        }
        Err(e) => Err(WasmError::new(
            ErrorCode::CorruptData,
            format!("Could not parse OpenPGP key: {e}"),
        )),
    }
}

pub fn parse_public(bytes: &[u8]) -> WResult<SignedPublicKey> {
    match parse_any_key(bytes)? {
        AnyKey::Public(pk) => Ok(*pk),
        AnyKey::Secret(sk) => Ok(sk.to_public_key()),
    }
}

pub fn parse_secret(bytes: &[u8]) -> WResult<SignedSecretKey> {
    match parse_any_key(bytes)? {
        AnyKey::Secret(sk) => Ok(*sk),
        AnyKey::Public(_) => Err(WasmError::new(
            ErrorCode::InvalidKey,
            "Expected a secret key but found a public key",
        )),
    }
}

fn subkey_info_public(sub: &pgp::composed::SignedPublicSubKey) -> SubkeyInfo {
    let (label, bits) = algo_label(sub.algorithm(), sub.public_params());
    SubkeyInfo {
        fingerprint: hex_upper(sub.fingerprint().as_bytes()),
        key_id: hex_upper(sub.legacy_key_id().as_ref()),
        algorithm: label,
        bits,
        created_at: sub.created_at().as_secs() as i64,
        can_encrypt: sub.algorithm().can_encrypt(),
        can_sign: sub.algorithm().can_sign(),
    }
}

/// Extract metadata from a public certificate.
pub fn key_info_public(pk: &SignedPublicKey, has_secret: bool) -> KeyInfo {
    let (label, bits) = algo_label(pk.algorithm(), pk.primary_key.public_params());
    let key_id = hex_upper(pk.legacy_key_id().as_ref());
    let short_id = key_id.chars().rev().take(8).collect::<String>()
        .chars().rev().collect::<String>();

    let user_ids: Vec<String> = pk
        .details
        .users
        .iter()
        .map(|u| String::from_utf8_lossy(u.id.id()).into_owned())
        .collect();

    KeyInfo {
        fingerprint: hex_upper(pk.fingerprint().as_bytes()),
        key_id,
        short_id,
        algorithm: label,
        bits,
        created_at: pk.primary_key.created_at().as_secs() as i64,
        has_secret,
        primary_user_id: user_ids.first().cloned(),
        user_ids,
        can_encrypt: pk.algorithm().can_encrypt(),
        can_sign: pk.algorithm().can_sign(),
        subkeys: pk.public_subkeys.iter().map(subkey_info_public).collect(),
    }
}

pub fn key_info_secret(sk: &SignedSecretKey) -> KeyInfo {
    key_info_public(&sk.to_public_key(), true)
}

/// Metadata for arbitrary imported key bytes.
pub fn info_for_bytes(bytes: &[u8]) -> WResult<KeyInfo> {
    match parse_any_key(bytes)? {
        AnyKey::Secret(sk) => Ok(key_info_secret(&sk)),
        AnyKey::Public(pk) => Ok(key_info_public(&pk, false)),
    }
}

/// Generate a brand new secret key + certificate, passphrase-protected.
pub fn generate(opts: GenerateOptions) -> WResult<KeyBundle> {
    if opts.passphrase.is_empty() {
        return Err(WasmError::new(
            ErrorCode::BadParams,
            "A passphrase is required to protect the secret key",
        ));
    }
    let uid = build_uid(&opts.name, &opts.email, opts.comment.as_deref());
    let pass = opts.passphrase.clone();

    // Map the requested algorithm to (primary signing type, encryption subkey
    // type, key version). Legacy-compatible families use v4 keys (understood by
    // every GnuPG); modern (Ed448) and post-quantum families require v6.
    let (primary_type, enc_type, version) = match opts.algo {
        KeyAlgo::Ed25519 => (
            KeyType::Ed25519Legacy,
            KeyType::ECDH(ECCCurve::Curve25519Legacy),
            KeyVersion::V4,
        ),
        KeyAlgo::Rsa2048 => (KeyType::Rsa(2048), KeyType::Rsa(2048), KeyVersion::V4),
        KeyAlgo::Rsa3072 => (KeyType::Rsa(3072), KeyType::Rsa(3072), KeyVersion::V4),
        KeyAlgo::Rsa4096 => (KeyType::Rsa(4096), KeyType::Rsa(4096), KeyVersion::V4),
        KeyAlgo::NistP256 => (
            KeyType::ECDSA(ECCCurve::P256),
            KeyType::ECDH(ECCCurve::P256),
            KeyVersion::V4,
        ),
        KeyAlgo::NistP384 => (
            KeyType::ECDSA(ECCCurve::P384),
            KeyType::ECDH(ECCCurve::P384),
            KeyVersion::V4,
        ),
        KeyAlgo::Ed448 => (KeyType::Ed448, KeyType::X448, KeyVersion::V6),
        KeyAlgo::PostQuantum => (
            KeyType::MlDsa65Ed25519,
            KeyType::MlKem768X25519,
            KeyVersion::V6,
        ),
    };

    // Encryption subkey — locked with the same passphrase as the primary.
    let mut enc_subkey = SubkeyParamsBuilder::default();
    enc_subkey
        .version(version)
        .key_type(enc_type)
        .can_encrypt(EncryptionCaps::All)
        .can_sign(false)
        .can_authenticate(false)
        .passphrase(Some(pass.clone()));

    let enc_subkey = enc_subkey
        .build()
        .map_err(|e| WasmError::new(ErrorCode::BadParams, e.to_string()))?;

    let mut params = SecretKeyParamsBuilder::default();
    params
        .version(version)
        .key_type(primary_type)
        .can_certify(true)
        .can_sign(true)
        .can_encrypt(EncryptionCaps::None)
        .can_authenticate(false)
        .primary_user_id(uid)
        .passphrase(Some(pass))
        .preferred_symmetric_algorithms(smallvec![
            SymmetricKeyAlgorithm::AES256,
            SymmetricKeyAlgorithm::AES192,
            SymmetricKeyAlgorithm::AES128,
        ])
        .preferred_hash_algorithms(smallvec![
            HashAlgorithm::Sha512,
            HashAlgorithm::Sha256,
        ])
        .subkeys(vec![enc_subkey]);

    let secret_params = params
        .build()
        .map_err(|e| WasmError::new(ErrorCode::BadParams, e.to_string()))?;

    let ssk = secret_params
        .generate(rand::rngs::OsRng)
        .map_err(WasmError::from)?;

    let info = key_info_secret(&ssk);
    let public_key = ssk
        .to_public_key()
        .to_armored_string(armor_opts())
        .map_err(WasmError::from)?;
    let secret_key = ssk.to_armored_string(armor_opts()).map_err(WasmError::from)?;

    Ok(KeyBundle {
        info,
        public_key,
        secret_key,
    })
}

/// Re-armor the public certificate of any key input.
pub fn export_public(bytes: &[u8]) -> WResult<String> {
    let pk = parse_public(bytes)?;
    pk.to_armored_string(armor_opts()).map_err(WasmError::from)
}

/// Re-armor a secret key (input must contain secret material). Never derives
/// an unencrypted form — the S2K-encrypted packets are preserved as-is.
pub fn export_secret(bytes: &[u8]) -> WResult<String> {
    let sk = parse_secret(bytes)?;
    sk.to_armored_string(armor_opts()).map_err(WasmError::from)
}

/// Generate an armored revocation certificate for the primary key.
/// `reason_code`: 0 = no reason, 1 = superseded, 2 = compromised, 3 = retired.
pub fn generate_revocation(
    secret_bytes: &[u8],
    passphrase: &str,
    reason_code: u8,
    reason: &str,
) -> WResult<String> {
    let sk = parse_secret(secret_bytes)?;
    let primary = &sk.primary_key;
    let pw = Password::from(passphrase.to_string());

    let code = match reason_code {
        1 => RevocationCode::KeySuperseded,
        2 => RevocationCode::KeyCompromised,
        3 => RevocationCode::KeyRetired,
        _ => RevocationCode::NoReason,
    };

    let mut config = match primary.version() {
        KeyVersion::V4 => SignatureConfig::v4(
            SignatureType::KeyRevocation,
            primary.algorithm(),
            HashAlgorithm::Sha512,
        ),
        KeyVersion::V6 => SignatureConfig::v6(
            rand::rngs::OsRng,
            SignatureType::KeyRevocation,
            primary.algorithm(),
            HashAlgorithm::Sha512,
        )
        .map_err(WasmError::from)?,
        v => {
            return Err(WasmError::new(
                ErrorCode::InvalidKey,
                format!("Unsupported key version for revocation: {v:?}"),
            ))
        }
    };

    config.hashed_subpackets = vec![
        Subpacket::regular(SubpacketData::SignatureCreationTime(Timestamp::now()))
            .map_err(WasmError::from)?,
        Subpacket::regular(SubpacketData::IssuerFingerprint(primary.fingerprint()))
            .map_err(WasmError::from)?,
        Subpacket::regular(SubpacketData::RevocationReason(
            code,
            reason.as_bytes().to_vec().into(),
        ))
        .map_err(WasmError::from)?,
    ];
    config.unhashed_subpackets =
        vec![
            Subpacket::regular(SubpacketData::IssuerKeyId(primary.legacy_key_id()))
                .map_err(WasmError::from)?,
        ];

    let sig: Signature = config
        .sign_key(primary, &pw, primary.public_key())
        .map_err(WasmError::from)?;

    pgp::composed::DetachedSignature::new(sig)
        .to_armored_string(armor_opts())
        .map_err(WasmError::from)
}
