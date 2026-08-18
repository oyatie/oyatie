//! Pure SPF identifier-alignment evaluation per RFC 7208 §2.6 / RFC 7489 §3.1.
//!
//! No DNS lookup, no network I/O, no cryptographic operations.
//! Alignment is purely a string comparison over already-resolved domain names.

use crate::governance::organizational_domain;

/// RFC 7208 §2.6 / RFC 7489 §3.1 alignment mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpfAlignmentMode {
    /// Only an exact domain match (case-insensitive, trailing-dot normalised)
    /// counts as aligned.
    Strict,
    /// Organizational-domain (registrable domain) match is sufficient.  A
    /// subdomain of the same registered domain is aligned under relaxed mode.
    Relaxed,
}

/// Result of SPF identifier alignment evaluation.
///
/// Consistent with the `SendingDomainAuthReason` vocabulary used by
/// `evaluate_sending_domain_authentication`: a verdict that is not `Aligned`
/// maps to `SenderDomainMismatch` at the admission layer; this type carries
/// finer-grained information for routing / audit decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpfAlignmentVerdict {
    /// Exact domain match (valid for both strict and relaxed modes).
    Aligned,
    /// Organizational-domain match under relaxed mode only (the envelope-from
    /// domain is a subdomain sharing the same registered domain as
    /// header-from).
    RelaxedAligned,
    /// No alignment.  Always returned under strict mode for non-exact matches.
    Misaligned,
}

/// Evaluate SPF identifier alignment between the SMTP envelope-from domain and
/// the RFC5322.From (header-from) domain.
///
/// # Arguments
/// * `envelope_from_domain` — domain from SMTP `MAIL FROM` (envelope-from).
/// * `header_from_domain`   — RFC5322.From domain extracted from the message.
/// * `mode`                 — alignment strictness.
///
/// # Returns
/// [`SpfAlignmentVerdict`] — never performs DNS lookup or network I/O.
pub fn evaluate_spf_alignment(
    envelope_from_domain: &str,
    header_from_domain: &str,
    mode: SpfAlignmentMode,
) -> SpfAlignmentVerdict {
    let envelope_norm = normalize(envelope_from_domain);
    let header_norm = normalize(header_from_domain);

    // RFC 7208 §2.6: a null sender (empty envelope-from) must never be
    // considered aligned with any domain, including another null sender.
    // Likewise a missing header-from domain must not produce a false positive.
    if envelope_norm.is_empty() || header_norm.is_empty() {
        return SpfAlignmentVerdict::Misaligned;
    }

    if envelope_norm == header_norm {
        return SpfAlignmentVerdict::Aligned;
    }

    if mode == SpfAlignmentMode::Relaxed {
        let envelope_org = normalize(organizational_domain(&envelope_norm));
        let header_org = normalize(organizational_domain(&header_norm));
        if envelope_org == header_org {
            return SpfAlignmentVerdict::RelaxedAligned;
        }
    }

    SpfAlignmentVerdict::Misaligned
}

/// Normalise a domain for comparison: ASCII-lowercase + strip trailing dot.
fn normalize(domain: &str) -> String {
    domain.trim().trim_end_matches('.').to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_strict_aligned() {
        assert_eq!(
            evaluate_spf_alignment("example.com", "example.com", SpfAlignmentMode::Strict),
            SpfAlignmentVerdict::Aligned
        );
    }

    #[test]
    fn exact_match_relaxed_aligned() {
        assert_eq!(
            evaluate_spf_alignment("example.com", "example.com", SpfAlignmentMode::Relaxed),
            SpfAlignmentVerdict::Aligned
        );
    }

    #[test]
    fn subdomain_relaxed_relaxed_aligned() {
        assert_eq!(
            evaluate_spf_alignment("mail.example.com", "example.com", SpfAlignmentMode::Relaxed),
            SpfAlignmentVerdict::RelaxedAligned
        );
    }

    #[test]
    fn subdomain_strict_misaligned() {
        assert_eq!(
            evaluate_spf_alignment("mail.example.com", "example.com", SpfAlignmentMode::Strict),
            SpfAlignmentVerdict::Misaligned
        );
    }

    #[test]
    fn unrelated_domain_misaligned() {
        assert_eq!(
            evaluate_spf_alignment("unrelated.net", "example.com", SpfAlignmentMode::Relaxed),
            SpfAlignmentVerdict::Misaligned
        );
    }

    #[test]
    fn case_insensitive_normalization() {
        assert_eq!(
            evaluate_spf_alignment("EXAMPLE.COM", "example.com", SpfAlignmentMode::Strict),
            SpfAlignmentVerdict::Aligned
        );
    }

    #[test]
    fn trailing_dot_normalization() {
        assert_eq!(
            evaluate_spf_alignment("example.com.", "example.com", SpfAlignmentMode::Strict),
            SpfAlignmentVerdict::Aligned
        );
    }
}
