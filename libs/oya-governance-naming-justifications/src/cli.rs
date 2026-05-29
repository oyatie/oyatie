use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Result, anyhow};
use oya_governance_naming_justifications::{enforce_naming_justifications, format_text_report};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Json,
    Text,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CliArgs {
    root: PathBuf,
    format: OutputFormat,
    strict: bool,
}

fn main() -> ExitCode {
    match run(std::env::args().skip(1)) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<ExitCode> {
    let Some(args) = parse_args(args)? else {
        return Ok(ExitCode::SUCCESS);
    };
    let outcome = enforce_naming_justifications(&args.root)?;

    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&outcome)?);
        }
        OutputFormat::Text => {
            print!("{}", format_text_report(&outcome));
        }
    }

    if args.strict && !outcome.is_success() {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Option<CliArgs>> {
    let mut root = PathBuf::from(".");
    let mut format = OutputFormat::Text;
    let mut strict = false;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--root" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--root requires a path value"))?;
                root = PathBuf::from(value);
            }
            "--format" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("--format requires json or text"))?;
                format = match value.as_str() {
                    "json" => OutputFormat::Json,
                    "text" => OutputFormat::Text,
                    other => return Err(anyhow!("unsupported --format value: {other}")),
                };
            }
            "--strict" => strict = true,
            "--help" | "-h" => {
                println!(
                    "usage: oya-governance-naming-justifications [--root PATH] [--format json|text] [--strict]"
                );
                return Ok(None);
            }
            other => return Err(anyhow!("unknown argument: {other}")),
        }
    }

    Ok(Some(CliArgs {
        root,
        format,
        strict,
    }))
}
