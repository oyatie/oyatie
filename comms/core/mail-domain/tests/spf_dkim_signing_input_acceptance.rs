// Acceptance tests for the mail-spf-dkim-signing-input-domain slice.
//
// These tests cover edge cases and RFC-compliance requirements not yet exercised
// by the 71 in-crate unit tests.  They are written first (TDD red) and describe
// expected behavior at the crate's public API surface.
//
// RFC references:
//   RFC 6376 §3.4   — DKIM header/body canonicalization
//   RFC 6376 §3.5   — DKIM-Signature tag list
//   RFC 6376 §3.7   — Computing the message hash
//   RFC 6376 §5.4   — Determine the header fields to sign (last-occurrence rule)
//   RFC 7208 §2.6   — SPF identifier alignment
//   RFC 7489 §3.1   — DMARC identifier alignment

// ---------------------------------------------------------------------------
// ST1: SPF alignment edge cases
// ---------------------------------------------------------------------------

#[cfg(test)]
mod spf_alignment_edge_cases {
    use comms_mail_domain::{SpfAlignmentMode, SpfAlignmentVerdict, evaluate_spf_alignment};

    /// RFC 7208 §2.6: a null sender (empty envelope-from domain, as used in
    /// SMTP bounce messages "MAIL FROM:<>") must NOT be treated as aligned with
    /// any header-from domain.  Two empty strings normalizing to equal is a
    /// false positive.
    #[test]
    fn empty_envelope_from_domain_is_misaligned_strict() {
        assert_eq!(
            evaluate_spf_alignment("", "example.com", SpfAlignmentMode::Strict),
            SpfAlignmentVerdict::Misaligned,
            "empty envelope-from must not align with a real domain under strict mode"
        );
    }

    /// Same invariant under relaxed mode: an empty envelope-from must not
    /// match any real header-from domain.
    #[test]
    fn empty_envelope_from_domain_is_misaligned_relaxed() {
        assert_eq!(
            evaluate_spf_alignment("", "example.com", SpfAlignmentMode::Relaxed),
            SpfAlignmentVerdict::Misaligned,
            "empty envelope-from must not align with a real domain under relaxed mode"
        );
    }

    /// Two empty domains must not be considered aligned with each other — that
    /// would make all null-sender messages appear SPF-aligned against each other.
    #[test]
    fn both_empty_domains_are_misaligned() {
        assert_eq!(
            evaluate_spf_alignment("", "", SpfAlignmentMode::Strict),
            SpfAlignmentVerdict::Misaligned,
            "two empty domains must not be aligned"
        );
    }

    /// RFC 7208 §2.6 relaxed: two different subdomains of the same
    /// organizational domain must be RelaxedAligned.
    /// e.g. mail.example.com (envelope) vs smtp.example.com (header-from)
    /// share the org domain example.com → relaxed aligned.
    #[test]
    fn two_subdomains_of_same_org_are_relaxed_aligned() {
        assert_eq!(
            evaluate_spf_alignment(
                "mail.example.com",
                "smtp.example.com",
                SpfAlignmentMode::Relaxed,
            ),
            SpfAlignmentVerdict::RelaxedAligned,
            "two different subdomains sharing the same org domain must be RelaxedAligned"
        );
    }

    /// Under strict mode the same two subdomains must be Misaligned because
    /// strict requires exact match.
    #[test]
    fn two_subdomains_of_same_org_are_misaligned_under_strict() {
        assert_eq!(
            evaluate_spf_alignment(
                "mail.example.com",
                "smtp.example.com",
                SpfAlignmentMode::Strict,
            ),
            SpfAlignmentVerdict::Misaligned,
            "two different subdomains must be Misaligned under strict mode"
        );
    }

    /// Whitespace-padded domains must normalize correctly: leading/trailing
    /// whitespace must not cause a false mismatch or a false match.
    #[test]
    fn whitespace_padding_normalized_for_strict_alignment() {
        assert_eq!(
            evaluate_spf_alignment("  example.com  ", "example.com", SpfAlignmentMode::Strict),
            SpfAlignmentVerdict::Aligned,
            "leading/trailing whitespace must be stripped before comparison"
        );
    }

