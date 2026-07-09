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

    let options = parse_dry_run_options(&mut args)?;

    let mut original = String::new();
    std::io::stdin()
        .read_to_string(&mut original)
        .map_err(|err| format!("failed to read original text from stdin: {err}"))?;

    let range = ByteRange::new(options.start, options.end).map_err(|err| err.to_string())?;
    let remediation = Remediation::AutoFix(Edit::new(options.path, range, options.replacement));
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

#[derive(Debug, Default)]
struct DryRunOptions {
    path: String,
    start: usize,
    end: usize,
    replacement: String,
}

fn parse_dry_run_options(args: &mut VecDeque<String>) -> Result<DryRunOptions, String> {
    let mut path = None;
    let mut start = None;
    let mut end = None;
    let mut replacement = None;

    while let Some(flag) = args.pop_front() {
        match flag.as_str() {
            "--path" => set_once(&mut path, "--path", take_value(args, "--path")?)?,
            "--start" => set_once(
                &mut start,
                "--start",
                parse_usize(&take_value(args, "--start")?, "--start")?,
            )?,
            "--end" => set_once(
                &mut end,
                "--end",
                parse_usize(&take_value(args, "--end")?, "--end")?,
            )?,
            "--replacement" => set_once(
                &mut replacement,
                "--replacement",
                take_value(args, "--replacement")?,
            )?,
            _ => return Err(format!("unexpected dry-run flag {flag}\n{}", usage())),
        }
    }

    Ok(DryRunOptions {
        path: require_option(path, "--path")?,
        start: require_option(start, "--start")?,
        end: require_option(end, "--end")?,
        replacement: require_option(replacement, "--replacement")?,
    })
}

fn set_once<T>(slot: &mut Option<T>, name: &str, value: T) -> Result<(), String> {
    if slot.replace(value).is_some() {
        Err(format!("duplicate {name}\n{}", usage()))
    } else {
        Ok(())
    }
}

fn require_option<T>(value: Option<T>, name: &str) -> Result<T, String> {
    value.ok_or_else(|| format!("missing {name}\n{}", usage()))
}

fn take_value(args: &mut VecDeque<String>, name: &str) -> Result<String, String> {
    args.pop_front()
        .ok_or_else(|| format!("missing value for {name}\n{}", usage()))
}

fn parse_usize(value: &str, name: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|err| format!("{name} must be a byte offset: {err}"))
}

fn usage() -> String {
    "usage: oya-bot-autofix dry-run [--path <repo-relative-path> --start <byte> --end <byte> --replacement <text>] < original-file\nflags may be supplied in any order; forbidden: oya-bot-autofix merge | bypass-gates".to_owned()
}
