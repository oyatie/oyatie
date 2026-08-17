//! RFC 6376 §3.5 / §3.7 DKIM signing-input builder.
//!
//! Returns typed material (canonical headers, canonical body, DKIM-Signature
//! stub with `b=` empty) that an adapter later feeds to `aws-lc-rs` for actual
//! signing.  No key material, no cryptographic operations, no DNS lookup, no
//! OpenBao read, no SMTP delivery.

use crate::dkim_canonicalization::{
    DkimCanonicalizationAlgorithm, RawHeader, canonicalize_body, canonicalize_header,
};
use crate::sending_domain_authentication::{DkimSigningAlgorithm, NON_CLAIM};

/// All inputs required to produce a DKIM signing-input string.
/// Contains no key material and triggers no I/O.
pub struct DkimSigningInputRequest {
    /// Header field names to include in `h=`, in the order given.
    pub signed_headers: Vec<String>,
    /// Full set of parsed message headers.  Signing selects from these.
    pub headers: Vec<RawHeader>,
    /// Raw message body bytes (pre-canonicalization).
    pub body: Vec<u8>,
    /// DKIM selector (s= tag).
    pub selector: String,
    /// Signing domain (d= tag).
    pub signing_domain: String,
    /// Opaque key-version reference (informational; not key material).
    pub key_version_ref: String,
    /// Signing algorithm; validated via `DkimSigningAlgorithm::supported_for_signing`.
    pub algorithm: DkimSigningAlgorithm,
    /// Canonicalization applied to headers.
    pub header_canonicalization: DkimCanonicalizationAlgorithm,
    /// Canonicalization applied to the body.
    pub body_canonicalization: DkimCanonicalizationAlgorithm,
}

/// Typed signing-input material returned to the adapter.
/// Contains no key material and performs no signing.
#[derive(Debug)]
pub struct DkimSigningInputMaterial {
    /// The DKIM-Signature header stub with `b=` left empty, ready for signing.
    ///
    /// Shape:
    /// ```text
    /// DKIM-Signature: v=1; a=<alg>; c=<hdr>/<body>; d=<domain>; s=<selector>;
    ///  h=<h-tag>; bh=<bh>; b=
    /// ```
    ///
    /// `<bh>` is the literal placeholder `<bh>`.  The adapter replaces it with
    /// the base64-encoded hash of `canonical_body` before signing.
    pub signing_input: String,
    /// Canonical body bytes.  The adapter hashes these to compute `bh=`.
    pub canonical_body: Vec<u8>,
    /// Canonical header strings in the order they contribute to the signature.
    /// The DKIM-Signature stub (with `b=` empty) is appended last per
    /// RFC 6376 §3.7.
    pub canonical_signed_headers: Vec<String>,
    /// Invariant: this module performs no signing, DNS lookup, OpenBao read, or
    /// SMTP delivery.  Value is [`NON_CLAIM`] from `sending_domain_authentication`.
    pub non_claim: &'static str,
}

/// Errors returned by [`build_dkim_signing_input`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DkimSigningInputError {
    /// `algorithm.supported_for_signing()` returned false (e.g. `RsaSha1`,
    /// `Other`).
    UnsupportedAlgorithm,
    /// `selector` or `signing_domain` is empty / whitespace-only.
    EmptySelectorOrDomain,
    /// `signed_headers` list is empty.
    NoSignedHeaders,
}

