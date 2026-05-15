// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod audit;
pub mod commands;
pub mod runner;

use std::io::{self, Write};

use audit::{AuditOutcome, Auditor, FileAuditor};
use runner::ProcessRunner;

pub const BIN_NAME: &str = "oya-tooling-agent-read";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub message: String,
}

impl CliError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

pub type CliResult<T> = Result<T, CliError>;

pub fn cli_main<I>(args: I) -> i32
where
    I: IntoIterator<Item = String>,
{
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let runner = ProcessRunner;
    let auditor = FileAuditor::from_env();
    match run_cli(args, &runner, &auditor, &mut stdout, &mut stderr) {
        Ok(code) => code,
        Err(err) => {
            let _ = writeln!(stderr, "{err}");
            2
        }
    }
}

pub fn run_cli<I, R, A, W, E>(
    args: I,
    runner: &R,
    auditor: &A,
    stdout: &mut W,
    stderr: &mut E,
) -> CliResult<i32>
where
    I: IntoIterator<Item = String>,
    R: runner::CommandRunner,
    A: Auditor,
    W: Write,
    E: Write,
{
    let args: Vec<String> = args.into_iter().collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        auditor
            .emit(&AuditOutcome::success("help", "internal", &args, 0))
            .map_err(|err| CliError::new(format!("audit emission failed: {err}")))?;
        write_usage(stdout).map_err(io_err)?;
        return Ok(0);
    }

    let verb = args[0].as_str();
    let known_tool = match verb {
        "log" | "diff" => Some("git"),
        "pr-view" | "pr-comments" => Some("gh"),
        _ => None,
    };

    let result = match verb {
        "log" => commands::log::run(&args[1..], runner, auditor, stdout, stderr),
        "diff" => commands::diff::run(&args[1..], runner, auditor, stdout, stderr),
        "pr-view" => commands::pr_view::run(&args[1..], runner, auditor, stdout, stderr),
        "pr-comments" => commands::pr_comments::run(&args[1..], runner, auditor, stdout, stderr),
        other => {
            let outcome = AuditOutcome::failure(
                other,
                "reject",
                &args[1..],
                None,
                "unsupported command; read-only helper allows only log,diff,pr-view,pr-comments",
            );
            if let Err(err) = auditor.emit(&outcome) {
                return Err(CliError::new(format!("audit emission failed: {err}")));
            }
            Err(CliError::new(format!(
                "unsupported command '{other}'; allowed commands: log, diff, pr-view, pr-comments"
            )))
        }
    };

    match result {
        Ok(code) => Ok(code),
        Err(err) => {
            if let Some(tool) = known_tool
                && !err.message.starts_with("audit emission failed")
            {
                auditor
                    .emit(&AuditOutcome::failure(
                        verb,
                        tool,
                        &args[1..],
                        None,
                        &err.to_string(),
                    ))
                    .map_err(|audit_err| {
                        CliError::new(format!(
                            "audit emission failed after command rejection: {audit_err}; original error: {err}"
                        ))
                    })?;
            }
            let _ = writeln!(stderr, "{err}");
            Ok(2)
        }
    }
}

fn write_usage<W: Write>(stdout: &mut W) -> io::Result<()> {
    writeln!(stdout, "{BIN_NAME} <command> [args]")?;
    writeln!(stdout)?;
    writeln!(stdout, "Read-only sanctioned agent helper. Commands:")?;
    writeln!(stdout, "  log [N] [--range A..B] [--paths PATH ...]")?;
    writeln!(stdout, "  diff --base REF --head REF [--paths PATH ...]")?;
    writeln!(stdout, "  pr-view PR_NUMBER")?;
    writeln!(stdout, "  pr-comments PR_NUMBER")?;
    Ok(())
}

pub(crate) fn io_err(error: io::Error) -> CliError {
    CliError::new(error.to_string())
}

pub(crate) fn validate_ref(value: &str, label: &str) -> CliResult<()> {
    if value.is_empty() || value.starts_with('-') || value.contains('\0') {
        return Err(CliError::new(format!("{label} is not a safe ref")));
    }
    if !value.bytes().all(|b| {
        b.is_ascii_alphanumeric() || matches!(b, b'.' | b'/' | b'_' | b'-' | b'~' | b'^' | b':')
    }) {
        return Err(CliError::new(format!(
            "{label} contains unsupported characters"
        )));
    }
    Ok(())
}

pub(crate) fn validate_path(value: &str) -> CliResult<()> {
    if value.is_empty() || value.starts_with('-') || value.contains('\0') {
        return Err(CliError::new(format!("path '{value}' is not safe")));
    }
    Ok(())
}

pub(crate) fn validate_pr_number(value: &str) -> CliResult<u64> {
    if value.starts_with('-') || value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(CliError::new("PR number must be a positive integer"));
    }
    let parsed = value
        .parse::<u64>()
        .map_err(|_| CliError::new("PR number is too large"))?;
    if parsed == 0 {
        return Err(CliError::new("PR number must be greater than zero"));
    }
    Ok(parsed)
}

