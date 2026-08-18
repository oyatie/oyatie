//! Git/object-derived facts for the history-only retirement validator.
//!
//! This module is part of the repository's single sanctioned Git boundary. It observes
//! candidate, first-parent, and immutable predecessor objects and emits a controller-owned,
//! untracked facts bundle. It never decides PASS, never creates receipts, and never copies a
//! retired body into the generated face.

use std::path::Path;
use std::sync::atomic::AtomicU64;

#[path = "retirement_types.rs"]
mod retirement_types;
#[path = "retirement_parse.rs"]
mod retirement_parse;
#[path = "retirement_git.rs"]
mod retirement_git;
#[path = "retirement_path.rs"]
mod retirement_path;
#[path = "retirement_validate.rs"]
mod retirement_validate;
#[path = "retirement_validate_control.rs"]
mod retirement_validate_control;
#[path = "retirement_validate_event.rs"]
mod retirement_validate_event;
#[path = "retirement_facts_more.rs"]
mod retirement_facts_more;
#[path = "retirement_facts.rs"]
mod retirement_facts;
#[path = "retirement_materialize.rs"]
mod retirement_materialize;
#[path = "retirement_emit.rs"]
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

#[cfg(unix)]
#[path = "retirement_unix_fs.rs"]
mod retirement_unix_fs;
#[cfg(unix)]
pub(crate) use retirement_unix_fs::{
    atomic_replace_ignored_generated_file, create_temporary_file_with_prefix,
    open_canonical_retirement_facts_parent, open_or_create_directory_at,
};

#[cfg(unix)]
#[path = "retirement_unix.rs"]
mod retirement_unix;
#[cfg(unix)]
pub use retirement_unix::{CanonicalIgnoredGeneratedWriter, CanonicalRetirementFactsWriter};

#[cfg(windows)]
#[path = "retirement_windows.rs"]
mod retirement_windows;
#[cfg(windows)]
pub use retirement_windows::{CanonicalIgnoredGeneratedWriter, CanonicalRetirementFactsWriter};

#[cfg(not(any(unix, windows)))]
#[path = "retirement_stub.rs"]
mod retirement_stub;
#[cfg(not(any(unix, windows)))]
pub use retirement_stub::{CanonicalIgnoredGeneratedWriter, CanonicalRetirementFactsWriter};

#[cfg(all(test, unix))]
#[path = "retirement_unix_tests.rs"]
mod retirement_unix_tests;
#[cfg(all(test, windows))]
#[path = "retirement_windows_tests.rs"]
mod retirement_windows_tests;

/// Atomically write the canonical ignored retirement-facts file.
///
/// Public only for the package-local integration target's filesystem defenses.
/// The path is intentionally not caller-controlled: this seam can write only
/// [`GENERATED_FACTS_PATH`], after rerunning the ignore/untracked boundary.
pub fn write_canonical_retirement_facts(repo_root: &Path, bytes: &[u8]) -> Result<(), String> {
    CanonicalRetirementFactsWriter::open(repo_root)?.write(bytes)
}

/// Write another canonical ignored generated face through the platform writer.
pub fn write_canonical_ignored_generated_file(
    repo_root: &Path,
    relative_path: &Path,
    bytes: &[u8],
) -> Result<(), String> {
    CanonicalIgnoredGeneratedWriter::open(repo_root, relative_path)?.write(bytes)
}

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
