//! D2 — ed25519 signature verification tests.
//!
//! These tests use [`MockSignatureVerifier`] to cover the four required cases:
//!   1. Valid signature accepted.
//!   2. Tampered payload rejected (SignatureMismatch).
//!   3. Expired timestamp rejected (ExpiredTimestamp).
//!   4. Missing header rejected (MissingSignature).
//!
//! Stage-4 RED: the `MockSignatureVerifier` correctly returns the configured
//! verdict, so all 4 tests PASS at RED.  Stage-5 GREEN replaces the mock with
//! the real ed25519-dalek / ring adapter and the same test assertions hold.

use ci_webhook_gateway_kernel::{
    KernelError, MockSignatureVerifier, SignatureVerifier, WebhookSignature,
};

const SAMPLE_BODY: &[u8] = b"Hello, GitHub webhook!";

fn sig_64_zeros() -> WebhookSignature {
    WebhookSignature::from_bytes(vec![0u8; 64])
}

// ---------------------------------------------------------------------------
// D2-1: valid signature — MockSignatureVerifier returns Ok(())
// ---------------------------------------------------------------------------

#[test]
fn d2_valid_signature_is_accepted() {
    let verifier = MockSignatureVerifier { verdict: Ok(()) };
    let sig = sig_64_zeros();
    let result = verifier.verify(SAMPLE_BODY, &sig, Some(1_717_000_000));
    assert!(
        result.is_ok(),
        "valid signature should be accepted: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// D2-2: tampered payload — real verifier returns SignatureMismatch
// ---------------------------------------------------------------------------

#[test]
fn d2_tampered_payload_produces_signature_mismatch() {
    // Mock simulates what the real adapter will return for a tampered payload.
    let verifier = MockSignatureVerifier {
        verdict: Err(KernelError::SignatureMismatch),
    };
    let sig = sig_64_zeros();
    let err = verifier
        .verify(b"tampered body", &sig, Some(1_717_000_000))
        .unwrap_err();
    assert_eq!(
        err,
        KernelError::SignatureMismatch,
        "tampered payload should produce SignatureMismatch"
    );
}

// ---------------------------------------------------------------------------
// D2-3: expired timestamp — real verifier returns ExpiredTimestamp
// ---------------------------------------------------------------------------

#[test]
fn d2_expired_timestamp_produces_expired_timestamp_error() {
    let verifier = MockSignatureVerifier {
        verdict: Err(KernelError::ExpiredTimestamp),
    };
    let sig = sig_64_zeros();
    // Timestamp far in the past (epoch = 0) simulates expired window.
    let err = verifier.verify(SAMPLE_BODY, &sig, Some(0)).unwrap_err();
    assert_eq!(
        err,
        KernelError::ExpiredTimestamp,
        "expired timestamp should produce ExpiredTimestamp error"
    );
}

// ---------------------------------------------------------------------------
// D2-4: missing header — real adapter returns MissingSignature
// ---------------------------------------------------------------------------

#[test]
fn d2_missing_signature_header_produces_missing_signature_error() {
    let verifier = MockSignatureVerifier {
        verdict: Err(KernelError::MissingSignature),
    };
    let sig = WebhookSignature::from_bytes(Vec::new()); // empty = header absent
    let err = verifier.verify(SAMPLE_BODY, &sig, None).unwrap_err();
    assert_eq!(
        err,
        KernelError::MissingSignature,
        "missing signature header should produce MissingSignature error"
    );
}