pub(crate) fn parse_count(value: &str) -> CliResult<u16> {
    if value.starts_with('-') || value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(CliError::new("log count must be a positive integer"));
    }
    let count = value
        .parse::<u16>()
        .map_err(|_| CliError::new("log count is too large"))?;
    if count == 0 || count > 200 {
        return Err(CliError::new("log count must be between 1 and 200"));
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditOutcome, MemoryAuditor};
    use crate::runner::{CommandOutput, CommandSpec, MemoryRunner};

    struct FailingAuditor;

    impl Auditor for FailingAuditor {
        fn emit(&self, _: &AuditOutcome) -> io::Result<()> {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "audit sink denied",
            ))
        }
    }

    fn ok_runner(program: &str, args: &[&str], stdout: &str) -> MemoryRunner {
        MemoryRunner::new(vec![(
            CommandSpec::new(program, args.iter().map(|s| s.to_string()).collect()),
            CommandOutput {
                status: 0,
                stdout: stdout.to_string(),
                stderr: String::new(),
            },
        )])
    }

    #[test]
    fn unsupported_command_fails_closed_and_audits() {
        let runner = MemoryRunner::new(vec![]);
        let auditor = MemoryAuditor::default();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run_cli(
            vec!["commit".to_string()],
            &runner,
            &auditor,
            &mut out,
            &mut err,
        )
        .unwrap();
        assert_eq!(code, 2);
        assert!(
            String::from_utf8(err)
                .unwrap()
                .contains("unsupported command")
        );
        assert_eq!(auditor.records().len(), 1);
        assert_eq!(auditor.records()[0].verb, "commit");
        assert!(!auditor.records()[0].success);
    }

    #[test]
    fn log_builds_fixed_read_only_git_command() {
        let runner = ok_runner(
            "git",
            &[
                "log",
                "--no-ext-diff",
                "--date=iso-strict",
                "--format=%H %cI %s",
                "-n",
                "5",
                "--",
                "docs",
            ],
            "abc 2026-05-14T00:00:00Z msg\n",
        );
        let auditor = MemoryAuditor::default();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run_cli(
            vec!["log", "5", "--paths", "docs"]
                .into_iter()
                .map(String::from),
            &runner,
            &auditor,
            &mut out,
            &mut err,
        )
        .unwrap();
        assert_eq!(code, 0);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "abc 2026-05-14T00:00:00Z msg\n"
        );
        assert_eq!(runner.calls().len(), 1);
        assert_eq!(auditor.records()[0].verb, "log");
    }

    #[test]
    fn allowed_read_fails_closed_when_audit_emit_fails() {
        let runner = ok_runner(
            "git",
            &[
                "log",
                "--no-ext-diff",
                "--date=iso-strict",
                "--format=%H %cI %s",
                "-n",
                "5",
                "--",
                "docs",
            ],
            "commit output must not be written without audit\n",
        );
        let auditor = FailingAuditor;
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run_cli(
            vec!["log", "5", "--paths", "docs"]
                .into_iter()
                .map(String::from),
            &runner,
            &auditor,
            &mut out,
            &mut err,
        )
        .unwrap();
        assert_eq!(code, 2);
        assert!(out.is_empty());
        let err = String::from_utf8(err).unwrap();
        assert!(err.contains("audit emission failed"));
        assert!(!err.contains("commit output must not be written"));
        assert_eq!(runner.calls().len(), 1);
    }

    #[test]
    fn help_invocation_audits_before_usage() {
        let runner = MemoryRunner::new(vec![]);
        let auditor = MemoryAuditor::default();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run_cli(
            vec!["--help"].into_iter().map(String::from),
            &runner,
            &auditor,
            &mut out,
            &mut err,
        )
        .unwrap();
        assert_eq!(code, 0);
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("Read-only sanctioned agent helper")
        );
        assert!(err.is_empty());
        assert_eq!(auditor.records().len(), 1);
        assert_eq!(auditor.records()[0].event, "EVT-AGENT-READ-HELP");
        assert_eq!(runner.calls().len(), 0);
    }

    #[test]
    fn help_invocation_fails_closed_when_audit_emit_fails() {
        let runner = MemoryRunner::new(vec![]);
        let auditor = FailingAuditor;
        let mut out = Vec::new();
        let mut err = Vec::new();
        let failure = run_cli(
            vec!["--help"].into_iter().map(String::from),
            &runner,
            &auditor,
            &mut out,
            &mut err,
        )
        .unwrap_err();
        assert!(failure.message.contains("audit emission failed"));
        assert!(out.is_empty());
        assert!(err.is_empty());
        assert_eq!(runner.calls().len(), 0);
    }

    #[test]
    fn diff_rejects_option_injected_path() {
        let runner = MemoryRunner::new(vec![]);
        let auditor = MemoryAuditor::default();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run_cli(
            vec![
                "diff",
                "--base",
                "main",
                "--head",
                "HEAD",
                "--paths",
                "--output=/tmp/x",
            ]
            .into_iter()
            .map(String::from),
            &runner,
            &auditor,
            &mut out,
            &mut err,
        )
        .unwrap();
        assert_eq!(code, 2);
        assert!(String::from_utf8(err).unwrap().contains("not safe"));
        assert_eq!(runner.calls().len(), 0);
        assert_eq!(auditor.records()[0].tool, "git");
        assert!(!auditor.records()[0].success);
    }

    #[test]
    fn pr_view_requires_numeric_pr() {
        let runner = MemoryRunner::new(vec![]);
        let auditor = MemoryAuditor::default();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run_cli(
            vec!["pr-view", "feature/foo"].into_iter().map(String::from),
            &runner,
            &auditor,
            &mut out,
            &mut err,
        )
        .unwrap();
        assert_eq!(code, 2);
        assert!(String::from_utf8(err).unwrap().contains("PR number"));
        assert_eq!(runner.calls().len(), 0);
    }
}
