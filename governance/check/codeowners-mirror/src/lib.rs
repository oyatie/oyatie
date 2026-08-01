//! Foundry CODEOWNERS mirror fitness kernel.

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeownersEntry {
    pub line_number: usize,  // data_class: INTERNAL_ONLY
    pub pattern: String,     // data_class: INTERNAL_ONLY
    pub owners: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodeownersMirrorReport {
    pub entries_checked: usize, // data_class: INTERNAL_ONLY
    pub owners_checked: usize,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeownersMirrorError {
    NoCodeownersEntries,
    EmptyPattern { line_number: usize },
    DuplicatePattern { pattern: String },
    MissingOwners { pattern: String },
    UnsupportedOwnerShape { pattern: String, owner: String },
    UnknownTeamOwner { pattern: String, owner: String },
    MissingRequiredPattern { pattern: String },
}

const REQUIRED_PATTERNS: &[&str] = &[
    "*",
    "crates/oya-foundry-*",
    "registry/catalog/",
    "docs/teams/*/CHARTER.md",
    "docs/RACI-OWNERSHIP.md",
];

pub fn validate_codeowners_mirror<T>(
    entries: &[CodeownersEntry],
    team_ids: T,
) -> Result<CodeownersMirrorReport, CodeownersMirrorError>
where
    T: IntoIterator,
    T::Item: AsRef<str>,
{
    if entries.is_empty() {
        return Err(CodeownersMirrorError::NoCodeownersEntries);
    }
    let team_ids = team_ids
        .into_iter()
        .map(|team_id| team_id.as_ref().to_string())
        .collect::<BTreeSet<_>>();
    let mut patterns = BTreeSet::new();
    let mut owners_checked = 0;

    for entry in entries {
        let pattern = entry.pattern.trim();
        if pattern.is_empty() {
            return Err(CodeownersMirrorError::EmptyPattern {
                line_number: entry.line_number,
            });
        }
        if !patterns.insert(pattern.to_string()) {
            return Err(CodeownersMirrorError::DuplicatePattern {
                pattern: pattern.to_string(),
            });
        }
        if entry.owners.is_empty() {
            return Err(CodeownersMirrorError::MissingOwners {
                pattern: pattern.to_string(),
            });
        }
        for owner in &entry.owners {
            let owner = owner.trim();
            owners_checked += 1;
            let Some(team_id) = owner.strip_prefix("@teams/") else {
                return Err(CodeownersMirrorError::UnsupportedOwnerShape {
                    pattern: pattern.to_string(),
                    owner: owner.to_string(),
                });
            };
            if team_id.is_empty() || !team_ids.contains(team_id) {
                return Err(CodeownersMirrorError::UnknownTeamOwner {
                    pattern: pattern.to_string(),
                    owner: owner.to_string(),
                });
            }
        }
    }

    for required_pattern in REQUIRED_PATTERNS {
        if !patterns.contains(*required_pattern) {
            return Err(CodeownersMirrorError::MissingRequiredPattern {
                pattern: (*required_pattern).to_string(),
            });
        }
    }

    Ok(CodeownersMirrorReport {
        entries_checked: entries.len(),
        owners_checked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_team_owner() {
        assert_eq!(
            validate_codeowners_mirror(
                &required_entries_with_owner("@teams/missing-team"),
                ["council-architecture", "axis-foundry"]
            ),
            Err(CodeownersMirrorError::UnknownTeamOwner {
                pattern: "*".into(),
                owner: "@teams/missing-team".into()
            })
        );
    }

    #[test]
    fn rejects_unsupported_non_team_owner_shape() {
        assert_eq!(
            validate_codeowners_mirror(
                &required_entries_with_owner("@octocat"),
                ["council-architecture", "axis-foundry"]
            ),
            Err(CodeownersMirrorError::UnsupportedOwnerShape {
                pattern: "*".into(),
                owner: "@octocat".into()
            })
        );
    }

    #[test]
    fn rejects_missing_required_pattern() {
        assert_eq!(
            validate_codeowners_mirror(
                &[
                    entry(1, "*", ["@teams/council-architecture"]),
                    entry(2, "crates/oya-foundry-*", ["@teams/axis-foundry"]),
                    entry(3, "registry/catalog/", ["@teams/axis-foundry"]),
                    entry(
                        4,
                        "docs/teams/*/CHARTER.md",
                        ["@teams/council-architecture"]
                    ),
                ],
                ["council-architecture", "axis-foundry"]
            ),
            Err(CodeownersMirrorError::MissingRequiredPattern {
                pattern: "docs/RACI-OWNERSHIP.md".into()
            })
        );
    }

    #[test]
    fn accepts_team_owned_required_patterns() {
        assert_eq!(
            validate_codeowners_mirror(
                &required_entries_with_owner("@teams/council-architecture"),
                ["council-architecture", "axis-foundry"]
            ),
            Ok(CodeownersMirrorReport {
                entries_checked: 5,
                owners_checked: 5,
            })
        );
    }

    fn required_entries_with_owner(owner: &str) -> Vec<CodeownersEntry> {
        vec![
            entry(1, "*", [owner]),
            entry(2, "crates/oya-foundry-*", ["@teams/axis-foundry"]),
            entry(3, "registry/catalog/", ["@teams/axis-foundry"]),
            entry(
                4,
                "docs/teams/*/CHARTER.md",
                ["@teams/council-architecture"],
            ),
            entry(5, "docs/RACI-OWNERSHIP.md", ["@teams/council-architecture"]),
        ]
    }

    fn entry<I>(line_number: usize, pattern: &str, owners: I) -> CodeownersEntry
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        CodeownersEntry {
            line_number,
            pattern: pattern.into(),
            owners: owners
                .into_iter()
                .map(|owner| owner.as_ref().to_string())
                .collect(),
        }
    }
}
