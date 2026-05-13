use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "oya-shared-semver-check-cli",
    about = "LEAN-A4: cargo-semver-checks wrapper with rename-baseline-reset classifier"
)]
struct Cli {
    /// Path to rename map TSV (old<TAB>new); used to classify BASELINE-RESET failures.
    #[arg(long)]
    rename_map: Option<String>,

    /// Output format: text (default) or json.
    #[arg(long, default_value = "text")]
    format: String,

    /// Run in report-only mode (exit 0 regardless of violations).
    #[arg(long)]
    report_only: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    println!(
        "semver-check: SCAFFOLD (populated in Shard 1) rename_map={:?} format={} report_only={}",
        cli.rename_map, cli.format, cli.report_only
    );
    Ok(())
}