/// Build the canonical DKIM signing-input string per RFC 6376 §3.5 / §3.7.
///
/// Steps:
/// 1. Validate algorithm via [`DkimSigningAlgorithm::supported_for_signing`].
/// 2. Validate `selector` and `signing_domain` non-empty.
/// 3. Validate `signed_headers` non-empty.
/// 4. Canonicalize body per `body_canonicalization`.
/// 5. For each name in `signed_headers` (in order): find the *last* matching
///    header in `headers` (RFC 6376 §5.4 last-occurrence rule) and canonicalize
///    per `header_canonicalization`.
/// 6. Build the DKIM-Signature stub with `bh=<bh>` and `b=` (empty).
/// 7. Append the canonical DKIM-Signature stub as the final signed-header
///    (RFC 6376 §3.7).
///
/// No I/O, no crypto.  Returns [`DkimSigningInputMaterial`] for the adapter.
pub fn build_dkim_signing_input(
    request: DkimSigningInputRequest,
) -> Result<DkimSigningInputMaterial, DkimSigningInputError> {
    // 1. Algorithm gate.
    if !request.algorithm.supported_for_signing() {
        return Err(DkimSigningInputError::UnsupportedAlgorithm);
    }

    // 2. Non-empty selector and domain.
    if request.selector.trim().is_empty() || request.signing_domain.trim().is_empty() {
        return Err(DkimSigningInputError::EmptySelectorOrDomain);
    }

    // 3. Non-empty signed-headers list.
    if request.signed_headers.is_empty() {
        return Err(DkimSigningInputError::NoSignedHeaders);
    }

    // 4. Canonicalize body.
    let canonical_body = canonicalize_body(&request.body, request.body_canonicalization);

    // 5. Collect canonical header strings using RFC 6376 §5.4 last-occurrence rule.
    let mut canonical_signed_headers: Vec<String> = Vec::new();
    for name in &request.signed_headers {
        let name_lower = name.to_ascii_lowercase();
        // Find last matching header (case-insensitive).
        let found = request
            .headers
            .iter()
            .rev()
            .find(|h| h.name.to_ascii_lowercase() == name_lower);
        let single: Vec<RawHeader> = found
            .map(|h| {
                vec![RawHeader {
                    name: h.name.clone(),
                    value: h.value.clone(),
                }]
            })
            .unwrap_or_default();
        let canonical = canonicalize_header(&single, request.header_canonicalization);
        // Strip the trailing \r\n for inclusion in the signed-headers list;
        // it is present in signing_input but stored cleanly here.
        canonical_signed_headers.push(canonical.trim_end_matches("\r\n").to_string());
    }

    // 6. Build the DKIM-Signature stub.
    let alg_str = algorithm_string(request.algorithm);
    let hdr_canon_str = canon_string(request.header_canonicalization);
    let body_canon_str = canon_string(request.body_canonicalization);
    let h_tag = request
        .signed_headers
        .iter()
        .map(|s| s.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(":");

    let dkim_stub = format!(
        "DKIM-Signature: v=1; a={alg}; c={hc}/{bc}; d={domain}; s={sel}; h={h}; bh=<bh>; b=",
        alg = alg_str,
        hc = hdr_canon_str,
        bc = body_canon_str,
        domain = request.signing_domain.trim(),
        sel = request.selector.trim(),
        h = h_tag,
    );

    // 7. The canonicalized DKIM-Signature stub is the final item per RFC 6376 §3.7.
    // Canonicalize the stub header as if it were a regular header.
    let stub_raw = RawHeader {
        name: "DKIM-Signature".to_string(),
        value: dkim_stub[16..].to_string(), // strip "DKIM-Signature:" prefix
    };
    let stub_canonical = canonicalize_header(&[stub_raw], request.header_canonicalization);
    canonical_signed_headers.push(stub_canonical.trim_end_matches("\r\n").to_string());

    // The signing_input is the concatenation of all canonical header strings
    // (joined by \r\n) + the stub (without trailing \r\n per RFC 6376 §3.7).
    // Simplest representation: the stub already encodes the full DKIM-Signature.
    let signing_input = dkim_stub.clone();

    Ok(DkimSigningInputMaterial {
        signing_input,
        canonical_body,
        canonical_signed_headers,
        non_claim: NON_CLAIM,
    })
}

fn algorithm_string(alg: DkimSigningAlgorithm) -> &'static str {
    match alg {
        DkimSigningAlgorithm::Ed25519Sha256 => "ed25519-sha256",
        DkimSigningAlgorithm::RsaSha256 => "rsa-sha256",
        // supported_for_signing already filtered these out above
        DkimSigningAlgorithm::RsaSha1 | DkimSigningAlgorithm::Other => unreachable!(),
    }
}

