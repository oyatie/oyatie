//! Windows same-directory writers for canonical ignored generated faces.
//!
//! Exclusive temp + `write_all` + `sync_all`, then same-directory best-effort
//! replace (`remove_file` if present, then `rename`). Not `renameat`-atomic and
//! not dirfd / TOCTOU-closed.

use std::path::{Path, PathBuf};

use super::{
    GENERATED_FACTS_PATH, canonical_generated_facts_output_path, canonical_ignored_generated_path,
};

#[path = "retirement_windows_fs.rs"]
mod fs;

/// Windows same-directory writer for the canonical retirement-facts face.
pub struct CanonicalRetirementFactsWriter {
    directory: PathBuf,
}

impl CanonicalRetirementFactsWriter {
    /// Re-run the canonical boundary, then walk/create a real parent.
    pub fn open(repo_root: &Path) -> Result<Self, String> {
        canonical_generated_facts_output_path(repo_root, Path::new(GENERATED_FACTS_PATH))?;
        Ok(Self {
            directory: fs::open_real_windows_parent(
                repo_root,
                &["ci", "facade", "scm-facts-snapshot"],
                "retirement facts",
            )?,
        })
    }

    /// Best-effort replace of the fixed canonical facts basename.
    pub fn write(&self, bytes: &[u8]) -> Result<(), String> {
        const FINAL_NAME: &str = "history-only-retirement-facts.generated.json";
        fs::replace_regular_file_best_effort(
            &self.directory,
            FINAL_NAME,
            ".retirement-facts",
            bytes,
        )
    }
}

/// Windows same-directory writer for another canonical ignored generated face.
#[derive(Debug)]
pub struct CanonicalIgnoredGeneratedWriter {
    directory: PathBuf,
    final_name: String,
}

impl CanonicalIgnoredGeneratedWriter {
    /// Re-run the ignored/untracked boundary, then walk/create a real parent.
    pub fn open(repo_root: &Path, relative_path: &Path) -> Result<Self, String> {
        let (parent_components, final_name) =
            canonical_ignored_generated_path(repo_root, relative_path)?;
        Ok(Self {
            directory: fs::open_real_windows_parent(
                repo_root,
                &parent_components,
                "ignored generated",
            )?,
            final_name: final_name.to_owned(),
        })
    }

    /// Best-effort replace of the fixed canonical basename.
    pub fn write(&self, bytes: &[u8]) -> Result<(), String> {
        fs::replace_regular_file_best_effort(
            &self.directory,
            &self.final_name,
            ".ignored-generated",
            bytes,
        )
    }
}
