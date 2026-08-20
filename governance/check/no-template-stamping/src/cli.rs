use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Result, anyhow};
use check_no_template_stamping::{enforce_no_template_stamping, format_text_report};

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
    let outcome = enforce_no_template_stamping(&args.root)?;
    let _strict_compat = args.strict;

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&outcome)?),
        OutputFormat::Text => print!("{}", format_text_report(&outcome)),
    }

    if outcome.is_success() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
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
                root = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| anyhow!("--root requires a path value"))?,
                );
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
                    "usage: check-no-template-stamping [--root PATH] [--format json|text] [--strict]\n       violations fail closed by default; --strict is accepted for compatibility"
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn violations_fail_closed_without_strict() {
        let root = temp_root("template-cli-fail-closed");
        let docs = root.join("docs/run");
        fs::create_dir_all(&docs).expect("create docs dir");
        let stamped = "# Title\n## Scope\n- alpha 100\n- beta 200\n| A | B |\n|---|---|\n";
        for name in ["a.md", "b.md", "c.md"] {
            fs::write(docs.join(name), stamped).expect("write stamped doc");
        }

        let exit = run([
            "--root".to_string(),
            root.display().to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("run succeeds");

        assert_eq!(exit, ExitCode::from(1));
        fs::remove_dir_all(root).expect("cleanup temp root");
    }

    fn temp_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
    }
}