    /// A trailing dot (FQDN notation) on one side and none on the other must
    /// still align under strict mode.
    #[test]
    fn trailing_dot_on_header_from_normalizes_for_strict_alignment() {
        assert_eq!(
            evaluate_spf_alignment("example.com", "example.com.", SpfAlignmentMode::Strict),
            SpfAlignmentVerdict::Aligned,
            "trailing dot on header-from must be stripped before strict comparison"
        );
    }
}

// ---------------------------------------------------------------------------
// ST2: DKIM canonicalization RFC 6376 coverage
// ---------------------------------------------------------------------------

#[cfg(test)]
mod dkim_canonicalization_coverage {
    use comms_mail_domain::{
        DkimCanonicalizationAlgorithm, RawHeader, canonicalize_body, canonicalize_header,
    };

    fn rh(name: &str, value: &str) -> RawHeader {
        RawHeader {
            name: name.into(),
            value: value.into(),
        }
    }

    // --- RFC 6376 §3.4.1 relaxed header: multiple headers with same name ---

    /// Relaxed header canonicalization must produce one output line per input
    /// header, each ending with CRLF.  Two headers of the same name must both
    /// appear in the output, in input order.
    #[test]
    fn relaxed_two_received_headers_both_canonicalized() {
        let headers = vec![
            rh("Received", " from a.example.com"),
            rh("Received", " from b.example.com"),
        ];
        let result = canonicalize_header(&headers, DkimCanonicalizationAlgorithm::Relaxed);
        assert_eq!(
            result, "received:from a.example.com\r\nreceived:from b.example.com\r\n",
            "both Received headers must appear in relaxed canonical form"
        );
    }

    // --- RFC 6376 §3.4.2 simple header: multiple headers ---

    /// Simple canonicalization must preserve each header verbatim and terminate
    /// each with CRLF.  The bug candidate: the current impl checks
    /// `out.ends_with("\r\n")` on the whole accumulated output rather than on
    /// the current header's value, which means the second header will not get
    /// its own CRLF appended if the first one already added CRLF.
    #[test]
    fn simple_two_headers_each_ends_with_crlf() {
        let headers = vec![rh("From", " alice@example.com"), rh("Subject", " Q4 close")];
        let result = canonicalize_header(&headers, DkimCanonicalizationAlgorithm::Simple);
        // Each header must have its own CRLF terminator.
        assert_eq!(
            result, "From: alice@example.com\r\nSubject: Q4 close\r\n",
            "simple canonicalization must append CRLF to each header individually"
        );
    }

    /// Simple mode: a header value that already ends with CRLF must not have
    /// a second CRLF appended — but the next header in the list still needs
    /// its own CRLF.
    #[test]
    fn simple_header_value_already_crlf_terminated_does_not_double_crlf() {
        let headers = vec![rh("From", " alice@example.com\r\n"), rh("Subject", " test")];
        let result = canonicalize_header(&headers, DkimCanonicalizationAlgorithm::Simple);
        // "From: alice@example.com\r\n" already has CRLF; Subject must still get one.
        assert_eq!(
            result, "From: alice@example.com\r\nSubject: test\r\n",
            "pre-terminated header must not have double CRLF; subsequent header must still have its own CRLF"
        );
    }

    // --- RFC 6376 §3.4.3 relaxed body: whitespace-only line ---

    /// RFC 6376 §3.4.3: relaxed body strips trailing whitespace from each line.
    /// A line consisting entirely of spaces/tabs becomes an empty line (not
    /// deleted — only trailing empty lines at the end of the body are dropped).
    #[test]
    fn relaxed_body_whitespace_only_line_in_middle_becomes_empty_line() {
        // A body with a whitespace-only middle line: "foo\r\n   \r\nbar\r\n"
        let body = b"foo\r\n   \r\nbar\r\n";
        let result = canonicalize_body(body, DkimCanonicalizationAlgorithm::Relaxed);
        // Middle "   " stripped to "" → empty line preserved; trailing single CRLF.
        assert_eq!(
            result, b"foo\r\n\r\nbar\r\n" as &[u8],
            "whitespace-only middle line must become an empty line (not removed)"
        );
    }

