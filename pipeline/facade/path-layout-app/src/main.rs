//! Event-independent ADR-0719 D-8 changed-path admission facade.

use std::collections::BTreeSet;
use std::env;
use std::process::{Command, ExitCode, Output};

use pipeline_admission::{
    APP_PRODUCT_DIRS, BUILD_ROOT_DIRS, base_admission_violations, cargo_manifest_violations,
    changed_layout_violations, git_change_paths_from_name_status_z,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let base = required_sha("OYATIE_LAYOUT_BASE")?;
    let head = required_sha("OYATIE_LAYOUT_HEAD")?;
    let merge_base = git_text(&["merge-base", &base, &head])?;
    let output = git_output(&[
        "diff",
        "--name-status",
        "-z",
        "-M",
        &merge_base,
        &head,
        "--",
    ])?;
    let changes =
        git_change_paths_from_name_status_z(&output.stdout).map_err(|error| error.message())?;
    let existing_owner_dirs = existing_owner_dirs(&merge_base)?;
    let mut violations = changed_layout_violations(&changes, &existing_owner_dirs);
    let mut manifests = Vec::new();
    for path in changes
        .layout_candidates
        .iter()
        .filter(|path| path.ends_with("/Cargo.toml"))
    {
        let contents = git_blob_text(&head, path)?;
        violations.extend(cargo_manifest_violations(path, &contents));
        manifests.push((path.clone(), contents));
    }
    let first_base = !existing_owner_dirs.contains("base")
        && changes
            .layout_candidates
            .iter()
            .any(|path| path.starts_with("base/"));
    if first_base {
        violations.extend(base_admission_violations(&manifests));
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "ADR-0719 D-8 layout refused:\n{}",
            violations.join("\n")
        ))
    }
}

fn git_blob_text(commit: &str, path: &str) -> Result<String, String> {
    let object = format!("{commit}:{path}");
    let output = git_output(&["cat-file", "blob", &object])?;
    String::from_utf8(output.stdout)
        .map_err(|_| format!("git cat-file returned non-UTF-8 for {path}"))
}

fn required_sha(name: &str) -> Result<String, String> {
    let value = env::var(name).map_err(|_| format!("{name} is required"))?;
    if value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be a 40-digit Git object id"));
    }
    Ok(value)
}

fn existing_owner_dirs(merge_base: &str) -> Result<BTreeSet<String>, String> {
    let mut owners = BTreeSet::new();
    for root in BUILD_ROOT_DIRS {
        record_existing_dir(merge_base, root, &mut owners)?;
    }
    for product in APP_PRODUCT_DIRS {
        record_existing_dir(merge_base, &format!("app/{product}"), &mut owners)?;
    }
    Ok(owners)
}

fn record_existing_dir(
    merge_base: &str,
    owner: &str,
    existing: &mut BTreeSet<String>,
) -> Result<(), String> {
    let output = git_output(&["ls-tree", "-d", "--name-only", merge_base, "--", owner])?;
    let path = String::from_utf8(output.stdout)
        .map_err(|_| format!("git ls-tree returned non-UTF-8 for {owner}"))?;
    match path.trim_end() {
        "" => Ok(()),
        present if present == owner => {
            existing.insert(owner.to_owned());
            Ok(())
        }
        other => Err(format!(
            "unexpected git ls-tree output for {owner}: {other:?}"
        )),
    }
}

fn git_text(args: &[&str]) -> Result<String, String> {
    let output = git_output(args)?;
    let text = String::from_utf8(output.stdout)
        .map_err(|_| format!("git {} returned non-UTF-8", args.join(" ")))?;
    let text = text.trim().to_owned();
    if text.is_empty() {
        return Err(format!("git {} returned empty output", args.join(" ")));
    }
    Ok(text)
}

fn git_output(args: &[&str]) -> Result<Output, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|error| format!("spawn git {}: {error}", args.join(" ")))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}
