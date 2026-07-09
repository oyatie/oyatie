#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::io::Read;
use std::process::ExitCode;

use oya_bot_autofix_app::{Action, BotPolicy, DryRunInput, render_dry_run};
use oya_ci_gate_contract::{ByteRange, Edit, Remediation};

fn main() -> ExitCode {
    match run() {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<String, String> {
    let mut args: VecDeque<String> = std::env::args().skip(1).collect();
    let Some(command) = args.pop_front() else {
        return Err(usage());
    };

    match command.as_str() {
        "dry-run" => run_dry_run(args),
        "merge" => deny(Action::MergePullRequest),
        "bypass-gates" => deny(Action::BypassGates),
        _ => Err(usage()),
    }
}

fn run_dry_run(mut args: VecDeque<String>) -> Result<String, String> {
    BotPolicy::propose_only()
        .authorize(Action::DryRun)
        .map_err(|err| err.to_string())?;

    let path = take_value(&mut args, "--path")?;
    let start = parse_usize(&take_value(&mut args, "--start")?, "--start")?;
    let end = parse_usize(&take_value(&mut args, "--end")?, "--end")?;
    let replacement = take_value(&mut args, "--replacement")?;
    if !args.is_empty() {
        return Err(format!(
            "unexpected arguments after dry-run options: {args:?}"
        ));
    }

    let mut original = String::new();
    std::io::stdin()
        .read_to_string(&mut original)
        .map_err(|err| format!("failed to read original text from stdin: {err}"))?;

    let range = ByteRange::new(start, end).map_err(|err| err.to_string())?;
    let remediation = Remediation::AutoFix(Edit::new(path, range, replacement));
    let report = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: &original,
    })
    .map_err(|err| err.to_string())?;
    Ok(report.diff)
}

fn deny(action: Action) -> Result<String, String> {
    BotPolicy::propose_only()
        .authorize(action)
        .map(|()| String::new())
        .map_err(|err| err.to_string())
}

fn take_value(args: &mut VecDeque<String>, name: &str) -> Result<String, String> {
    let Some(flag) = args.pop_front() else {
        return Err(format!("missing {name}\n{}", usage()));
    };
    if flag != name {
        return Err(format!("expected {name}, got {flag}\n{}", usage()));
    }
    args.pop_front()
        .ok_or_else(|| format!("missing value for {name}\n{}", usage()))
}

fn parse_usize(value: &str, name: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|err| format!("{name} must be a byte offset: {err}"))
}

fn usage() -> String {
    "usage: oya-bot-autofix dry-run --path <repo-relative-path> --start <byte> --end <byte> --replacement <text> < original-file\nforbidden: oya-bot-autofix merge | bypass-gates".to_owned()
}
