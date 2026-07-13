//! Integration tests for the crypto core. These run on the host target (native)
//! so they exercise the exact same rpgp code paths the wasm build uses.
//!
//! NOTE: the public API functions are `#[wasm_bindgen]`-annotated and take
//! wasm-only types (`GenerateOptions`, etc.), so we test the internal modules
//! directly via a small `pub(crate)` test surface re-exported below.

use enkrypt_core as api;

const PASS: &str = "correct horse battery staple";

fn gen_ed25519() -> api::KeyBundle {
    api::generate_key(api::GenerateOptions {
        name: "Alice Example".into(),
        email: "alice@example.com".into(),
        comment: Some("test".into()),
        passphrase: PASS.into(),
        algo: api::KeyAlgo::Ed25519,
        expiry_secs: None,
    })
    .expect("keygen")
}

fn gen(algo: api::KeyAlgo) -> api::KeyBundle {
    api::generate_key(api::GenerateOptions {
        name: "Multi Algo".into(),
        email: "multi@example.com".into(),
        comment: None,
        passphrase: PASS.into(),
        algo,
        expiry_secs: None,
    })
    .expect("keygen")
}

/// Every offered algorithm must generate a usable key that can encrypt→decrypt
/// (and, where it signs, verify). Covers legacy (RSA/NIST) through modern
/// (Ed448) and post-quantum (ML-KEM/ML-DSA) families.
#[test]
fn all_algorithms_encrypt_decrypt_roundtrip() {
    let algos = [
        api::KeyAlgo::Ed25519,
        api::KeyAlgo::Rsa2048,
        api::KeyAlgo::NistP256,
        api::KeyAlgo::NistP384,
        api::KeyAlgo::Ed448,
        api::KeyAlgo::PostQuantum,
    ];
    for algo in algos {
        let bundle = gen(algo);
        assert!(bundle.info.has_secret, "{algo:?} should have secret");
        assert!(
            bundle.info.subkeys.iter().any(|s| s.can_encrypt),
            "{algo:?} should expose an encryption subkey"
        );
        let plaintext = format!("payload for {algo:?}").into_bytes();
        let ct = api::encrypt(&plaintext, vec![bundle.public_key.clone()], None, None, true)
            .unwrap_or_else(|e| panic!("encrypt {algo:?}: {}", e.message));
        let res = api::decrypt(&ct, vec![bundle.secret_key.clone()], PASS.into(), vec![])
            .unwrap_or_else(|e| panic!("decrypt {algo:?}: {}", e.message));
        assert_eq!(res.data, plaintext, "roundtrip mismatch for {algo:?}");
    }
}

#[test]
fn keygen_produces_valid_metadata() {
    let bundle = gen_ed25519();
    assert!(bundle.public_key.contains("BEGIN PGP PUBLIC KEY"));
    assert!(bundle.secret_key.contains("BEGIN PGP PRIVATE KEY"));
    assert!(bundle.info.has_secret);
    assert_eq!(bundle.info.fingerprint.len(), 40); // v4 SHA-1 fingerprint
    assert!(bundle.info.can_sign);
    assert!(!bundle.info.subkeys.is_empty());
    assert!(bundle.info.subkeys[0].can_encrypt);
    assert_eq!(
        bundle.info.primary_user_id.as_deref(),
        Some("Alice Example (test) <alice@example.com>")
    );
}

#[test]
fn secret_key_is_passphrase_protected() {
    let bundle = gen_ed25519();
    // The armored secret must not contain unlocked key material markers; the
    // simplest robust check: re-parsing works and it is recognised as secret.
    let info = api::parse_key(bundle.secret_key.as_bytes()).expect("parse secret");
    assert!(info.has_secret);
    // A wrong passphrase must fail to decrypt anything signed/encrypted by it.
    let other = gen_ed25519();
    let ct = api::encrypt(b"x", vec![other.public_key.clone()], None, None, true).unwrap();
    let err = api::decrypt(
        &ct,
        vec![other.secret_key.clone()],
        "wrong-pass".into(),
        vec![],
    )
    .unwrap_err();
    assert!(matches!(
        err.code,
        api::ErrorCode::WrongPassphrase | api::ErrorCode::NoMatchingKey
    ));
}

#[test]
fn encrypt_decrypt_roundtrip() {
    let bundle = gen_ed25519();
    let plaintext = b"The quick brown fox jumps over the lazy dog.";
    let ct = api::encrypt(plaintext, vec![bundle.public_key.clone()], None, None, false)
        .expect("encrypt");
    // Output must be valid OpenPGP (starts with a packet tag byte, not armor).
    assert!(ct[0] & 0x80 != 0, "first byte should be an OpenPGP packet tag");

    let res = api::decrypt(&ct, vec![bundle.secret_key.clone()], PASS.into(), vec![])
        .expect("decrypt");
    assert_eq!(res.data, plaintext);
    assert!(res.signatures.is_empty());
}

#[test]
fn encrypt_sign_then_decrypt_verifies() {
    let alice = gen_ed25519();
    let plaintext = b"signed and encrypted";
    let ct = api::encrypt(
        plaintext,
        vec![alice.public_key.clone()],
        Some(alice.secret_key.clone()),
        Some(PASS.into()),
        true,
    )
    .expect("encrypt+sign");

    let res = api::decrypt(
        &ct,
        vec![alice.secret_key.clone()],
        PASS.into(),
        vec![alice.public_key.clone()],
    )
    .expect("decrypt");
    assert_eq!(res.data, plaintext);
    assert_eq!(res.signatures.len(), 1);
    assert_eq!(res.signatures[0].status, api::SignatureStatus::Valid);
}

