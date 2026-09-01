//! Unagreed pipeline-local repository read port.

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepositoryEntryKind {
    Tree,
    Blob,
    ExecutableBlob,
    Symlink,
    Gitlink,
}

pub trait RepositoryRead {
    fn working_tree_root(&self) -> Result<PathBuf, String>;

    fn repository_identity(&self) -> Result<String, String>;

    fn resolve_commit(&self, revision: &str) -> Result<String, String>;

    fn tree_id(&self, commit: &str) -> Result<String, String>;

    fn merge_base(&self, left: &str, right: &str) -> Result<String, String>;

    fn changed_name_status(&self, base: &str, head: &str) -> Result<Vec<u8>, String>;

    fn blob_text(&self, commit: &str, path: &str) -> Result<String, String>;

    fn blob_bytes(&self, commit: &str, path: &str) -> Result<Vec<u8>, String>;

    fn files_under(&self, commit: &str, path: &str) -> Result<Vec<String>, String>;

    fn path_exists(&self, commit: &str, path: &str) -> Result<bool, String>;

    fn directory_exists(&self, commit: &str, path: &str) -> Result<bool, String>;

    fn entry_kind(&self, commit: &str, path: &str) -> Result<Option<RepositoryEntryKind>, String>;
}
