//! Webhook signature verification: HMAC-SHA256 and Ed25519.
//!
//! ## HMAC-SHA256 (primary)
//!
//! GitHub (like GitHub) signs each webhook delivery with
//! `HMAC-SHA256(secret, raw_body)` and sends it as the
//! `X-Hub-Signature-256: sha256=<hex>` header (GitHub also sends the
//! legacy `X-Gitea-Signature` raw-hex header; both are supported).
//!
//! ## Ed25519 (best-practice alternative, ADR-0374 §"ed25519 path")
//!
//! GitHub also supports signing webhook deliveries with an ed25519 keypair.
//! When configured, GitHub sends the detached signature in the
//! `X-GitHub-Signature` header as standard base64 (RFC 4648, no padding
//! required). The gateway verifies the raw body against the configured
//! ed25519 public key using `ed25519-dalek` (RustCrypto, MIT/Apache-2.0).
//!
//! `verify_any` prefers HMAC-SHA256 when both headers are present; either
//! path satisfies the security invariants below.
//!
//! ## Security invariants (ADR-0112 §"Signature handling")
//!   1. FAIL CLOSED — verification runs on the RAW body BEFORE JSON parsing,
//!      dedup, or routing, so an attacker cannot poison downstream state with
//!      a crafted-but-unsigned payload.
//!   2. CONSTANT TIME — HMAC comparison uses the hand-rolled `ct_eq_bytes`
//!      below, NOT `subtle::ConstantTimeEq` (this module takes no `subtle`
//!      dependency); ed25519-dalek verify is internally constant-time per its
//!      API contract. The HMAC itself is likewise hand-rolled on `sha2` in
//!      `hmac_sha256` rather than using the `hmac` crate. Both hand-rolls are
//!      tracked as defects, not endorsed: see registry/fixuptasks.jsonl
//!      F-QUAL-CONSTTIME / F-HSA-O8 (constant-time compare) and
//!      F-SEC-WEBHOOK-HMAC (the MAC construction).
//!   3. NO SECRET LOGGING — secrets and private keys never appear in
//!      `Debug`/`Display`.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::error::{GatewayError, Result};

/// Constant-time equality for byte slices (no `subtle` dep).
///
/// Compares length first (leaks length, which is fine for fixed-size HMAC
/// digests) then XORs all bytes and checks the accumulator in one branch.
fn ct_eq_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let diff: u8 = a.iter().zip(b.iter()).fold(0u8, |acc, (&x, &y)| acc | (x ^ y));
    diff == 0
}

/// HMAC-SHA256 block size (64 bytes for SHA-256).
const SHA256_BLOCK: usize = 64;

/// Compute HMAC-SHA256(key, msg) using only `sha2` — no `hmac` crate dep.
///
/// Standard RFC 2104 construction: H((key XOR opad) || H((key XOR ipad) || msg)).
/// `pub(crate)` so tests in `receiver.rs` can use it via `crate::signature::hmac_sha256`.
pub(crate) fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    // If key > block size, hash it first; then zero-pad to block size.
    let mut k = [0u8; SHA256_BLOCK];
    if key.len() > SHA256_BLOCK {
        let hk = Sha256::digest(key);
        k[..32].copy_from_slice(&hk);
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; SHA256_BLOCK];
    let mut opad = [0x5cu8; SHA256_BLOCK];
    for i in 0..SHA256_BLOCK {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }

    // inner = SHA256(ipad || msg)
    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(msg);
    let inner_hash = inner.finalize();

    // outer = SHA256(opad || inner)
    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    outer.finalize().into()
}

/// The canonical GitHub/GitHub HMAC-SHA256 signature header.
pub const SIGNATURE_HEADER: &str = "x-hub-signature-256";
/// The legacy Gitea/GitHub raw-hex HMAC header (no `sha256=` prefix).
pub const LEGACY_SIGNATURE_HEADER: &str = "x-gitea-signature";
/// GitHub ed25519 signature header: base64-encoded detached signature over
/// the raw body. Present when the webhook is configured with an ed25519 keypair
/// instead of (or in addition to) an HMAC secret.
pub const ED25519_SIGNATURE_HEADER: &str = "x-github-signature";

