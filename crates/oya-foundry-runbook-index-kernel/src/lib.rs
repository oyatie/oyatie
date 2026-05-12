//! Foundry runbook-index validation kernel.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunbookIndexReport {
    pub indexed_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunbookIndexError {
    EmptyRunbookPath,
    DuplicateRunbookEntry,
    MissingRunbook,
}

pub fn validate_runbook_index_resolves<I, E>(
    indexed_paths: &[I],
    existing_paths: &[E],
) -> Result<RunbookIndexReport, RunbookIndexError>
where
    I: AsRef<str>,
    E: AsRef<str>,
{
    let existing_paths = existing_paths
        .iter()
        .map(|path| path.as_ref())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for path in indexed_paths {
        let path = path.as_ref();
        if path.trim().is_empty() {
            return Err(RunbookIndexError::EmptyRunbookPath);
        }
        if !seen.insert(path) {
            return Err(RunbookIndexError::DuplicateRunbookEntry);
        }
        if !existing_paths.contains(path) {
            return Err(RunbookIndexError::MissingRunbook);
        }
    }
    Ok(RunbookIndexReport {
        indexed_count: indexed_paths.len(),
    })
}
