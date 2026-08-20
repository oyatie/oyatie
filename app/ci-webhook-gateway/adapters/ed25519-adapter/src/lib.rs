//! # ci-webhook-gateway-ed25519-adapter
//!
//! ed25519 signature verifier adapter for the CI webhook gateway (ADR-0387 D2).
//!
//! Implements [`SignatureVerifier`] using `ed25519-dalek 2.x`.
//! The public key is injected via constructor — OpenBao client integration
//! is deferred to Stage-7.
//!
//! ## Security invariants
//!
//! - `VerifyingKey::verify_strict` provides constant-time comparison.
//! - Timestamp window is ±300 seconds (5 minutes).
//! - Signature is verified on raw bytes BEFORE any JSON parsing.
//! - ADR-0083 Tier-3: no `unwrap`/`expect`/`panic` on the request path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use ci_webhook_gateway_kernel::{KernelError, Result, SignatureVerifier, WebhookSignature};
use ed25519_dalek::{Signature, VerifyingKey};

/// Timestamp tolerance window in seconds (±5 minutes).
const TIMESTAMP_TOLERANCE_S: u64 = 300;

/// ed25519 [`SignatureVerifier`] adapter.
///
/// The public key is injected at construction time.  On the request path,
/// `verify_strict` is used for constant-time byte comparison.
pub struct Ed25519Verifier {
    verifying_key: VerifyingKey, // data_class: INTERNAL_ONLY
}

impl Ed25519Verifier {
    /// Construct with a caller-supplied public key.
    ///
    /// The key is typically loaded from the keyring / OpenBao at startup
    /// (Stage-7 will wire the OpenBao client here).
    pub fn new(public_key: VerifyingKey) -> Self {
        Self {
            verifying_key: public_key,
        }
    }
}

impl SignatureVerifier for Ed25519Verifier {
    /// Verify the ed25519 `signature` over `body`.
    ///
    /// If `timestamp_unix_s` is `Some`, it is checked against the current
    /// system time; values outside ±[`TIMESTAMP_TOLERANCE_S`] return
    /// [`KernelError::ExpiredTimestamp`].
    fn verify(
        &self,
        body: &[u8],
        signature: &WebhookSignature,
        timestamp_unix_s: Option<u64>,
    ) -> Result<()> {
        // D2 timestamp window check — must happen before crypto work.
        if let Some(ts) = timestamp_unix_s {
            let now = current_unix_s();
            let delta = now.max(ts) - now.min(ts);
            if delta > TIMESTAMP_TOLERANCE_S {
                return Err(KernelError::ExpiredTimestamp);
            }
        }

        // Parse the 64-byte ed25519 signature from the WebhookSignature bytes.
        let sig_bytes: &[u8; 64] = signature
            .as_bytes()
            .try_into()
            .map_err(|_| KernelError::MalformedSignature)?;
        let dalek_sig = Signature::from_bytes(sig_bytes);

        // Constant-time verification (ADR-0387 D2 security invariant).
        self.verifying_key
            .verify_strict(body, &dalek_sig)
            .map_err(|_| KernelError::SignatureMismatch)
    }
}

/// Returns the current UNIX timestamp in seconds.
/// On the request path this is the only source of time; no panic possible.
fn current_unix_s() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
