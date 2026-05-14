//! Foundry account-auth kernel — neutral identity + reference value types.
//!
//! Per ADR-0056 (12-layer enum, port-in-kernel): the kernel holds the
//! value types that ports exchange across product boundaries. Adapter
//! kernels in outer rings consume directly from here; the domain
//! re-exports for backwards-compat so existing call sites stay valid.
//!
//! No I/O. No provider-specific code. No state-machine behavior — that
//! lives in `oya-foundry-account-domain`.

use std::fmt;

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AccountId(pub String);

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SessionId(pub String);

/// Allowlisted provider family. Adding a family requires an ADR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderFamily {
    Aws,
    Oci,
    Claude,
    OpenAiOrCodex,
    Gemini,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderFamilyError(pub String);

impl fmt::Display for ProviderFamilyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "provider family not allowlisted: {}", self.0)
    }
}

impl TryFrom<&str> for ProviderFamily {
    type Error = ProviderFamilyError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "AWS" => Ok(Self::Aws),
            "OCI" => Ok(Self::Oci),
            "Claude" => Ok(Self::Claude),
            "OpenAIOrCodex" => Ok(Self::OpenAiOrCodex),
            "Gemini" => Ok(Self::Gemini),
            other => Err(ProviderFamilyError(other.to_owned())),
        }
    }
}

/// Reference to a secret in some external store. Carries no raw
/// bytes — `Debug` redacts, no `Display` impl.
#[derive(Clone, Eq, PartialEq)]
pub struct SecretReference(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretReferenceError(pub String);

impl fmt::Display for SecretReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid secret reference: {}", self.0)
    }
}

impl SecretReference {
    pub fn new(sref: String) -> Result<Self, SecretReferenceError> {
        if !sref.starts_with("sref://") {
            return Err(SecretReferenceError("must use sref:// scheme".to_owned()));
        }
        if sref.len() <= 7 {
            return Err(SecretReferenceError("reference body is empty".to_owned()));
        }
        Ok(Self(sref))
    }
}

impl fmt::Debug for SecretReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretReference(sref://[REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_family_aws() {
        assert_eq!(ProviderFamily::try_from("AWS"), Ok(ProviderFamily::Aws));
    }

    #[test]
    fn provider_family_claude() {
        assert_eq!(
            ProviderFamily::try_from("Claude"),
            Ok(ProviderFamily::Claude)
        );
    }

    #[test]
    fn provider_family_rejects_unknown() {
        assert!(ProviderFamily::try_from("Anthropic").is_err());
    }

    #[test]
    fn provider_family_error_carries_input() {
        let err = ProviderFamily::try_from("BadProvider").unwrap_err();
        assert!(err.0.contains("BadProvider"));
    }

    #[test]
    fn secret_ref_valid_sref_scheme() {
        assert!(SecretReference::new("sref://my-secret-id".to_owned()).is_ok());
    }

    #[test]
    fn secret_ref_rejects_non_sref_scheme() {
        assert!(SecretReference::new("http://my-secret".to_owned()).is_err());
    }

    #[test]
    fn secret_ref_rejects_bare_sref() {
        assert!(SecretReference::new("sref://".to_owned()).is_err());
    }

    #[test]
    fn secret_ref_debug_is_redacted() {
        let r = SecretReference::new("sref://very-secret-value".to_owned()).unwrap();
        let dbg = format!("{r:?}");
        assert!(dbg.contains("[REDACTED]"));
        assert!(!dbg.contains("very-secret"));
    }
}
