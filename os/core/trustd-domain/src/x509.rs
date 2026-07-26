//! Low-level X.509 / PEM primitives.
//!
//! Mirrors the `pkg/grpc/gen` and `pkg/machinery/x509` boundary of Talos. We do
//! not implement real ASN.1/DER cryptography here (that would require external
//! crates and syscalls); instead we model the *shape* of the data: PEM
//! envelopes, key material, validity windows, and distinguished names, with the
//! invariants Talos enforces. Cryptographic signing is delegated to a
//! [`crate::signer::SigningBackend`] trait so a real backend can be plugged in
//! while tests use an in-memory stub.

use crate::error::TrustError;

/// The kind of object carried inside a PEM envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PEMLabel {
    /// `-----BEGIN CERTIFICATE-----`
    Certificate,
    /// `-----BEGIN CERTIFICATE REQUEST-----`
    CertificateRequest,
    /// `-----BEGIN EC PRIVATE KEY-----` / generic private key.
    PrivateKey,
    /// `-----BEGIN PUBLIC KEY-----`
    PublicKey,
}

impl PEMLabel {
    /// The textual label used between the PEM dashes.
    pub fn as_str(self) -> &'static str {
        match self {
            PEMLabel::Certificate => "CERTIFICATE",
            PEMLabel::CertificateRequest => "CERTIFICATE REQUEST",
            PEMLabel::PrivateKey => "PRIVATE KEY",
            PEMLabel::PublicKey => "PUBLIC KEY",
        }
    }

    /// Parse a label from the text found inside the PEM dashes.
    pub fn parse(s: &str) -> Result<Self, TrustError> {
        match s.trim() {
            "CERTIFICATE" => Ok(PEMLabel::Certificate),
            "CERTIFICATE REQUEST" | "NEW CERTIFICATE REQUEST" => Ok(PEMLabel::CertificateRequest),
            "PRIVATE KEY" | "EC PRIVATE KEY" | "RSA PRIVATE KEY" => Ok(PEMLabel::PrivateKey),
            "PUBLIC KEY" => Ok(PEMLabel::PublicKey),
            other => Err(TrustError::pem(format!("unknown PEM label '{other}'"))),
        }
    }
}

/// A PEM-encoded blob: a label plus raw (already-decoded) bytes.
///
/// Talos passes certificates and keys around as PEM throughout its gRPC PKI
/// API; this type is the universal envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PEMEncoded {
    label: PEMLabel,
    der: Vec<u8>,
}

impl PEMEncoded {
    /// Construct from a label and raw DER bytes.
    pub fn new(label: PEMLabel, der: impl Into<Vec<u8>>) -> Self {
        PEMEncoded {
            label,
            der: der.into(),
        }
    }

    /// The PEM object kind.
    pub fn label(&self) -> PEMLabel {
        self.label
    }

    /// The underlying DER bytes.
    pub fn der(&self) -> &[u8] {
        &self.der
    }

    /// Whether this envelope carries a certificate.
    pub fn is_certificate(&self) -> bool {
        self.label == PEMLabel::Certificate
    }

    /// Render to a textual PEM document. The body is a deterministic hex
    /// encoding of the bytes (a stand-in for base64 DER) so round-tripping is
    /// exact and dependency-free.
    pub fn encode(&self) -> String {
        let mut out = String::new();
        out.push_str("-----BEGIN ");
        out.push_str(self.label.as_str());
        out.push_str("-----\n");
        out.push_str(&hex_encode(&self.der));
        out.push('\n');
        out.push_str("-----END ");
        out.push_str(self.label.as_str());
        out.push_str("-----\n");
        out
    }