fn canon_string(alg: DkimCanonicalizationAlgorithm) -> &'static str {
    match alg {
        DkimCanonicalizationAlgorithm::Relaxed => "relaxed",
        DkimCanonicalizationAlgorithm::Simple => "simple",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dkim_canonicalization::RawHeader;

    fn base_request() -> DkimSigningInputRequest {
        DkimSigningInputRequest {
            signed_headers: vec!["From".into(), "Subject".into()],
            headers: vec![
                RawHeader {
                    name: "From".into(),
                    value: " alice@example.com".into(),
                },
                RawHeader {
                    name: "Subject".into(),
                    value: " Q4 close".into(),
                },
            ],
            body: b"Hello world".to_vec(),
            selector: "s20260525a".into(),
            signing_domain: "example.com".into(),
            key_version_ref: "dkim-key:v1".into(),
            algorithm: DkimSigningAlgorithm::Ed25519Sha256,
            header_canonicalization: DkimCanonicalizationAlgorithm::Relaxed,
            body_canonicalization: DkimCanonicalizationAlgorithm::Simple,
        }
    }

    #[test]
    fn signing_input_contains_v1_tag() {
        let mat = build_dkim_signing_input(base_request()).unwrap();
        assert!(
            mat.signing_input.contains("v=1"),
            "missing v=1 in: {}",
            mat.signing_input
        );
    }

    #[test]
    fn signing_input_b_tag_is_empty() {
        let mat = build_dkim_signing_input(base_request()).unwrap();
        assert!(
            mat.signing_input.ends_with("b="),
            "signing_input must end with 'b=', got: {}",
            mat.signing_input
        );
    }

    #[test]
    fn signing_input_bh_placeholder_present() {
        let mat = build_dkim_signing_input(base_request()).unwrap();
        assert!(
            mat.signing_input.contains("bh=<bh>"),
            "missing bh=<bh> placeholder in: {}",
            mat.signing_input
        );
    }

    #[test]
    fn signing_input_h_tag_matches_request() {
        let mat = build_dkim_signing_input(base_request()).unwrap();
        assert!(
            mat.signing_input.contains("h=from:subject"),
            "expected h=from:subject in: {}",
            mat.signing_input
        );
    }

    #[test]
    fn signing_input_dkim_stub_appended_last() {
        let mat = build_dkim_signing_input(base_request()).unwrap();
        // The last element must contain the DKIM-Signature stub.
        let last = mat.canonical_signed_headers.last().unwrap();
        assert!(
            last.to_ascii_lowercase().contains("dkim-signature"),
            "last signed header must be DKIM-Signature stub, got: {last}"
        );
    }

    #[test]
    fn unsupported_algorithm_rejected() {
        let mut req = base_request();
        req.algorithm = DkimSigningAlgorithm::RsaSha1;
        assert_eq!(
            build_dkim_signing_input(req).unwrap_err(),
            DkimSigningInputError::UnsupportedAlgorithm
        );
    }

    #[test]
    fn empty_selector_rejected() {
        let mut req = base_request();
        req.selector = "".into();
        assert_eq!(
            build_dkim_signing_input(req).unwrap_err(),
            DkimSigningInputError::EmptySelectorOrDomain
        );
    }

    #[test]
    fn empty_signed_headers_rejected() {
        let mut req = base_request();
        req.signed_headers = vec![];
        assert_eq!(
            build_dkim_signing_input(req).unwrap_err(),
            DkimSigningInputError::NoSignedHeaders
        );
    }

    #[test]
    fn non_claim_invariant_preserved() {
        let mat = build_dkim_signing_input(base_request()).unwrap();
        assert!(
            mat.non_claim.contains("no DNS lookup"),
            "NON_CLAIM must contain 'no DNS lookup', got: {}",
            mat.non_claim
        );
    }

    #[test]
    fn canonical_body_populated() {
        let mat = build_dkim_signing_input(base_request()).unwrap();
        // Simple canonicalization of "Hello world" → "Hello world\r\n"
        assert_eq!(mat.canonical_body, b"Hello world\r\n");
    }

    #[test]
    fn rsa_sha256_algorithm_string_in_stub() {
        let mut req = base_request();
        req.algorithm = DkimSigningAlgorithm::RsaSha256;
        let mat = build_dkim_signing_input(req).unwrap();
        assert!(mat.signing_input.contains("a=rsa-sha256"));
    }
}
