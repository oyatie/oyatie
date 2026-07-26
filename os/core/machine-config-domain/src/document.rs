//! The [`Document`] trait and document metadata, mirroring the Talos
//! `config.Document` interface and `meta` header that every config document
//! carries (`apiVersion` + `kind`).

use core::fmt;
use os_kernel::error::{Error, Result};

/// The configuration schema version of a document.
///
/// Mirrors the Talos `apiVersion`/`version` field. `V1Alpha1` is the legacy
/// monolithic document; `V1Alpha1Doc` is the apiVersion used by the newer
/// multi-document typed configs (`v1alpha1` with an explicit `kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigVersion {
    /// Legacy single document (`version: v1alpha1`, no `apiVersion`).
    V1Alpha1,
    /// Typed multi-document config (`apiVersion: v1alpha1`).
    V1Alpha1Doc,
}

impl ConfigVersion {
    /// Canonical string form.
    pub fn as_str(self) -> &'static str {
        match self {
            ConfigVersion::V1Alpha1 | ConfigVersion::V1Alpha1Doc => "v1alpha1",
        }
    }

    /// Parse from the `apiVersion` / `version` string.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim() {
            "v1alpha1" => Ok(ConfigVersion::V1Alpha1),
            other => Err(Error::parse(format!(
                "unsupported config version '{other}'"
            ))),
        }
    }
}

impl fmt::Display for ConfigVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The header common to every config document: the schema version and the
/// document `kind`. Equivalent to Talos `meta` (apiVersion + kind).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentMeta {
    /// Schema version (`apiVersion`).
    pub version: ConfigVersion,
    /// Document kind (e.g. `"v1alpha1"`, `"SideroLinkConfig"`, ...). The legacy
    /// monolithic document uses kind `"v1alpha1"`.
    pub kind: String,
}

impl DocumentMeta {
    /// Build metadata.
    pub fn new(version: ConfigVersion, kind: impl Into<String>) -> Self {
        DocumentMeta {
            version,
            kind: kind.into(),
        }
    }

    /// Metadata for the legacy monolithic v1alpha1 document.
    pub fn v1alpha1() -> Self {
        DocumentMeta {
            version: ConfigVersion::V1Alpha1,
            kind: "v1alpha1".to_string(),
        }
    }
}

/// A single configuration document.
///
/// Mirrors the Talos `config.Document` interface: each document reports its
/// `apiVersion`/`kind`, and is independently validatable. Concrete
/// implementations include the legacy [`crate::v1alpha1::V1Alpha1Config`] and
/// the auxiliary typed documents.
pub trait Document: fmt::Debug {
    /// The document metadata header.
    fn meta(&self) -> DocumentMeta;

    /// The document kind. Defaults to the kind in [`Document::meta`].
    fn kind(&self) -> String {
        self.meta().kind
    }

    /// The schema version. Defaults to the version in [`Document::meta`].
    fn version(&self) -> ConfigVersion {
        self.meta().version
    }

    /// Validate the document in isolation, returning the first error.
    fn validate_document(&self) -> Result<()>;

    /// Whether this document may appear more than once in a container. Most
    /// documents are singletons; some (e.g. extra network rules) are not.
    fn allow_multiple(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_parse_and_display() {
        assert_eq!(
            ConfigVersion::parse("v1alpha1").unwrap(),
            ConfigVersion::V1Alpha1
        );
        assert_eq!(
            ConfigVersion::parse("  v1alpha1 ").unwrap(),
            ConfigVersion::V1Alpha1
        );
        assert!(ConfigVersion::parse("v1beta9").is_err());
        assert_eq!(ConfigVersion::V1Alpha1.to_string(), "v1alpha1");
    }

    #[test]
    fn v1alpha1_meta_defaults() {
        let m = DocumentMeta::v1alpha1();
        assert_eq!(m.kind, "v1alpha1");
        assert_eq!(m.version, ConfigVersion::V1Alpha1);
    }
}