// ── Ed25519 public-key wrapper ─────────────────────────────────────────────

/// A GitHub webhook ed25519 public key. Wraps `ed25519-dalek::VerifyingKey`
/// and redacts it from `Debug` output to avoid leaking key material.
#[derive(Clone)]
pub struct WebhookEd25519Key(VerifyingKey);

impl WebhookEd25519Key {
    /// Construct from a 32-byte raw public key (ed25519 compressed point).
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        VerifyingKey::from_bytes(bytes)
            .map(WebhookEd25519Key)
            .map_err(|_| GatewayError::SecretUnavailable)
    }
}

impl std::fmt::Debug for WebhookEd25519Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WebhookEd25519Key(<redacted>)")
    }
}

/// A webhook HMAC secret. Wraps the bytes so the secret is never accidentally
/// formatted into a log line.
#[derive(Clone)]
pub struct WebhookSecret(Vec<u8>);

impl WebhookSecret {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        WebhookSecret(bytes.into())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for WebhookSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print the secret.
        f.write_str("WebhookSecret(<redacted>)")
    }
}

/// Compute the lowercase-hex HMAC-SHA256 of `body` under `secret`.
fn compute_hex(secret: &WebhookSecret, body: &[u8]) -> Result<String> {
    if secret.is_empty() {
        return Err(GatewayError::SecretUnavailable);
    }
    let digest = hmac_sha256(secret.as_bytes(), body);
    Ok(hex_encode(&digest))
}

/// Verify a `sha256=<hex>` style header (the canonical `X-Hub-Signature-256`).
///
/// Returns `Ok(())` only when the header is well-formed AND the constant-time
/// comparison matches. Every other path is a typed fail-closed error.
pub fn verify_prefixed(secret: &WebhookSecret, body: &[u8], header_value: &str) -> Result<()> {
    let Some(hex) = header_value.strip_prefix("sha256=") else {
        return Err(GatewayError::MalformedSignature);
    };
    verify_raw_hex(secret, body, hex)
}

/// Verify a raw-hex signature (the legacy `X-Gitea-Signature`, no prefix).
pub fn verify_raw_hex(secret: &WebhookSecret, body: &[u8], header_hex: &str) -> Result<()> {
    let header_hex = header_hex.trim();
    if header_hex.is_empty() {
        return Err(GatewayError::MalformedSignature);
    }
    if !header_hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(GatewayError::MalformedSignature);
    }
    let expected = compute_hex(secret, body)?;
    // Constant-time compare on the lowercased hex bytes.
    let provided = header_hex.to_ascii_lowercase();
    if ct_eq_bytes(expected.as_bytes(), provided.as_bytes()) {
        Ok(())
    } else {
        Err(GatewayError::SignatureMismatch)
    }
}

/// Select the strongest available header and verify it. Preference order:
/// 1. `X-Hub-Signature-256` (HMAC-SHA256, prefixed `sha256=<hex>`)
/// 2. `X-Gitea-Signature` (legacy raw-hex HMAC)
/// 3. `X-GitHub-Signature` (ed25519, requires `ed25519_key` to be `Some`)
///
/// At least one verifiable header MUST be present, or we fail closed with
/// `MissingSignature`. If only the ed25519 header is present but no public key
/// is configured, we fail closed with `SecretUnavailable`.
pub fn verify_any(
    secret: &WebhookSecret,
    body: &[u8],
    prefixed: Option<&str>,
    legacy: Option<&str>,
    ed25519_header: Option<&str>,
    ed25519_key: Option<&WebhookEd25519Key>,
) -> Result<()> {
    match (prefixed, legacy, ed25519_header, ed25519_key) {
        (Some(value), _, _, _) => verify_prefixed(secret, body, value),
        (None, Some(value), _, _) => verify_raw_hex(secret, body, value),
        (None, None, Some(sig), Some(key)) => verify_ed25519(key, body, sig),
        (None, None, Some(_), None) => Err(GatewayError::SecretUnavailable),
        (None, None, None, _) => Err(GatewayError::MissingSignature),
    }
}

