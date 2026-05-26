//! HMAC-SHA256 webhook signature verification.
//!
//! Forgejo (like GitHub) signs each webhook delivery with
//! `HMAC-SHA256(secret, raw_body)` and sends it as the
//! `X-Hub-Signature-256: sha256=<hex>` header (Forgejo also sends the
//! legacy `X-Gitea-Signature` raw-hex header; both are supported).
//!
//! Security invariants (ADR-0112 §"Signature handling", carried forward to
//! the ADR-0363 Forgejo substrate):
//!   1. FAIL CLOSED — verification runs on the RAW body BEFORE JSON parsing,
//!      dedup, or routing, so an attacker cannot poison downstream state with
//!      a crafted-but-unsigned payload.
//!   2. CONSTANT TIME — the digest comparison uses `subtle::ConstantTimeEq`
//!      so the verifier does not leak the secret via timing.
//!   3. NO SECRET LOGGING — the secret never appears in `Debug`/`Display`.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::error::{GatewayError, Result};

type HmacSha256 = Hmac<Sha256>;

/// The canonical Forgejo/GitHub signature header.
pub const SIGNATURE_HEADER: &str = "x-hub-signature-256";
/// The legacy Gitea/Forgejo raw-hex header (no `sha256=` prefix).
pub const LEGACY_SIGNATURE_HEADER: &str = "x-gitea-signature";

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
    // `new_from_slice` only errors on a zero-length key, which we reject above.
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| GatewayError::SecretUnavailable)?;
    mac.update(body);
    let digest = mac.finalize().into_bytes();
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
    let matches: bool = expected.as_bytes().ct_eq(provided.as_bytes()).into();
    if matches {
        Ok(())
    } else {
        Err(GatewayError::SignatureMismatch)
    }
}

/// Select the strongest available header and verify it. `prefixed` is the
/// `X-Hub-Signature-256` value; `legacy` is the `X-Gitea-Signature` value.
/// At least one MUST be present, or we fail closed with `MissingSignature`.
pub fn verify_any(
    secret: &WebhookSecret,
    body: &[u8],
    prefixed: Option<&str>,
    legacy: Option<&str>,
) -> Result<()> {
    match (prefixed, legacy) {
        (Some(value), _) => verify_prefixed(secret, body, value),
        (None, Some(value)) => verify_raw_hex(secret, body, value),
        (None, None) => Err(GatewayError::MissingSignature),
    }
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
        let err = verify_any(&secret, KAT_BODY, None, None).unwrap_err();
        assert!(matches!(err, GatewayError::MissingSignature));
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
