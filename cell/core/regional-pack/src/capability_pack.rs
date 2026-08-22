//! Vertical capability pack types for M04-P01 (merge-variant 2026-05-17).
//!
//! A `CapabilityPack` identifies a versioned bundle of per-vertical capabilities
//! that binds to a regional pack.  The elected vertical for M04 is
//! `vertical-corporate` (council-resolution 2026-05-17).

use data_boundary_kernel::{Classified, DataClass};

/// Semantic version variant for a capability pack.
///
/// Variants correspond to the three SemVer axes; the `PackVersion` value is
/// embedded in pack IDs and compared during upgrade checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackVersion {
    /// Backward-compatible additions only.
    Minor { major: u32, minor: u32, patch: u32 },
    /// Breaking interface change — requires tenant migration window.
    Major { major: u32, minor: u32, patch: u32 },
    /// Bug-fix / security-patch only — no interface change.
    Patch { major: u32, minor: u32, patch: u32 },
}

impl PackVersion {
    /// Returns `(major, minor, patch)` tuple regardless of variant.
    pub const fn triplet(self) -> (u32, u32, u32) {
        match self {
            Self::Minor {
                major,
                minor,
                patch,
            }
            | Self::Major {
                major,
                minor,
                patch,
            }
            | Self::Patch {
                major,
                minor,
                patch,
            } => (major, minor, patch),
        }
    }

    /// Canonical string form: `"<major>.<minor>.<patch>"`.
    pub fn display(self) -> String {
        let (maj, min, pat) = self.triplet();
        format!("{maj}.{min}.{pat}")
    }
}

/// A versioned bundle of per-vertical capabilities bound to a regional pack.
///
/// The `vertical_id` must match the elected vertical slug (`"vertical-corporate"`).
/// `pack_ref` must start with `"pack-"` to align with the `RegionalPack` id
/// invariant.
///
/// All fields are private to enforce invariants through [`CapabilityPack::new`].
/// Use the accessor methods to read field values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityPack {
    vertical_id: Classified<String>,
    pack_ref: String, // data_class: INTERNAL_ONLY; references RegionalPack.id
    version: PackVersion,
    capabilities: Classified<Vec<String>>,
}

/// The elected vertical slug for M04 (council-resolution 2026-05-17).
pub const ELECTED_VERTICAL_SLUG: &str = "vertical-corporate";

/// Errors produced by [`CapabilityPack::new`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapabilityPackError {
    EmptyVerticalId,
    /// `vertical_id` is non-empty but does not match the elected slug
    /// (`vertical-corporate`).  Non-elected verticals are rejected at
    /// construction time to prevent invalid bindings propagating into
    /// downstream pack-selection and upgrade flows.
    NonElectedVerticalId,
    InvalidPackRef,
    EmptyCapabilities,
}

impl CapabilityPack {
    /// Returns the elected vertical ID.
    pub fn vertical_id(&self) -> &str {
        &self.vertical_id.value
    }

    /// Returns the pack reference (e.g. `"pack-alpha"`).
    pub fn pack_ref(&self) -> &str {
        &self.pack_ref
    }

    /// Returns the pack version.
    pub fn version(&self) -> PackVersion {
        self.version
    }

    /// Returns the capability list.
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities.value
    }

    /// Constructs and validates a new [`CapabilityPack`].
    ///
    /// # Errors
    /// Returns [`CapabilityPackError`] if any invariant is violated.
    pub fn new(
        vertical_id: String,
        pack_ref: String,
        version: PackVersion,
        capabilities: Vec<String>,
    ) -> Result<Self, CapabilityPackError> {
        if vertical_id.trim().is_empty() {
            return Err(CapabilityPackError::EmptyVerticalId);
        }
        if vertical_id != ELECTED_VERTICAL_SLUG {
            return Err(CapabilityPackError::NonElectedVerticalId);
        }
        if !pack_ref.starts_with("pack-") {
            return Err(CapabilityPackError::InvalidPackRef);
        }
        if capabilities.is_empty() {
            return Err(CapabilityPackError::EmptyCapabilities);
        }
        Ok(Self {
            vertical_id: Classified::new(vertical_id, DataClass::InternalOnly),
            pack_ref,
            version,
            capabilities: Classified::new(capabilities, DataClass::InternalOnly),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corporate_pack() -> CapabilityPack {
        CapabilityPack::new(
            "vertical-corporate".to_string(),
            "pack-alpha".to_string(),
            PackVersion::Minor {
                major: 1,
                minor: 0,
                patch: 0,
            },
            vec!["payroll.close".to_string(), "gl.reconcile".to_string()],
        )
        .expect("canonical corporate pack must be accepted")
    }

    #[test]
    fn accepts_canonical_corporate_pack() {
        let pack = corporate_pack();
        assert_eq!(pack.pack_ref(), "pack-alpha");
        assert_eq!(pack.version().triplet(), (1, 0, 0));
        assert_eq!(pack.version().display(), "1.0.0");
    }

    #[test]
    fn rejects_non_elected_vertical_id() {
        // Synthetic violation: a non-empty but non-elected slug must be
        // rejected at construction time (M04 election invariant).
        let err = CapabilityPack::new(
            "vertical-healthcare".to_string(),
            "pack-alpha".to_string(),
            PackVersion::Minor {
                major: 1,
                minor: 0,
                patch: 0,
            },
            vec!["claims.adjudicate".to_string()],
        )
        .expect_err("non-elected vertical must be rejected");
        assert_eq!(err, CapabilityPackError::NonElectedVerticalId);
    }

    #[test]
    fn rejects_empty_vertical_id() {
        let err = CapabilityPack::new(
            "".to_string(),
            "pack-alpha".to_string(),
            PackVersion::Patch {
                major: 1,
                minor: 0,
                patch: 1,
            },
            vec!["payroll.close".to_string()],
        )
        .expect_err("empty vertical_id must be rejected");
        assert_eq!(err, CapabilityPackError::EmptyVerticalId);
    }

    #[test]
    fn rejects_invalid_pack_ref() {
        let err = CapabilityPack::new(
            "vertical-corporate".to_string(),
            "pack-kr".to_string(), // missing "pack-" prefix
            PackVersion::Patch {
                major: 1,
                minor: 0,
                patch: 1,
            },
            vec!["payroll.close".to_string()],
        )
        .expect_err("pack_ref without pack- prefix must be rejected");
        assert_eq!(err, CapabilityPackError::InvalidPackRef);
    }

    #[test]
    fn rejects_empty_capabilities() {
        let err = CapabilityPack::new(
            "vertical-corporate".to_string(),
            "pack-alpha".to_string(),
            PackVersion::Minor {
                major: 1,
                minor: 0,
                patch: 0,
            },
            vec![],
        )
        .expect_err("empty capabilities must be rejected");
        assert_eq!(err, CapabilityPackError::EmptyCapabilities);
    }

    #[test]
    fn pack_version_display_and_triplet() {
        let v = PackVersion::Minor {
            major: 2,
            minor: 3,
            patch: 4,
        };
        assert_eq!(v.triplet(), (2, 3, 4));
        assert_eq!(v.display(), "2.3.4");

        let v_patch = PackVersion::Patch {
            major: 1,
            minor: 0,
            patch: 5,
        };
        assert_eq!(v_patch.display(), "1.0.5");

        let v_major = PackVersion::Major {
            major: 3,
            minor: 0,
            patch: 0,
        };
        assert_eq!(v_major.triplet(), (3, 0, 0));
    }
}
