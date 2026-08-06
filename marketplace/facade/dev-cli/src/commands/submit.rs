//! `oya submit` — canonical local-to-PR shipping primitive.
//!
//! Bundles the three steps every developer (human or agent) does
//! every time they want their work to land:
//!
//! 1. `oya verify --ci-required` (delegates to `gate run-all`, the
//!    canonical pre-merge aggregator plus required hosted-check mirrors).
//! 2. `oya git push` (with `-u origin HEAD` when no upstream is set).
//! 3. `gh pr create --fill` (only when no PR exists for the branch
//!    yet; if a PR is already open, the prior `oya git push` simply
//!    extends it — no second PR is opened).
//!
//! Per [[feedback_no_exceptions_canonical]], `oya submit` is the
//! single canonical shipping command. The retired manual sequence
//! (`cargo run … -- verify && git push && gh pr create`) is no longer
//! sanctioned; the [`oya-governance-retired-vocabulary`] lane
//! catches re-introductions in docs.
//!
//! Naming justification: top-level subcommand `submit` (kebab-case);
//! module file `src/commands/submit.rs` (snake_case, no redundant
//! `_command` suffix because it lives under `commands/`); handler
//! `run` (snake_case verb). Conforms to ADR-0105 v4 BNF and the
//! 12-value layer enum at
//! `crates/oya-governance-predictable-naming-kernel::ALLOWED_ROLES`.
//!
//! Tooling dependencies: this subcommand shells out to the repo-local
//! `oya git` transport wrapper for git history operations, and `gh`
//! for GitHub PR operations. Replacing those external CLIs with native
//! Rust clients is an orthogonal milestone (octocrab + gitoxide
//! migrations are tracked separately). The shell-out surface is
//! minimal: 3 subprocesses (`oya git status`, `oya git push`,
//! `gh pr view` / `gh pr create`) with explicit arg arrays and no
//! `bash -c` interpolation — see
//! [`feedback_no_silent_regression`] for the no-shell-injection
//! invariant.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::process::{Command, ExitCode, Stdio};

use super::verify;

const SUBMIT_VERIFY_ARGS: &[&str] = &["--ci-required"];

const USAGE: &str = "oya submit \
                     [--no-verify] \
                     [--push-only] \
                     [--draft] \
                     [--title <text>] \
                     [--body <text>]";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SubmitArgs {
    skip_verify: bool,
    push_only: bool,
    draft: bool,
    title: Option<String>,
    body: Option<String>,
}

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_submit_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    if !parsed.skip_verify {
        let verify_exit = verify::run(
            SUBMIT_VERIFY_ARGS
                .iter()
                .map(|arg| (*arg).to_string())
                .collect(),
            usage,
        );
        if !is_success(&verify_exit) {
            eprintln!(
                "oya submit: aborting — `oya verify --ci-required` did not pass. \
                 Fix the failing local/hosted-required mirrors and re-run `oya submit`."
            );
            return verify_exit;
        }
    }

    if !git_working_tree_is_clean() {
        eprintln!(
            "oya submit: aborting — working tree has uncommitted changes. \
             Commit (or stash) first, then re-run `oya submit`."
        );
        return ExitCode::FAILURE;
    }

    if let Err(message) = git_push_current_branch() {
        eprintln!("oya submit: `oya git push` failed: {message}");
        return ExitCode::FAILURE;
    }

    if parsed.push_only {
        println!("oya submit: push complete (--push-only; no PR opened).");
        return ExitCode::SUCCESS;
    }

    match existing_pr_url() {
        Ok(Some(url)) => {
            println!("oya submit: existing PR extended → {url}");
            ExitCode::SUCCESS
        }
        Ok(None) => match create_pr(&parsed) {
            Ok(url) => {
                println!("oya submit: PR opened → {url}");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("oya submit: `gh pr create` failed: {message}");
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("oya submit: `gh pr view` failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn parse_submit_args(args: Vec<String>) -> Result<SubmitArgs, String> {
    let mut parsed = SubmitArgs::default();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--no-verify" => parsed.skip_verify = true,
            "--push-only" => parsed.push_only = true,
            "--draft" => parsed.draft = true,
            "--title" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.title = Some(value);
            }
            "--body" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.body = Some(value);
            }
            _ => return Err(USAGE.to_owned()),
        }
    }
    Ok(parsed)
}

