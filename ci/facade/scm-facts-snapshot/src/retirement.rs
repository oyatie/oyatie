//! Git/object-derived facts for the history-only retirement validator.
//!
//! This module is part of the repository's single sanctioned Git boundary. It observes
//! candidate, first-parent, and immutable predecessor objects and emits a controller-owned,
//! untracked facts bundle. It never decides PASS, never creates receipts, and never copies a
//! retired body into the generated face.
//!
//! Writer dispatch only: Unix dirfd, Windows same-directory replace, fail-closed elsewhere.

use std::sync::atomic::AtomicU64;

static NEXT_ATOMIC_WRITE_ID: AtomicU64 = AtomicU64::new(0);

#[cfg(unix)]
#[path = "retirement_unix.rs"]
mod unix;
#[cfg(unix)]
pub use unix::{
    CanonicalIgnoredGeneratedWriter, CanonicalRetirementFactsWriter,
    write_canonical_ignored_generated_file, write_canonical_retirement_facts,
};

#[cfg(windows)]
#[path = "retirement_windows.rs"]
mod windows;
#[cfg(windows)]
pub use windows::{CanonicalIgnoredGeneratedWriter, CanonicalRetirementFactsWriter};

#[cfg(all(test, windows))]
#[path = "retirement_windows_tests.rs"]
mod windows_tests;

/// Non-Unix, non-Windows placeholder that preserves the public API while failing closed.
#[cfg(not(any(unix, windows)))]
pub struct CanonicalRetirementFactsWriter;

#[cfg(not(any(unix, windows)))]
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

#[cfg(not(unix))]
pub fn write_canonical_retirement_facts(_repo_root: &Path, _bytes: &[u8]) -> Result<(), String> {
    CanonicalRetirementFactsWriter::open(_repo_root)?.write(_bytes)
}

/// Non-Unix, non-Windows placeholder that preserves the public API while failing closed.
///
/// Integration targets import this type on all platforms; Unix-only tests that
/// exercise dirfd semantics stay behind `#[cfg(unix)]`. Without this stub,
/// Windows soft-smoke fails at compile time with `unresolved import`.
#[cfg(not(any(unix, windows)))]
#[derive(Debug)]
pub struct CanonicalIgnoredGeneratedWriter;

#[cfg(not(any(unix, windows)))]
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

#[cfg(not(unix))]
pub fn write_canonical_ignored_generated_file(
    repo_root: &Path,
    relative_path: &Path,
    bytes: &[u8],
) -> Result<(), String> {
    CanonicalIgnoredGeneratedWriter::open(repo_root, relative_path)?.write(bytes)
}


include!("retirement_facts.rs");
