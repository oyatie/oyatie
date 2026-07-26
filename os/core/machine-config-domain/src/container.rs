//! The multi-document [`Config`] container: one mandatory legacy v1alpha1
//! document plus any number of auxiliary typed documents.
//!
//! Mirrors the Talos `container.Config`, which wraps a v1alpha1 document and a
//! list of additional documents and presents the merged [`crate::Provider`]
//! surface.

use crate::document::{ConfigVersion, Document, DocumentMeta};
use crate::v1alpha1::V1Alpha1Config;
use crate::validation::{ValidationMode, ValidationReport, Validator};
use os_kernel::error::{Error, Result};

/// An auxiliary typed document held alongside the core v1alpha1 document.
///
/// In a full port these would be concrete typed documents; here each carries its
/// metadata and an opaque body, which is enough to enforce the container
/// invariants (singleton kinds, version agreement) and round-trip ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuxDocument {
    /// The document header.
    pub meta: DocumentMeta,
    /// Opaque document body.
    pub body: String,
    /// Whether this kind may appear more than once.
    pub allow_multiple: bool,
}

impl AuxDocument {
    /// Build an auxiliary singleton document.
    pub fn new(kind: impl Into<String>, body: impl Into<String>) -> Self {
        AuxDocument {
            meta: DocumentMeta::new(ConfigVersion::V1Alpha1Doc, kind),
            body: body.into(),
            allow_multiple: false,
        }
    }

    /// Mark this kind as repeatable.
    pub fn repeatable(mut self) -> Self {
        self.allow_multiple = true;
        self
    }
}

impl Document for AuxDocument {
    fn meta(&self) -> DocumentMeta {
        self.meta.clone()
    }

    fn validate_document(&self) -> Result<()> {
        if self.meta.kind.is_empty() {
            return Err(Error::invalid("auxiliary document kind is empty"));
        }
        Ok(())
    }

    fn allow_multiple(&self) -> bool {
        self.allow_multiple
    }
}

/// The multi-document config container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The mandatory legacy v1alpha1 document.
    core: V1Alpha1Config,
    /// Auxiliary typed documents, in declaration order.
    documents: Vec<AuxDocument>,
}

impl Config {
    /// Build a container around a v1alpha1 document.
    pub fn new(core: V1Alpha1Config) -> Self {
        Config {
            core,
            documents: Vec::new(),
        }
    }

    /// The core v1alpha1 document.
    pub fn core(&self) -> &V1Alpha1Config {
        &self.core
    }

    /// Mutable access to the core document (for patching).
    pub fn core_mut(&mut self) -> &mut V1Alpha1Config {
        &mut self.core
    }

    /// The auxiliary documents.
    pub fn documents(&self) -> &[AuxDocument] {
        &self.documents
    }

    /// Total document count (core + auxiliary).
    pub fn len(&self) -> usize {
        1 + self.documents.len()
    }

    /// A container always holds at least the core document, so it is never
    /// empty; this exists for lint-compliance with [`Config::len`].
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Look up the first auxiliary document of a given kind.
    pub fn document(&self, kind: &str) -> Option<&AuxDocument> {
        self.documents.iter().find(|d| d.meta.kind == kind)
    }

    /// Add an auxiliary document, enforcing singleton kinds.
    ///
    /// Returns an error if a non-repeatable kind already exists, mirroring the
    /// Talos rule that most document kinds may appear at most once.
    pub fn add_document(&mut self, doc: AuxDocument) -> Result<()> {
        if !doc.allow_multiple
            && let Some(existing) = self.document(&doc.meta.kind)
            && !existing.allow_multiple
        {
            return Err(Error::invalid(format!(
                "duplicate document of kind '{}'",
                doc.meta.kind
            )));
        }
        self.documents.push(doc);
        Ok(())
    }

    /// Validate the whole container against a runtime mode: each document is
    /// validated, the core sub-trees are validated, and singleton constraints
    /// are enforced.
    pub fn validate(&self, mode: ValidationMode) -> Result<Vec<String>> {
        let mut report = ValidationReport::new();
        self.validate_into(mode, &mut report);
        report.into_result()
    }
}

impl Validator for Config {
    fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport) {
        self.core.validate_into(mode, report);
        // Enforce singleton-kind uniqueness across auxiliary docs.
        for (i, doc) in self.documents.iter().enumerate() {
            if let Err(e) = doc.validate_document() {
                report.push(crate::validation::ValidationError::invalid(
                    format!("documents[{i}]"),
                    e.to_string(),
                ));
            }
            if !doc.allow_multiple {
                let dup = self.documents[..i]
                    .iter()
                    .any(|d| d.meta.kind == doc.meta.kind);
                if dup {
                    report.push(crate::validation::ValidationError::Conflict(format!(
                        "duplicate singleton document kind '{}'",
                        doc.meta.kind
                    )));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::{ClusterConfig, ControlPlaneEndpoint};
    use crate::machine::{InstallConfig, MachineConfig};
    use os_kernel::machine_type::MachineType;

    fn core() -> V1Alpha1Config {
        let mut machine = MachineConfig::new(MachineType::ControlPlane);
        machine.token = "tok".to_string();
        machine.ca_crt = "ca".to_string();
        machine.install = InstallConfig::new("/dev/sda", "img");
        let cluster = ClusterConfig::new(
            "prod",
            ControlPlaneEndpoint::parse("https://10.0.0.1:6443").unwrap(),
        );
        V1Alpha1Config::new(machine, cluster)
    }

    #[test]
    fn container_has_core_plus_aux() {
        let mut c = Config::new(core());
        assert_eq!(c.len(), 1);
        c.add_document(AuxDocument::new("SideroLinkConfig", "apiUrl: grpc://x"))
            .unwrap();
        assert_eq!(c.len(), 2);
        assert!(!c.is_empty());
        assert!(c.document("SideroLinkConfig").is_some());
        assert!(c.document("Nope").is_none());
    }

    #[test]
    fn singleton_kinds_rejected_on_add() {
        let mut c = Config::new(core());
        c.add_document(AuxDocument::new("KmsgLogConfig", "a"))
            .unwrap();
        let err = c
            .add_document(AuxDocument::new("KmsgLogConfig", "b"))
            .unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn repeatable_kinds_allowed() {
        let mut c = Config::new(core());
        c.add_document(AuxDocument::new("NetworkRuleConfig", "a").repeatable())
            .unwrap();
        c.add_document(AuxDocument::new("NetworkRuleConfig", "b").repeatable())
            .unwrap();
        assert_eq!(c.len(), 3);
    }

    #[test]
    fn container_validates_core() {
        let c = Config::new(core());
        assert!(c.validate(ValidationMode::Metal).is_ok());

        let bad = Config::new(V1Alpha1Config::default());
        assert!(bad.validate(ValidationMode::Metal).is_err());
    }

    #[test]
    fn core_mut_allows_in_place_edit() {
        let mut c = Config::new(core());
        c.core_mut().machine.machine_type = MachineType::Worker;
        assert_eq!(c.core().machine.machine_type, MachineType::Worker);
    }
}
