//! `oya-check-doc-coverage` CLI entry — LEAN-A5 fitness lane.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Verify oyatie documentation suite coverage per ADR-0063"
)]
struct Cli {
    /// Run against the entire workspace. Default: yes.
    #[arg(long, default_value = "true")]
    workspace: bool,

    /// Report-only mode — print violations and exit 0 regardless. Default (until M02-P22).
    #[arg(long, conflicts_with = "blocker")]
    report_only: bool,

    /// Blocker mode — exit nonzero if any violation. Active post-M02-P22 per ADR-0063.
    #[arg(long)]
    blocker: bool,

    /// Repository root. Default: current working directory.
    #[arg(long)]
    repo_root: Option<PathBuf>,
}

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();
    let repo_root = cli
        .repo_root
        .unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let _ = cli.workspace;

    let report = oya_check_doc_coverage::run(&repo_root)?;
    println!("{}", report.render_markdown());

    if cli.blocker && !report.is_clean() {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}
