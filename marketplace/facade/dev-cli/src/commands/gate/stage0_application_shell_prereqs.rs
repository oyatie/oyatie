// Purpose: validate Stage-0 application-shell prerequisites for M02 substrate.
// Ported from `scripts/check-stage0-application-shell-prereqs.py` per
// `evidence/audits/shell-python-replacement-audit-2026-05-15.md` row B-6.
// Naming-justification: `stage0_application_shell_prereqs_gate` is a
// check-family (`*_gate`) module under `oya-dev-cli`, satisfying
// predictable-naming-kernel `is_check_family(name)`. Surface command
// `gate validate stage0-prereqs` is canonical kebab-case verb-noun pair
// (ADR-0105 v4 BNF).

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::usage;

const REQUIRED_PATHS: &[&str] = &[
    "Cargo.toml",
    "crates/oya-application-app/Cargo.toml",
    "crates/oya-application-app/src/lib.rs",
    "docs/decisions/ADR-0061-application-b2b-unified-shell.md",
];

const EXPECTED_APP_EDITION: &str = "2024";
const EXPECTED_APP_RUST_VERSION: &str = "1.97.1";
const APP_PACKAGE_NAME: &str = "oya-application-app";
const APP_WORKSPACE_MEMBER: &str = "crates/oya-application-app";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Stage0PrereqsValidateArgs {
    pub repo_root: PathBuf,
    pub self_test: bool,
}

pub(crate) fn parse_stage0_prereqs_validate_args(
    args: Vec<String>,
) -> Result<Stage0PrereqsValidateArgs, String> {
    let mut parsed = Stage0PrereqsValidateArgs {
        repo_root: PathBuf::from("."),
        self_test: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--self-test" => parsed.self_test = true,
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.repo_root = PathBuf::from(value);
            }
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Stage0PrereqsReport {
    pub required_paths_checked: usize,
    pub workspace_member_present: bool,
    pub package_edition: String,
    pub package_rust_version: String,
}

pub(crate) fn validate_stage0_prereqs_gate(
    args: Stage0PrereqsValidateArgs,
) -> Result<Stage0PrereqsReport, String> {
    let errors = check_repo(&args.repo_root)?;
    if !errors.is_empty() {
        return Err(format_errors(&errors));
    }
    let workspace_text = std::fs::read_to_string(args.repo_root.join("Cargo.toml"))
        .map_err(|error| format!("Cargo.toml unreadable at repo root: {error}"))?;
    let members = parse_workspace_members(&workspace_text);
    let (edition, rust_version) = read_application_app_metadata(&args.repo_root)?;
    Ok(Stage0PrereqsReport {
        required_paths_checked: REQUIRED_PATHS.len(),
        workspace_member_present: members.iter().any(|member| member == APP_WORKSPACE_MEMBER),
        package_edition: edition,
        package_rust_version: rust_version,
    })
}

fn check_repo(root: &Path) -> Result<Vec<String>, String> {
    let mut errors: Vec<String> = Vec::new();
    for rel_path in REQUIRED_PATHS {
        if !root.join(rel_path).exists() {
            errors.push(format!("missing required path: {rel_path}"));
        }
    }

    let cargo_toml_path = root.join("Cargo.toml");
    let cargo_toml_text = std::fs::read_to_string(&cargo_toml_path).map_err(|error| {
        format!(
            "Cargo.toml unreadable {}: {error}",
            cargo_toml_path.display()
        )
    })?;
    let members = parse_workspace_members(&cargo_toml_text);
    if !members.iter().any(|member| member == APP_WORKSPACE_MEMBER) {
        errors.push(format!(
            "workspace members does not include {APP_WORKSPACE_MEMBER}"
        ));
    }

    match run_cargo_metadata(root) {
        Ok((edition, rust_version)) => {
            if edition != EXPECTED_APP_EDITION {
                errors.push(format!(
                    "{APP_PACKAGE_NAME} edition is {edition}, expected {EXPECTED_APP_EDITION}"
                ));
            }
            if rust_version != EXPECTED_APP_RUST_VERSION {
                errors.push(format!(
                    "{APP_PACKAGE_NAME} rust-version is {rust_version}, expected {EXPECTED_APP_RUST_VERSION}"
                ));
            }
        }
        Err(error) => errors.push(error),
    }
    Ok(errors)
}

pub(crate) fn parse_workspace_members(cargo_toml_text: &str) -> Vec<String> {
    let mut members: Vec<String> = Vec::new();
    let mut in_members = false;
    for raw_line in cargo_toml_text.lines() {
        let line = raw_line.trim();
        if line.starts_with("members") && line.contains('[') {
            collect_quoted_values(
                line.split_once('[').map_or("", |(_, rest)| rest),
                &mut members,
            );
            if !line.contains(']') {
                in_members = true;
            }
            continue;
        }
        if in_members && line.starts_with(']') {
            break;
        }
        if in_members {
            collect_quoted_values(line, &mut members);
            if line.contains(']') {
                break;
            }
        }
    }
    members
}

fn collect_quoted_values(text: &str, values: &mut Vec<String>) {
    let mut rest = text;
    while let Some(start) = rest.find('"') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('"') else {
            break;
        };
        values.push(after_start[..end].to_string());
        rest = &after_start[end + 1..];
    }
}

fn read_application_app_metadata(root: &Path) -> Result<(String, String), String> {
    run_cargo_metadata(root)
}

fn run_cargo_metadata(root: &Path) -> Result<(String, String), String> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version")
        .arg("1")
        .current_dir(root)
        .output()
        .map_err(|error| format!("cargo metadata failed to start: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cargo metadata failed: {}", stderr.trim_end()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let metadata: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|error| format!("cargo metadata JSON invalid: {error}"))?;
    let packages = metadata
        .get("packages")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "cargo metadata missing packages array".to_string())?;
    let package = packages
        .iter()
        .find(|entry| {
            entry
                .get("name")
                .and_then(|value| value.as_str())
                .is_some_and(|name| name == APP_PACKAGE_NAME)
        })
        .ok_or_else(|| format!("cargo metadata does not include {APP_PACKAGE_NAME}"))?;
    let edition = package
        .get("edition")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();
    let rust_version = package
        .get("rust_version")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();
    Ok((edition, rust_version))
}

fn format_errors(errors: &[String]) -> String {
    errors
        .iter()
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_workspace_members_returns_quoted_paths() {
        let toml = "[workspace]\nmembers = [\n  \"crates/a\",\n  \"crates/b\",\n]\n";
        let members = parse_workspace_members(toml);
        assert_eq!(members, vec!["crates/a", "crates/b"]);
    }

    #[test]
    fn parse_workspace_members_accepts_inline_array() {
        let toml = "[workspace]\nmembers = [\"crates/a\", \"crates/b\"]\n";
        let members = parse_workspace_members(toml);
        assert_eq!(members, vec!["crates/a", "crates/b"]);
    }

    #[test]
    fn parse_workspace_members_skips_until_open_bracket() {
        let toml = "[workspace]\nresolver = \"3\"\nmembers = [\n  \"crates/x\",\n]\n";
        let members = parse_workspace_members(toml);
        assert_eq!(members, vec!["crates/x"]);
    }

    #[test]
    fn parse_args_accepts_self_test_flag() {
        let args = parse_stage0_prereqs_validate_args(vec!["--self-test".to_string()])
            .expect("flag must parse");
        assert!(args.self_test);
    }

    #[test]
    fn parse_args_rejects_unknown_flag() {
        let result = parse_stage0_prereqs_validate_args(vec!["--bogus".to_string()]);
        assert!(result.is_err());
    }
}
