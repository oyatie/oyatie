//! Extension manifest parsing and metadata, mirroring Talos
//! `pkg/machinery/extensions` (`manifest.yaml`).
//!
//! A Talos system extension ships as an OCI image whose rootfs contains a
//! `manifest.yaml` describing the extension's identity and the Talos versions it
//! is compatible with. This module models that manifest, a small dependency-free
//! parser for the subset of YAML Talos actually uses, and the [`ExtensionKind`]
//! taxonomy (rootfs payload vs. firmware vs. extension service).

use std::collections::BTreeMap;
use std::fmt;

use os_kernel::error::{Error, Result};
use os_kernel::version::Version;

/// The kind of payload an extension contributes to the running system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionKind {
    /// Files merged into the root filesystem (binaries, libraries, configs).
    Rootfs,
    /// Firmware blobs installed under `/lib/firmware`.
    Firmware,
    /// A long-running service launched by the extension service controller.
    Service,
    /// Kernel modules installed under `/lib/modules`.
    KernelModule,
}

impl ExtensionKind {
    /// Stable lowercase identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            ExtensionKind::Rootfs => "rootfs",
            ExtensionKind::Firmware => "firmware",
            ExtensionKind::Service => "service",
            ExtensionKind::KernelModule => "kernel-module",
        }
    }

    /// Parse a kind from its lowercase identifier.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim() {
            "rootfs" => Ok(ExtensionKind::Rootfs),
            "firmware" => Ok(ExtensionKind::Firmware),
            "service" => Ok(ExtensionKind::Service),
            "kernel-module" | "kernel_module" => Ok(ExtensionKind::KernelModule),
            other => Err(Error::parse(format!("unknown extension kind '{other}'"))),
        }
    }
}

impl fmt::Display for ExtensionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Compatibility constraints declared by an extension manifest.
///
/// Talos manifests carry a `compatibility.talos.version` constraint such as
/// `">= v1.6.0"`. We model the common operators Talos accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compatibility {
    /// Talos version constraint, if declared.
    pub talos: Option<VersionConstraint>,
}

impl Compatibility {
    /// An empty (unconstrained) compatibility block.
    pub fn unconstrained() -> Self {
        Compatibility { talos: None }
    }

    /// Whether `running` satisfies the Talos compatibility constraint. An absent
    /// constraint is treated as compatible with everything.
    pub fn is_satisfied_by(&self, running: &Version) -> bool {
        match &self.talos {
            Some(c) => c.matches(running),
            None => true,
        }
    }
}

/// A single comparison operator used in version constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintOp {
    /// `>=`
    GreaterOrEqual,
    /// `>`
    Greater,
    /// `<=`
    LessOrEqual,
    /// `<`
    Less,
    /// `==`
    Equal,
}

impl ConstraintOp {
    fn parse(s: &str) -> Result<Self> {
        match s {
            ">=" => Ok(ConstraintOp::GreaterOrEqual),
            ">" => Ok(ConstraintOp::Greater),
            "<=" => Ok(ConstraintOp::LessOrEqual),
            "<" => Ok(ConstraintOp::Less),
            "==" | "=" => Ok(ConstraintOp::Equal),
            other => Err(Error::parse(format!(
                "unknown constraint operator '{other}'"
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            ConstraintOp::GreaterOrEqual => ">=",
            ConstraintOp::Greater => ">",
            ConstraintOp::LessOrEqual => "<=",
            ConstraintOp::Less => "<",
            ConstraintOp::Equal => "==",
        }
    }
}

/// A version constraint: an operator plus a reference version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionConstraint {
    /// The comparison operator.
    pub op: ConstraintOp,
    /// The reference version compared against.
    pub version: Version,
}

impl VersionConstraint {
    /// Construct a constraint directly.
    pub fn new(op: ConstraintOp, version: Version) -> Self {
        VersionConstraint { op, version }
    }

    /// Parse a constraint like `">= v1.6.0"`. A bare version is treated as `==`.
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Err(Error::parse("empty version constraint"));
        }
        // Find the boundary between operator characters and the version.
        let op_len = s
            .char_indices()
            .find(|(_, c)| !matches!(c, '>' | '<' | '=' | '!'))
            .map(|(i, _)| i)
            .unwrap_or(s.len());

        let (op_str, ver_str) = s.split_at(op_len);
        let op_str = op_str.trim();
        let ver_str = ver_str.trim();
        let op = if op_str.is_empty() {
            ConstraintOp::Equal
        } else {
            ConstraintOp::parse(op_str)?
        };
        let version = Version::parse(ver_str)?;
        Ok(VersionConstraint { op, version })
    }

