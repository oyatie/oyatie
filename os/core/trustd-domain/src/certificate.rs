//! The [`Certificate`] type: an issued, signed X.509 certificate.

use crate::error::{Result, TrustError};
use crate::x509::{DistinguishedName, PEMEncoded, PEMLabel, SubjectAltNames, Validity};
use os_kernel::role::RoleSet;

/// The intended usage of a certificate, gating what kinds of certs the CA will
/// issue (mirrors Talos KeyUsage/ExtKeyUsage handling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertUsage {
    /// A certificate authority (can sign other certs).
    CertificateAuthority,
    /// A TLS server certificate (apid, machined endpoints).
    ServerAuth,
    /// A TLS client certificate (talosctl, node-to-node).
    ClientAuth,
}

impl CertUsage {
    /// Whether a certificate with this usage may sign other certificates.
    pub fn can_sign(self) -> bool {
        matches!(self, CertUsage::CertificateAuthority)
    }
}

/// An issued certificate. Equivalent to the parsed contents of an
/// `x509.Certificate` in Talos, plus the signature linking it to its issuer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate {
    /// Monotonic serial number assigned by the issuing CA.
    pub serial: u64,
    /// Subject distinguished name.
    pub subject: DistinguishedName,
    /// Issuer distinguished name (equals subject for a self-signed CA).
    pub issuer: DistinguishedName,
    /// Validity window.
    pub validity: Validity,
    /// Intended usage.
    pub usage: CertUsage,
    /// Subject alternative names.
    pub sans: SubjectAltNames,
    /// The DER bytes of the subject's public key.
    pub public_key_der: Vec<u8>,
    /// The signature produced by the issuer over this certificate's TBS bytes.
    pub signature: Vec<u8>,
}

impl Certificate {
    /// The RBAC roles encoded in the subject's organizational units.
    pub fn roles(&self) -> RoleSet {
        RoleSet::parse_ous(self.subject.organizations.iter().map(String::as_str))
    }

    /// Whether this is a CA certificate.
    pub fn is_ca(&self) -> bool {
        self.usage.can_sign()
    }

    /// Whether the certificate is currently valid at `now`.
    pub fn is_valid_at(&self, now: u64) -> bool {
        self.validity.contains(now)
    }

    /// Deterministic "to-be-signed" bytes used as the signing input. In a real
    /// implementation this is the DER `TBSCertificate`; here it is a stable
    /// serialization of the load-bearing fields.
    pub fn tbs_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.serial.to_be_bytes());
        buf.extend_from_slice(self.subject.to_rfc().as_bytes());
        buf.push(b'|');
        buf.extend_from_slice(self.issuer.to_rfc().as_bytes());
        buf.push(b'|');
        buf.extend_from_slice(&self.validity.not_before.to_be_bytes());
        buf.extend_from_slice(&self.validity.not_after.to_be_bytes());
        buf.push(self.usage as u8);
        for d in &self.sans.dns_names {
            buf.extend_from_slice(d.as_bytes());
            buf.push(b',');
        }
        for ip in &self.sans.ip_addresses {
            buf.extend_from_slice(ip.as_bytes());
            buf.push(b',');
        }
        // URI SANs are signed too: a SPIFFE SVID's identity is the URI SAN, so
        // it MUST be inside the to-be-signed bytes or an attacker could append
        // a forged identity to a validly-signed cert without breaking the
        // signature. The `|` separator keeps the URI section unambiguous from
        // the DNS/IP sections above.
        buf.push(b'|');
        for uri in &self.sans.uris {
            buf.extend_from_slice(uri.as_bytes());
            buf.push(b',');
        }
        buf.extend_from_slice(&self.public_key_der);
        buf
    }

    /// PEM envelope of the certificate. The body is the TBS bytes followed by
    /// the signature, so a verifier can reconstruct both.
    pub fn to_pem(&self) -> PEMEncoded {
        let mut der = self.tbs_bytes();
        der.extend_from_slice(&self.signature);
        PEMEncoded::new(PEMLabel::Certificate, der)
    }

    /// Validate basic structural invariants Talos enforces before trusting a
    /// certificate (non-empty CN, signature present, sane validity).
    pub fn validate(&self) -> Result<()> {
        if self.subject.common_name.is_empty() {
            return Err(TrustError::invalid("certificate has empty common name"));
        }
        if self.signature.is_empty() {
            return Err(TrustError::invalid("certificate is unsigned"));
        }
        if self.validity.not_after <= self.validity.not_before {
            return Err(TrustError::invalid("certificate validity window is empty"));
        }
        Ok(())
    }

    /// Common name accessor.
    pub fn common_name(&self) -> &str {
        &self.subject.common_name
    }
}

/// A bundle of a certificate with its matching private key PEM, as returned to
/// a node that just had its identity issued.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedIdentity {
    /// The signed certificate.
    pub certificate: Certificate,
    /// PEM-encoded certificate.
    pub cert_pem: PEMEncoded,
    /// PEM-encoded private key.
    pub key_pem: PEMEncoded,
    /// PEM-encoded issuing CA certificate, for chain building.
    pub ca_pem: PEMEncoded,
}

impl IssuedIdentity {
    /// The hostname / common name of the issued identity.
    pub fn name(&self) -> &str {
        self.certificate.common_name()
    }

    /// Convenience: the issued certificate's textual PEM.
    pub fn cert_pem_text(&self) -> String {
        self.cert_pem.encode()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x509::DistinguishedName;

    fn sample(usage: CertUsage) -> Certificate {
        Certificate {
            serial: 1,
            subject: DistinguishedName::common("node-1").with_org("os:reader"),
            issuer: DistinguishedName::common("talos-ca"),
            validity: Validity::from_duration(0, 100).unwrap(),
            usage,
            sans: SubjectAltNames::default(),
            public_key_der: vec![1, 2, 3],
            signature: vec![9, 9],
        }
    }

    #[test]
    fn roles_decoded_from_ou() {
        let c = sample(CertUsage::ClientAuth);
        assert!(c.roles().can_read());
        assert!(!c.roles().can_write());
    }

    #[test]
    fn validate_rejects_unsigned() {
        let mut c = sample(CertUsage::ClientAuth);
        c.signature.clear();
        assert_eq!(c.validate().unwrap_err().kind(), "invalid");
    }

    #[test]
    fn ca_flag_follows_usage() {
        assert!(sample(CertUsage::CertificateAuthority).is_ca());
        assert!(!sample(CertUsage::ServerAuth).is_ca());
    }

    #[test]
    fn tbs_changes_with_serial() {
        let a = sample(CertUsage::ClientAuth);
        let mut b = a.clone();
        b.serial = 2;
        assert_ne!(a.tbs_bytes(), b.tbs_bytes());
    }
}
