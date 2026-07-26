//! Self-signed certificate bootstrap for the maintenance API server.
//!
//! In maintenance mode there is no cluster PKI yet: the node has never seen a
//! machine config, so it cannot have been issued a node certificate signed by
//! the cluster CA. Talos therefore generates a *self-signed* server certificate
//! on the fly (see `internal/app/maintenance` / `pkg/grpc/gen`) and serves the
//! maintenance gRPC + HTTP endpoints with TLS but **without** client-cert
//! verification — anyone who can reach the node may push a config.
//!
//! Real Talos uses ECDSA/Ed25519 key material and x509 ASN.1 DER. Here the
//! cryptographic boundary is modeled as the [`CertBootstrap`] trait; the
//! in-memory implementation produces deterministic placeholder material so the
//! surrounding logic (SAN handling, validity windows, regeneration) is testable
//! offline.

use std::collections::BTreeSet;
use std::fmt;

use os_kernel::Clock;

/// The set of Subject Alternative Names a maintenance certificate is valid for.
///
/// The maintenance cert must cover every address an operator might dial the
/// node on: its hostname, `localhost`, and each of its configured IP addresses
/// (plus the loopback addresses). Talos adds these so `talosctl --nodes <ip>`
/// works against a freshly booted node.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubjectAltNames {
    dns_names: BTreeSet<String>,
    ip_addresses: BTreeSet<String>,
}

impl SubjectAltNames {
    /// An empty SAN set.
    pub fn new() -> Self {
        Self::default()
    }

    /// A SAN set seeded with the loopback names every maintenance cert carries.
    pub fn with_loopback() -> Self {
        let mut san = Self::new();
        san.add_dns("localhost");
        san.add_ip("127.0.0.1");
        san.add_ip("::1");
        san
    }

    /// Add a DNS name (case-insensitive, deduplicated). Empty names are ignored.
    pub fn add_dns(&mut self, name: impl Into<String>) -> &mut Self {
        let name = name.into();
        if !name.is_empty() {
            self.dns_names.insert(name.to_ascii_lowercase());
        }
        self
    }

    /// Add an IP address (deduplicated). Empty values are ignored.
    pub fn add_ip(&mut self, ip: impl Into<String>) -> &mut Self {
        let ip = ip.into();
        if !ip.is_empty() {
            self.ip_addresses.insert(ip);
        }
        self
    }

    /// The sorted DNS names.
    pub fn dns_names(&self) -> impl Iterator<Item = &str> {
        self.dns_names.iter().map(String::as_str)
    }

    /// The sorted IP addresses.
    pub fn ip_addresses(&self) -> impl Iterator<Item = &str> {
        self.ip_addresses.iter().map(String::as_str)
    }

    /// Whether the certificate would be valid for the given host (DNS or IP).
    pub fn covers(&self, host: &str) -> bool {
        self.dns_names.contains(&host.to_ascii_lowercase()) || self.ip_addresses.contains(host)
    }

    /// Total number of SAN entries.
    pub fn len(&self) -> usize {
        self.dns_names.len() + self.ip_addresses.len()
    }

    /// Whether there are no SAN entries at all.
    pub fn is_empty(&self) -> bool {
        self.dns_names.is_empty() && self.ip_addresses.is_empty()
    }
}

/// A generated server certificate + private key (opaque PEM-ish bytes here).
#[derive(Clone, PartialEq, Eq)]
pub struct Certificate {
    /// Whether this certificate is self-signed (issuer == subject).
    pub self_signed: bool,
    /// The SANs the certificate is valid for.
    pub sans: SubjectAltNames,
    /// `notBefore`, as a unix timestamp (seconds).
    pub not_before: u64,
    /// `notAfter`, as a unix timestamp (seconds).
    pub not_after: u64,
    /// Opaque certificate bytes (DER/PEM in the real world).
    pub cert_pem: Vec<u8>,
    /// Opaque private-key bytes.
    pub key_pem: Vec<u8>,
}

