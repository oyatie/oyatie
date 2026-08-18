//! Shared ignored/untracked path boundary for retirement writers.

use std::path::Path;
use std::process::Command;

use super::GENERATED_FACTS_PATH;

pub(crate) fn canonical_ignored_generated_path<'a>(
    repo_root: &Path,
    relative_path: &'a Path,
) -> Result<(Vec<&'a str>, &'a str), String> {
    let components = relative_path
        .components()
        .map(|component| match component {
            std::path::Component::Normal(value) => value
                .to_str()
                .ok_or_else(|| "ignored generated path is not UTF-8".to_owned()),
            _ => Err("ignored generated path must be normal and repo-relative".to_owned()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (final_name, parents) = components
        .split_last()
        .ok_or_else(|| "ignored generated path must name a file".to_owned())?;
    let mut ignored_command = Command::new("git");
    ignored_command
        .args(["check-ignore", "--quiet", "--"])
        .arg(relative_path)
        .current_dir(repo_root);
    // Capture git's stderr on both boundary probes. `check-ignore` and `ls-files` report a
    // genuine policy answer through their exit CODE and a fault (`fatal: not a git
    // repository`, a stale index lock) only on stderr — with stderr discarded, a fault is
    // indistinguishable from the policy violation and gets reported as one.
    let (status, ignored_suffix) = crate::command_status_with_captured_stderr(
        ignored_command,
        crate::P2_HISTORICAL_GIT_TIMEOUT,
        "check ignored generated output boundary",
    )?;
    // Three arms, not two: `check-ignore --quiet` answers 0 = ignored, 1 = NOT ignored, and
    // 128 = git itself faulted (no repository, a stale `index.lock`, a broken gitfile).
    // Collapsing 1 and 128 into "not ignored" reports a git fault to the consuming gate as a
    // POLICY VIOLATION — a wrong answer, not merely a terse one. Same shape as
    // `canonical_generated_facts_output_path` and `is_ancestor`.
    match status.code() {
        Some(0) => {}
        Some(1) => {
            return Err(format!(
                "ignored generated output {} must be ignored and untracked{ignored_suffix}",
                relative_path.display()
            ));
        }
        code => {
            return Err(format!(
                "check ignored generated output boundary exited with {code:?}{ignored_suffix}"
            ));
        }
    }
    let mut tracked_command = Command::new("git");
    tracked_command
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(relative_path)
        .current_dir(repo_root);
    let (tracked_status, tracked_suffix) = crate::command_status_with_captured_stderr(
        tracked_command,
        crate::P2_HISTORICAL_GIT_TIMEOUT,
        "check tracked generated output boundary",
    )?;
    if tracked_status.code() == Some(0) {
        return Err(format!(
            "ignored generated output {} must be untracked",
            relative_path.display()
        ));
    }
    if tracked_status.code() != Some(1) {
        return Err(format!(
            "check tracked generated output boundary exited with {:?}{tracked_suffix}",
            tracked_status.code()
        ));
    }
    Ok((parents.to_vec(), final_name))
}

#[cfg(unix)]
pub(crate) fn write_all(file: &OwnedFd, mut bytes: &[u8]) -> Result<(), String> {
    while !bytes.is_empty() {
        let written = rustix::io::write(file, bytes)
            .map_err(|error| format!("write retirement facts temporary file: {error}"))?;
        if written == 0 {
            return Err("write retirement facts temporary file made no progress".to_owned());
        }
        bytes = &bytes[written..];
    }
    Ok(())
}

pub(crate) fn canonical_generated_facts_output_path(
    repo_root: &Path,
    output_path: &Path,
) -> Result<(), String> {
    if output_path != Path::new(GENERATED_FACTS_PATH)
        || !output_path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(format!(
            "retirement facts output must be the exact canonical repo-relative generated facts path {GENERATED_FACTS_PATH}"
        ));
    }
    ensure_existing_canonical_parent_is_real(repo_root)?;
    let status = Command::new("git")
        .args(["check-ignore", "--quiet", "--", GENERATED_FACTS_PATH])
        .current_dir(repo_root)
        .status()
        .map_err(|error| format!("check retirement facts output ignore boundary: {error}"))?;
    match status.code() {
        Some(0) => Ok(()),
        Some(1) => Err(format!(
            "retirement facts output {GENERATED_FACTS_PATH} must be ignored and untracked"
        )),
        code => Err(format!(
            "check retirement facts output ignore boundary: git exited {code:?}"
        )),
    }
}

pub(crate) fn ensure_existing_canonical_parent_is_real(repo_root: &Path) -> Result<(), String> {
    let mut current = repo_root.to_path_buf();
    for component in ["ci", "facade", "scm-facts-snapshot"] {
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(format!(
                    "retirement facts directory {component:?} is not a real directory"
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => {
                return Err(format!(
                    "inspect retirement facts directory {component:?}: {error}"
                ));
            }
        }
    }
    Ok(())
}

