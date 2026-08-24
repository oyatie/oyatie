//! Event-independent ADR-0719 D-8 changed-path admission.

use std::collections::BTreeSet;
use std::env;
use std::process::{Command, ExitCode, Output};

use pipeline_admission::{
    BUILD_ROOT_DIRS, changed_layout_violations, git_change_paths_from_name_status_z,
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
    let existing_build_roots = existing_build_roots(&merge_base)?;
    let violations = changed_layout_violations(&changes, &existing_build_roots);
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "ADR-0719 D-8 layout refused:\n{}",
            violations.join("\n")
        ))
    }
}

fn required_sha(name: &str) -> Result<String, String> {
    let value = env::var(name).map_err(|_| format!("{name} is required"))?;
    if value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be a 40-digit Git object id"));
    }
    Ok(value)
}

fn existing_build_roots(merge_base: &str) -> Result<BTreeSet<String>, String> {
    let mut roots = BTreeSet::new();
    for root in BUILD_ROOT_DIRS {
        let output = git_output(&["ls-tree", "-d", "--name-only", merge_base, "--", root])?;
        let path = String::from_utf8(output.stdout)
            .map_err(|_| format!("git ls-tree returned non-UTF-8 for {root}"))?;
        match path.trim_end() {
            "" => {}
            present if present == *root => {
                roots.insert((*root).to_owned());
            }
            other => {
                return Err(format!(
                    "unexpected git ls-tree output for {root}: {other:?}"
                ));
            }
        }
    }
    Ok(roots)
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
