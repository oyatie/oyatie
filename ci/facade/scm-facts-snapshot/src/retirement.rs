//! Git/object-derived facts for the history-only retirement validator.
//!
//! This module is part of the repository's single sanctioned Git boundary. It observes
//! candidate, first-parent, and immutable predecessor objects and emits a controller-owned,
//! untracked facts bundle. It never decides PASS, never creates receipts, and never copies a
//! retired body into the generated face.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use rustix::fd::OwnedFd;
#[cfg(unix)]
use rustix::fs::{AtFlags, FileType, Mode, OFlags};

mod retirement_types;
mod retirement_parse;
mod retirement_git;
mod retirement_path;
mod retirement_validate;
mod retirement_validate_control;
mod retirement_validate_event;
mod retirement_facts_more;
mod retirement_facts;
mod retirement_materialize;
mod retirement_emit;

pub(crate) use retirement_types::*;
pub(crate) use retirement_parse::*;
pub(crate) use retirement_git::*;
pub(crate) use retirement_path::*;
pub(crate) use retirement_validate::*;
pub(crate) use retirement_validate_control::*;
pub(crate) use retirement_validate_event::*;
pub(crate) use retirement_facts_more::*;
pub(crate) use retirement_facts::*;
pub(crate) use retirement_materialize::*;
pub(crate) use retirement_emit::*;

pub use retirement_types::{BlobVisitor, GENERATED_FACTS_PATH};
pub use retirement_git::visit_git_blobs;
pub use retirement_emit::{
    RetirementMaterializationContext, emit_history_only_retirement_facts,
    historical_dev_push_context,
};
pub(super) use retirement_emit::census_revision_from_event;

pub(crate) static NEXT_ATOMIC_WRITE_ID: AtomicU64 = AtomicU64::new(0);

/// A Unix capability bound to the canonical retirement-facts parent directory.
///
/// It owns the opened directory descriptor, so its finalization remains bound
/// to that directory even if a pathname ancestor is replaced after [`Self::open`].
#[cfg(unix)]
pub struct CanonicalRetirementFactsWriter {
    directory: OwnedFd,
}

#[cfg(unix)]
impl CanonicalRetirementFactsWriter {
    /// Open the fixed canonical retirement-facts parent without following symlinks.
    pub fn open(repo_root: &Path) -> Result<Self, String> {
        canonical_generated_facts_output_path(repo_root, Path::new(GENERATED_FACTS_PATH))?;
        Ok(Self {
            directory: open_canonical_retirement_facts_parent(repo_root)?,
        })
    }

    /// Atomically replace only the fixed canonical facts basename through this directory fd.
    pub fn write(&self, bytes: &[u8]) -> Result<(), String> {
        const FINAL_NAME: &str = "history-only-retirement-facts.generated.json";
        atomic_replace_ignored_generated_file(
            &self.directory,
            FINAL_NAME,
            ".retirement-facts",
            bytes,
        )
    }
}

/// Windows same-directory writers. Not `renameat`-atomic and not dirfd / TOCTOU-closed.
#[cfg(windows)]
mod retirement_windows;
#[cfg(windows)]
#[doc(inline)]
pub use retirement_windows::{CanonicalIgnoredGeneratedWriter, CanonicalRetirementFactsWriter};

#[cfg(all(test, windows))]
mod retirement_windows_tests;

#[cfg(not(any(unix, windows)))]
mod retirement_stub;
#[cfg(not(any(unix, windows)))]
pub use retirement_stub::{CanonicalIgnoredGeneratedWriter, CanonicalRetirementFactsWriter};

/// Atomically write the canonical ignored retirement-facts file.
///
/// Public only for the package-local integration target's filesystem defenses.
/// The path is intentionally not caller-controlled: this seam can write only
/// [`GENERATED_FACTS_PATH`], after rerunning the ignore/untracked boundary.
pub fn write_canonical_retirement_facts(repo_root: &Path, bytes: &[u8]) -> Result<(), String> {
    CanonicalRetirementFactsWriter::open(repo_root)?.write(bytes)
}

#[cfg(unix)]
fn open_canonical_retirement_facts_parent(repo_root: &Path) -> Result<OwnedFd, String> {
    let mut directory = rustix::fs::openat(
        rustix::fs::CWD,
        repo_root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| format!("open retirement facts repository directory: {error}"))?;
    for component in ["ci", "facade", "scm-facts-snapshot"] {
        directory = open_or_create_directory_at(&directory, component)?;
    }
    Ok(directory)
}

#[cfg(unix)]
fn open_or_create_directory_at(parent: &OwnedFd, name: &str) -> Result<OwnedFd, String> {
    if name.contains('\0') {
        return Err(format!("retirement facts directory contains NUL: {name:?}"));
    }
    match rustix::fs::openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    ) {
        Ok(directory) => Ok(directory),
        Err(error) if error == rustix::io::Errno::NOENT => {
            match rustix::fs::mkdirat(parent, name, Mode::from_bits_retain(0o755)) {
                Ok(()) | Err(rustix::io::Errno::EXIST) => {}
                Err(error) => {
                    return Err(format!(
                        "create retirement facts directory {name:?}: {error}"
                    ));
                }
            }
            rustix::fs::openat(
                parent,
                name,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|error| format!("open retirement facts directory {name:?}: {error}"))
        }
        Err(error) if error == rustix::io::Errno::NOTDIR || error == rustix::io::Errno::LOOP => {
            Err(format!(
                "retirement facts directory {name:?} is not a real directory"
            ))
        }
        Err(error) => Err(format!("open retirement facts directory {name:?}: {error}")),
    }
}