#[test]
fn detached_sign_and_verify() {
    let alice = gen_ed25519();
    let data = b"detached signature target";
    let sig = api::sign_detached(data, &alice.secret_key, PASS, true).expect("sign");
    assert!(sig.starts_with(b"-----BEGIN PGP SIGNATURE"));

    let res = api::verify_detached(data, &sig, vec![alice.public_key.clone()]).expect("verify");
    assert_eq!(res.signatures[0].status, api::SignatureStatus::Valid);

    // Tampered data must not verify as valid.
    let bad = api::verify_detached(b"tampered", &sig, vec![alice.public_key.clone()]).unwrap();
    assert_ne!(bad.signatures[0].status, api::SignatureStatus::Valid);
}

#[test]
fn cleartext_sign_and_verify() {
    let alice = gen_ed25519();
    let text = "hello\nworld\n";
    let signed = api::sign_cleartext(text, &alice.secret_key, PASS).expect("clearsign");
    assert!(signed.contains("BEGIN PGP SIGNED MESSAGE"));
    let res = api::verify_cleartext(&signed, vec![alice.public_key.clone()]).expect("verify");
    assert_eq!(res.text.trim_end(), text.trim_end());
    assert_eq!(res.signatures[0].status, api::SignatureStatus::Valid);
}

#[test]
fn wrong_passphrase_on_detached_sign_fails() {
    let alice = gen_ed25519();
    let err = api::sign_detached(b"x", &alice.secret_key, "nope", true).unwrap_err();
    assert!(matches!(
        err.code,
        api::ErrorCode::WrongPassphrase | api::ErrorCode::Internal
    ));
}

#[test]
fn revocation_certificate_is_generated() {
    let alice = gen_ed25519();
    let rev = api::generate_revocation(alice.secret_key.as_bytes(), PASS, 1, "superseded")
        .expect("revocation");
    assert!(rev.contains("BEGIN PGP SIGNATURE") || rev.contains("BEGIN PGP PUBLIC KEY"));
}

#[test]
fn vault_seal_open_roundtrip() {
    let data = b"keyring blob bytes";
    let blob = api::vault_seal(data, "vault-pass").expect("seal");
    assert_ne!(&blob[..], &data[..]);
    let opened = api::vault_open(&blob, "vault-pass").expect("open");
    assert_eq!(opened, data);
    // Wrong passphrase must fail.
    assert!(api::vault_open(&blob, "wrong").is_err());
}

// --- GnuPG interoperability ------------------------------------------------
//
// These fixtures contain a throwaway GnuPG private key, so they are NOT
// committed (see .gitignore). Regenerate them locally with
// `scripts/gen-fixtures.sh`; the tests skip cleanly when the fixtures are
// absent (e.g. a fresh clone).

/// Read a fixture relative to the crate root; `None` if it isn't present.
fn read_fixture(name: &str) -> Option<Vec<u8>> {
    std::fs::read(format!("tests/fixtures/{name}")).ok()
}

#[test]
fn decrypts_message_created_by_gnupg() {
    // Fixtures produced by GnuPG 2.4 (ed25519 / cv25519 key, passphrase "test1234").
    let (Some(seckey), Some(msg)) = (read_fixture("seckey.asc"), read_fixture("msg.asc")) else {
        eprintln!("skipping GnuPG interop: fixtures absent (run scripts/gen-fixtures.sh)");
        return;
    };
    let seckey = String::from_utf8(seckey).expect("armored secret key is UTF-8");
    let res = api::decrypt(&msg, vec![seckey], "test1234".into(), vec![])
        .expect("decrypt GnuPG message");
    assert_eq!(res.data, b"Hello from GnuPG interop!\n");
}

/// Modern GnuPG emits its proprietary OCB/AEAD packet (type 20) by default for
/// AEAD-capable keys. Real users have such files, so we must decrypt them.
#[test]
fn decrypts_gnupg_aead_message() {
    let (Some(seckey), Some(msg)) = (read_fixture("aead-sec.asc"), read_fixture("aead.asc")) else {
        eprintln!("skipping GnuPG AEAD interop: fixtures absent (run scripts/gen-fixtures.sh)");
        return;
    };
    let seckey = String::from_utf8(seckey).expect("armored secret key is UTF-8");
    let res = api::decrypt(&msg, vec![seckey], "test1234".into(), vec![])
        .expect("decrypt GnuPG OCB/AEAD message");
    assert_eq!(res.data, b"AEAD interop works!\n");
}

#[test]
fn parses_gnupg_public_key_metadata() {
    let Some(pubkey) = read_fixture("pubkey.asc") else {
        eprintln!("skipping GnuPG interop: fixtures absent (run scripts/gen-fixtures.sh)");
        return;
    };
    let info = api::parse_key(&pubkey).expect("parse gpg pubkey");
    assert!(!info.has_secret);
    assert!(info
        .primary_user_id
        .as_deref()
        .unwrap_or("")
        .contains("fixture@example.com"));
    assert!(info.subkeys.iter().any(|s| s.can_encrypt));
}