impl fmt::Debug for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never print key material.
        f.debug_struct("Certificate")
            .field("self_signed", &self.self_signed)
            .field("sans", &self.sans)
            .field("not_before", &self.not_before)
            .field("not_after", &self.not_after)
            .field("cert_pem_len", &self.cert_pem.len())
            .finish()
    }
}

impl Certificate {
    /// Whether the certificate is valid (not expired / not yet valid) at `now`.
    pub fn is_valid_at(&self, now: u64) -> bool {
        now >= self.not_before && now < self.not_after
    }

    /// Whether the certificate has expired at `now`.
    pub fn is_expired_at(&self, now: u64) -> bool {
        now >= self.not_after
    }

    /// Remaining lifetime in seconds at `now` (0 once expired).
    pub fn remaining_secs(&self, now: u64) -> u64 {
        self.not_after.saturating_sub(now)
    }
}

/// The TLS configuration the maintenance server runs with.
///
/// Maintenance mode serves TLS but does **not** require (or verify) client
/// certificates, because no CA exists to validate them against yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsConfig {
    /// The active server certificate.
    pub certificate: Certificate,
    /// Whether client certificates are required. Always `false` in maintenance.
    pub require_client_cert: bool,
}

impl TlsConfig {
    /// Build a maintenance TLS config (never requires client certs).
    pub fn maintenance(certificate: Certificate) -> Self {
        TlsConfig {
            certificate,
            require_client_cert: false,
        }
    }
}

/// The certificate-generation boundary.
///
/// Real implementations call into the crypto stack (ECDSA keygen, x509
/// self-sign). The maintenance flow only needs "give me a self-signed cert for
/// these SANs, valid for this long".
pub trait CertBootstrap {
    /// Generate a self-signed server certificate valid for `sans`, with the
    /// given validity duration in seconds starting at `now`.
    fn generate_self_signed(
        &self,
        sans: SubjectAltNames,
        now: u64,
        validity_secs: u64,
    ) -> Certificate;
}

/// The default validity window for a maintenance certificate (one year),
/// matching Talos's self-signed maintenance cert lifetime.
pub const DEFAULT_VALIDITY_SECS: u64 = 365 * 24 * 60 * 60;

/// Deterministic in-memory [`CertBootstrap`] used by tests and offline builds.
#[derive(Debug, Default, Clone)]
pub struct InMemoryCertBootstrap {
    /// Monotonic counter so successive generations produce distinct material.
    serial: std::cell::Cell<u64>,
}

impl InMemoryCertBootstrap {
    /// A fresh bootstrap with serial counter at zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience: generate using a [`Clock`]'s current time.
    pub fn generate_with_clock<C: Clock>(
        &self,
        clock: &C,
        sans: SubjectAltNames,
        validity_secs: u64,
    ) -> Certificate {
        self.generate_self_signed(sans, clock.now_unix_secs(), validity_secs)
    }
}

