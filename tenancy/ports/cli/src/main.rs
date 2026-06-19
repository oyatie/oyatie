//! Tenant-facing `oya` CLI surface per ADR-0167.
//!
//! # Scope
//!
//! This crate ships the TENANT-facing CLI surface. It is distinct from
//! the internal `oya-dev-cli` per ADR-0167 §"Decision":
//!
//! - Tier-A semver-protected per ADR-0037.
//! - Depends on the public SDK only (no `oya-check-*`, no `oya-foundry-*`).
//! - Distributed to tenants via Homebrew tap, apt repo, winget, ghcr.
//! - Built in the workspace as `oya-tenant` to avoid colliding with
//!   the internal `oya-dev-cli` binary target.
//!
//! # Skeleton scope (v0.1)
//!
//! Per ADR-0167 §"Migration / rollout plan" Phase 1, the v0.1 surface
//! ships `auth`, `version`, `completion` only. Remaining command groups
//! (`workflow`, `messenger`, `tasks`, `foundry`, `ontology`, `audit`,
//! `webhook`, `status`) ship at M01.5 / M02 / M03.
//!
//! Each subcommand currently prints an `unimplemented` notice referring
//! the user to the ADR. Full impl tracked under
//! `registry/placeholder-debt/adr-follow-ups.yaml#adr-0167-tenant-cli-commands`.
//!
//! # Naming justification
//!
//! The crate de-brands to `tenancy-cli` (ADR-0562 capability-first home
//! tenancy/ports/cli); the tenant-facing binary NAME `oya-tenant` is
//! deliberately preserved (Tier-A distribution channel id, ADR-0167) and
//! distinguishes this CLI from the internal `oya-dev-cli`.
//!
//! # References
//!
//! - ADR-0167 — tenant-facing CLI binary `oya` (this skeleton).
//! - ADR-0037 — public API stability tiers (Tier-A semver).
//! - ADR-0120 — Rust-first on-prem tooling authority.

#![forbid(unsafe_code)]

use std::io::stdout;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};

/// Tenant-facing `oya` CLI per ADR-0167.
#[derive(Parser, Debug)]
#[command(name = "oya", version, about = "Oyatie tenant CLI (ADR-0167)")]
struct Cli {
    /// Output format: `human` (default), `json`, or `ndjson`.
    #[arg(long, global = true, default_value = "human", env = "OYA_OUTPUT")]
    output: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Authentication commands per ADR-0167 OAuth-2.1 device-code flow.
    #[command(subcommand)]
    Auth(AuthCommand),

    /// Workflow run + status commands (M01.5).
    #[command(subcommand)]
    Workflow(WorkflowCommand),

    /// Messenger send + search commands (M01.5).
    #[command(subcommand)]
    Messenger(MessengerCommand),

    /// Tasks create + list commands (M01.5).
    #[command(subcommand)]
    Tasks(TasksCommand),

    /// Foundry capability invoke + list commands (M02).
    #[command(subcommand)]
    Foundry(FoundryCommand),

    /// Ontology entity get commands (M02).
    #[command(subcommand)]
    Ontology(OntologyCommand),

    /// Audit-chain query commands (M03).
    #[command(subcommand)]
    Audit(AuditCommand),

    /// Webhook endpoint + delivery commands (M02.5, depends on ADR-0169).
    #[command(subcommand)]
    Webhook(WebhookCommand),

    /// Public status surface (M03, depends on ADR-0168).
    Status,

    /// Print build version + API version compatibility.
    Version,

    /// Emit shell-completion scripts.
    Completion { shell: Shell },
}

#[derive(Subcommand, Debug)]
enum AuthCommand {
    /// OAuth-2.1 device-code login (RFC 8628).
    Login,
    /// Revoke local token + clear OS credential store.
    Logout,
    /// Show current tenant + principal.
    Whoami,
}

#[derive(Subcommand, Debug)]
enum WorkflowCommand {
    Run { flow_id: String },
    Status { run_id: String },
}

#[derive(Subcommand, Debug)]
enum MessengerCommand {
    Send {
        #[arg(long)]
        to: String,
        #[arg(long)]
        body: String,
    },
    Search {
        #[arg(long)]
        query: String,
    },
}

#[derive(Subcommand, Debug)]
enum TasksCommand {
    Create {
        #[arg(long)]
        title: String,
    },
    List,
}

#[derive(Subcommand, Debug)]
enum FoundryCommand {
    #[command(subcommand)]
    Capability(FoundryCapabilityCommand),
}

#[derive(Subcommand, Debug)]
enum FoundryCapabilityCommand {
    Invoke {
        capability_id: String,
        #[arg(long)]
        args: String,
    },
    List,
}

#[derive(Subcommand, Debug)]
enum OntologyCommand {
    #[command(subcommand)]
    Entity(OntologyEntityCommand),
}

#[derive(Subcommand, Debug)]
enum OntologyEntityCommand {
    Get { urn: String },
}

#[derive(Subcommand, Debug)]
enum AuditCommand {
    #[command(subcommand)]
    Chain(AuditChainCommand),
}

#[derive(Subcommand, Debug)]
enum AuditChainCommand {
    Query {
        #[arg(long)]
        since: String,
    },
}

#[derive(Subcommand, Debug)]
enum WebhookCommand {
    ListDeliveries {
        #[arg(long)]
        endpoint_id: String,
    },
    Retry {
        #[arg(long)]
        delivery_id: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Version => {
            println!("oya {} (ADR-0167 skeleton)", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Command::Completion { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "oya", &mut stdout());
            ExitCode::SUCCESS
        }
        other => {
            eprintln!(
                "oya: error: command not yet implemented in skeleton: {:?}\n\
                 see ADR-0167 §\"Migration / rollout plan\" for the schedule",
                other
            );
            // Exit-code 1 per ADR-0167 §"Decision" command-surface contract
            // (user error). Once impl lands, this dispatches to the public
            // SDK. Full impl tracked under
            // registry/placeholder-debt/adr-follow-ups.yaml
            //   #adr-0167-tenant-cli-commands.
            ExitCode::from(1)
        }
    }
}
