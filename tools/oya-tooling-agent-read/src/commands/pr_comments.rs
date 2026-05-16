use std::io::Write;

use crate::audit::{AuditOutcome, Auditor};
use crate::runner::{CommandRunner, CommandSpec};
use crate::{CliError, CliResult, validate_pr_number};

pub fn run<R, A, W, E>(
    args: &[String],
    runner: &R,
    auditor: &A,
    stdout: &mut W,
    stderr: &mut E,
) -> CliResult<i32>
where
    R: CommandRunner,
    A: Auditor,
    W: Write,
    E: Write,
{
    if args.len() != 1 {
        return Err(CliError::new("pr-comments requires exactly one PR number"));
    }
    validate_pr_number(&args[0])?;
    let gh_args = vec![
        "pr".to_string(),
        "view".to_string(),
        args[0].clone(),
        "--comments".to_string(),
        "--json".to_string(),
        "number,comments,reviews".to_string(),
    ];
    let spec = CommandSpec::new("gh", gh_args);
    let output = runner
        .run(&spec)
        .map_err(|err| CliError::new(err.to_string()))?;
    let outcome = if output.status == 0 {
        AuditOutcome::success("pr-comments", "gh", &spec.args, output.status)
    } else {
        AuditOutcome::failure(
            "pr-comments",
            "gh",
            &spec.args,
            Some(output.status),
            "gh pr comments failed",
        )
    };
    auditor
        .emit(&outcome)
        .map_err(|err| CliError::new(format!("audit emission failed: {err}")))?;
    write!(stdout, "{}", output.stdout).map_err(crate::io_err)?;
    write!(stderr, "{}", output.stderr).map_err(crate::io_err)?;
    Ok(output.status)
}
