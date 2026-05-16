//! Archive-orphan fitness kernel.
//!
//! The kernel is I/O-free. Runners parse the inventory ledger, check filesystem
//! presence, collect references from living files, and pass typed records into
//! [`check`].
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub const DEFAULT_ARCHIVE_ROOT: &str =
    "bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchivedPath {
    pub original_path: String, // data_class: INTERNAL_ONLY
    pub archive_path: String,  // data_class: INTERNAL_ONLY
    pub original_exists: bool, // data_class: INTERNAL_ONLY
    pub archive_exists: bool,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InboundRef {
    pub source_path: String, // data_class: INTERNAL_ONLY
    pub target_path: String, // data_class: INTERNAL_ONLY
    pub line: u32,           // data_class: INTERNAL_ONLY
    pub context: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveOrphanFitnessReport {
    pub archives_checked: usize,      // data_class: INTERNAL_ONLY
    pub archive_files_present: usize, // data_class: INTERNAL_ONLY
    pub originals_absent: usize,      // data_class: INTERNAL_ONLY
    pub inbound_refs_checked: usize,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArchiveOrphanFitnessError {
    EmptyArchiveSet,
    ArchiveOutsideRoot {
        archive_path: String,
        expected_root: String,
    },
    ArchiveMissing {
        original_path: String,
        archive_path: String,
    },
    OriginalStillActive {
        original_path: String,
        archive_path: String,
    },
    LiveRefToArchived {
        source_path: String,
        target_path: String,
        line: u32,
    },
}

impl ArchiveOrphanFitnessError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyArchiveSet => "archive ledger contains no ARCHIVE rows".into(),
            Self::ArchiveOutsideRoot {
                archive_path,
                expected_root,
            } => format!(
                "archive path '{archive_path}' is outside expected archive root '{expected_root}'"
            ),
            Self::ArchiveMissing {
                original_path,
                archive_path,
            } => format!("ARCHIVE row '{original_path}' is missing archived copy '{archive_path}'"),
            Self::OriginalStillActive {
                original_path,
                archive_path,
            } => format!(
                "ARCHIVE row '{original_path}' still exists in active path after archived copy '{archive_path}'"
            ),
            Self::LiveRefToArchived {
                source_path,
                target_path,
                line,
            } => format!(
                "{source_path}:{line} has living reference to archived path '{target_path}'"
            ),
        }
    }
}

pub fn check(
    archived: &[ArchivedPath],
    inbound_refs: &[InboundRef],
) -> Result<ArchiveOrphanFitnessReport, ArchiveOrphanFitnessError> {
    if archived.is_empty() {
        return Err(ArchiveOrphanFitnessError::EmptyArchiveSet);
    }

    let mut archive_files_present = 0usize;
    let mut originals_absent = 0usize;

    for path in archived {
        if !is_under(&path.archive_path, DEFAULT_ARCHIVE_ROOT) {
            return Err(ArchiveOrphanFitnessError::ArchiveOutsideRoot {
                archive_path: path.archive_path.clone(),
                expected_root: DEFAULT_ARCHIVE_ROOT.into(),
            });
        }
        if !path.archive_exists {
            return Err(ArchiveOrphanFitnessError::ArchiveMissing {
                original_path: path.original_path.clone(),
                archive_path: path.archive_path.clone(),
            });
        }
        archive_files_present += 1;

        if path.original_exists {
            return Err(ArchiveOrphanFitnessError::OriginalStillActive {
                original_path: path.original_path.clone(),
                archive_path: path.archive_path.clone(),
            });
        }
        originals_absent += 1;
    }

    if let Some(inbound) = inbound_refs.first() {
        return Err(ArchiveOrphanFitnessError::LiveRefToArchived {
            source_path: inbound.source_path.clone(),
            target_path: inbound.target_path.clone(),
            line: inbound.line,
        });
    }

    Ok(ArchiveOrphanFitnessReport {
        archives_checked: archived.len(),
        archive_files_present,
        originals_absent,
        inbound_refs_checked: inbound_refs.len(),
    })
}

fn is_under(path: &str, root: &str) -> bool {
    path == root
        || path
            .strip_prefix(root)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_moved_archives_without_living_refs() {
        let report = check(&[archived("ledger.jsonl", false, true)], &[]).expect("valid archive");

        assert_eq!(report.archives_checked, 1);
        assert_eq!(report.archive_files_present, 1);
        assert_eq!(report.originals_absent, 1);
        assert_eq!(report.inbound_refs_checked, 0);
    }

    #[test]
    fn rejects_empty_archive_set() {
        assert_eq!(
            check(&[], &[]),
            Err(ArchiveOrphanFitnessError::EmptyArchiveSet)
        );
    }

    #[test]
    fn rejects_archive_outside_root() {
        let mut path = archived("ledger.jsonl", false, true);
        path.archive_path = wrong_archive("ledger.jsonl");

        assert_eq!(
            check(&[path], &[]),
            Err(ArchiveOrphanFitnessError::ArchiveOutsideRoot {
                archive_path: wrong_archive("ledger.jsonl"),
                expected_root: DEFAULT_ARCHIVE_ROOT.into(),
            })
        );
    }

    #[test]
    fn rejects_missing_archive_copy() {
        let path = archived("goals.json", false, false);

        assert_eq!(
            check(&[path], &[]),
            Err(ArchiveOrphanFitnessError::ArchiveMissing {
                original_path: original("goals.json"),
                archive_path: format!("{DEFAULT_ARCHIVE_ROOT}/goals.json"),
            })
        );
    }

    #[test]
    fn rejects_original_still_active() {
        let path = archived("PAUSE.md", true, true);

        assert_eq!(
            check(&[path], &[]),
            Err(ArchiveOrphanFitnessError::OriginalStillActive {
                original_path: original("PAUSE.md"),
                archive_path: format!("{DEFAULT_ARCHIVE_ROOT}/PAUSE.md"),
            })
        );
    }

    #[test]
    fn rejects_living_ref_to_archived_path() {
        let inbound = InboundRef {
            source_path: "docs/runbooks/live.md".into(),
            target_path: original("ledger.jsonl"),
            line: 7,
            context: "uses old ledger".into(),
        };

        assert_eq!(
            check(&[archived("ledger.jsonl", false, true)], &[inbound]),
            Err(ArchiveOrphanFitnessError::LiveRefToArchived {
                source_path: "docs/runbooks/live.md".into(),
                target_path: original("ledger.jsonl"),
                line: 7,
            })
        );
    }

    fn archived(name: &str, original_exists: bool, archive_exists: bool) -> ArchivedPath {
        ArchivedPath {
            original_path: original(name),
            archive_path: format!("{DEFAULT_ARCHIVE_ROOT}/{name}"),
            original_exists,
            archive_exists,
        }
    }

    fn original(name: &str) -> String {
        ["bominal", "agents", "ultragoal", name].join("/")
    }

    fn wrong_archive(name: &str) -> String {
        ["bominal", "agents", "ultragoal", "archive", "wrong", name].join("/")
    }
}