fn is_success(exit: &ExitCode) -> bool {
    // `ExitCode` does not expose its inner value publicly; compare by
    // formatting to detect the canonical success case. This avoids
    // depending on `process::Termination::report` (unstable) and
    // keeps the check fail-closed: any non-`ExitCode::SUCCESS` shape
    // is treated as failure.
    format!("{exit:?}") == format!("{:?}", ExitCode::SUCCESS)
}

/// Path prefixes that are agent-harness sidecars — they get written
/// mid-run by hooks/tooling and are never the intended scope of a
/// PR. The submit-time clean-check IGNORES these so the harness
/// noise doesn't trip the gate. If a real edit lands under one of
/// these prefixes, the user can override by committing it
/// explicitly.
///
/// History: surfaced 2026-05-16 when the dogfood `oya submit` of
/// PR #4 was rejected by the clean-check because the .omc/state/
/// hud-stdin-cache.json kept being rewritten by hook activity
/// during the cargo-run of submit itself.
const HARNESS_SIDECAR_PREFIXES: &[&str] = &[".omc/", ".omx/", ".grit/", "target/"];

fn git_working_tree_is_clean() -> bool {
    let output = match oya_git_output(&["status", "--porcelain"]) {
        Ok(output) => output,
        Err(_) => return false,
    };
    if !output.status.success() {
        return false;
    }
    let stdout = match std::str::from_utf8(&output.stdout) {
        Ok(text) => text,
        Err(_) => return false,
    };
    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }
        // Porcelain format: `XY <path>` (or `XY <old> -> <new>` for
        // renames). Strip the 3-char status prefix to get the path.
        let path = match line.get(3..) {
            Some(rest) => rest,
            None => return false,
        };
        let path = path.split(" -> ").next().unwrap_or(path).trim();
        if HARNESS_SIDECAR_PREFIXES
            .iter()
            .any(|prefix| path.starts_with(prefix))
        {
            continue;
        }
        // Non-sidecar, non-empty entry → tree is dirty.
        return false;
    }
    true
}

#[cfg(test)]
mod working_tree_tests {
    // Pure-Rust tests for the path-filter logic don't need a real
    // git repo. We exercise the same parsing against synthetic
    // porcelain output.

    fn is_sidecar(path: &str) -> bool {
        super::HARNESS_SIDECAR_PREFIXES
            .iter()
            .any(|prefix| path.starts_with(prefix))
    }

    #[test]
    fn omc_state_files_are_sidecars() {
        assert!(is_sidecar(".omc/state/mission-state.json"));
        assert!(is_sidecar(".omc/sessions/123.json"));
    }

    #[test]
    fn omx_and_grit_and_target_are_sidecars() {
        assert!(is_sidecar(".omx/notepad.md"));
        assert!(is_sidecar(".grit/worktrees/something/Cargo.toml"));
        assert!(is_sidecar("target/debug/build.lock"));
    }

    #[test]
    fn real_source_paths_are_not_sidecars() {
        assert!(!is_sidecar("crates/oya-dev-cli/src/commands/submit.rs"));
        assert!(!is_sidecar("docs/adr-archive/ADR-0110-changeset-state-machine.md"));
        assert!(!is_sidecar(".github/workflows/pr-tests.yml"));
    }

    #[test]
    fn similarly_named_non_prefix_paths_are_not_sidecars() {
        // `.omc-archived/...` is NOT under `.omc/` so it should
        // still count as real (avoid accidental over-broad
        // matching).
        assert!(!is_sidecar(".omc-archived/old.json"));
        assert!(!is_sidecar("docs/.omc-notes.md"));
    }
}

fn git_push_current_branch() -> Result<(), String> {
    // First attempt: plain `oya git push`. Succeeds when an upstream is
    // already set for the current branch.
    let mut plain_cmd = oya_git_command()?;
    let plain = plain_cmd
        .arg("push")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("could not invoke `oya git push`: {error}"))?;
    if plain.success() {
        return Ok(());
    }
    // Fallback: set the upstream to `origin/HEAD`. This handles a
    // freshly-created local branch that has never been pushed.
    let mut upstream_cmd = oya_git_command()?;
    let with_upstream = upstream_cmd
        .args(["push", "-u", "origin", "HEAD"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("could not invoke `oya git push -u`: {error}"))?;
    if with_upstream.success() {
        Ok(())
    } else {
        Err(format!(
            "oya git push exited with {}",
            with_upstream.code().unwrap_or(-1)
        ))
    }
}

