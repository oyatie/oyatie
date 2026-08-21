use std::io::Write;

use crate::audit::{AuditOutcome, Auditor};
use crate::runner::{CommandRunner, CommandSpec};
use crate::{CliError, CliResult, parse_count, validate_path, validate_ref};

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
        "log".to_string(),
        "--no-ext-diff".to_string(),
        "--date=iso-strict".to_string(),
        "--format=%H %cI %s".to_string(),
    ];
    if let Some(range) = parsed.range {
        git_args.push(range);
    } else {
        git_args.push("-n".to_string());
        git_args.push(parsed.count.unwrap_or(20).to_string());
    }
    git_args.push("--".to_string());
    git_args.extend(parsed.paths);
    execute("log", git_args, runner, auditor, stdout, stderr)
}

struct ParsedLog {
    count: Option<u16>,
    range: Option<String>,
    paths: Vec<String>,
}

fn parse_args(args: &[String]) -> CliResult<ParsedLog> {
    let mut count = None;
    let mut range = None;
    let mut paths = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--range" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| CliError::new("--range requires a value"))?;
                validate_ref(value, "range")?;
                range = Some(value.clone());
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
            value if !value.starts_with('-') && count.is_none() && range.is_none() => {
                count = Some(parse_count(value)?);
            }
            other => return Err(CliError::new(format!("unexpected log argument '{other}'"))),
        }
        i += 1;
    }
    Ok(ParsedLog {
        count,
        range,
        paths,
    })
}

fn execute<R, A, W, E>(
    verb: &str,
    git_args: Vec<String>,
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
    let spec = CommandSpec::new("git", git_args);
    let output = runner
        .run(&spec)
        .map_err(|err| CliError::new(err.to_string()))?;
    let outcome = if output.status == 0 {
        AuditOutcome::success(verb, "git", &spec.args, output.status)
    } else {
        AuditOutcome::failure(
            verb,
            "git",
            &spec.args,
            Some(output.status),
            "git read command failed",
        )
    };
    auditor
        .emit(&outcome)
        .map_err(|err| CliError::new(format!("audit emission failed: {err}")))?;
    write!(stdout, "{}", output.stdout).map_err(crate::io_err)?;
    write!(stderr, "{}", output.stderr).map_err(crate::io_err)?;
    Ok(output.status)
}
