//! Unix dirfd writers for canonical ignored generated faces.
//!
//! `openat` / `NOFOLLOW` / `renameat` / dir fsync. Implementation bodies are
//! the frozen Unix path extracted from retirement.rs.

use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::{Mode, OFlags};

use super::{
    GENERATED_FACTS_PATH, canonical_generated_facts_output_path,
    canonical_ignored_generated_path,
};

#[path = "retirement_unix_fs.rs"]
mod fs;

/// A Unix capability bound to the canonical retirement-facts parent directory.
///
/// It owns the opened directory descriptor, so its finalization remains bound
/// to that directory even if a pathname ancestor is replaced after [`Self::open`].
pub struct CanonicalRetirementFactsWriter {
    directory: OwnedFd,
}

impl CanonicalRetirementFactsWriter {
    /// Open the fixed canonical retirement-facts parent without following symlinks.
    pub fn open(repo_root: &Path) -> Result<Self, String> {
        canonical_generated_facts_output_path(repo_root, Path::new(GENERATED_FACTS_PATH))?;
        Ok(Self {
            directory: fs::open_canonical_retirement_facts_parent(repo_root)?,
        })
    }

    /// Atomically replace only the fixed canonical facts basename through this directory fd.
    pub fn write(&self, bytes: &[u8]) -> Result<(), String> {
        const FINAL_NAME: &str = "history-only-retirement-facts.generated.json";
        fs::atomic_replace_ignored_generated_file(
            &self.directory,
            FINAL_NAME,
            ".retirement-facts",
            bytes,
        )
    }
}

/// Atomically write the canonical ignored retirement-facts file.
///
/// Public only for the package-local integration target's filesystem defenses.
/// The path is intentionally not caller-controlled: this seam can write only
/// [`GENERATED_FACTS_PATH`], after rerunning the ignore/untracked boundary.
pub fn write_canonical_retirement_facts(repo_root: &Path, bytes: &[u8]) -> Result<(), String> {
    CanonicalRetirementFactsWriter::open(repo_root)?.write(bytes)
}

/// Descriptor-relative, no-follow atomic write for another canonical ignored generated face.
///
/// The supplied path must be a normal, repo-relative path below `repo_root`, must be ignored and
/// untracked, and is opened component-by-component without following links. This is deliberately
/// available only to sibling controller-owned writers, not arbitrary callers.
pub struct CanonicalIgnoredGeneratedWriter {
    directory: OwnedFd,
    final_name: String,
}

impl CanonicalIgnoredGeneratedWriter {
    /// Opens the fixed canonical output directory without following any path component links.
    pub fn open(repo_root: &Path, relative_path: &Path) -> Result<Self, String> {
        let (parent_components, final_name) =
            canonical_ignored_generated_path(repo_root, relative_path)?;
        let mut directory = rustix::fs::openat(
            rustix::fs::CWD,
            repo_root,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| format!("open ignored generated repository directory: {error}"))?;
        for component in parent_components {
            directory = fs::open_or_create_directory_at(&directory, component)?;
        }
        Ok(Self {
            directory,
            final_name: final_name.to_owned(),
        })
    }

    /// Atomically replaces the fixed canonical basename through the already-open directory fd.
    pub fn write(&self, bytes: &[u8]) -> Result<(), String> {
        fs::atomic_replace_ignored_generated_file(
            &self.directory,
            &self.final_name,
            ".ignored-generated",
            bytes,
        )
    }
}

pub fn write_canonical_ignored_generated_file(
    repo_root: &Path,
    relative_path: &Path,
    bytes: &[u8],
) -> Result<(), String> {
    CanonicalIgnoredGeneratedWriter::open(repo_root, relative_path)?.write(bytes)
}

