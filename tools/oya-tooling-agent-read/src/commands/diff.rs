use std::io::Write;

use crate::audit::{AuditOutcome, Auditor};
use crate::runner::{CommandRunner, CommandSpec};
use crate::{CliError, CliResult, validate_path, validate_ref};

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
    let parsed = parse_args(args)?;
    let mut git_args = vec![
        "diff".to_string(),
        "--no-ext-diff".to_string(),
        format!("{}..{}", parsed.base, parsed.head),
        "--".to_string(),
    ];
    git_args.extend(parsed.paths);
    let spec = CommandSpec::new("git", git_args);
    let output = runner
        .run(&spec)
        .map_err(|err| CliError::new(err.to_string()))?;
    let outcome = if output.status == 0 {
        AuditOutcome::success("diff", "git", &spec.args, output.status)
    } else {
        AuditOutcome::failure(
            "diff",
            "git",
            &spec.args,
            Some(output.status),
            "git diff failed",
        )
    };
    auditor
        .emit(&outcome)
        .map_err(|err| CliError::new(format!("audit emission failed: {err}")))?;
    write!(stdout, "{}", output.stdout).map_err(crate::io_err)?;
    write!(stderr, "{}", output.stderr).map_err(crate::io_err)?;
    Ok(output.status)
}

struct ParsedDiff {
    base: String,
    head: String,
    paths: Vec<String>,
}

fn parse_args(args: &[String]) -> CliResult<ParsedDiff> {
    let mut base = None;
    let mut head = None;
    let mut paths = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| CliError::new("--base requires a value"))?;
                validate_ref(value, "base")?;
                base = Some(value.clone());
            }
            "--head" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| CliError::new("--head requires a value"))?;
                validate_ref(value, "head")?;
                head = Some(value.clone());
            }
            "--paths" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::new("--paths requires at least one path"));
                }
                while i < args.len() {
                    validate_path(&args[i])?;
                    paths.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            other => return Err(CliError::new(format!("unexpected diff argument '{other}'"))),
        }
        i += 1;
    }
    Ok(ParsedDiff {
        base: base.ok_or_else(|| CliError::new("diff requires --base"))?,
        head: head.ok_or_else(|| CliError::new("diff requires --head"))?,
        paths,
    })
}
