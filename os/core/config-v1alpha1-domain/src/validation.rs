//! Validation surface for the v1alpha1 config schema.
//!
//! This module re-exports the shared validation primitives from
//! `talos-machine-config` (the [`Validator`] trait, [`ValidationMode`],
//! [`ValidationReport`], and the [`ValidationError`] taxonomy) so that every
//! type in this crate validates through the same machinery as the rest of the
//! config subsystem. It then adds a handful of v1alpha1-specific helper
//! predicates used by the field validators in the other modules (CIDR, IP,
//! hostname, image reference, and key/value identifier checks), mirroring the
//! validation helpers Talos keeps in
//! `pkg/machinery/config/types/v1alpha1/v1alpha1_validation.go`.

pub use os_machine_config_domain::validation::{
    ValidationError, ValidationMode, ValidationReport, Validator,
};

/// Whether `s` parses as a bare IPv4 dotted-quad literal.
///
/// This deliberately does not accept IPv6 or CIDR; callers that need those use
/// [`is_ip`] / [`is_cidr`].
pub fn is_ipv4(s: &str) -> bool {
    let mut count = 0;
    for octet in s.split('.') {
        count += 1;
        if octet.is_empty() || octet.len() > 3 {
            return false;
        }
        if !octet.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        match octet.parse::<u16>() {
            Ok(n) if n <= 255 => {}
            _ => return false,
        }
        // Reject non-canonical leading zeros like `01`.
        if octet.len() > 1 && octet.starts_with('0') {
            return false;
        }
    }
    count == 4
}

/// Whether `s` looks like an IPv6 literal (loosely: contains `:` and only hex /
/// colon characters, with at most one `::`).
pub fn is_ipv6(s: &str) -> bool {
    if !s.contains(':') {
        return false;
    }
    if s.matches("::").count() > 1 {
        return false;
    }
    s.bytes().all(|b| b.is_ascii_hexdigit() || b == b':')
}

/// Whether `s` is an IP literal (v4 or v6).
pub fn is_ip(s: &str) -> bool {
    is_ipv4(s) || is_ipv6(s)
}

/// Whether `s` is a CIDR (`<ip>/<prefix>`), with the prefix length in range for
/// the address family.
pub fn is_cidr(s: &str) -> bool {
    let Some((addr, prefix)) = s.split_once('/') else {
        return false;
    };
    let Ok(bits) = prefix.parse::<u8>() else {
        return false;
    };
    if is_ipv4(addr) {
        bits <= 32
    } else if is_ipv6(addr) {
        bits <= 128
    } else {
        false
    }
}

/// Whether `s` is a syntactically valid DNS hostname / label sequence (RFC 1123
/// subdomain): non-empty, <= 253 chars, dot-separated labels of `[a-z0-9-]`
/// not starting/ending with a hyphen.
pub fn is_hostname(s: &str) -> bool {
    if s.is_empty() || s.len() > 253 {
        return false;
    }
    s.split('.').all(is_dns_label)
}

fn is_dns_label(label: &str) -> bool {
    if label.is_empty() || label.len() > 63 {
        return false;
    }
    if label.starts_with('-') || label.ends_with('-') {
        return false;
    }
    label
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// Whether `s` looks like a container image reference: `host[:port]/path:tag` or
/// `path@sha256:...`. We accept anything non-empty that has a registry/repo
/// shape and no whitespace; this mirrors the lightweight check Talos performs
/// before handing the reference to containerd.
pub fn is_image_ref(s: &str) -> bool {
    if s.is_empty() || s.contains(char::is_whitespace) {
        return false;
    }
    // Must contain a repository component (no leading `/` or `:`).
    !s.starts_with('/') && !s.starts_with(':')
}

/// Whether `s` is a valid Kubernetes-style label/identifier key: non-empty,
/// composed of alphanumerics plus `-_.`/`/`. Used to vet kubelet extra args,
/// sysctl keys, and registry hosts.
pub fn is_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'/' | b':'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv4_canonical_only() {
        assert!(is_ipv4("10.0.0.1"));
        assert!(is_ipv4("255.255.255.255"));
        assert!(!is_ipv4("256.0.0.1"));
        assert!(!is_ipv4("10.0.0"));
        assert!(!is_ipv4("01.0.0.1")); // leading zero
        assert!(!is_ipv4("10.0.0.1.2"));
        assert!(!is_ipv4("::1"));
    }

    #[test]
    fn ipv6_loose() {
        assert!(is_ipv6("::1"));
        assert!(is_ipv6("fd00::1"));
        assert!(!is_ipv6("10.0.0.1"));
        assert!(!is_ipv6("fd00::1::2")); // two ::
    }

    #[test]
    fn cidr_checks_family_and_prefix() {
        assert!(is_cidr("10.244.0.0/16"));
        assert!(is_cidr("fd00::/64"));
        assert!(!is_cidr("10.244.0.0/33"));
        assert!(!is_cidr("fd00::/129"));
        assert!(!is_cidr("10.244.0.0"));
        assert!(!is_cidr("notacidr/8"));
    }

    #[test]
    fn hostnames() {
        assert!(is_hostname("api.example.com"));
        assert!(is_hostname("localhost"));
        assert!(!is_hostname(""));
        assert!(!is_hostname("-bad.com"));
        assert!(!is_hostname("bad-.com"));
        assert!(!is_hostname("a..b"));
    }

    #[test]
    fn image_refs() {
        assert!(is_image_ref("ghcr.io/siderolabs/installer:v1.7.0"));
        assert!(is_image_ref("registry.k8s.io/pause:3.9"));
        assert!(!is_image_ref(""));
        assert!(!is_image_ref("has space"));
        assert!(!is_image_ref("/leadingslash"));
    }

    #[test]
    fn identifiers() {
        assert!(is_identifier("max-pods"));
        assert!(is_identifier("net.ipv4.ip_forward"));
        assert!(!is_identifier(""));
        assert!(!is_identifier("has space"));
    }
}
