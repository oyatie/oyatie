#![forbid(unsafe_code)]

use std::process::ExitCode;

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use buck_test_wiring_app::{
    CommandMode, Options, Outcome, discover_repo_root, render_unsupported_member_diagnostic, run,
};

#[derive(Debug, Parser)]
#[command(name = "buck-test-wiring")]
#[command(about = "Local bridge generator for ADR-0540 missing rust_test BUCK wiring")]
struct Cli {
    #[arg(long)]
    list: bool,
    #[arg(long)]
    apply: bool,
    #[arg(long)]
    check: bool,
    #[arg(long)]
    limit: Option<usize>,
    #[arg(long)]
    root: Option<String>,
}

impl Cli {
    fn mode(&self) -> Result<CommandMode> {
        let selected = [self.list, self.apply, self.check]
            .into_iter()
            .filter(|enabled| *enabled)
            .count();
        if selected != 1 {
            bail!("select exactly one mode: --list, --apply, or --check");
        }
        if self.list {
            Ok(CommandMode::List)
        } else if self.apply {
            Ok(CommandMode::Apply)
        } else if self.check {
            Ok(CommandMode::Check)
        } else {
            Err(anyhow!("unreachable mode selection"))
        }
    }
}

fn main() -> ExitCode {
    match run_main() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run_main() -> Result<ExitCode> {
    let cli = Cli::parse();
    let mode = cli.mode()?;
    let cwd = std::env::current_dir()?;
    let repo_root = discover_repo_root(&cwd)?;
    let outcome = run(
        &repo_root,
        Options {
            mode,
            root_filter: cli.root,
            limit: cli.limit,
        },
    )?;

    match outcome {
        Outcome::Listed(candidates) => {
            for candidate in candidates {
                println!(
                    "{}\t{}",
                    candidate.member_path,
                    candidate.target_labels.join(" ")
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        Outcome::Checked(report) => {
            for diagnostic in &report.diagnostics {
                eprintln!("{}", render_unsupported_member_diagnostic(diagnostic));
            }

            if report.candidates.is_empty() {
                println!("no rust_test wiring candidates");
                Ok(ExitCode::SUCCESS)
            } else {
                for candidate in &report.candidates {
                    eprintln!(
                        "{}\t{}",
                        candidate.member_path,
                        candidate.target_labels.join(" ")
                    );
                }
                eprintln!(
                    "{} rust_test wiring candidates remain",
                    report.candidates.len()
                );
                Ok(ExitCode::from(1))
            }
        }
        Outcome::Applied(applied) => {
            for member in applied {
                println!("{}\t{}", member.member_path, member.target_labels.join(" "));
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}
