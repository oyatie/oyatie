use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use check_quality_lane::{
    QualityLaneDocRow, QualityLaneRecord, QualityLaneStage, QualityLaneStatus,
    validate_quality_lanes,
};
use check_gate_catalog_domain::all_canonical_commands_rendered;

use crate::{clean_yaml_value, list_team_ids, usage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QualityLanesValidateArgs {
    registry_path: PathBuf,
    ci_lanes_path: PathBuf,
    /// Optional test-only override for the wired-commands corpus. When
    /// `None` (production default) the kernel sources its wired-commands
    /// catalog from `oya-governance-gate-catalog-domain` per the .sh-removal
    /// chain IP-C. When `Some(path)`, the CLI reads the path verbatim —
    /// used by the integration-test fixtures in `tests/gate_cli.rs` to
    /// exercise rejection paths.
    check_script_path: Option<PathBuf>,
    teams_dir: PathBuf,
}

pub(crate) fn parse_quality_lanes_validate_args(
    args: Vec<String>,
) -> Result<QualityLanesValidateArgs, String> {
    let mut parsed = QualityLanesValidateArgs {
        registry_path: PathBuf::from("registry/quality/lanes.yaml"),
        ci_lanes_path: PathBuf::from("docs/standards/ci-lanes.md"),
        check_script_path: None,
        teams_dir: PathBuf::from("docs/teams"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--registry" => parsed.registry_path = PathBuf::from(path),
            "--ci-lanes" => parsed.ci_lanes_path = PathBuf::from(path),
            "--check-script" => parsed.check_script_path = Some(PathBuf::from(path)),
            "--teams-dir" => parsed.teams_dir = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_quality_lanes_gate(
    args: QualityLanesValidateArgs,
) -> Result<(usize, usize, usize, usize), String> {
    let records = read_quality_lane_registry(&args.registry_path)?;
    let markdown_rows = read_quality_lane_markdown_rows(&args.ci_lanes_path)?;
    let owner_teams = list_team_ids(&args.teams_dir)?;
    // Canonical catalog replaces the legacy `scripts/check.sh` file read
    // (audit `evidence/audits/shell-python-replacement-audit-2026-05-15.md`
    // row B-1, .sh-removal chain IP-C). Test-only override:
    // `--check-script <path>` swaps the canonical catalog for the file
    // body so integration-test fixtures can exercise rejection paths.
    let wired_commands = match args.check_script_path.as_ref() {
        Some(path) => fs::read_to_string(path).map_err(|error| {
            format!(
                "quality lane check script unreadable {}: {error}",
                path.display()
            )
        })?,
        None => all_canonical_commands_rendered(),
    };
    let report = validate_quality_lanes(records, markdown_rows, owner_teams, &wired_commands)
        .map_err(|error| format!("quality lanes invalid: {error:?}"))?;
    Ok((
        report.registry_records,
        report.markdown_rows,
        report.active_commands_checked,
        report.owner_teams_checked,
    ))
}

fn read_quality_lane_registry(path: &Path) -> Result<Vec<QualityLaneRecord>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "quality lane registry unreadable {}: {error}",
            path.display()
        )
    })?;
    let mut records = Vec::new();
    let mut current = BTreeMap::<String, String>::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "lanes:" {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- ") {
            if !current.is_empty() {
                records.push(quality_lane_record_from_map(path, index, &current)?);
                current.clear();
            }
            let Some((key, value)) = rest.split_once(':') else {
                return Err(format!(
                    "quality lane registry {}:{} has invalid record opener",
                    path.display(),
                    index + 1
                ));
            };
            current.insert(key.trim().to_string(), clean_yaml_value(value));
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            return Err(format!(
                "quality lane registry {}:{} must be '<key>: <value>'",
                path.display(),
                index + 1
            ));
        };
        current.insert(key.trim().to_string(), clean_yaml_value(value));
    }
    if !current.is_empty() {
        records.push(quality_lane_record_from_map(
            path,
            contents.lines().count(),
            &current,
        )?);
    }
    Ok(records)
}

fn quality_lane_record_from_map(
    path: &Path,
    index: usize,
    fields: &BTreeMap<String, String>,
) -> Result<QualityLaneRecord, String> {
    let id = required_quality_lane_field(path, index, fields, "id")?;
    let stage =
        QualityLaneStage::parse(&required_quality_lane_field(path, index, fields, "stage")?)
            .ok_or_else(|| {
                format!(
                    "quality lane registry {}:{} has unknown stage",
                    path.display(),
                    index + 1
                )
            })?;
    let status =
        QualityLaneStatus::parse(&required_quality_lane_field(path, index, fields, "status")?)
            .ok_or_else(|| {
                format!(
                    "quality lane registry {}:{} has unknown status",
                    path.display(),
                    index + 1
                )
            })?;
    let check_command = fields
        .get("check_command")
        .map(|command| command.trim().to_string())
        .filter(|command| !command.is_empty());
    let runtime_budget_seconds =
        required_quality_lane_field(path, index, fields, "runtime_budget_seconds")?
            .parse::<u64>()
            .map_err(|error| {
                format!(
                    "quality lane registry {}:{} has invalid runtime_budget_seconds: {error}",
                    path.display(),
                    index + 1
                )
            })?;
    Ok(QualityLaneRecord {
        id,
        stage,
        status,
        owner_team: required_quality_lane_field(path, index, fields, "owner_team")?,
        purpose: required_quality_lane_field(path, index, fields, "purpose")?,
        source: required_quality_lane_field(path, index, fields, "source")?,
        runtime_budget_seconds,
        check_command,
    })
}

fn required_quality_lane_field(
    path: &Path,
    index: usize,
    fields: &BTreeMap<String, String>,
    key: &str,
) -> Result<String, String> {
    fields
        .get(key)
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "quality lane registry {}:{} missing required field {key}",
                path.display(),
                index + 1
            )
        })
}

fn read_quality_lane_markdown_rows(path: &Path) -> Result<Vec<QualityLaneDocRow>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "quality lane markdown unreadable {}: {error}",
            path.display()
        )
    })?;
    let mut rows = Vec::new();
    let mut stage = None;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("### 1.1 ") {
            stage = Some(QualityLaneStage::Foundation);
            continue;
        }
        if trimmed.starts_with("### 1.2 ") {
            stage = Some(QualityLaneStage::PerPr);
            continue;
        }
        if trimmed.starts_with("### 1.3 ") {
            stage = Some(QualityLaneStage::Nightly);
            continue;
        }
        if trimmed.starts_with("### 1.4 ") {
            stage = Some(QualityLaneStage::PerRelease);
            continue;
        }
        if !trimmed.starts_with("| `") || trimmed.starts_with("| `Lane") {
            continue;
        }
        let columns = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if columns.len() < 2 {
            continue;
        }
        let Some(stage) = stage else {
            return Err(format!(
                "quality lane markdown row appears before a known stage heading: {trimmed}"
            ));
        };
        let id = columns[0].trim_matches('`').to_string();
        let purpose = columns[1].to_string();
        rows.push(QualityLaneDocRow { id, stage, purpose });
    }
    Ok(rows)
}
