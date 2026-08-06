//! Foundry ADR citation fitness kernel.

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdrCitationDocument {
    pub path: String,           // data_class: INTERNAL_ONLY
    pub contents: String,       // data_class: INTERNAL_ONLY
    pub forensic_allowed: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdrCitationReport {
    pub documents_checked: usize, // data_class: INTERNAL_ONLY
    pub citations_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdrCitationError {
    NoDocuments,
    NoAllowedPackAdrs,
    DisallowedAdrCitation {
        path: String,
        line: usize,
        adr: String,
    },
}

pub fn validate_adr_citations<D, A>(
    documents: D,
    allowed_pack_adrs: A,
) -> Result<AdrCitationReport, AdrCitationError>
where
    D: IntoIterator<Item = AdrCitationDocument>,
    A: IntoIterator,
    A::Item: AsRef<str>,
{
    let allowed_pack_adrs = allowed_pack_adrs
        .into_iter()
        .map(|adr| adr.as_ref().trim().to_string())
        .filter(|adr| !adr.is_empty())
        .collect::<BTreeSet<_>>();
    if allowed_pack_adrs.is_empty() {
        return Err(AdrCitationError::NoAllowedPackAdrs);
    }

    let mut documents_checked = 0usize;
    let mut citations_checked = 0usize;
    for document in documents {
        documents_checked += 1;
        for (line_index, line) in document.contents.lines().enumerate() {
            for adr in adr_citations(line) {
                citations_checked += 1;
                if !document.forensic_allowed && !allowed_pack_adrs.contains(&adr) {
                    return Err(AdrCitationError::DisallowedAdrCitation {
                        path: document.path,
                        line: line_index + 1,
                        adr,
                    });
                }
            }
        }
    }

    if documents_checked == 0 {
        Err(AdrCitationError::NoDocuments)
    } else {
        Ok(AdrCitationReport {
            documents_checked,
            citations_checked,
        })
    }
}

fn adr_citations(line: &str) -> Vec<String> {
    let bytes = line.as_bytes();
    let mut citations = Vec::new();
    let mut index = 0usize;
    while index + 8 <= bytes.len() {
        if &bytes[index..index + 4] != b"ADR-" {
            index += 1;
            continue;
        }
        if index >= 8 && &bytes[index - 8..index] == b"Bominal-" {
            index += 8;
            continue;
        }
        let digits = &bytes[index + 4..index + 8];
        if digits.iter().all(u8::is_ascii_digit) {
            citations.push(line[index..index + 8].to_string());
            index += 8;
        } else {
            index += 4;
        }
    }
    citations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_existing_new_pack_adr_citations_in_active_docs() {
        assert_eq!(
            validate_adr_citations(
                [doc("docs/README.md", "See ADR-0001 and ADR-0051.", false)],
                ["ADR-0001", "ADR-0051"],
            ),
            Ok(AdrCitationReport {
                documents_checked: 1,
                citations_checked: 2,
            })
        );
    }

    #[test]
    fn rejects_legacy_or_future_adr_numbers_in_active_docs() {
        assert_eq!(
            validate_adr_citations(
                [doc("docs/README.md", "Do not cite ADR-0201 here.", false)],
                ["ADR-0001", "ADR-0051"],
            ),
            Err(AdrCitationError::DisallowedAdrCitation {
                path: "docs/README.md".into(),
                line: 1,
                adr: "ADR-0201".into(),
            })
        );
        assert_eq!(
            validate_adr_citations(
                [doc("docs/README.md", "ADR-0052 lands later.", false)],
                ["ADR-0001", "ADR-0051"],
            ),
            Err(AdrCitationError::DisallowedAdrCitation {
                path: "docs/README.md".into(),
                line: 1,
                adr: "ADR-0052".into(),
            })
        );
    }

    #[test]
    fn allows_legacy_numbers_only_on_forensic_surfaces() {
        assert_eq!(
            validate_adr_citations(
                [doc(
                    "docs/ADR-LEGACY-REGRESSION-MAPPING.md",
                    "Legacy ADR-0201 maps to ADR-0051.",
                    true,
                )],
                ["ADR-0051"],
            ),
            Ok(AdrCitationReport {
                documents_checked: 1,
                citations_checked: 2,
            })
        );
    }

    #[test]
    fn rejects_empty_inputs() {
        assert_eq!(
            validate_adr_citations(
                [doc("docs/README.md", "ADR-0001", false)],
                Vec::<String>::new(),
            ),
            Err(AdrCitationError::NoAllowedPackAdrs)
        );
        assert_eq!(
            validate_adr_citations([], ["ADR-0001"]),
            Err(AdrCitationError::NoDocuments)
        );
    }

    #[test]
    fn skips_bominal_namespace_adr_citations() {
        assert_eq!(
            validate_adr_citations(
                [doc(
                    "docs/adr-archive/ADR-0122-ontology-crate-rename-from-object-graph.md",
                    "related: [ADR-0056, Bominal-ADR-0133]",
                    false,
                )],
                ["ADR-0056"],
            ),
            Ok(AdrCitationReport {
                documents_checked: 1,
                citations_checked: 1,
            })
        );
    }

    #[test]
    fn bominal_prefix_does_not_byte_collide_with_local_adr() {
        assert_eq!(
            validate_adr_citations(
                [doc(
                    "docs/adr-archive/ADR-0122-ontology-crate-rename-from-object-graph.md",
                    "Cited ADR-0056 alongside Bominal-ADR-0056.",
                    false,
                )],
                ["ADR-0056"],
            ),
            Ok(AdrCitationReport {
                documents_checked: 1,
                citations_checked: 1,
            })
        );
    }

    fn doc(path: &str, contents: &str, forensic_allowed: bool) -> AdrCitationDocument {
        AdrCitationDocument {
            path: path.into(),
            contents: contents.into(),
            forensic_allowed,
        }
    }
}