impl CertBootstrap for InMemoryCertBootstrap {
    fn generate_self_signed(
        &self,
        sans: SubjectAltNames,
        now: u64,
        validity_secs: u64,
    ) -> Certificate {
        let serial = self.serial.get().wrapping_add(1);
        self.serial.set(serial);

        // Deterministic placeholder material that varies by serial + SAN count
        // so distinct certs compare unequal.
        let tag = format!("maintenance-self-signed#{serial}");
        let mut cert_pem = b"-----BEGIN CERTIFICATE-----\n".to_vec();
        cert_pem.extend_from_slice(tag.as_bytes());
        cert_pem.push(b'\n');
        cert_pem.extend_from_slice(format!("sans={}", sans.len()).as_bytes());
        cert_pem.extend_from_slice(b"\n-----END CERTIFICATE-----\n");

        let mut key_pem = b"-----BEGIN PRIVATE KEY-----\n".to_vec();
        key_pem.extend_from_slice(format!("key#{serial}").as_bytes());
        key_pem.extend_from_slice(b"\n-----END PRIVATE KEY-----\n");

        Certificate {
            self_signed: true,
            sans,
            not_before: now,
            not_after: now.saturating_add(validity_secs),
            cert_pem,
            key_pem,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::ManualClock;

    #[test]
    fn loopback_sans_cover_localhost_and_loopback_ips() {
        let san = SubjectAltNames::with_loopback();
        assert!(san.covers("localhost"));
        assert!(san.covers("LOCALHOST")); // case-insensitive DNS
        assert!(san.covers("127.0.0.1"));
        assert!(san.covers("::1"));
        assert!(!san.covers("10.0.0.5"));
        assert_eq!(san.len(), 3);
    }

    #[test]
    fn sans_dedup_and_ignore_empty() {
        let mut san = SubjectAltNames::new();
        san.add_dns("node-1").add_dns("NODE-1").add_dns("");
        san.add_ip("10.0.0.5").add_ip("10.0.0.5").add_ip("");
        assert_eq!(san.len(), 2);
        assert!(san.covers("node-1"));
        assert!(san.covers("10.0.0.5"));
    }

    #[test]
    fn generated_cert_is_self_signed_and_valid_in_window() {
        let boot = InMemoryCertBootstrap::new();
        let mut san = SubjectAltNames::with_loopback();
        san.add_ip("10.0.0.5");
        let cert = boot.generate_self_signed(san, 1_000, DEFAULT_VALIDITY_SECS);

        assert!(cert.self_signed);
        assert!(cert.is_valid_at(1_000));
        assert!(cert.is_valid_at(1_000 + 10));
        assert!(!cert.is_valid_at(999)); // before notBefore
        assert!(cert.is_expired_at(1_000 + DEFAULT_VALIDITY_SECS));
        assert!(cert.sans.covers("10.0.0.5"));
    }

    #[test]
    fn successive_generations_differ() {
        let boot = InMemoryCertBootstrap::new();
        let a = boot.generate_self_signed(SubjectAltNames::with_loopback(), 0, 100);
        let b = boot.generate_self_signed(SubjectAltNames::with_loopback(), 0, 100);
        assert_ne!(a.cert_pem, b.cert_pem);
        assert_ne!(a, b);
    }

    #[test]
    fn remaining_lifetime_tracks_clock() {
        let boot = InMemoryCertBootstrap::new();
        let cert = boot.generate_self_signed(SubjectAltNames::new(), 100, 1_000);
        assert_eq!(cert.remaining_secs(100), 1_000);
        assert_eq!(cert.remaining_secs(600), 500);
        assert_eq!(cert.remaining_secs(2_000), 0);
    }

    #[test]
    fn generate_with_clock_uses_clock_time() {
        let boot = InMemoryCertBootstrap::new();
        let clock = ManualClock::new(5_000 * 1_000_000_000);
        let cert = boot.generate_with_clock(&clock, SubjectAltNames::with_loopback(), 100);
        assert_eq!(cert.not_before, 5_000);
        assert_eq!(cert.not_after, 5_100);
    }

    #[test]
    fn maintenance_tls_never_requires_client_certs() {
        let boot = InMemoryCertBootstrap::new();
        let cert = boot.generate_self_signed(SubjectAltNames::with_loopback(), 0, 100);
        let tls = TlsConfig::maintenance(cert);
        assert!(!tls.require_client_cert);
    }

    #[test]
    fn debug_does_not_leak_key_material() {
        let boot = InMemoryCertBootstrap::new();
        let cert = boot.generate_self_signed(SubjectAltNames::new(), 0, 100);
        let dbg = format!("{cert:?}");
        assert!(!dbg.contains("PRIVATE KEY"));
        assert!(dbg.contains("cert_pem_len"));
    }
}