#[cfg(unix)]
fn ensure_regular_or_absent(directory: &OwnedFd, name: &str) -> Result<(), String> {
    match rustix::fs::statat(directory, name, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(stat) if !FileType::from_raw_mode(stat.st_mode).is_file() => {
            Err("retirement facts output must be a regular file".to_owned())
        }
        Ok(_) | Err(rustix::io::Errno::NOENT) => Ok(()),
        Err(error) => Err(format!("inspect retirement facts output: {error}")),
    }
}

#[cfg(unix)]
fn create_temporary_file_with_prefix(
    directory: &OwnedFd,
    prefix: &str,
) -> Result<(String, OwnedFd), String> {
    for _ in 0..32 {
        let name = format!(
            "{prefix}-{}-{}",
            std::process::id(),
            NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed)
        );
        match rustix::fs::openat(
            directory,
            &name,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::from_bits_retain(0o600),
        ) {
            Ok(file) => return Ok((name, file)),
            Err(rustix::io::Errno::EXIST) => continue,
            Err(error) => {
                return Err(format!(
                    "create temporary file with prefix {prefix:?}: {error}"
                ));
            }
        }
    }
    Err(format!(
        "exhausted temporary file names with prefix {prefix:?}"
    ))
}

/// Descriptor-relative, no-follow atomic write for another canonical ignored generated face.
///
/// The supplied path must be a normal, repo-relative path below `repo_root`, must be ignored and
/// untracked, and is opened component-by-component without following links. This is deliberately
/// available only to sibling controller-owned writers, not arbitrary callers.
#[cfg(unix)]
pub struct CanonicalIgnoredGeneratedWriter {
    directory: OwnedFd,
    final_name: String,
}

#[cfg(unix)]
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
            directory = open_or_create_directory_at(&directory, component)?;
        }
        Ok(Self {
            directory,
            final_name: final_name.to_owned(),
        })
    }

    /// Atomically replaces the fixed canonical basename through the already-open directory fd.
    pub fn write(&self, bytes: &[u8]) -> Result<(), String> {
        atomic_replace_ignored_generated_file(
            &self.directory,
            &self.final_name,
            ".ignored-generated",
            bytes,
        )
    }
}

/// Write another canonical ignored generated face through the platform writer.
pub fn write_canonical_ignored_generated_file(
    repo_root: &Path,
    relative_path: &Path,
    bytes: &[u8],
) -> Result<(), String> {
    CanonicalIgnoredGeneratedWriter::open(repo_root, relative_path)?.write(bytes)
}

#[cfg(unix)]
fn atomic_replace_ignored_generated_file(
    directory: &OwnedFd,
    final_name: &str,
    temporary_prefix: &str,
    bytes: &[u8],
) -> Result<(), String> {
    ensure_regular_or_absent(directory, final_name)?;
    let (temporary_name, temporary) =
        create_temporary_file_with_prefix(directory, temporary_prefix)?;
    let result = (|| {
        write_all(&temporary, bytes)?;
        rustix::fs::fsync(&temporary)
            .map_err(|error| format!("sync ignored generated temporary file: {error}"))?;
        rustix::fs::renameat(directory, &temporary_name, directory, final_name)
            .map_err(|error| format!("replace ignored generated output: {error}"))?;
        rustix::fs::fsync(directory)
            .map_err(|error| format!("sync ignored generated directory: {error}"))
    })();
    if result.is_err() {
        let _ = rustix::fs::unlinkat(directory, &temporary_name, AtFlags::empty());
    }
    result
}

#[cfg(unix)]
fn write_all(file: &OwnedFd, mut bytes: &[u8]) -> Result<(), String> {
    while !bytes.is_empty() {
        let written = rustix::io::write(file, bytes)
            .map_err(|error| format!("write retirement facts temporary file: {error}"))?;
        if written == 0 {
            return Err("write retirement facts temporary file made no progress".to_owned());
        }
        bytes = &bytes[written..];
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod retirement_unix_tests;

#[cfg(test)]
#[path = "retirement_test_fixtures.rs"]
mod test_fixtures;
#[cfg(test)]
#[path = "retirement_test_receipts.rs"]
mod test_receipts;
#[cfg(test)]
#[path = "retirement_tests_public.rs"]
mod tests_public;
#[cfg(test)]
#[path = "retirement_tests_event.rs"]
mod tests_event;
#[cfg(test)]
#[path = "retirement_tests_bootstrap.rs"]
mod tests_bootstrap;
#[cfg(test)]
#[path = "retirement_tests_prepared.rs"]
mod tests_prepared;
#[cfg(test)]
#[path = "retirement_tests_closure.rs"]
mod tests_closure;
#[cfg(test)]
#[path = "retirement_tests_misc.rs"]
mod tests_misc;
#[cfg(test)]
#[path = "retirement_tests_parser.rs"]
mod tests_parser;
