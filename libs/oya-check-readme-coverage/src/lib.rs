//! Foundry README doc coverage fitness kernel.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadmeDocCoverageReport {
    pub documents_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadmeDocCoverageError {
    NoRootDocuments,
    DocumentMissingCatalogRow { path: String },
    CatalogPathMissingRootDocument { path: String },
    DocumentMissingReadmeLink { path: String },
    ReadmeLinkMissingRootDocument { path: String },
}

pub fn validate_readme_doc_coverage<R, C, L>(
    root_doc_paths: R,
    catalog_doc_paths: C,
    readme_linked_doc_paths: L,
) -> Result<ReadmeDocCoverageReport, ReadmeDocCoverageError>
where
    R: IntoIterator,
    R::Item: AsRef<str>,
    C: IntoIterator,
    C::Item: AsRef<str>,
    L: IntoIterator,
    L::Item: AsRef<str>,
{
    let root_doc_paths = root_doc_paths
        .into_iter()
        .map(|path| path.as_ref().to_string())
        .collect::<BTreeSet<_>>();
    if root_doc_paths.is_empty() {
        return Err(ReadmeDocCoverageError::NoRootDocuments);
    }
    let catalog_doc_paths = catalog_doc_paths
        .into_iter()
        .map(|path| path.as_ref().to_string())
        .collect::<BTreeSet<_>>();
    let readme_linked_doc_paths = readme_linked_doc_paths
        .into_iter()
        .map(|path| path.as_ref().to_string())
        .collect::<BTreeSet<_>>();

    for path in &root_doc_paths {
        if !catalog_doc_paths.contains(path) {
            return Err(ReadmeDocCoverageError::DocumentMissingCatalogRow { path: path.clone() });
        }
        if !readme_linked_doc_paths.contains(path) {
            return Err(ReadmeDocCoverageError::DocumentMissingReadmeLink { path: path.clone() });
        }
    }

    for path in &catalog_doc_paths {
        if !root_doc_paths.contains(path) {
            return Err(ReadmeDocCoverageError::CatalogPathMissingRootDocument {
                path: path.clone(),
            });
        }
    }

    for path in &readme_linked_doc_paths {
        if is_root_doc_path(path) && !root_doc_paths.contains(path) {
            return Err(ReadmeDocCoverageError::ReadmeLinkMissingRootDocument {
                path: path.clone(),
            });
        }
    }

    Ok(ReadmeDocCoverageReport {
        documents_checked: root_doc_paths.len(),
    })
}

fn is_root_doc_path(path: &str) -> bool {
    path.starts_with("docs/") && path.ends_with(".md") && !path[5..].contains('/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_root_doc_missing_readme_link() {
        assert_eq!(
            validate_readme_doc_coverage(
                ["docs/README.md", "docs/CONSTITUTION.md"],
                ["docs/README.md", "docs/CONSTITUTION.md"],
                ["docs/README.md"],
            ),
            Err(ReadmeDocCoverageError::DocumentMissingReadmeLink {
                path: "docs/CONSTITUTION.md".into()
            })
        );
    }

    #[test]
    fn rejects_root_doc_missing_catalog_row() {
        assert_eq!(
            validate_readme_doc_coverage(
                ["docs/README.md", "docs/CONSTITUTION.md"],
                ["docs/README.md"],
                ["docs/README.md", "docs/CONSTITUTION.md"],
            ),
            Err(ReadmeDocCoverageError::DocumentMissingCatalogRow {
                path: "docs/CONSTITUTION.md".into()
            })
        );
    }

    #[test]
    fn rejects_stale_catalog_path() {
        assert_eq!(
            validate_readme_doc_coverage(
                ["docs/README.md"],
                ["docs/README.md", "docs/MISSING.md"],
                ["docs/README.md"],
            ),
            Err(ReadmeDocCoverageError::CatalogPathMissingRootDocument {
                path: "docs/MISSING.md".into()
            })
        );
    }

    #[test]
    fn rejects_stale_readme_root_link() {
        assert_eq!(
            validate_readme_doc_coverage(
                ["docs/README.md"],
                ["docs/README.md"],
                ["docs/README.md", "docs/MISSING.md"],
            ),
            Err(ReadmeDocCoverageError::ReadmeLinkMissingRootDocument {
                path: "docs/MISSING.md".into()
            })
        );
    }

    #[test]
    fn accepts_root_docs_with_catalog_rows_and_readme_links() {
        assert_eq!(
            validate_readme_doc_coverage(
                ["docs/README.md", "docs/CONSTITUTION.md"],
                ["docs/README.md", "docs/CONSTITUTION.md"],
                [
                    "docs/README.md",
                    "docs/CONSTITUTION.md",
                    "docs/standards/doc-style.md",
                ],
            ),
            Ok(ReadmeDocCoverageReport {
                documents_checked: 2
            })
        );
    }
}