    /// Whether `candidate` satisfies this constraint.
    pub fn matches(&self, candidate: &Version) -> bool {
        let cand = candidate.to_release();
        let reference = self.version.to_release();
        match self.op {
            ConstraintOp::GreaterOrEqual => cand >= reference,
            ConstraintOp::Greater => cand > reference,
            ConstraintOp::LessOrEqual => cand <= reference,
            ConstraintOp::Less => cand < reference,
            ConstraintOp::Equal => cand == reference,
        }
    }
}

impl fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.op.as_str(), self.version)
    }
}

/// The parsed contents of an extension `manifest.yaml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionManifest {
    /// Extension name, e.g. `gvisor`, `nvidia-container-toolkit`.
    pub name: String,
    /// Extension version (its own version, not Talos's).
    pub version: Version,
    /// Author / vendor.
    pub author: String,
    /// Short human-readable description.
    pub description: String,
    /// What the extension contributes to the system.
    pub kind: ExtensionKind,
    /// Talos compatibility constraints.
    pub compatibility: Compatibility,
}

impl ExtensionManifest {
    /// Build a manifest programmatically with sensible defaults.
    pub fn new(name: impl Into<String>, version: Version, kind: ExtensionKind) -> Self {
        ExtensionManifest {
            name: name.into(),
            version,
            author: String::new(),
            description: String::new(),
            kind,
            compatibility: Compatibility::unconstrained(),
        }
    }

