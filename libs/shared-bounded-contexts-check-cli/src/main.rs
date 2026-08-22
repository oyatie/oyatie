use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "shared-bounded-contexts-check-cli",
    about = "LEAN-A2: BC registry validation + cross-vertical refusal + transitive walker + BC overlap governance"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate all crate bounded_context fields are registered in workspace metadata.
    RegistryValidation,
    /// Refuse direct cross-vertical dependencies (vertical-A -> vertical-B where A != B).
    CrossVerticalRefusal,
    /// Walk transitive deps; refuse shared-crate -> vertical-crate edges.
    TransitiveWalker,
    /// Check public_layers allowlist at every cross-vertical hop.
    PublicLayersHop,
    /// Run Jaro-Winkler similarity check on all BC names; flag >0.85 for manual review.
    OverlapGovernance,
    /// Run all checks; non-zero exit if any blocking check fails.
    #[command(alias = "all")]
    Report,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::RegistryValidation => {
            println!("registry-validation: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::CrossVerticalRefusal => {
            println!("cross-vertical-refusal: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::TransitiveWalker => {
            println!("transitive-walker: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::PublicLayersHop => {
            println!("public-layers-hop: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::OverlapGovernance => {
            println!("overlap-governance: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::Report => {
            println!("report: SCAFFOLD (populated in Shard 1) -- report-only mode");
            Ok(())
        }
    }
}