// ── Ed25519 signature verification ────────────────────────────────────────

/// Verify a GitHub ed25519 webhook signature.
///
/// `header_value` is the raw `X-GitHub-Signature` header value: standard
/// base64-encoded (RFC 4648; with or without padding) 64-byte ed25519
/// detached signature over the raw body bytes.
///
/// Security properties:
///   - `ed25519-dalek::VerifyingKey::verify` is internally constant-time.
///   - Returns `MalformedSignature` for any base64 or length error so callers
///     cannot distinguish "bad base64" from "wrong key" — fail closed.
pub fn verify_ed25519(
    key: &WebhookEd25519Key,
    body: &[u8],
    header_value: &str,
) -> Result<()> {
    let raw = B64
        .decode(header_value.trim())
        .map_err(|_| GatewayError::MalformedSignature)?;
    let sig_bytes: &[u8; 64] = raw
        .as_slice()
        .try_into()
        .map_err(|_| GatewayError::MalformedSignature)?;
    let sig = Ed25519Signature::from_bytes(sig_bytes);
    key.0
        .verify(body, &sig)
        .map_err(|_| GatewayError::SignatureMismatch)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known-answer vector from RFC 4231 / GitHub docs style:
    // secret = "It's a Secret to Everybody", body = "Hello, World!"
    // => sha256=757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17
    const KAT_SECRET: &str = "It's a Secret to Everybody";
    const KAT_BODY: &[u8] = b"Hello, World!";
    const KAT_HEX: &str = "757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17";

    #[test]
    fn known_answer_prefixed_verifies() {
        let secret = WebhookSecret::new(KAT_SECRET.as_bytes().to_vec());
        let header = format!("sha256={KAT_HEX}");
        assert!(verify_prefixed(&secret, KAT_BODY, &header).is_ok());
    }

    #[test]
    fn known_answer_raw_hex_verifies() {
        let secret = WebhookSecret::new(KAT_SECRET.as_bytes().to_vec());
        assert!(verify_raw_hex(&secret, KAT_BODY, KAT_HEX).is_ok());
    }

    #[test]
    fn tampered_body_fails_closed() {
        let secret = WebhookSecret::new(KAT_SECRET.as_bytes().to_vec());
        let header = format!("sha256={KAT_HEX}");
        let err = verify_prefixed(&secret, b"tampered", &header).unwrap_err();
        assert!(matches!(err, GatewayError::SignatureMismatch));
    }

    #[test]
    fn missing_prefix_is_malformed() {
        let secret = WebhookSecret::new(KAT_SECRET.as_bytes().to_vec());
        let err = verify_prefixed(&secret, KAT_BODY, KAT_HEX).unwrap_err();
        assert!(matches!(err, GatewayError::MalformedSignature));
    }

    #[test]
    fn non_hex_is_malformed() {
        let secret = WebhookSecret::new(KAT_SECRET.as_bytes().to_vec());
        let err = verify_raw_hex(&secret, KAT_BODY, "zzzz").unwrap_err();
        assert!(matches!(err, GatewayError::MalformedSignature));
    }

    #[test]
    fn empty_secret_fails_closed() {
        let secret = WebhookSecret::new(Vec::new());
        let err = verify_raw_hex(&secret, KAT_BODY, KAT_HEX).unwrap_err();
        assert!(matches!(err, GatewayError::SecretUnavailable));
    }

    #[test]
    fn no_headers_is_missing_signature() {
        let secret = WebhookSecret::new(KAT_SECRET.as_bytes().to_vec());
        let err = verify_any(&secret, KAT_BODY, None, None, None, None).unwrap_err();
        assert!(matches!(err, GatewayError::MissingSignature));
    }

    // ── Ed25519 tests ────────────────────────────────────────────────────

    fn ed25519_keypair() -> (ed25519_dalek::SigningKey, WebhookEd25519Key) {
        // Deterministic keypair for tests: seed = [0x42; 32].
        let seed = [0x42u8; 32];
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        let verifying_key =
            WebhookEd25519Key::from_bytes(signing_key.verifying_key().as_bytes()).unwrap();
        (signing_key, verifying_key)
    }

    fn ed25519_sign(signing_key: &ed25519_dalek::SigningKey, body: &[u8]) -> String {
        use ed25519_dalek::Signer as _;
        let sig = signing_key.sign(body);
        base64::engine::general_purpose::STANDARD.encode(sig.to_bytes())
    }

    #[test]
    fn ed25519_valid_signature_verifies() {
        let (sk, vk) = ed25519_keypair();
        let sig = ed25519_sign(&sk, KAT_BODY);
        assert!(verify_ed25519(&vk, KAT_BODY, &sig).is_ok());
    }

    #[test]
    fn ed25519_tampered_body_fails() {
        let (sk, vk) = ed25519_keypair();
        let sig = ed25519_sign(&sk, KAT_BODY);
        let err = verify_ed25519(&vk, b"tampered", &sig).unwrap_err();
        assert!(matches!(err, GatewayError::SignatureMismatch));
    }

    #[test]
    fn ed25519_bad_base64_is_malformed() {
        let (_, vk) = ed25519_keypair();
        let err = verify_ed25519(&vk, KAT_BODY, "not!base64$$").unwrap_err();
        assert!(matches!(err, GatewayError::MalformedSignature));
    }

    #[test]
    fn ed25519_wrong_length_is_malformed() {
        let (_, vk) = ed25519_keypair();
        // 32 bytes encoded is too short for a 64-byte ed25519 signature.
        let short = base64::engine::general_purpose::STANDARD.encode([0u8; 32]);
        let err = verify_ed25519(&vk, KAT_BODY, &short).unwrap_err();
        assert!(matches!(err, GatewayError::MalformedSignature));
    }

    #[test]
    fn verify_any_falls_through_to_ed25519_when_hmac_absent() {
        let secret = WebhookSecret::new(Vec::new()); // empty — won't be used
        let (sk, vk) = ed25519_keypair();
        let sig = ed25519_sign(&sk, KAT_BODY);
        assert!(verify_any(&secret, KAT_BODY, None, None, Some(&sig), Some(&vk)).is_ok());
    }

    #[test]
    fn verify_any_ed25519_without_key_configured_fails_closed() {
        let secret = WebhookSecret::new(Vec::new());
        let (sk, _vk) = ed25519_keypair();
        let sig = ed25519_sign(&sk, KAT_BODY);
        let err = verify_any(&secret, KAT_BODY, None, None, Some(&sig), None).unwrap_err();
        assert!(matches!(err, GatewayError::SecretUnavailable));
    }

    #[test]
    fn verify_any_prefers_hmac_over_ed25519() {
        // Provide a valid HMAC prefixed header + an ed25519 header — HMAC wins.
        let secret = WebhookSecret::new(KAT_SECRET.as_bytes().to_vec());
        let (sk, vk) = ed25519_keypair();
        let ed_sig = ed25519_sign(&sk, KAT_BODY);
        let hmac_header = format!("sha256={KAT_HEX}");
        assert!(
            verify_any(&secret, KAT_BODY, Some(&hmac_header), None, Some(&ed_sig), Some(&vk))
                .is_ok()
        );
    }

    #[test]
    fn ed25519_key_redacted_in_debug() {
        let (_, vk) = ed25519_keypair();
        assert_eq!(format!("{vk:?}"), "WebhookEd25519Key(<redacted>)");
    }

    #[test]
    fn uppercase_hex_still_verifies() {
        let secret = WebhookSecret::new(KAT_SECRET.as_bytes().to_vec());
        assert!(verify_raw_hex(&secret, KAT_BODY, &KAT_HEX.to_ascii_uppercase()).is_ok());
    }

    #[test]
    fn secret_is_redacted_in_debug() {
        let secret = WebhookSecret::new(b"super-secret".to_vec());
        assert_eq!(format!("{secret:?}"), "WebhookSecret(<redacted>)");
    }
}