    // --- RFC 6376 §3.4.3 relaxed body: bare-LF input robustness ---

    /// The implementation accepts bare LF (in addition to CRLF) for robustness.
    /// A body with bare LF separators must canonicalize identically to the
    /// CRLF variant.
    #[test]
    fn relaxed_body_bare_lf_input_equivalent_to_crlf() {
        let bare_lf = b"foo   \nbar\n" as &[u8];
        let crlf = b"foo   \r\nbar\r\n" as &[u8];
        let result_bare = canonicalize_body(bare_lf, DkimCanonicalizationAlgorithm::Relaxed);
        let result_crlf = canonicalize_body(crlf, DkimCanonicalizationAlgorithm::Relaxed);
        assert_eq!(
            result_bare, result_crlf,
            "bare-LF and CRLF input must produce identical relaxed canonical bodies"
        );
    }

    // --- RFC 6376 §3.4.4 simple body: bare-LF input ---

    /// Simple body canonicalization must also handle bare-LF input gracefully,
    /// outputting CRLF-terminated lines.
    #[test]
    fn simple_body_bare_lf_input_produces_crlf_output() {
        let result = canonicalize_body(b"hello\n", DkimCanonicalizationAlgorithm::Simple);
        assert_eq!(
            result, b"hello\r\n" as &[u8],
            "simple body with bare-LF must produce CRLF-terminated output"
        );
    }

    // --- Empty header list ---

    /// Canonicalizing an empty header list must produce an empty string (no
    /// spurious CRLF output) for both modes.
    #[test]
    fn empty_header_list_relaxed_produces_empty_string() {
        assert_eq!(
            canonicalize_header(&[], DkimCanonicalizationAlgorithm::Relaxed),
            "",
            "empty header list must produce empty string under relaxed"
        );
    }

    #[test]
    fn empty_header_list_simple_produces_empty_string() {
        assert_eq!(
            canonicalize_header(&[], DkimCanonicalizationAlgorithm::Simple),
            "",
            "empty header list must produce empty string under simple"
        );
    }
}

// ---------------------------------------------------------------------------
// ST3: DKIM signing-input builder acceptance
// ---------------------------------------------------------------------------