    /// Parse a textual PEM document back into a [`PEMEncoded`].
    pub fn decode(text: &str) -> Result<Self, TrustError> {
        let mut label: Option<PEMLabel> = None;
        let mut body = String::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix("-----BEGIN ") {
                let name = rest
                    .strip_suffix("-----")
                    .ok_or_else(|| TrustError::pem("malformed BEGIN line"))?;
                label = Some(PEMLabel::parse(name)?);
            } else if !line.starts_with("-----END ") {
                body.push_str(line);
            }
        }
        let label = label.ok_or_else(|| TrustError::pem("no PEM block found"))?;
        let der = hex_decode(&body)?;
        Ok(PEMEncoded::new(label, der))
    }
}

/// A distinguished name as Talos uses it for CA and leaf certificates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DistinguishedName {
    /// Common name (typically the node hostname or CA name).
    pub common_name: String,
    /// Organizational units — Talos encodes RBAC roles here as `os:<role>`.
    pub organizations: Vec<String>,
}

impl DistinguishedName {
    /// Construct a DN with a common name and no organizations.
    pub fn common(cn: impl Into<String>) -> Self {
        DistinguishedName {
            common_name: cn.into(),
            organizations: Vec::new(),
        }
    }

    /// Add an organizational unit, returning self for chaining.
    pub fn with_org(mut self, org: impl Into<String>) -> Self {
        self.organizations.push(org.into());
        self
    }

    /// RFC-4514-ish string form, used for logging and equality in tests.
    pub fn to_rfc(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("CN={}", self.common_name));
        for o in &self.organizations {
            parts.push(format!("O={o}"));
        }
        parts.join(",")
    }

    /// Parse the RFC-4514-ish form produced by [`DistinguishedName::to_rfc`]
    /// back into a [`DistinguishedName`]. Only the `CN` and `O` attribute types
    /// are recognised (the only ones Talos populates); a missing or empty `CN`
    /// is an error.
    pub fn parse_rfc(s: &str) -> Result<Self, TrustError> {
        let mut common_name = None;
        let mut organizations = Vec::new();
        for raw in s.split(',') {
            let part = raw.trim();
            if part.is_empty() {
                continue;
            }
            let (key, value) = part
                .split_once('=')
                .ok_or_else(|| TrustError::invalid("DN component missing '='"))?;
            match key.trim() {
                "CN" => common_name = Some(value.trim().to_string()),
                "O" => organizations.push(value.trim().to_string()),
                other => {
                    return Err(TrustError::invalid(format!(
                        "unsupported DN attribute '{other}'"
                    )));
                }
            }
        }
        let common_name = common_name
            .filter(|cn| !cn.is_empty())
            .ok_or_else(|| TrustError::invalid("DN has no common name"))?;
        Ok(DistinguishedName {
            common_name,
            organizations,
        })
    }

    /// The RBAC role OUs (`os:<role>`) carried in the organizations.
    pub fn role_ous(&self) -> impl Iterator<Item = &str> {
        self.organizations
            .iter()
            .map(String::as_str)
            .filter(|s| s.starts_with("os:"))
    }
}

/// Subject Alternative Names attached to a leaf certificate.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubjectAltNames {
    /// DNS names.
    pub dns_names: Vec<String>,
    /// IP addresses, stored as their string representation.
    pub ip_addresses: Vec<String>,
    /// URI SANs. This is the SPIFFE-identity carrier: a SPIFFE SVID encodes its
    /// identity as a single `spiffe://…` URI in the `uniformResourceIdentifier`
    /// general-name slot (SPIFFE X.509-SVID §2). Talos node certs use only
    /// DNS/IP SANs; workload SVIDs additionally carry exactly one URI SAN.
    pub uris: Vec<String>,
}

impl SubjectAltNames {
    /// True when there are no SANs at all.
    pub fn is_empty(&self) -> bool {
        self.dns_names.is_empty() && self.ip_addresses.is_empty() && self.uris.is_empty()
    }

    /// Whether the given URI literal appears in the URI SANs (exact match — a
    /// SPIFFE id is compared byte-for-byte, never normalised).
    pub fn covers_uri(&self, uri: &str) -> bool {
        self.uris.iter().any(|entry| entry == uri)
    }

