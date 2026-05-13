use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "oya-shared-architecture-check-cli",
    about = "LEAN-A1: Clean Architecture enforcement orchestrator (7 subcommands)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: text (default) or json.
    #[arg(long, default_value = "text", global = true)]
    format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Check each crate's declared layer matches its actual code shape.
    LayerCorrectness,
    /// Enforce 12-value layer dependency direction matrix (§2.2.5).
    DependencyDirection,
    /// Verify no two crates share the same <vertical>-<bc>-<layer> tuple.
    NamingCollision,
    /// Verify every Cargo.toml has a valid [package.metadata.oya] block.
    MetadataSchema,
    /// Check Cargo.lock contains zero references to old crate names.
    LockfileParity,
    /// Check [lib] name (snake) equals snake_case of [package] name (kebab).
    LibNameParity,
    /// Check every oya-check-* crate name matches ^oya_check_[a-z][a-z0-9_]*$.
    CheckNamespace,
    /// Run all 7 subcommands; non-zero exit if any fails.
    #[command(alias = "all")]
    Report,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::LayerCorrectness => {
            println!("layer-correctness: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::DependencyDirection => {
            println!("dependency-direction: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::NamingCollision => {
            println!("naming-collision: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::MetadataSchema => {
            println!("metadata-schema: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::LockfileParity => {
            println!("lockfile-parity: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::LibNameParity => {
            println!("lib-name-parity: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::CheckNamespace => {
            println!("check-namespace: SCAFFOLD (populated in Shard 1)");
            Ok(())
        }
        Commands::Report => {
            println!("report: SCAFFOLD (populated in Shard 1) -- report-only mode");
            Ok(())
        }
    }
}
