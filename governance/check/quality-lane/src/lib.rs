//! Foundry quality-lane catalog fitness kernel.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum QualityLaneStage {
    Foundation,
    PerPr,
    Nightly,
    PerRelease,
}

impl QualityLaneStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Foundation => "foundation",
            Self::PerPr => "per-pr",
            Self::Nightly => "nightly",
            Self::PerRelease => "per-release",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "foundation" => Some(Self::Foundation),
            "per-pr" => Some(Self::PerPr),
            "nightly" => Some(Self::Nightly),
            "per-release" => Some(Self::PerRelease),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum QualityLaneStatus {
    Active,
    Planned,
}

impl QualityLaneStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Planned => "planned",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "planned" => Some(Self::Planned),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityLaneRecord {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub stage: QualityLaneStage,       // data_class: INTERNAL_ONLY
    pub status: QualityLaneStatus,     // data_class: INTERNAL_ONLY
    pub owner_team: String,            // data_class: INTERNAL_ONLY
    pub purpose: String,               // data_class: INTERNAL_ONLY
    pub source: String,                // data_class: INTERNAL_ONLY
    pub runtime_budget_seconds: u64,   // data_class: INTERNAL_ONLY
    pub check_command: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityLaneDocRow {
    pub id: String,              // data_class: INTERNAL_ONLY
    pub stage: QualityLaneStage, // data_class: INTERNAL_ONLY
    pub purpose: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityLaneReport {
    pub registry_records: usize,        // data_class: INTERNAL_ONLY
    pub markdown_rows: usize,           // data_class: INTERNAL_ONLY
    pub active_commands_checked: usize, // data_class: INTERNAL_ONLY
    pub owner_teams_checked: usize,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QualityLaneError {
    NoRegistryRecords,
    DuplicateRegistryLane {
        id: String,
    },
    DuplicateMarkdownLane {
        id: String,
    },
    InvalidRegistryLane {
        id: String,
        reason: String,
    },
    MissingMarkdownMirror {
        id: String,
    },
    ExtraMarkdownLane {
        id: String,
    },
    StageDrift {
        id: String,
        expected: String,
        actual: String,
    },
    PurposeDrift {
        id: String,
        expected: String,
        actual: String,
    },
    MissingActiveCheckCommand {
        id: String,
    },
    CheckCommandNotWired {
        id: String,
        command: String,
    },
    NoOwnerTeams,
    UnknownOwnerTeam {
        id: String,
        owner_team: String,
    },
}

/// Validate the quality-lane registry against its markdown mirror and
/// the canonical wired-commands catalog.
///
/// `wired_commands` is the substring-tolerant catalog of canonical
/// commands the gate aggregator wires (sourced from
/// `oya-governance-gate-catalog-domain::all_canonical_commands_rendered`).
/// This replaces the legacy `check_script_contents: &str` parameter
/// which read `scripts/check.sh`'s body verbatim — the canonical
/// catalog crate now owns that data per the `.sh-removal` chain IP-C
/// (audit `evidence/audits/shell-python-replacement-audit-2026-05-15.md`).
pub fn validate_quality_lanes<R, D, O>(
    registry_records: R,
    markdown_rows: D,
    known_owner_teams: O,
    wired_commands: &str,
) -> Result<QualityLaneReport, QualityLaneError>
where
    R: IntoIterator<Item = QualityLaneRecord>,
    D: IntoIterator<Item = QualityLaneDocRow>,
    O: IntoIterator,
    O::Item: AsRef<str>,
{
    let registry = registry_map(registry_records)?;
    if registry.is_empty() {
        return Err(QualityLaneError::NoRegistryRecords);
    }
    let markdown = markdown_map(markdown_rows)?;
    let known_owner_teams = owner_team_set(known_owner_teams)?;

    for (id, record) in &registry {
        validate_registry_record(record)?;
        if !known_owner_teams.contains(&record.owner_team) {
            return Err(QualityLaneError::UnknownOwnerTeam {
                id: id.clone(),
                owner_team: record.owner_team.clone(),
            });
        }
        let Some(row) = markdown.get(id) else {
            return Err(QualityLaneError::MissingMarkdownMirror { id: id.clone() });
        };
        if row.stage != record.stage {
            return Err(QualityLaneError::StageDrift {
                id: id.clone(),
                expected: record.stage.as_str().into(),
                actual: row.stage.as_str().into(),
            });
        }
        if normalize_purpose(&row.purpose) != normalize_purpose(&record.purpose) {
            return Err(QualityLaneError::PurposeDrift {
                id: id.clone(),
                expected: normalize_purpose(&record.purpose),
                actual: normalize_purpose(&row.purpose),
            });
        }
    }
    if let Some(id) = markdown.keys().find(|id| !registry.contains_key(*id)) {
        return Err(QualityLaneError::ExtraMarkdownLane { id: id.clone() });
    }

    let mut active_commands_checked = 0;
    for record in registry.values() {
        if record.status != QualityLaneStatus::Active {
            continue;
        }
        let Some(command) = record.check_command.as_ref().map(|command| command.trim()) else {
            return Err(QualityLaneError::MissingActiveCheckCommand {
                id: record.id.clone(),
            });
        };
        if command.is_empty() {
            return Err(QualityLaneError::MissingActiveCheckCommand {
                id: record.id.clone(),
            });
        }
        if !wired_commands.contains(command) {
            return Err(QualityLaneError::CheckCommandNotWired {
                id: record.id.clone(),
                command: command.into(),
            });
        }
        active_commands_checked += 1;
    }

    Ok(QualityLaneReport {
        registry_records: registry.len(),
        markdown_rows: markdown.len(),
        active_commands_checked,
        owner_teams_checked: known_owner_teams.len(),
    })
}

fn owner_team_set<O>(owner_teams: O) -> Result<BTreeSet<String>, QualityLaneError>
where
    O: IntoIterator,
    O::Item: AsRef<str>,
{
    let owners = owner_teams
        .into_iter()
        .map(|owner| owner.as_ref().trim().to_string())
        .filter(|owner| !owner.is_empty())
        .collect::<BTreeSet<_>>();
    if owners.is_empty() {
        Err(QualityLaneError::NoOwnerTeams)
    } else {
        Ok(owners)
    }
}

fn registry_map<R>(records: R) -> Result<BTreeMap<String, QualityLaneRecord>, QualityLaneError>
where
    R: IntoIterator<Item = QualityLaneRecord>,
{
    let mut registry = BTreeMap::new();
    for record in records {
        if registry.contains_key(&record.id) {
            return Err(QualityLaneError::DuplicateRegistryLane { id: record.id });
        }
        registry.insert(record.id.clone(), record);
    }
    Ok(registry)
}

fn markdown_map<D>(rows: D) -> Result<BTreeMap<String, QualityLaneDocRow>, QualityLaneError>
where
    D: IntoIterator<Item = QualityLaneDocRow>,
{
    let mut markdown = BTreeMap::new();
    for row in rows {
        if markdown.insert(row.id.clone(), row.clone()).is_some() {
            return Err(QualityLaneError::DuplicateMarkdownLane { id: row.id });
        }
    }
    Ok(markdown)
}

fn validate_registry_record(record: &QualityLaneRecord) -> Result<(), QualityLaneError> {
    if !valid_lane_id(&record.id) {
        return Err(QualityLaneError::InvalidRegistryLane {
            id: record.id.clone(),
            reason: "lane id must be lowercase alphanumeric plus hyphen".into(),
        });
    }
    for (field, value) in [
        ("owner_team", &record.owner_team),
        ("purpose", &record.purpose),
        ("source", &record.source),
    ] {
        if value.trim().is_empty() || value.contains('\t') {
            return Err(QualityLaneError::InvalidRegistryLane {
                id: record.id.clone(),
                reason: format!("{field} must be non-empty and tab-free"),
            });
        }
    }
    if record.runtime_budget_seconds == 0 {
        return Err(QualityLaneError::InvalidRegistryLane {
            id: record.id.clone(),
            reason: "runtime_budget_seconds must be greater than zero".into(),
        });
    }
    if record.status == QualityLaneStatus::Planned && record.check_command.is_some() {
        return Err(QualityLaneError::InvalidRegistryLane {
            id: record.id.clone(),
            reason: "planned lanes must not claim active check-script wiring".into(),
        });
    }
    Ok(())
}

fn valid_lane_id(id: &str) -> bool {
    !id.is_empty()
        && id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        && id
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_lowercase())
        && id
            .chars()
            .last()
            .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
}

fn normalize_purpose(value: &str) -> String {
    value
        .replace('`', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_matching_registry_doc_and_check_wiring() {
        let report = validate_quality_lanes(
            [record(
                "cargo-fmt",
                QualityLaneStage::PerPr,
                QualityLaneStatus::Active,
                "cargo fmt --all -- --check",
            )],
            [row(
                "cargo-fmt",
                QualityLaneStage::PerPr,
                "cargo fmt --all -- --check",
            )],
            ["axis-foundry"],
            "cargo fmt --all -- --check",
        )
        .expect("quality lanes validate");

        assert_eq!(report.registry_records, 1);
        assert_eq!(report.markdown_rows, 1);
        assert_eq!(report.active_commands_checked, 1);
        assert_eq!(report.owner_teams_checked, 1);
    }

    #[test]
    fn rejects_missing_markdown_mirror() {
        assert_eq!(
            validate_quality_lanes(
                [record(
                    "cargo-fmt",
                    QualityLaneStage::PerPr,
                    QualityLaneStatus::Active,
                    "cargo fmt --all -- --check",
                )],
                [],
                ["axis-foundry"],
                "cargo fmt --all -- --check",
            ),
            Err(QualityLaneError::MissingMarkdownMirror {
                id: "cargo-fmt".into(),
            })
        );
    }

    #[test]
    fn rejects_purpose_drift() {
        assert_eq!(
            validate_quality_lanes(
                [record(
                    "cargo-fmt",
                    QualityLaneStage::PerPr,
                    QualityLaneStatus::Active,
                    "cargo fmt --all -- --check",
                )],
                [row(
                    "cargo-fmt",
                    QualityLaneStage::PerPr,
                    "different purpose"
                )],
                ["axis-foundry"],
                "cargo fmt --all -- --check",
            ),
            Err(QualityLaneError::PurposeDrift {
                id: "cargo-fmt".into(),
                expected: "cargo fmt --all -- --check".into(),
                actual: "different purpose".into(),
            })
        );
    }

    #[test]
    fn rejects_stage_drift() {
        assert_eq!(
            validate_quality_lanes(
                [record(
                    "cargo-fmt",
                    QualityLaneStage::PerPr,
                    QualityLaneStatus::Active,
                    "cargo fmt --all -- --check",
                )],
                [row(
                    "cargo-fmt",
                    QualityLaneStage::Foundation,
                    "cargo fmt --all -- --check"
                )],
                ["axis-foundry"],
                "cargo fmt --all -- --check",
            ),
            Err(QualityLaneError::StageDrift {
                id: "cargo-fmt".into(),
                expected: "per-pr".into(),
                actual: "foundation".into(),
            })
        );
    }

    #[test]
    fn rejects_unwired_active_command() {
        assert_eq!(
            validate_quality_lanes(
                [record(
                    "cargo-fmt",
                    QualityLaneStage::PerPr,
                    QualityLaneStatus::Active,
                    "cargo fmt --all -- --check",
                )],
                [row(
                    "cargo-fmt",
                    QualityLaneStage::PerPr,
                    "cargo fmt --all -- --check"
                )],
                ["axis-foundry"],
                "cargo check",
            ),
            Err(QualityLaneError::CheckCommandNotWired {
                id: "cargo-fmt".into(),
                command: "cargo fmt --all -- --check".into(),
            })
        );
    }

    #[test]
    fn accepts_planned_lane_without_command() {
        let report = validate_quality_lanes(
            [QualityLaneRecord {
                id: "pnpm-test".into(),
                stage: QualityLaneStage::PerPr,
                status: QualityLaneStatus::Planned,
                owner_team: "axis-foundry".into(),
                purpose: "TS unit + integration".into(),
                source: "TOOLCHAIN.md".into(),
                runtime_budget_seconds: 300,
                check_command: None,
            }],
            [row(
                "pnpm-test",
                QualityLaneStage::PerPr,
                "TS unit + integration",
            )],
            ["axis-foundry"],
            "",
        )
        .expect("planned lane accepted");

        assert_eq!(report.active_commands_checked, 0);
    }

    #[test]
    fn rejects_duplicate_and_invalid_records() {
        assert!(matches!(
            validate_quality_lanes(
                [
                    record(
                        "cargo-fmt",
                        QualityLaneStage::PerPr,
                        QualityLaneStatus::Active,
                        "cargo fmt"
                    ),
                    record(
                        "cargo-fmt",
                        QualityLaneStage::PerPr,
                        QualityLaneStatus::Active,
                        "cargo fmt"
                    ),
                ],
                [row(
                    "cargo-fmt",
                    QualityLaneStage::PerPr,
                    "cargo fmt --all -- --check"
                )],
                ["axis-foundry"],
                "cargo fmt",
            ),
            Err(QualityLaneError::DuplicateRegistryLane { .. })
        ));
        assert!(matches!(
            validate_quality_lanes(
                [QualityLaneRecord {
                    id: "CargoFmt".into(),
                    stage: QualityLaneStage::PerPr,
                    status: QualityLaneStatus::Planned,
                    owner_team: "axis-foundry".into(),
                    purpose: "format".into(),
                    source: "TOOLCHAIN.md".into(),
                    runtime_budget_seconds: 300,
                    check_command: None,
                }],
                [row("CargoFmt", QualityLaneStage::PerPr, "format")],
                ["axis-foundry"],
                "",
            ),
            Err(QualityLaneError::InvalidRegistryLane { .. })
        ));
    }

    #[test]
    fn rejects_unknown_owner_and_missing_runtime_budget() {
        assert_eq!(
            validate_quality_lanes(
                [QualityLaneRecord {
                    id: "cargo-fmt".into(),
                    stage: QualityLaneStage::PerPr,
                    status: QualityLaneStatus::Active,
                    owner_team: "unknown-team".into(),
                    purpose: "cargo fmt".into(),
                    source: "TOOLCHAIN.md".into(),
                    runtime_budget_seconds: 300,
                    check_command: Some("cargo fmt".into()),
                }],
                [row("cargo-fmt", QualityLaneStage::PerPr, "cargo fmt")],
                ["axis-foundry"],
                "cargo fmt",
            ),
            Err(QualityLaneError::UnknownOwnerTeam {
                id: "cargo-fmt".into(),
                owner_team: "unknown-team".into(),
            })
        );
        assert!(matches!(
            validate_quality_lanes(
                [QualityLaneRecord {
                    id: "cargo-fmt".into(),
                    stage: QualityLaneStage::PerPr,
                    status: QualityLaneStatus::Active,
                    owner_team: "axis-foundry".into(),
                    purpose: "cargo fmt".into(),
                    source: "TOOLCHAIN.md".into(),
                    runtime_budget_seconds: 0,
                    check_command: Some("cargo fmt".into()),
                }],
                [row("cargo-fmt", QualityLaneStage::PerPr, "cargo fmt")],
                ["axis-foundry"],
                "cargo fmt",
            ),
            Err(QualityLaneError::InvalidRegistryLane { .. })
        ));
    }

    fn record(
        id: &str,
        stage: QualityLaneStage,
        status: QualityLaneStatus,
        command: &str,
    ) -> QualityLaneRecord {
        QualityLaneRecord {
            id: id.into(),
            stage,
            status,
            owner_team: "axis-foundry".into(),
            purpose: command.into(),
            source: "TOOLCHAIN.md".into(),
            runtime_budget_seconds: 300,
            check_command: Some(command.into()),
        }
    }

    fn row(id: &str, stage: QualityLaneStage, purpose: &str) -> QualityLaneDocRow {
        QualityLaneDocRow {
            id: id.into(),
            stage,
            purpose: purpose.into(),
        }
    }
}
