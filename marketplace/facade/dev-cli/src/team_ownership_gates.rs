use std::fs;
use std::path::{Path, PathBuf};

use check_codeowners_mirror::{CodeownersEntry, validate_codeowners_mirror};
use check_raci_coverage::validate_raci_team_coverage;

use crate::{extract_first_backticked_value, usage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CodeownersMirrorValidateArgs {
    codeowners_path: PathBuf,
    teams_dir: PathBuf,
}

pub(crate) fn parse_codeowners_mirror_validate_args(
    args: Vec<String>,
) -> Result<CodeownersMirrorValidateArgs, String> {
    let mut parsed = CodeownersMirrorValidateArgs {
        codeowners_path: PathBuf::from(".github/CODEOWNERS"),
        teams_dir: PathBuf::from("docs/teams"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--codeowners" => parsed.codeowners_path = PathBuf::from(path),
            "--teams-dir" => parsed.teams_dir = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_codeowners_mirror_gate(
    args: CodeownersMirrorValidateArgs,
) -> Result<(usize, usize), String> {
    let entries = read_codeowners_entries(&args.codeowners_path)?;
    let team_ids = list_team_ids(&args.teams_dir)?;
    let report = validate_codeowners_mirror(&entries, team_ids)
        .map_err(|error| format!("codeowners mirror invalid: {error:?}"))?;
    Ok((report.entries_checked, report.owners_checked))
}

fn read_codeowners_entries(path: &Path) -> Result<Vec<CodeownersEntry>, String> {
    let contents =
        fs::read_to_string(path).map_err(|error| format!("CODEOWNERS unreadable: {error}"))?;
    let mut entries = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(pattern) = parts.next() else {
            continue;
        };
        entries.push(CodeownersEntry {
            line_number: index + 1,
            pattern: pattern.to_string(),
            owners: parts.map(str::to_string).collect(),
        });
    }
    Ok(entries)
}

pub(crate) fn list_team_ids(teams_dir: &Path) -> Result<Vec<String>, String> {
    let mut team_ids = Vec::new();
    for entry in fs::read_dir(teams_dir).map_err(|error| {
        format!(
            "teams directory unreadable {}: {error}",
            teams_dir.display()
        )
    })? {
        let entry = entry.map_err(|error| format!("teams directory entry unreadable: {error}"))?;
        let path = entry.path();
        if !path.is_dir() || !path.join("CHARTER.md").is_file() {
            continue;
        }
        let team_id = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("team path has invalid name: {}", path.display()))?;
        team_ids.push(team_id.to_string());
    }
    if team_ids.is_empty() {
        Err("teams directory contains no CHARTER.md files".to_string())
    } else {
        Ok(team_ids)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RaciTeamCoverageValidateArgs {
    teams_dir: PathBuf,
    raci_path: PathBuf,
    codeowners_path: PathBuf,
}

pub(crate) fn parse_raci_team_coverage_validate_args(
    args: Vec<String>,
) -> Result<RaciTeamCoverageValidateArgs, String> {
    let mut parsed = RaciTeamCoverageValidateArgs {
        teams_dir: PathBuf::from("docs/teams"),
        raci_path: PathBuf::from("docs/RACI-OWNERSHIP.md"),
        codeowners_path: PathBuf::from(".github/CODEOWNERS"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--teams-dir" => parsed.teams_dir = PathBuf::from(path),
            "--raci" => parsed.raci_path = PathBuf::from(path),
            "--codeowners" => parsed.codeowners_path = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_raci_team_coverage_gate(
    args: RaciTeamCoverageValidateArgs,
) -> Result<usize, String> {
    let team_ids = list_team_ids(&args.teams_dir)?;
    let raci_team_ids = read_raci_team_coverage_ids(&args.raci_path)?;
    let codeowners_team_ids = read_codeowners_team_ids(&args.codeowners_path)?;
    let report = validate_raci_team_coverage(team_ids, raci_team_ids, codeowners_team_ids)
        .map_err(|error| format!("raci team coverage invalid: {error:?}"))?;
    Ok(report.teams_checked)
}

fn read_raci_team_coverage_ids(path: &Path) -> Result<Vec<String>, String> {
    let contents =
        fs::read_to_string(path).map_err(|error| format!("RACI ownership unreadable: {error}"))?;
    let mut ids = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("| `") {
            continue;
        }
        let Some(team_id) = extract_first_backticked_value(trimmed) else {
            continue;
        };
        ids.push(team_id);
    }
    Ok(ids)
}

fn read_codeowners_team_ids(path: &Path) -> Result<Vec<String>, String> {
    let entries = read_codeowners_entries(path)?;
    Ok(entries
        .into_iter()
        .flat_map(|entry| entry.owners)
        .filter_map(|owner| owner.strip_prefix("@teams/").map(str::to_string))
        .collect())
}
