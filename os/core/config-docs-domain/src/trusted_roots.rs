//! `TrustedRootsConfig` — extra trusted CA roots.
//!
//! Mirrors `pkg/machinery/config/types/security`. Each document is keyed by
//! `name:` and supplies one or more PEM-encoded CA certificates that Talos adds
//! to the host trust store (used by containerd registry pulls, extension
//! services, etc.).

use crate::document::{ConfigDocument, DocId, DocKind};
use os_kernel::error::{Error, Result};

const PEM_BEGIN: &str = "-----BEGIN CERTIFICATE-----";
const PEM_END: &str = "-----END CERTIFICATE-----";

/// The `TrustedRootsConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedRootsConfig {
    /// Bundle name (document key).
    pub name: String,
    /// Concatenated PEM certificate bundle.
    pub certificates: String,
}

impl TrustedRootsConfig {
    /// Construct a trusted roots bundle.
    pub fn new(name: impl Into<String>, certificates: impl Into<String>) -> Self {
        TrustedRootsConfig {
            name: name.into(),
            certificates: certificates.into(),
        }
    }

    /// Count the number of PEM certificate blocks in the bundle.
    #[must_use]
    pub fn certificate_count(&self) -> usize {
        self.certificates.matches(PEM_BEGIN).count()
    }
}

impl ConfigDocument for TrustedRootsConfig {
    fn kind(&self) -> DocKind {
        DocKind::TrustedRoots
    }

    fn id(&self) -> DocId {
        DocId::keyed(DocKind::TrustedRoots, self.name.clone())
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("TrustedRootsConfig: name is required"));
        }
        if self.certificates.trim().is_empty() {
            return Err(Error::invalid(
                "TrustedRootsConfig: certificates are required",
            ));
        }
        let begins = self.certificates.matches(PEM_BEGIN).count();
        let ends = self.certificates.matches(PEM_END).count();
        if begins == 0 {
            return Err(Error::invalid(
                "TrustedRootsConfig: certificates must contain at least one PEM CERTIFICATE block",
            ));
        }
        if begins != ends {
            return Err(Error::invalid(format!(
                "TrustedRootsConfig: unbalanced PEM markers ({begins} begin, {ends} end)"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pem(n: usize) -> String {
        let mut s = String::new();
        for _ in 0..n {
            s.push_str(PEM_BEGIN);
            s.push_str("\nMIIB...base64...\n");
            s.push_str(PEM_END);
            s.push('\n');
        }
        s
    }

    #[test]
    fn valid_single_cert() {
        let c = TrustedRootsConfig::new("corp-ca", pem(1));
        assert!(c.validate().is_ok());
        assert_eq!(c.certificate_count(), 1);
        assert!(c.allows_multiple());
    }

    #[test]
    fn valid_multi_cert() {
        let c = TrustedRootsConfig::new("bundle", pem(3));
        assert!(c.validate().is_ok());
        assert_eq!(c.certificate_count(), 3);
    }

    #[test]
    fn empty_name_rejected() {
        assert!(TrustedRootsConfig::new("", pem(1)).validate().is_err());
    }

    #[test]
    fn empty_certs_rejected() {
        assert!(TrustedRootsConfig::new("ca", "   ").validate().is_err());
    }

    #[test]
    fn non_pem_rejected() {
        assert!(
            TrustedRootsConfig::new("ca", "just some text")
                .validate()
                .is_err()
        );
    }

    #[test]
    fn unbalanced_pem_rejected() {
        let bad = format!("{PEM_BEGIN}\nbody\n");
        assert!(TrustedRootsConfig::new("ca", bad).validate().is_err());
    }
}
