//! Fail-closed writers for non-Unix, non-Windows targets.

use std::path::Path;

/// Non-Unix, non-Windows placeholder that preserves the public API while failing closed.
pub struct CanonicalRetirementFactsWriter;

impl CanonicalRetirementFactsWriter {
    /// The descriptor-relative writer is unavailable on this platform.
    pub fn open(_repo_root: &Path) -> Result<Self, String> {
        Err("canonical retirement facts writer requires Unix dirfd support".to_owned())
    }

    /// The descriptor-relative writer is unavailable on this platform.
    pub fn write(&self, _bytes: &[u8]) -> Result<(), String> {
        Err("canonical retirement facts writer requires Unix dirfd support".to_owned())
    }
}

/// Non-Unix, non-Windows placeholder that preserves the public API while failing closed.
///
/// Integration targets import this type on all platforms; Unix-only tests that
/// exercise dirfd semantics stay behind `#[cfg(unix)]`. Without this stub,
/// Windows soft-smoke fails at compile time with `unresolved import`.
#[derive(Debug)]
pub struct CanonicalIgnoredGeneratedWriter;

impl CanonicalIgnoredGeneratedWriter {
    /// The descriptor-relative writer is unavailable on this platform.
    pub fn open(_repo_root: &Path, _relative_path: &Path) -> Result<Self, String> {
        Err("canonical ignored generated writer requires Unix dirfd support".to_owned())
    }

    /// The descriptor-relative writer is unavailable on this platform.
    pub fn write(&self, _bytes: &[u8]) -> Result<(), String> {
        Err("canonical ignored generated writer requires Unix dirfd support".to_owned())
    }
}