fn oya_git_command() -> Result<Command, String> {
    // PR-3 / ADR-0363: the `oya git` wrapper is retired — use plain `git`.
    Ok(Command::new("git"))
}

fn oya_git_output(args: &[&str]) -> Result<std::process::Output, String> {
    let mut command = oya_git_command()?;
    command
        .args(args)
        .output()
        .map_err(|error| format!("could not invoke `oya git {}`: {error}", args.join(" ")))
}

fn existing_pr_url() -> Result<Option<String>, String> {
    let output = Command::new("gh")
        .args(["pr", "view", "--json", "url", "--jq", ".url"])
        .output()
        .map_err(|error| format!("could not invoke `gh pr view`: {error}"))?;
    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if url.is_empty() {
            Ok(None)
        } else {
            Ok(Some(url))
        }
    } else {
        // `gh pr view` returns non-zero when no PR exists for the
        // current branch — that's the canonical "no PR yet" signal,
        // not a failure. We disambiguate by checking stderr.
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no pull requests found") || stderr.contains("not found") {
            Ok(None)
        } else {
            Err(format!(
                "gh pr view failed (exit {}): {stderr}",
                output.status.code().unwrap_or(-1)
            ))
        }
    }
}

fn create_pr(args: &SubmitArgs) -> Result<String, String> {
    let mut cmd = Command::new("gh");
    cmd.args(["pr", "create"]);
    if let Some(title) = &args.title {
        cmd.args(["--title", title]);
    }
    if let Some(body) = &args.body {
        cmd.args(["--body", body]);
    }
    if args.title.is_none() && args.body.is_none() {
        cmd.arg("--fill");
    }
    if args.draft {
        cmd.arg("--draft");
    }
    let output = cmd
        .output()
        .map_err(|error| format!("could not invoke `gh pr create`: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "gh pr create failed (exit {}): {stderr}",
            output.status.code().unwrap_or(-1)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uses_canonical_defaults() {
        let args = parse_submit_args(Vec::new()).expect("no flags is valid");
        assert!(!args.skip_verify);
        assert!(!args.push_only);
        assert!(!args.draft);
        assert!(args.title.is_none());
        assert!(args.body.is_none());
    }

    #[test]
    fn parse_accepts_no_verify_flag() {
        let args = parse_submit_args(vec!["--no-verify".to_string()]).expect("--no-verify parses");
        assert!(args.skip_verify);
    }

    #[test]
    fn parse_accepts_push_only_flag() {
        let args = parse_submit_args(vec!["--push-only".to_string()]).expect("--push-only parses");
        assert!(args.push_only);
    }

    #[test]
    fn parse_accepts_draft_flag() {
        let args = parse_submit_args(vec!["--draft".to_string()]).expect("--draft parses");
        assert!(args.draft);
    }

    #[test]
    fn parse_accepts_title_and_body() {
        let args = parse_submit_args(vec![
            "--title".to_string(),
            "ship: my change".to_string(),
            "--body".to_string(),
            "## Summary\nshort body".to_string(),
        ])
        .expect("title + body parse");
        assert_eq!(args.title.as_deref(), Some("ship: my change"));
        assert_eq!(args.body.as_deref(), Some("## Summary\nshort body"));
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        let result = parse_submit_args(vec!["--unknown".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_rejects_dangling_value_flag() {
        let result = parse_submit_args(vec!["--title".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn is_success_matches_canonical_success_exit_code() {
        assert!(is_success(&ExitCode::SUCCESS));
        assert!(!is_success(&ExitCode::FAILURE));
        assert!(!is_success(&ExitCode::from(2)));
    }

    #[test]
    fn submit_default_verify_profile_is_ci_required() {
        assert_eq!(SUBMIT_VERIFY_ARGS, &["--ci-required"]);
    }
}