    /// Validate structural invariants of the manifest.
    ///
    /// Talos requires the extension name to be a non-empty lowercase DNS-ish
    /// label (letters, digits, `-`, `_`) so it can be used as a directory and
    /// resource id.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::invalid("extension name must not be empty"));
        }
        if !self
            .name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Err(Error::invalid(format!(
                "extension name '{}' contains invalid characters",
                self.name
            )));
        }
        if self.name.starts_with('-') || self.name.ends_with('-') {
            return Err(Error::invalid(
                "extension name must not start or end with '-'",
            ));
        }
        Ok(())
    }

    /// Whether this extension is compatible with the running Talos version.
    pub fn is_compatible_with(&self, talos: &Version) -> bool {
        self.compatibility.is_satisfied_by(talos)
    }

    /// Parse a manifest from the small YAML subset Talos uses. Supports a flat
    /// set of `key: value` lines plus a nested `compatibility:` block:
    ///
    /// ```yaml
    /// name: gvisor
    /// version: v20231214.0
    /// author: Sidero Labs
    /// description: gVisor container runtime
    /// kind: service
    /// compatibility:
    ///   talos:
    ///     version: ">= v1.6.0"
    /// ```
    pub fn parse(input: &str) -> Result<Self> {
        let mut top: BTreeMap<String, String> = BTreeMap::new();
        let mut compat_version: Option<String> = None;

        // Section tracking: 0 = top-level, 1 = under compatibility, 2 = under
        // compatibility.talos.
        let mut section: u8 = 0;
        for raw in input.lines() {
            // Drop comments.
            let line = match raw.split_once('#') {
                Some((before, _)) => before,
                None => raw,
            };
            if line.trim().is_empty() {
                continue;
            }
            let indent = line.len() - line.trim_start().len();
            let trimmed = line.trim();
            let (key, value) = match trimmed.split_once(':') {
                Some((k, v)) => (k.trim(), v.trim().trim_matches('"').trim_matches('\'')),
                None => {
                    return Err(Error::parse(format!(
                        "malformed manifest line: '{trimmed}'"
                    )));
                }
            };

            if indent == 0 {
                section = 0;
                if key == "compatibility" {
                    section = 1;
                    continue;
                }
                top.insert(key.to_string(), value.to_string());
            } else if section >= 1 {
                if key == "talos" {
                    section = 2;
                    continue;
                }
                if section == 2 && key == "version" {
                    compat_version = Some(value.to_string());
                }
            }
        }

        let name = top
            .get("name")
            .cloned()
            .ok_or_else(|| Error::parse("manifest missing 'name'"))?;
        let version = top
            .get("version")
            .ok_or_else(|| Error::parse("manifest missing 'version'"))
            .and_then(|v| Version::parse(v))?;
        let kind = top
            .get("kind")
            .map(|k| ExtensionKind::parse(k))
            .transpose()?
            .unwrap_or(ExtensionKind::Rootfs);

        let compatibility = match compat_version {
            Some(c) => Compatibility {
                talos: Some(VersionConstraint::parse(&c)?),
            },
            None => Compatibility::unconstrained(),
        };

        let manifest = ExtensionManifest {
            name,
            version,
            author: top.get("author").cloned().unwrap_or_default(),
            description: top.get("description").cloned().unwrap_or_default(),
            kind,
            compatibility,
        };
        manifest.validate()?;
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_manifest() {
        let yaml = "\
name: gvisor
version: v20231214.0.0
author: Sidero Labs
description: gVisor container runtime sandbox
kind: service
compatibility:
  talos:
    version: \">= v1.6.0\"
";
        let m = ExtensionManifest::parse(yaml).unwrap();
        assert_eq!(m.name, "gvisor");
        assert_eq!(m.version, Version::new(20231214, 0, 0));
        assert_eq!(m.author, "Sidero Labs");
        assert_eq!(m.kind, ExtensionKind::Service);
        assert!(m.is_compatible_with(&Version::new(1, 7, 0)));
        assert!(!m.is_compatible_with(&Version::new(1, 5, 0)));
    }

    #[test]
    fn parse_defaults_kind_to_rootfs() {
        let yaml = "name: hello\nversion: v1.0.0\n";
        let m = ExtensionManifest::parse(yaml).unwrap();
        assert_eq!(m.kind, ExtensionKind::Rootfs);
        assert!(m.compatibility.talos.is_none());
        assert!(m.is_compatible_with(&Version::new(99, 0, 0)));
    }

    #[test]
    fn parse_rejects_missing_required_fields() {
        assert!(ExtensionManifest::parse("version: v1.0.0\n").is_err());
        assert!(ExtensionManifest::parse("name: x\n").is_err());
    }

    #[test]
    fn parse_handles_comments_and_quotes() {
        let yaml = "\
# top comment
name: 'nvidia-driver'   # inline comment
version: v535.0.0
";
        let m = ExtensionManifest::parse(yaml).unwrap();
        assert_eq!(m.name, "nvidia-driver");
        assert_eq!(m.version, Version::new(535, 0, 0));
    }

    #[test]
    fn validate_rejects_bad_names() {
        let mut m =
            ExtensionManifest::new("ok-name_1", Version::new(1, 0, 0), ExtensionKind::Rootfs);
        assert!(m.validate().is_ok());
        m.name = "Bad Name".to_string();
        assert!(m.validate().is_err());
        m.name = "-leading".to_string();
        assert!(m.validate().is_err());
        m.name = String::new();
        assert!(m.validate().is_err());
    }

    #[test]
    fn constraint_operators_parse_and_match() {
        let ge = VersionConstraint::parse(">= v1.6.0").unwrap();
        assert_eq!(ge.op, ConstraintOp::GreaterOrEqual);
        assert!(ge.matches(&Version::new(1, 6, 0)));
        assert!(ge.matches(&Version::new(1, 7, 0)));
        assert!(!ge.matches(&Version::new(1, 5, 9)));

        let lt = VersionConstraint::parse("< v2.0.0").unwrap();
        assert!(lt.matches(&Version::new(1, 9, 9)));
        assert!(!lt.matches(&Version::new(2, 0, 0)));

        let eq = VersionConstraint::parse("v1.6.3").unwrap();
        assert_eq!(eq.op, ConstraintOp::Equal);
        assert!(eq.matches(&Version::new(1, 6, 3)));
        assert!(!eq.matches(&Version::new(1, 6, 4)));
    }

    #[test]
    fn kind_round_trip() {
        for k in [
            ExtensionKind::Rootfs,
            ExtensionKind::Firmware,
            ExtensionKind::Service,
            ExtensionKind::KernelModule,
        ] {
            assert_eq!(ExtensionKind::parse(k.as_str()).unwrap(), k);
        }
        assert!(ExtensionKind::parse("nope").is_err());
    }

    #[test]
    fn constraint_ignores_prerelease_for_matching() {
        let ge = VersionConstraint::parse(">= v1.6.0").unwrap();
        let pre = Version::parse("v1.6.0-alpha.1").unwrap();
        // to_release() is used so a pre-release of the boundary still matches.
        assert!(ge.matches(&pre));
    }
}