    /// Add a URI SAN after validating it parses as an absolute
    /// `scheme://authority[/path]` URI with no whitespace/control characters.
    /// A SPIFFE id (`spiffe://…`) satisfies this shape.
    pub fn push_uri(&mut self, uri: impl AsRef<str>) -> Result<(), TrustError> {
        let uri = uri.as_ref();
        if !is_valid_uri(uri) {
            return Err(TrustError::invalid(format!("invalid URI SAN '{uri}'")));
        }
        self.uris.push(uri.to_string());
        Ok(())
    }

    /// Whether the given DNS name is covered (exact match only; Talos node
    /// certs do not use wildcards for the SAN check here).
    pub fn covers_dns(&self, name: &str) -> bool {
        self.dns_names.iter().any(|n| n == name)
    }

    /// Whether `name` is covered, honouring a single leftmost `*` wildcard label
    /// (e.g. `*.cluster.local` matches `node.cluster.local` but not
    /// `a.b.cluster.local` nor the bare `cluster.local`), per RFC 6125.
    pub fn covers_dns_wildcard(&self, name: &str) -> bool {
        self.dns_names
            .iter()
            .any(|pattern| dns_matches(pattern, name))
    }

    /// Whether the given IP literal appears in the IP SANs (after
    /// normalisation, so `192.168.000.1` matches `192.168.0.1`).
    pub fn covers_ip(&self, ip: &str) -> bool {
        match normalize_ip(ip) {
            Some(target) => self
                .ip_addresses
                .iter()
                .any(|entry| normalize_ip(entry).as_deref() == Some(target.as_str())),
            None => false,
        }
    }

    /// Add an IP SAN after validating it parses as an IPv4 dotted quad.
    pub fn push_ip(&mut self, ip: impl AsRef<str>) -> Result<(), TrustError> {
        let ip = ip.as_ref();
        let norm = normalize_ip(ip)
            .ok_or_else(|| TrustError::invalid(format!("invalid IPv4 address '{ip}'")))?;
        self.ip_addresses.push(norm);
        Ok(())
    }

    /// Validate every DNS, IP, and URI SAN; returns an error on the first
    /// malformed entry. Talos rejects a CSR carrying a syntactically invalid
    /// SAN.
    pub fn validate(&self) -> Result<(), TrustError> {
        for d in &self.dns_names {
            if !is_valid_dns_name(d) {
                return Err(TrustError::invalid(format!("invalid DNS SAN '{d}'")));
            }
        }
        for ip in &self.ip_addresses {
            if normalize_ip(ip).is_none() {
                return Err(TrustError::invalid(format!("invalid IP SAN '{ip}'")));
            }
        }
        for uri in &self.uris {
            if !is_valid_uri(uri) {
                return Err(TrustError::invalid(format!("invalid URI SAN '{uri}'")));
            }
        }
        Ok(())
    }
}

/// Whether a string is a syntactically valid absolute URI SAN: a non-empty
/// `scheme://authority[/path]` with a non-empty authority and no whitespace or
/// control characters. Conservative on purpose — the only URI SANs this model
/// issues are SPIFFE ids, and a tighter SPIFFE shape is enforced by the
/// SVID-kernel `SpiffeId` parser, not here.
pub fn is_valid_uri(uri: &str) -> bool {
    if uri.is_empty() || uri.len() > 2048 {
        return false;
    }
    if uri.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return false;
    }
    let Some((scheme, rest)) = uri.split_once("://") else {
        return false;
    };
    if scheme.is_empty()
        || !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
        || !scheme
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
    {
        return false;
    }
    // Authority is everything up to the first '/' (path), '?' (query) or '#'.
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    !rest[..authority_end].is_empty()
}

