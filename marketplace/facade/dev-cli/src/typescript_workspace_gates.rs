use std::fs;
use std::path::{Path, PathBuf};

use check_typescript_workspace::{
    TypescriptWorkspaceEvidence, TypescriptWorkspaceLane, TypescriptWorkspaceScript,
    validate_typescript_workspace,
};

use crate::{
    extract_json_object_for_key, next_arg, parse_json_string_field, parse_json_string_value,
    quoted_json_len, usage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TypescriptWorkspaceValidateArgs {
    repo_root: PathBuf,
    lane: TypescriptWorkspaceLane,
}

pub(crate) fn parse_typescript_workspace_validate_args(
    args: Vec<String>,
) -> Result<TypescriptWorkspaceValidateArgs, String> {
    let mut repo_root = PathBuf::from(".");
    let mut lane = None;
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => repo_root = PathBuf::from(next_arg(&mut iter)?),
            "--lane" => {
                let value = next_arg(&mut iter)?;
                lane = Some(parse_typescript_workspace_lane(&value)?);
            }
            _ => return Err(usage()),
        }
    }
    Ok(TypescriptWorkspaceValidateArgs {
        repo_root,
        lane: lane.ok_or_else(usage)?,
    })
}

fn parse_typescript_workspace_lane(value: &str) -> Result<TypescriptWorkspaceLane, String> {
    match value {
        "typecheck" => Ok(TypescriptWorkspaceLane::Typecheck),
        "test" => Ok(TypescriptWorkspaceLane::Test),
        _ => Err(usage()),
    }
}

pub(crate) fn validate_typescript_workspace_gate(
    args: TypescriptWorkspaceValidateArgs,
) -> Result<(String, bool, usize, usize), String> {
    let evidence = read_typescript_workspace_evidence(&args.repo_root)?;
    let report = validate_typescript_workspace(evidence, args.lane)
        .map_err(|error| format!("typescript workspace invalid: {error:?}"))?;
    Ok((
        typescript_workspace_lane_name(report.lane).into(),
        report.workspace_present,
        report.markers_checked,
        report.scripts_checked,
    ))
}

fn typescript_workspace_lane_name(lane: TypescriptWorkspaceLane) -> &'static str {
    match lane {
        TypescriptWorkspaceLane::Typecheck => "typecheck",
        TypescriptWorkspaceLane::Test => "test",
    }
}

fn read_typescript_workspace_evidence(
    repo_root: &Path,
) -> Result<TypescriptWorkspaceEvidence, String> {
    if !repo_root.is_dir() {
        return Err(format!(
            "repo root is not a directory: {}",
            repo_root.display()
        ));
    }

    let package_json_path = repo_root.join("package.json");
    let root_package_json_present = package_json_path.is_file();
    let (package_manager, scripts) = if root_package_json_present {
        read_package_json_pnpm_lane_fields(&package_json_path)?
    } else {
        (None, Vec::new())
    };
    let mut marker_paths = Vec::new();
    collect_typescript_workspace_markers(repo_root, repo_root, &mut marker_paths)?;
    marker_paths.sort();
    marker_paths.dedup();
    Ok(TypescriptWorkspaceEvidence {
        marker_paths,
        root_package_json_present,
        pnpm_lock_present: repo_root.join("pnpm-lock.yaml").is_file(),
        pnpm_workspace_present: repo_root.join("pnpm-workspace.yaml").is_file(),
        package_manager,
        scripts,
    })
}

fn read_package_json_pnpm_lane_fields(
    package_json_path: &Path,
) -> Result<(Option<String>, Vec<TypescriptWorkspaceScript>), String> {
    let contents = fs::read_to_string(package_json_path)
        .map_err(|error| format!("package.json unreadable: {error}"))?;
    let package_manager = parse_json_string_field(&contents, "packageManager");
    let scripts = extract_json_object_for_key(&contents, "scripts")
        .map(parse_json_string_object_entries)
        .unwrap_or_default()
        .into_iter()
        .map(|(name, command)| TypescriptWorkspaceScript { name, command })
        .collect();
    Ok((package_manager, scripts))
}

fn parse_json_string_object_entries(object: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut rest = object;
    while let Some(key_quote_index) = rest.find('"') {
        let key_start = &rest[key_quote_index..];
        let Some(key) = parse_json_string_value(key_start) else {
            break;
        };
        let Some(key_len) = quoted_json_len(key_start) else {
            break;
        };
        let after_key = &key_start[key_len..];
        let Some(colon_index) = after_key.find(':') else {
            break;
        };
        let after_colon = after_key[colon_index + 1..].trim_start();
        let Some(value) = parse_json_string_value(after_colon) else {
            rest = after_colon;
            continue;
        };
        let Some(value_len) = quoted_json_len(after_colon) else {
            break;
        };
        entries.push((key, value));
        rest = &after_colon[value_len..];
    }
    entries
}

fn collect_typescript_workspace_markers(
    repo_root: &Path,
    dir: &Path,
    marker_paths: &mut Vec<String>,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)
        .map_err(|error| format!("typescript workspace marker directory unreadable: {error}"))?
    {
        let entry = entry
            .map_err(|error| format!("typescript workspace marker entry unreadable: {error}"))?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if path.is_dir() {
            if ignored_typescript_workspace_dir(&file_name) {
                continue;
            }
            collect_typescript_workspace_markers(repo_root, &path, marker_paths)?;
        } else if is_typescript_workspace_marker(&path) {
            let relative = path.strip_prefix(repo_root).unwrap_or(&path);
            marker_paths.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn ignored_typescript_workspace_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | ".omx" | "target" | "docs" | "registry" | ".github" | "node_modules"
    )
}

fn is_typescript_workspace_marker(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if matches!(
        file_name,
        "package.json"
            | "pnpm-lock.yaml"
            | "pnpm-workspace.yaml"
            | "tsconfig.json"
            | "tsconfig.build.json"
    ) || file_name.starts_with("vitest.config.")
        || file_name.starts_with("playwright.config.")
    {
        return true;
    }
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ts") | Some("tsx") | Some("mts") | Some("cts")
    )
}
