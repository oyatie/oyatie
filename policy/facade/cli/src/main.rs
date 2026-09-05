use std::fs::File;
use std::io::{Read, Write};
use std::process::ExitCode;

use policy_cli::{CommandOutput, qualify_json};

const MAX_PROJECT_BYTES: u64 = 16 * 1024 * 1024;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("policy qualification refused: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let (output, path) = parse_args(std::env::args_os().skip(1))?;
    let file = File::open(path).map_err(|error| format!("open project: {error}"))?;
    let input = read_project(file)?;
    let rendered = qualify_json(&input, output).map_err(|error| format!("{error:?}"))?;
    writeln!(std::io::stdout().lock(), "{rendered}")
        .map_err(|error| format!("write output: {error}"))
}

fn parse_args(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(CommandOutput, std::ffi::OsString), String> {
    let command = args.next().ok_or_else(usage)?;
    let output = match command.to_str() {
        Some("check") => CommandOutput::Report,
        Some("prepare") => CommandOutput::UnsignedBundle,
        _ => return Err(usage()),
    };
    let path = args.next().ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage());
    }
    Ok((output, path))
}

fn read_project(reader: impl Read) -> Result<Vec<u8>, String> {
    let mut input = Vec::new();
    reader
        .take(MAX_PROJECT_BYTES + 1)
        .read_to_end(&mut input)
        .map_err(|error| format!("read project: {error}"))?;
    if input.len() as u64 > MAX_PROJECT_BYTES {
        return Err("project exceeds 16 MiB command input limit".into());
    }
    Ok(input)
}

fn usage() -> String {
    "usage: policy-cli <check|prepare> <project.json>; prepare emits an unsigned candidate; signing requires PreparedPolicy::publish with configured trust and custody".into()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn command_arguments_require_exact_mode_and_path() {
        let args = |values: &[&str]| {
            values
                .iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>()
                .into_iter()
        };
        assert!(matches!(
            parse_args(args(&["check", "project.json"])).unwrap().0,
            CommandOutput::Report
        ));
        assert!(matches!(
            parse_args(args(&["prepare", "project.json"])).unwrap().0,
            CommandOutput::UnsignedBundle
        ));
        for invalid in [
            vec![],
            vec!["check"],
            vec!["publish", "project.json"],
            vec!["check", "project.json", "ignored"],
        ] {
            assert!(parse_args(args(&invalid)).is_err());
        }
    }

    #[test]
    fn project_read_is_bounded_and_preserves_io_refusals() {
        assert_eq!(read_project(&b"{}"[..]).unwrap(), b"{}");
        assert!(read_project(std::io::repeat(0).take(MAX_PROJECT_BYTES + 1)).is_err());
        struct FailedRead;
        impl Read for FailedRead {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("fixture refusal"))
            }
        }
        assert!(read_project(FailedRead).is_err());
    }
}