/// Whether a DNS name is syntactically valid (optionally with a leading `*`
/// wildcard label), using a conservative LDH (letter/digit/hyphen) check.
pub fn is_valid_dns_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 253 {
        return false;
    }
    let mut labels = name.split('.').peekable();
    let mut first = true;
    while let Some(label) = labels.next() {
        let is_last = labels.peek().is_none();
        if label.is_empty() {
            // trailing dot allowed only as the very last (root) label
            if is_last && !first {
                return true;
            }
            return false;
        }
        if label.len() > 63 {
            return false;
        }
        if first && label == "*" {
            first = false;
            continue;
        }
        first = false;
        let bytes = label.as_bytes();
        if bytes[0] == b'-' || bytes[bytes.len() - 1] == b'-' {
            return false;
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return false;
        }
    }
    true
}

/// RFC-6125 hostname matching with at most one leftmost wildcard label.
pub fn dns_matches(pattern: &str, name: &str) -> bool {
    if pattern == name {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        // wildcard matches exactly one leftmost label
        if let Some((label, rest)) = name.split_once('.') {
            return !label.is_empty() && rest == suffix;
        }
    }
    false
}

/// Normalise an IPv4 dotted-quad string to canonical form, or `None` if it is
/// not a valid IPv4 address. (IPv6 is out of scope for this model.)
pub fn normalize_ip(ip: &str) -> Option<String> {
    let octets: Vec<&str> = ip.trim().split('.').collect();
    if octets.len() != 4 {
        return None;
    }
    let mut parts = Vec::with_capacity(4);
    for o in octets {
        if o.is_empty() || o.len() > 3 || !o.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let v: u16 = o.parse().ok()?;
        if v > 255 {
            return None;
        }
        parts.push(v.to_string());
    }
    Some(parts.join("."))
}

/// A validity window, represented as Unix seconds. Talos uses a monotonic-ish
/// "now" from the runtime; we keep it as a plain `u64` to stay `no_std`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Validity {
    /// Not-before, Unix seconds.
    pub not_before: u64,
    /// Not-after, Unix seconds.
    pub not_after: u64,
}

impl Validity {
    /// Construct from a start time and a duration in seconds.
    pub fn from_duration(not_before: u64, ttl_secs: u64) -> Result<Self, TrustError> {
        let not_after = not_before
            .checked_add(ttl_secs)
            .ok_or_else(|| TrustError::invalid("validity overflow"))?;
        if ttl_secs == 0 {
            return Err(TrustError::invalid("zero TTL"));
        }
        Ok(Validity {
            not_before,
            not_after,
        })
    }

    /// Whether `now` falls within the window (inclusive of `not_before`).
    pub fn contains(&self, now: u64) -> bool {
        now >= self.not_before && now < self.not_after
    }

    /// Whether the certificate is expired at `now`.
    pub fn is_expired(&self, now: u64) -> bool {
        now >= self.not_after
    }

    /// Remaining lifetime in seconds at `now` (0 if expired).
    pub fn remaining(&self, now: u64) -> u64 {
        self.not_after.saturating_sub(now)
    }

    /// Whether the cert is within the renewal threshold: Talos renews when less
    /// than the given fraction (numerator/denominator) of life remains.
    pub fn needs_renewal(&self, now: u64, num: u64, den: u64) -> bool {
        if den == 0 {
            return false;
        }
        let total = self.not_after.saturating_sub(self.not_before);
        let threshold = total.saturating_mul(num) / den;
        self.remaining(now) <= threshold
    }
}

/// Lowercase hex encode.
pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Lowercase hex decode.
pub fn hex_decode(s: &str) -> Result<Vec<u8>, TrustError> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) {
        return Err(TrustError::pem("odd-length hex body"));
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(s.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_val(bytes[i])?;
        let lo = hex_val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn hex_val(c: u8) -> Result<u8, TrustError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(TrustError::pem("invalid hex digit")),
    }
}

/// A public/private key pair. The private key is held as opaque bytes; the
/// public key is derived deterministically so equality and matching work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPair {
    private_der: Vec<u8>,
    public_der: Vec<u8>,
}

impl KeyPair {
    /// Construct from explicit key material.
    pub fn new(private_der: impl Into<Vec<u8>>, public_der: impl Into<Vec<u8>>) -> Self {
        KeyPair {
            private_der: private_der.into(),
            public_der: public_der.into(),
        }
    }

