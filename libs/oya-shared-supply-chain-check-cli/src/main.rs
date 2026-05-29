use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "oya-shared-supply-chain-check-cli",
    about = "LEAN-A3: wraps cargo deny check with augmented JSON output"
)]
struct Cli {
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
        "supply-chain-check: SCAFFOLD (populated in Shard 1) format={} report_only={}",
        cli.format, cli.report_only
    );
    Ok(())
}
