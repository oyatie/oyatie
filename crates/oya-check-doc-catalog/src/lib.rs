//! Foundry doc-catalog fitness kernel.

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocCatalogRecord {
    pub doc_id: String,                 // data_class: INTERNAL_ONLY
    pub path: String,                   // data_class: INTERNAL_ONLY
    pub owner_team: String,             // data_class: INTERNAL_ONLY
    pub dependent_docs: Vec<String>,    // data_class: INTERNAL_ONLY
    pub validation_check_present: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocCatalogReport {
    pub documents_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocCatalogError {
    NoCatalogRecords,
    EmptyDocId,
    DuplicateDocId {
        doc_id: String,
    },
    EmptyPath {
        doc_id: String,
    },
    EmptyOwnerTeam {
        doc_id: String,
    },
    MissingValidationCheck {
        doc_id: String,
    },
    EmptyDependentDoc {
        doc_id: String,
    },
    UnknownDependentDoc {
        doc_id: String,
        dependent_doc_id: String,
    },
    UnresolvedDependentDocReference {
        doc_id: String,
        dependent_doc: String,
    },
    CatalogPathMissingFile {
        path: String,
    },
    DocumentMissingMachineRow {
        path: String,
    },
    CatalogPathMissingMarkdownRow {
        path: String,
    },
}

pub fn validate_doc_catalog<E, M, D>(
    records: &[DocCatalogRecord],
    existing_doc_paths: E,
    markdown_catalog_paths: M,
    dependency_reference_targets: D,
) -> Result<DocCatalogReport, DocCatalogError>
where
    E: IntoIterator,
    E::Item: AsRef<str>,
    M: IntoIterator,
    M::Item: AsRef<str>,
    D: IntoIterator,
    D::Item: AsRef<str>,
{
    if records.is_empty() {
        return Err(DocCatalogError::NoCatalogRecords);
    }
    let existing_doc_paths = existing_doc_paths
        .into_iter()
        .map(|path| path.as_ref().to_string())
        .collect::<BTreeSet<_>>();
    let markdown_catalog_paths = markdown_catalog_paths
        .into_iter()
        .map(|path| path.as_ref().to_string())
        .collect::<BTreeSet<_>>();
    let dependency_reference_targets = dependency_reference_targets
        .into_iter()
        .map(|path| normalize_dependency_reference(path.as_ref()))
        .collect::<BTreeSet<_>>();
    let mut seen_doc_ids = BTreeSet::new();
    let mut machine_paths = BTreeSet::new();

    for record in records {
        if record.doc_id.trim().is_empty() {
            return Err(DocCatalogError::EmptyDocId);
        }
        if !seen_doc_ids.insert(record.doc_id.clone()) {
            return Err(DocCatalogError::DuplicateDocId {
                doc_id: record.doc_id.clone(),
            });
        }
        if record.path.trim().is_empty() {
            return Err(DocCatalogError::EmptyPath {
                doc_id: record.doc_id.clone(),
            });
        }
        if record.owner_team.trim().is_empty() {
            return Err(DocCatalogError::EmptyOwnerTeam {
                doc_id: record.doc_id.clone(),
            });
        }
        if !record.validation_check_present {
            return Err(DocCatalogError::MissingValidationCheck {
                doc_id: record.doc_id.clone(),
            });
        }
        if !existing_doc_paths.contains(&record.path) {
            return Err(DocCatalogError::CatalogPathMissingFile {
                path: record.path.clone(),
            });
        }
        if !markdown_catalog_paths.contains(&record.path) {
            return Err(DocCatalogError::CatalogPathMissingMarkdownRow {
                path: record.path.clone(),
            });
        }
        machine_paths.insert(record.path.clone());
    }

    for record in records {
        for dependent_doc in &record.dependent_docs {
            let dependent_doc = dependent_doc.trim();
            if dependent_doc.is_empty() {
                return Err(DocCatalogError::EmptyDependentDoc {
                    doc_id: record.doc_id.clone(),
                });
            }
            if dependent_doc.starts_with("doc.") && !seen_doc_ids.contains(dependent_doc) {
                return Err(DocCatalogError::UnknownDependentDoc {
                    doc_id: record.doc_id.clone(),
                    dependent_doc_id: dependent_doc.to_string(),
                });
            }
            if !dependent_doc.starts_with("doc.")
                && !dependency_reference_resolves(dependent_doc, &dependency_reference_targets)
            {
                return Err(DocCatalogError::UnresolvedDependentDocReference {
                    doc_id: record.doc_id.clone(),
                    dependent_doc: dependent_doc.to_string(),
                });
            }
        }
    }

    for path in &existing_doc_paths {
        if !machine_paths.contains(path) {
            return Err(DocCatalogError::DocumentMissingMachineRow { path: path.clone() });
        }
    }

    Ok(DocCatalogReport {
        documents_checked: records.len(),
    })
}

fn dependency_reference_resolves(reference: &str, targets: &BTreeSet<String>) -> bool {
    let reference = normalize_dependency_reference(reference);
    if reference.is_empty() {
        return false;
    }
    if targets.contains(&reference) {
        return true;
    }
    if let Some(adr_id) = reference.strip_prefix("ADR-") {
        return !adr_id.is_empty()
            && targets.iter().any(|target| {
                target
                    .rsplit('/')
                    .next()
                    .is_some_and(|file_name| file_name.starts_with(&format!("ADR-{adr_id}-")))
            });
    }
    if let Some((prefix, suffix)) = reference.split_once('*') {
        return targets
            .iter()
            .any(|target| target.starts_with(prefix) && target.ends_with(suffix));
    }
    false
}

fn normalize_dependency_reference(reference: &str) -> String {
    reference
        .trim()
        .split_once(" (")
        .map(|(path, _)| path)
        .unwrap_or_else(|| reference.trim())
        .trim()
        .trim_start_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_root_doc_missing_machine_catalog_row() {
        assert_eq!(
            validate_doc_catalog(
                &[record("doc.readme", "docs/README.md")],
                ["docs/README.md", "docs/CONSTITUTION.md"],
                ["docs/README.md"],
                std::iter::empty::<&str>()
            ),
            Err(DocCatalogError::DocumentMissingMachineRow {
                path: "docs/CONSTITUTION.md".into()
            })
        );
    }

    #[test]
    fn rejects_machine_row_missing_markdown_catalog_row() {
        assert_eq!(
            validate_doc_catalog(
                &[record("doc.readme", "docs/README.md")],
                ["docs/README.md"],
                std::iter::empty::<&str>(),
                std::iter::empty::<&str>()
            ),
            Err(DocCatalogError::CatalogPathMissingMarkdownRow {
                path: "docs/README.md".into()
            })
        );
    }

    #[test]
    fn rejects_machine_row_pointing_at_missing_file() {
        assert_eq!(
            validate_doc_catalog(
                &[record("doc.readme", "docs/README.md")],
                std::iter::empty::<&str>(),
                ["docs/README.md"],
                std::iter::empty::<&str>()
            ),
            Err(DocCatalogError::CatalogPathMissingFile {
                path: "docs/README.md".into()
            })
        );
    }

    #[test]
    fn rejects_unknown_doc_id_dependency() {
        assert_eq!(
            validate_doc_catalog(
                &[record_with_deps(
                    "doc.readme",
                    "docs/README.md",
                    ["doc.missing"],
                )],
                ["docs/README.md"],
                ["docs/README.md"],
                std::iter::empty::<&str>()
            ),
            Err(DocCatalogError::UnknownDependentDoc {
                doc_id: "doc.readme".into(),
                dependent_doc_id: "doc.missing".into()
            })
        );
    }

    #[test]
    fn rejects_empty_dependency_value() {
        assert_eq!(
            validate_doc_catalog(
                &[record_with_deps("doc.readme", "docs/README.md", [" "])],
                ["docs/README.md"],
                ["docs/README.md"],
                std::iter::empty::<&str>()
            ),
            Err(DocCatalogError::EmptyDependentDoc {
                doc_id: "doc.readme".into()
            })
        );
    }

    #[test]
    fn rejects_unresolved_dependent_path_or_glob() {
        assert_eq!(
            validate_doc_catalog(
                &[record_with_deps(
                    "doc.readme",
                    "docs/README.md",
                    ["products/*/PRD.md"],
                )],
                ["docs/README.md"],
                ["docs/README.md"],
                ["machine-readable/catalog.json"]
            ),
            Err(DocCatalogError::UnresolvedDependentDocReference {
                doc_id: "doc.readme".into(),
                dependent_doc: "products/*/PRD.md".into()
            })
        );
    }

    #[test]
    fn accepts_resolved_paths_globs_parenthetical_paths_and_adr_ids() {
        assert_eq!(
            validate_doc_catalog(
                &[record_with_deps(
                    "doc.readme",
                    "docs/README.md",
                    [
                        "products/*/PRD.md",
                        "machine-readable/catalog.json (this file)",
                        ".github/CODEOWNERS",
                        "ADR-0050",
                    ],
                )],
                ["docs/README.md"],
                ["docs/README.md"],
                [
                    "products/foundry/PRD.md",
                    "machine-readable/catalog.json",
                    ".github/CODEOWNERS",
                    "decisions/ADR-0050-automation-first-pipeline.md",
                ]
            ),
            Ok(DocCatalogReport {
                documents_checked: 1
            })
        );
    }

    #[test]
    fn accepts_existing_docs_with_machine_and_markdown_rows() {
        assert_eq!(
            validate_doc_catalog(
                &[
                    record_with_deps("doc.readme", "docs/README.md", ["doc.constitution"]),
                    record("doc.constitution", "docs/CONSTITUTION.md"),
                ],
                ["docs/README.md", "docs/CONSTITUTION.md"],
                ["docs/README.md", "docs/CONSTITUTION.md"],
                std::iter::empty::<&str>()
            ),
            Ok(DocCatalogReport {
                documents_checked: 2
            })
        );
    }

    fn record(doc_id: &str, path: &str) -> DocCatalogRecord {
        record_with_deps(doc_id, path, std::iter::empty::<&str>())
    }

    fn record_with_deps<I>(doc_id: &str, path: &str, dependent_docs: I) -> DocCatalogRecord
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        DocCatalogRecord {
            doc_id: doc_id.into(),
            path: path.into(),
            owner_team: "council-architecture".into(),
            dependent_docs: dependent_docs
                .into_iter()
                .map(|dependent_doc| dependent_doc.as_ref().to_string())
                .collect(),
            validation_check_present: true,
        }
    }
}