    /// Deterministically derive a key pair from a seed. This stands in for a
    /// real keygen; the public key is a fixed transform of the seed so the same
    /// seed always yields the same identity.
    pub fn from_seed(seed: &[u8]) -> Self {
        let private_der = seed.to_vec();
        let public_der: Vec<u8> = seed.iter().rev().map(|b| b ^ 0xA5).collect();
        KeyPair {
            private_der,
            public_der,
        }
    }

    /// The private key DER bytes.
    pub fn private_der(&self) -> &[u8] {
        &self.private_der
    }

    /// The public key DER bytes.
    pub fn public_der(&self) -> &[u8] {
        &self.public_der
    }

    /// Whether the given public key belongs to this pair.
    pub fn matches_public(&self, public_der: &[u8]) -> bool {
        self.public_der == public_der
    }

    /// PEM envelope for the private key.
    pub fn private_pem(&self) -> PEMEncoded {
        PEMEncoded::new(PEMLabel::PrivateKey, self.private_der.clone())
    }

    /// PEM envelope for the public key.
    pub fn public_pem(&self) -> PEMEncoded {
        PEMEncoded::new(PEMLabel::PublicKey, self.public_der.clone())
    }

    /// A short stable fingerprint of the public key, hex-encoded.
    pub fn fingerprint(&self) -> String {
        // FNV-1a 64-bit over the public key for a deterministic id.
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in &self.public_der {
            hash ^= u64::from(b);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        hex_encode(&hash.to_be_bytes())
    }
}

impl core::fmt::Display for DistinguishedName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_rfc())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pem_round_trips() {
        let pem = PEMEncoded::new(PEMLabel::Certificate, vec![0xde, 0xad, 0xbe, 0xef]);
        let text = pem.encode();
        assert!(text.contains("BEGIN CERTIFICATE"));
        let back = PEMEncoded::decode(&text).unwrap();
        assert_eq!(pem, back);
    }

    #[test]
    fn validity_renewal_threshold() {
        let v = Validity::from_duration(1000, 1000).unwrap();
        assert!(v.contains(1500));
        assert!(!v.is_expired(1999));
        assert!(v.is_expired(2000));
        // less than 1/2 remaining -> needs renewal
        assert!(!v.needs_renewal(1400, 1, 2));
        assert!(v.needs_renewal(1600, 1, 2));
    }

    #[test]
    fn keypair_from_seed_is_deterministic() {
        let a = KeyPair::from_seed(b"node-seed");
        let b = KeyPair::from_seed(b"node-seed");
        assert_eq!(a, b);
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert!(a.matches_public(b.public_der()));
    }

    #[test]
    fn hex_decode_rejects_bad_input() {
        assert!(hex_decode("abc").is_err());
        assert!(hex_decode("zz").is_err());
        assert_eq!(
            hex_decode("deadbeef").unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn dn_round_trips_through_rfc() {
        let dn = DistinguishedName::common("node-1")
            .with_org("os:reader")
            .with_org("os:admin");
        let s = dn.to_rfc();
        let back = DistinguishedName::parse_rfc(&s).unwrap();
        assert_eq!(dn, back);
        assert_eq!(
            back.role_ous().collect::<Vec<_>>(),
            vec!["os:reader", "os:admin"]
        );
    }

    #[test]
    fn dn_parse_rejects_bad_input() {
        assert_eq!(
            DistinguishedName::parse_rfc("O=only-org")
                .unwrap_err()
                .kind(),
            "invalid"
        );
        assert_eq!(
            DistinguishedName::parse_rfc("CN=").unwrap_err().kind(),
            "invalid"
        );
        assert_eq!(
            DistinguishedName::parse_rfc("X=bad,CN=n")
                .unwrap_err()
                .kind(),
            "invalid"
        );
        assert_eq!(
            DistinguishedName::parse_rfc("noequals").unwrap_err().kind(),
            "invalid"
        );
    }

    #[test]
    fn dns_wildcard_matching() {
        assert!(dns_matches("node.cluster.local", "node.cluster.local"));
        assert!(dns_matches("*.cluster.local", "node.cluster.local"));
        assert!(!dns_matches("*.cluster.local", "a.b.cluster.local"));
        assert!(!dns_matches("*.cluster.local", "cluster.local"));
        assert!(!dns_matches("*.cluster.local", ".cluster.local"));
    }

    #[test]
    fn san_dns_validation() {
        assert!(is_valid_dns_name("node-1.cluster.local"));
        assert!(is_valid_dns_name("*.cluster.local"));
        assert!(!is_valid_dns_name(""));
        assert!(!is_valid_dns_name("-bad.example.com"));
        assert!(!is_valid_dns_name("bad-.example.com"));
        assert!(!is_valid_dns_name("under_score.com"));
        assert!(!is_valid_dns_name("a..b"));
    }

    #[test]
    fn ip_san_normalisation_and_coverage() {
        let mut sans = SubjectAltNames::default();
        sans.push_ip("192.168.000.1").unwrap();
        assert!(sans.covers_ip("192.168.0.1"));
        assert!(!sans.covers_ip("192.168.0.2"));
        assert_eq!(sans.push_ip("999.1.1.1").unwrap_err().kind(), "invalid");
        assert_eq!(sans.push_ip("1.2.3").unwrap_err().kind(), "invalid");
        assert!(!sans.covers_ip("not-an-ip"));
    }

    #[test]
    fn san_validate_catches_bad_entries() {
        let mut sans = SubjectAltNames::default();
        sans.dns_names.push("good.example.com".into());
        assert!(sans.validate().is_ok());
        sans.dns_names.push("_bad".into());
        assert!(sans.validate().is_err());
    }

    #[test]
    fn covers_dns_wildcard_helper() {
        let mut sans = SubjectAltNames::default();
        sans.dns_names.push("*.cluster.local".into());
        assert!(sans.covers_dns_wildcard("worker-1.cluster.local"));
        assert!(!sans.covers_dns("worker-1.cluster.local"));
    }

    #[test]
    fn uri_san_push_validate_and_coverage() {
        let mut sans = SubjectAltNames::default();
        assert!(sans.is_empty());
        sans.push_uri("spiffe://oyatie.cell-7/platform/cloud-iam-pdp")
            .unwrap();
        assert!(!sans.is_empty());
        assert!(sans.covers_uri("spiffe://oyatie.cell-7/platform/cloud-iam-pdp"));
        assert!(!sans.covers_uri("spiffe://oyatie.cell-7/platform/other"));
        assert!(sans.validate().is_ok());
    }

    #[test]
    fn uri_san_rejects_malformed() {
        let mut sans = SubjectAltNames::default();
        assert_eq!(sans.push_uri("").unwrap_err().kind(), "invalid");
        assert_eq!(sans.push_uri("not-a-uri").unwrap_err().kind(), "invalid");
        assert_eq!(sans.push_uri("spiffe://").unwrap_err().kind(), "invalid");
        assert_eq!(
            sans.push_uri("has space://authority").unwrap_err().kind(),
            "invalid"
        );
        // validate() also catches a URI smuggled directly into the vector.
        sans.uris.push("://no-scheme".into());
        assert!(sans.validate().is_err());
    }

    #[test]
    fn is_valid_uri_shapes() {
        assert!(is_valid_uri("spiffe://oyatie.cell-1/tenant/ten_acme/wl_x"));
        assert!(is_valid_uri("https://idp.example.com"));
        assert!(!is_valid_uri(""));
        assert!(!is_valid_uri("spiffe:/single-slash"));
        assert!(!is_valid_uri("1scheme://authority")); // scheme must start alpha
        assert!(!is_valid_uri("spiffe://")); // empty authority
        assert!(!is_valid_uri("spiffe://auth ority/x")); // whitespace
    }
}
