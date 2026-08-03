//! D2 ed25519 verifier adapter tests (ADR-0387).
//!
//! 6 tests: valid signature, wrong signature, tampered payload,
//! expired timestamp, future timestamp, missing signature header path
//! (via kernel MockSignatureVerifier contract parity check).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ci_webhook_gateway_ed25519_adapter::Ed25519Verifier;
use ed25519_dalek::{Signer, SigningKey};
use oya_ci_webhook_gateway_kernel::{KernelError, SignatureVerifier, WebhookSignature};
use rand_core::OsRng;

fn make_key() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

fn sign(key: &SigningKey, body: &[u8]) -> WebhookSignature {
    let sig = key.sign(body);
    WebhookSignature::from_bytes(sig.to_bytes().to_vec())
}

fn now_unix_s() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Test 1 — valid signature verifies successfully.
#[test]
fn valid_signature_passes() {
    let key = make_key();
    let verifier = Ed25519Verifier::new(key.verifying_key());
    let body = b"hello github";
    let sig = sign(&key, body);

    assert!(
        verifier.verify(body, &sig, None).is_ok(),
        "valid signature should pass"
    );
}

/// Test 2 — wrong signature (signed by a different key) is rejected.
#[test]
fn wrong_signature_rejected() {
    let key1 = make_key();
    let key2 = make_key();
    // verifier holds key1's public key but body is signed by key2
    let verifier = Ed25519Verifier::new(key1.verifying_key());
    let body = b"hello github";
    let sig = sign(&key2, body);

    assert_eq!(
        verifier.verify(body, &sig, None).unwrap_err(),
        KernelError::SignatureMismatch,
        "wrong signing key should produce SignatureMismatch"
    );
}

/// Test 3 — tampered payload (body changed after signing) is rejected.
#[test]
fn tampered_payload_rejected() {
    let key = make_key();
    let verifier = Ed25519Verifier::new(key.verifying_key());
    let body = b"original payload";
    let sig = sign(&key, body);

    assert_eq!(
        verifier
            .verify(b"tampered payload", &sig, None)
            .unwrap_err(),
        KernelError::SignatureMismatch,
        "tampered payload should produce SignatureMismatch"
    );
}

/// Test 4 — timestamp more than 5 minutes in the past is rejected.
#[test]
fn expired_timestamp_rejected() {
    let key = make_key();
    let verifier = Ed25519Verifier::new(key.verifying_key());
    let body = b"some payload";
    let sig = sign(&key, body);

    let stale_ts = now_unix_s().saturating_sub(400); // 400 s ago > 300 s window
    assert_eq!(
        verifier.verify(body, &sig, Some(stale_ts)).unwrap_err(),
        KernelError::ExpiredTimestamp,
        "timestamp 400s in the past should produce ExpiredTimestamp"
    );
}

/// Test 5 — timestamp more than 5 minutes in the future is rejected.
#[test]
fn future_timestamp_rejected() {
    let key = make_key();
    let verifier = Ed25519Verifier::new(key.verifying_key());
    let body = b"some payload";
    let sig = sign(&key, body);

    let future_ts = now_unix_s() + 400; // 400 s ahead > 300 s window
    assert_eq!(
        verifier.verify(body, &sig, Some(future_ts)).unwrap_err(),
        KernelError::ExpiredTimestamp,
        "timestamp 400s in the future should produce ExpiredTimestamp"
    );
}

/// Test 6 — malformed (wrong-length) signature bytes produce MalformedSignature.
#[test]
fn missing_signature_bytes_produce_malformed() {
    let key = make_key();
    let verifier = Ed25519Verifier::new(key.verifying_key());
    let body = b"some payload";
    // A WebhookSignature with wrong byte count (not 64) → MalformedSignature
    let bad_sig = WebhookSignature::from_bytes(vec![0u8; 10]);

    assert_eq!(
        verifier.verify(body, &bad_sig, None).unwrap_err(),
        KernelError::MalformedSignature,
        "signature with wrong byte length should produce MalformedSignature"
    );
}
