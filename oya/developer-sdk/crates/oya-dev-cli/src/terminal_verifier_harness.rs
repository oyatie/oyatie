use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use serde_json::json;

pub(crate) const CLEAN_CHECKOUT_EVIDENCE_CLASS: &str = "clean-checkout";
const SCHEMA_VERSION: &str = "g013-terminal-verifier-harness.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TerminalEvidenceArgs {
    pub evidence_class: String,     // data_class: INTERNAL_ONLY
    pub repo_root: Option<PathBuf>, // data_class: INTERNAL_ONLY
}

#[derive(Debug)]
pub(crate) struct TerminalEvidenceRun {
    pub stdout_json: String, // data_class: INTERNAL_ONLY
    pub exit: ExitCode,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TerminalEvidenceError {
    pub message: String, // data_class: INTERNAL_ONLY
}

impl TerminalEvidenceError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub(crate) fn parse_terminal_evidence_args(
    args: Vec<String>,
    usage: &str,
) -> Result<TerminalEvidenceArgs, TerminalEvidenceError> {
    let mut evidence_class = None;
    let mut repo_root = None;
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--terminal-evidence" => {
                let Some(value) = iter.next() else {
                    return Err(TerminalEvidenceError::new(format!(
                        "oya verify: --terminal-evidence requires an evidence class\n{usage}"
                    )));
                };
                evidence_class = Some(value);
            }
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return Err(TerminalEvidenceError::new(format!(
                        "oya verify: --repo-root requires a path\n{usage}"
                    )));
                };
                repo_root = Some(PathBuf::from(value));
            }
            other => {
                return Err(TerminalEvidenceError::new(format!(
                    "oya verify: unknown terminal-evidence flag {other:?}\n{usage}"
                )));
            }
        }
    }
    Ok(TerminalEvidenceArgs {
        evidence_class: evidence_class.ok_or_else(|| {
            TerminalEvidenceError::new(format!(
                "oya verify: --terminal-evidence requires an evidence class\n{usage}"
            ))
        })?,
        repo_root,
    })
}

pub(crate) fn run_terminal_evidence(
    args: TerminalEvidenceArgs,
) -> Result<TerminalEvidenceRun, TerminalEvidenceError> {
    match args.evidence_class.as_str() {
        CLEAN_CHECKOUT_EVIDENCE_CLASS => run_clean_checkout(args.repo_root.as_deref()),
        other => Err(TerminalEvidenceError::new(format!(
            "oya verify: unknown terminal evidence class {other:?}; supported: {CLEAN_CHECKOUT_EVIDENCE_CLASS}"
        ))),
    }
}

fn run_clean_checkout(
    repo_root_override: Option<&Path>,
) -> Result<TerminalEvidenceRun, TerminalEvidenceError> {
    let repo_root = match repo_root_override {
        Some(path) => path.to_path_buf(),
        None => PathBuf::from(run_git_text(
            Path::new("."),
            &["rev-parse", "--show-toplevel"],
        )?),
    };
    let checkout_ref = run_git_text(repo_root.as_path(), &["rev-parse", "HEAD"])?;
    let status = run_git_text(
        repo_root.as_path(),
        &["status", "--short", "--untracked-files=all"],
    )?;
    let dirty_paths = status
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let passed = dirty_paths.is_empty();
    let outcome = if passed { "pass" } else { "fail" };
    let document = json!({
        "schema_version": SCHEMA_VERSION,
        "emitter": "oya verify --terminal-evidence clean-checkout",
        "evidence_class": CLEAN_CHECKOUT_EVIDENCE_CLASS,
        "claim_scope": "slice_evidence",
        "outcome": outcome,
        "repo_root": repo_root.display().to_string(),
        "checkout_ref": checkout_ref,
        "dirty_paths": dirty_paths,
        "full_platform_terminal_closure_claimed": false,
        "g013_complete_claimed": false,
        "local_bridge_only": true,
        "notes": [
            "This result proves only the clean-checkout evidence class for a bounded verifier slice.",
            "It must not be used as a complete or terminal G013 full-platform closure claim."
        ]
    });
    let stdout_json = serde_json::to_string_pretty(&document).map_err(|error| {
        TerminalEvidenceError::new(format!(
            "oya verify: failed to serialize terminal evidence JSON: {error}"
        ))
    })?;
    Ok(TerminalEvidenceRun {
        stdout_json,
        exit: if passed {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

fn run_git_text(cwd: &Path, args: &[&str]) -> Result<String, TerminalEvidenceError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| {
            TerminalEvidenceError::new(format!("oya verify: git launch failed: {error}"))
        })?;
    if !output.status.success() {
        return Err(TerminalEvidenceError::new(format!(
            "oya verify: git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_clean_checkout_class() {
        let args = parse_terminal_evidence_args(
            vec![
                "--terminal-evidence".into(),
                CLEAN_CHECKOUT_EVIDENCE_CLASS.into(),
                "--repo-root".into(),
                "/tmp/repo".into(),
            ],
            "usage",
        )
        .expect("valid args");

        assert_eq!(args.evidence_class, CLEAN_CHECKOUT_EVIDENCE_CLASS);
        assert_eq!(args.repo_root, Some(PathBuf::from("/tmp/repo")));
    }

    #[test]
    fn parser_rejects_missing_class() {
        let error = parse_terminal_evidence_args(vec!["--terminal-evidence".into()], "usage")
            .expect_err("missing class");

        assert!(error.message.contains("requires an evidence class"));
    }
}