#[cfg(test)]
mod dkim_signing_input_builder_acceptance {
    use comms_mail_domain::dkim_canonicalization::RawHeader;
    use comms_mail_domain::sending_domain_authentication::DkimSigningAlgorithm;
    use comms_mail_domain::{
        DkimCanonicalizationAlgorithm, DkimSigningInputError, DkimSigningInputRequest,
        build_dkim_signing_input,
    };

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
            selector: "sel2026a".into(),
            signing_domain: "example.com".into(),
            key_version_ref: "dkim-key:v2".into(),
            algorithm: DkimSigningAlgorithm::Ed25519Sha256,
            header_canonicalization: DkimCanonicalizationAlgorithm::Relaxed,
            body_canonicalization: DkimCanonicalizationAlgorithm::Simple,
        }
    }

    // --- Validation: DkimSigningAlgorithm::Other must be rejected ---

    /// `DkimSigningAlgorithm::Other` must return `UnsupportedAlgorithm` because
    /// `supported_for_signing()` returns false for it — same as `RsaSha1`.
    #[test]
    fn algorithm_other_is_rejected_as_unsupported() {
        let mut req = base_request();
        req.algorithm = DkimSigningAlgorithm::Other;
        assert_eq!(
            build_dkim_signing_input(req).unwrap_err(),
            DkimSigningInputError::UnsupportedAlgorithm,
            "DkimSigningAlgorithm::Other must be rejected"
        );
    }

    /// Whitespace-only signing_domain must be rejected with EmptySelectorOrDomain.
    #[test]
    fn whitespace_only_signing_domain_is_rejected() {
        let mut req = base_request();
        req.signing_domain = "   ".into();
        assert_eq!(
            build_dkim_signing_input(req).unwrap_err(),
            DkimSigningInputError::EmptySelectorOrDomain,
            "whitespace-only signing_domain must be rejected"
        );
    }

    /// Whitespace-only selector must be rejected with EmptySelectorOrDomain.
    #[test]
    fn whitespace_only_selector_is_rejected() {
        let mut req = base_request();
        req.selector = "\t".into();
        assert_eq!(
            build_dkim_signing_input(req).unwrap_err(),
            DkimSigningInputError::EmptySelectorOrDomain,
            "whitespace-only selector must be rejected"
        );
    }

    // --- canonical_signed_headers length invariant ---

    /// Per RFC 6376 §3.7 the DKIM-Signature stub (with b= empty) must be
    /// appended as the final element of the canonical signed-headers list.
    /// Therefore: `canonical_signed_headers.len()` must equal
    /// `signed_headers.len() + 1`.
    #[test]
    fn canonical_signed_headers_count_includes_dkim_signature_stub() {
        let req = base_request();
        let signed_count = req.signed_headers.len();
        let mat = build_dkim_signing_input(req).unwrap();
        assert_eq!(
            mat.canonical_signed_headers.len(),
            signed_count + 1,
            "canonical_signed_headers must contain all signed headers plus the DKIM-Signature stub"
        );
    }

    // --- c= canonicalization tag reflects request ---

    /// The `c=` tag in the signing input must reflect the canonicalization
    /// algorithms requested: `c=relaxed/simple` when header=Relaxed, body=Simple.
    #[test]
    fn c_tag_reflects_relaxed_header_and_simple_body_canonicalization() {
        let mat = build_dkim_signing_input(base_request()).unwrap();
        assert!(
            mat.signing_input.contains("c=relaxed/simple"),
            "signing_input must contain c=relaxed/simple, got: {}",
            mat.signing_input
        );
    }

    /// `c=simple/simple` when both are Simple.
    #[test]
    fn c_tag_reflects_simple_simple_canonicalization() {
        let mut req = base_request();
        req.header_canonicalization = DkimCanonicalizationAlgorithm::Simple;
        req.body_canonicalization = DkimCanonicalizationAlgorithm::Simple;
        let mat = build_dkim_signing_input(req).unwrap();
        assert!(
            mat.signing_input.contains("c=simple/simple"),
            "signing_input must contain c=simple/simple, got: {}",
            mat.signing_input
        );
    }

    /// `c=relaxed/relaxed` when both are Relaxed.
    #[test]
    fn c_tag_reflects_relaxed_relaxed_canonicalization() {
        let mut req = base_request();
        req.header_canonicalization = DkimCanonicalizationAlgorithm::Relaxed;
        req.body_canonicalization = DkimCanonicalizationAlgorithm::Relaxed;
        let mat = build_dkim_signing_input(req).unwrap();
        assert!(
            mat.signing_input.contains("c=relaxed/relaxed"),
            "signing_input must contain c=relaxed/relaxed, got: {}",
            mat.signing_input
        );
    }

    // --- RFC 6376 §5.4 last-occurrence rule ---

    /// When a header name appears multiple times in the message, the signing
    /// input builder must use the LAST occurrence for inclusion in the
    /// canonical signed-headers string.
    #[test]
    fn last_occurrence_rule_selects_last_matching_header() {
        let req = DkimSigningInputRequest {
            signed_headers: vec!["Received".into()],
            headers: vec![
                RawHeader {
                    name: "Received".into(),
                    value: " from a.example.com".into(),
                },
                RawHeader {
                    name: "Received".into(),
                    value: " from b.example.com".into(),
                },
            ],
            body: b"body".to_vec(),
            selector: "sel".into(),
            signing_domain: "example.com".into(),
            key_version_ref: "ref".into(),
            algorithm: DkimSigningAlgorithm::Ed25519Sha256,
            header_canonicalization: DkimCanonicalizationAlgorithm::Relaxed,
            body_canonicalization: DkimCanonicalizationAlgorithm::Simple,
        };
        let mat = build_dkim_signing_input(req).unwrap();
        // The first signed-header entry (index 0) must be the LAST Received header.
        let received_canonical = &mat.canonical_signed_headers[0];
        assert!(
            received_canonical.contains("from b.example.com"),
            "last-occurrence rule: canonical signed-header must use the last Received header, got: {received_canonical}"
        );
        assert!(
            !received_canonical.contains("from a.example.com"),
            "last-occurrence rule: first Received header must NOT be selected, got: {received_canonical}"
        );
    }

    // --- Missing header graceful handling ---

    /// When a name in `signed_headers` has no corresponding header in the
    /// message `headers` list, the builder must still succeed (producing an
    /// empty canonical string for that slot per RFC 6376 §5.4), not panic or
    /// return an error.
    #[test]
    fn missing_header_in_message_does_not_cause_error() {
        let req = DkimSigningInputRequest {
            signed_headers: vec!["From".into(), "X-Nonexistent".into()],
            headers: vec![RawHeader {
                name: "From".into(),
                value: " alice@example.com".into(),
            }],
            body: b"body".to_vec(),
            selector: "sel".into(),
            signing_domain: "example.com".into(),
            key_version_ref: "ref".into(),
            algorithm: DkimSigningAlgorithm::Ed25519Sha256,
            header_canonicalization: DkimCanonicalizationAlgorithm::Relaxed,
            body_canonicalization: DkimCanonicalizationAlgorithm::Simple,
        };
        let result = build_dkim_signing_input(req);
        assert!(
            result.is_ok(),
            "missing header in message must not cause error, got: {result:?}"
        );
    }

    // --- NON_CLAIM invariant ---

    /// The `non_claim` field must assert that no signing, DNS lookup, or
    /// OpenBao read occurs.  The literal must contain "no DNS lookup" and
    /// "crypto signing" (or equivalent).
    #[test]
    fn non_claim_asserts_no_crypto_signing_performed() {
        let mat = build_dkim_signing_input(base_request()).unwrap();
        assert!(
            mat.non_claim.contains("no DNS lookup"),
            "non_claim must contain 'no DNS lookup', got: {}",
            mat.non_claim
        );
        assert!(
            mat.non_claim.contains("crypto signing")
                || mat.non_claim.contains("no signing")
                || mat.non_claim.contains("signing"),
            "non_claim must contain a crypto-signing disclaimer, got: {}",
            mat.non_claim
        );
    }

    // --- d= and s= tags present in signing input ---

    /// The signing input must contain the `d=` (signing domain) and `s=`
    /// (selector) tags as specified in RFC 6376 §3.5.
    #[test]
    fn signing_input_contains_d_and_s_tags() {
        let mat = build_dkim_signing_input(base_request()).unwrap();
        assert!(
            mat.signing_input.contains("d=example.com"),
            "signing_input must contain d=example.com, got: {}",
            mat.signing_input
        );
        assert!(
            mat.signing_input.contains("s=sel2026a"),
            "signing_input must contain s=sel2026a, got: {}",
            mat.signing_input
        );
    }

    // --- canonical_body non-empty for non-empty input ---

    /// For a non-empty body, `canonical_body` must be non-empty and must end
    /// with CRLF per RFC 6376 §3.4.3/§3.4.4.
    #[test]
    fn canonical_body_ends_with_crlf_for_nonempty_input() {
        let mat = build_dkim_signing_input(base_request()).unwrap();
        assert!(
            mat.canonical_body.ends_with(b"\r\n"),
            "canonical_body must end with CRLF, got: {:?}",
            mat.canonical_body
        );
    }

    // --- RsaSha256 accepted ---

    /// `DkimSigningAlgorithm::RsaSha256` must be accepted and produce
    /// `a=rsa-sha256` in the signing input.
    #[test]
    fn rsa_sha256_produces_correct_a_tag_and_is_not_rejected() {
        let mut req = base_request();
        req.algorithm = DkimSigningAlgorithm::RsaSha256;
        let mat = build_dkim_signing_input(req).unwrap();
        assert!(
            mat.signing_input.contains("a=rsa-sha256"),
            "rsa-sha256 algorithm must produce a=rsa-sha256, got: {}",
            mat.signing_input
        );
    }
}
