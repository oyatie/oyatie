//! Foundry placeholder-debt fitness kernel.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

pub const PLACEHOLDER_DEBT_TOKENS: &[&str] = &["TBD", "TODO"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaceholderDocument {
    pub path: String,     // data_class: INTERNAL_ONLY
    pub contents: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaceholderDebtRecord {
    pub token: String,      // data_class: INTERNAL_ONLY
    pub path: String,       // data_class: INTERNAL_ONLY
    pub occurrences: usize, // data_class: INTERNAL_ONLY
    pub excerpt: String,    // data_class: INTERNAL_ONLY
    pub rationale: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaceholderDebtFinding {
    pub token: String,      // data_class: INTERNAL_ONLY
    pub path: String,       // data_class: INTERNAL_ONLY
    pub occurrences: usize, // data_class: INTERNAL_ONLY
    pub excerpt: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaceholderDebtReport {
    pub documents_checked: usize,              // data_class: INTERNAL_ONLY
    pub open_placeholders: usize,              // data_class: INTERNAL_ONLY
    pub tracked_records: usize,                // data_class: INTERNAL_ONLY
    pub findings: Vec<PlaceholderDebtFinding>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlaceholderDebtError {
    NoDocuments,
    DuplicateRecord {
        record_id: String,
    },
    InvalidRecord {
        record_id: String,
        reason: String,
    },
    NewPlaceholderOutsideRegistry {
        record_id: String,
    },
    StalePlaceholderRecord {
        record_id: String,
    },
    PlaceholderCountChanged {
        record_id: String,
        expected: usize,
        actual: usize,
    },
}

pub fn validate_placeholder_debt<D, R>(
    documents: D,
    registry_records: R,
) -> Result<PlaceholderDebtReport, PlaceholderDebtError>
where
    D: IntoIterator<Item = PlaceholderDocument>,
    R: IntoIterator<Item = PlaceholderDebtRecord>,
{
    let registry = registry_map(registry_records)?;
    let (documents_checked, findings) = current_findings(documents)?;
    let current = findings
        .iter()
        .map(|finding| (finding.key(), finding.occurrences))
        .collect::<BTreeMap<_, _>>();

    for (key, actual) in &current {
        match registry.get(key) {
            Some(record) if record.occurrences == *actual => {}
            Some(record) => {
                return Err(PlaceholderDebtError::PlaceholderCountChanged {
                    record_id: key.id(),
                    expected: record.occurrences,
                    actual: *actual,
                });
            }
            None => {
                return Err(PlaceholderDebtError::NewPlaceholderOutsideRegistry {
                    record_id: key.id(),
                });
            }
        }
    }

    if let Some(key) = registry.keys().find(|key| !current.contains_key(*key)) {
        return Err(PlaceholderDebtError::StalePlaceholderRecord {
            record_id: key.id(),
        });
    }

    let open_placeholders = findings.iter().map(|finding| finding.occurrences).sum();
    Ok(PlaceholderDebtReport {
        documents_checked,
        open_placeholders,
        tracked_records: registry.len(),
        findings,
    })
}

pub fn discover_placeholder_debt<D>(
    documents: D,
) -> Result<PlaceholderDebtReport, PlaceholderDebtError>
where
    D: IntoIterator<Item = PlaceholderDocument>,
{
    let (documents_checked, findings) = current_findings(documents)?;
    let open_placeholders = findings.iter().map(|finding| finding.occurrences).sum();
    Ok(PlaceholderDebtReport {
        documents_checked,
        open_placeholders,
        tracked_records: findings.len(),
        findings,
    })
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PlaceholderKey {
    token: String,
    path: String,
    excerpt: String,
}

impl PlaceholderKey {
    fn id(&self) -> String {
        format!("{}\t{}\t{}", self.token, self.path, self.excerpt)
    }
}

impl PlaceholderDebtFinding {
    fn key(&self) -> PlaceholderKey {
        PlaceholderKey {
            token: self.token.clone(),
            path: self.path.clone(),
            excerpt: self.excerpt.clone(),
        }
    }
}

impl PlaceholderDebtRecord {
    fn key(&self) -> PlaceholderKey {
        PlaceholderKey {
            token: self.token.clone(),
            path: self.path.clone(),
            excerpt: self.excerpt.clone(),
        }
    }
}

fn registry_map<R>(
    records: R,
) -> Result<BTreeMap<PlaceholderKey, PlaceholderDebtRecord>, PlaceholderDebtError>
where
    R: IntoIterator<Item = PlaceholderDebtRecord>,
{
    let mut registry = BTreeMap::new();
    for record in records {
        validate_record(&record)?;
        let key = record.key();
        if registry.insert(key.clone(), record).is_some() {
            return Err(PlaceholderDebtError::DuplicateRecord {
                record_id: key.id(),
            });
        }
    }
    Ok(registry)
}

fn validate_record(record: &PlaceholderDebtRecord) -> Result<(), PlaceholderDebtError> {
    let key = record.key();
    let valid_tokens = PLACEHOLDER_DEBT_TOKENS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if !valid_tokens.contains(record.token.as_str()) {
        return Err(PlaceholderDebtError::InvalidRecord {
            record_id: key.id(),
            reason: "token is not an approved placeholder-debt marker".into(),
        });
    }
    if record.occurrences == 0 {
        return Err(PlaceholderDebtError::InvalidRecord {
            record_id: key.id(),
            reason: "occurrences must be greater than zero".into(),
        });
    }
    if !record.path.starts_with("docs/") || record.path.contains("\t") {
        return Err(PlaceholderDebtError::InvalidRecord {
            record_id: key.id(),
            reason: "path must be a docs/ path without tabs".into(),
        });
    }
    if record.excerpt.trim().is_empty() || record.excerpt.contains('\t') {
        return Err(PlaceholderDebtError::InvalidRecord {
            record_id: key.id(),
            reason: "excerpt must be non-empty and tab-free".into(),
        });
    }
    let rationale = record.rationale.trim();
    if rationale.is_empty() || rationale.contains('\t') {
        return Err(PlaceholderDebtError::InvalidRecord {
            record_id: key.id(),
            reason: "rationale must be non-empty and tab-free".into(),
        });
    }
    if rationale.contains("BOOTSTRAP_ONLY")
        || rationale.contains("owner=TBD")
        || rationale.contains("issue=TBD")
        || rationale.contains("existing placeholder debt captured")
    {
        return Err(PlaceholderDebtError::InvalidRecord {
            record_id: key.id(),
            reason: "rationale must be owner/issue-linked and not bootstrap-only".into(),
        });
    }
    for required in ["owner=", "issue=", "captured_at="] {
        if !rationale.contains(required) {
            return Err(PlaceholderDebtError::InvalidRecord {
                record_id: key.id(),
                reason: format!("rationale missing required marker {required}"),
            });
        }
    }
    Ok(())
}

fn current_findings<D>(
    documents: D,
) -> Result<(usize, Vec<PlaceholderDebtFinding>), PlaceholderDebtError>
where
    D: IntoIterator<Item = PlaceholderDocument>,
{
    let mut documents_checked = 0;
    let mut findings = BTreeMap::<PlaceholderKey, usize>::new();
    for document in documents {
        documents_checked += 1;
        for line in markdown_prose_lines(&document.contents) {
            let excerpt = normalize_excerpt(&line);
            if excerpt.is_empty() {
                continue;
            }
            for token in PLACEHOLDER_DEBT_TOKENS {
                let occurrences = count_token_occurrences(&line, token);
                if occurrences == 0 {
                    continue;
                }
                let key = PlaceholderKey {
                    token: (*token).into(),
                    path: document.path.clone(),
                    excerpt: excerpt.clone(),
                };
                *findings.entry(key).or_default() += occurrences;
            }
        }
    }
    if documents_checked == 0 {
        return Err(PlaceholderDebtError::NoDocuments);
    }
    Ok((
        documents_checked,
        findings
            .into_iter()
            .map(|(key, occurrences)| PlaceholderDebtFinding {
                token: key.token,
                path: key.path,
                occurrences,
                excerpt: key.excerpt,
            })
            .collect(),
    ))
}

fn markdown_prose_lines(contents: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_fence = false;
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        lines.push(strip_inline_code(line));
    }
    lines
}

fn strip_inline_code(line: &str) -> String {
    let mut output = String::new();
    let mut in_code = false;
    for character in line.chars() {
        if character == '`' {
            in_code = !in_code;
            output.push(' ');
        } else if in_code {
            output.push(' ');
        } else {
            output.push(character);
        }
    }
    output
}

fn normalize_excerpt(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn count_token_occurrences(line: &str, token: &str) -> usize {
    let mut count = 0;
    let mut search_start = 0;
    while let Some(offset) = line[search_start..].find(token) {
        let start = search_start + offset;
        let end = start + token.len();
        let previous = line[..start].chars().next_back();
        let next = line[end..].chars().next();
        if !is_word_character(previous) && !is_word_character(next) {
            count += 1;
        }
        search_start = end;
    }
    count
}

fn is_word_character(character: Option<char>) -> bool {
    character.is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_registered_placeholder_debt_and_reports_counts() {
        let report = validate_placeholder_debt(
            [doc("docs/ROADMAP.md", "TODO: ship this. TBD by council.")],
            [
                record(
                    "TODO",
                    "docs/ROADMAP.md",
                    1,
                    "TODO: ship this. TBD by council.",
                ),
                record(
                    "TBD",
                    "docs/ROADMAP.md",
                    1,
                    "TODO: ship this. TBD by council.",
                ),
            ],
        )
        .expect("registered debt accepted");

        assert_eq!(report.documents_checked, 1);
        assert_eq!(report.open_placeholders, 2);
        assert_eq!(report.tracked_records, 2);
    }

    #[test]
    fn rejects_new_placeholder_outside_registry() {
        assert_eq!(
            validate_placeholder_debt(
                [doc("docs/ROADMAP.md", "TODO: untracked.")],
                std::iter::empty::<PlaceholderDebtRecord>(),
            ),
            Err(PlaceholderDebtError::NewPlaceholderOutsideRegistry {
                record_id: "TODO\tdocs/ROADMAP.md\tTODO: untracked.".into(),
            })
        );
    }

    #[test]
    fn rejects_stale_placeholder_record() {
        assert_eq!(
            validate_placeholder_debt(
                [doc("docs/ROADMAP.md", "No placeholder debt.")],
                [record("TODO", "docs/ROADMAP.md", 1, "TODO: removed.")],
            ),
            Err(PlaceholderDebtError::StalePlaceholderRecord {
                record_id: "TODO\tdocs/ROADMAP.md\tTODO: removed.".into(),
            })
        );
    }

    #[test]
    fn rejects_placeholder_count_drift() {
        assert_eq!(
            validate_placeholder_debt(
                [doc("docs/ROADMAP.md", "TODO TODO")],
                [record("TODO", "docs/ROADMAP.md", 1, "TODO TODO")],
            ),
            Err(PlaceholderDebtError::PlaceholderCountChanged {
                record_id: "TODO\tdocs/ROADMAP.md\tTODO TODO".into(),
                expected: 1,
                actual: 2,
            })
        );
    }

    #[test]
    fn rejects_bootstrap_or_unowned_placeholder_records() {
        for rationale in [
            "BOOTSTRAP_ONLY owner=TBD issue=TBD captured_at=2026-05-10; replace before merge",
            "existing placeholder debt captured 2026-05-10; burn down later",
            "owner=council-architecture; captured_at=2026-05-10",
        ] {
            assert!(matches!(
                validate_placeholder_debt(
                    [doc("docs/ROADMAP.md", "TODO: tracked.")],
                    [PlaceholderDebtRecord {
                        token: "TODO".into(),
                        path: "docs/ROADMAP.md".into(),
                        occurrences: 1,
                        excerpt: "TODO: tracked.".into(),
                        rationale: rationale.into(),
                    }]
                ),
                Err(PlaceholderDebtError::InvalidRecord { .. })
            ));
        }
    }

    #[test]
    fn rejects_invalid_records() {
        assert!(matches!(
            validate_placeholder_debt(
                [doc("docs/ROADMAP.md", "TODO: tracked.")],
                [PlaceholderDebtRecord {
                    token: "FIXME".into(),
                    path: "docs/ROADMAP.md".into(),
                    occurrences: 1,
                    excerpt: "TODO: tracked.".into(),
                    rationale: "legacy debt".into(),
                }]
            ),
            Err(PlaceholderDebtError::InvalidRecord { .. })
        ));
        assert!(matches!(
            validate_placeholder_debt(
                [doc("docs/ROADMAP.md", "TODO: tracked.")],
                [PlaceholderDebtRecord {
                    token: "TODO".into(),
                    path: "docs/ROADMAP.md".into(),
                    occurrences: 1,
                    excerpt: "TODO: tracked.".into(),
                    rationale: "".into(),
                }]
            ),
            Err(PlaceholderDebtError::InvalidRecord { .. })
        ));
    }

    #[test]
    fn ignores_code_fences_inline_code_and_embedded_words() {
        let report = discover_placeholder_debt([doc(
            "docs/AGENTS.md",
            "`TODO` code. METHODTODO no.\n```\nTODO: generated fixture\n```\nTBD: prose.",
        )])
        .expect("discovery succeeds");

        assert_eq!(report.open_placeholders, 1);
        assert_eq!(report.findings[0].token, "TBD");
    }

    fn doc(path: &str, contents: &str) -> PlaceholderDocument {
        PlaceholderDocument {
            path: path.into(),
            contents: contents.into(),
        }
    }

    fn record(token: &str, path: &str, occurrences: usize, excerpt: &str) -> PlaceholderDebtRecord {
        PlaceholderDebtRecord {
            token: token.into(),
            path: path.into(),
            occurrences,
            excerpt: excerpt.into(),
            rationale: "owner=council-architecture; issue=PLACEHOLDER-DEBT-ROADMAP; captured_at=2026-05-10; action=burn-down".into(),
        }
    }
}
