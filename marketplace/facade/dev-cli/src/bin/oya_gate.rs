//! Minimal `oya-gate` binary — builds only architecture-boundaries and
//! workspace-topology validators without pulling in the full oya-dev-cli dep
//! tree (which includes crypto crates the gate lane does not need).
//!
//! This binary is the Buck2-built gate binary for CI use; it exposes:
//!   oya-gate gate validate architecture-boundaries [args…]
//!   oya-gate gate validate workspace-topology [args…]

use std::process::ExitCode;

// Pull in the self-contained helper module that the gate commands rely on.
#[path = "../workspace_manifest.rs"]
mod workspace_manifest;

// Provide the minimal `usage()` stub that workspace_topology_gate calls.
pub(crate) fn usage() -> String {
    "Usage: oya-gate gate validate <architecture-boundaries|workspace-topology> [--repo-root <path>] [--workspace <Cargo.toml>] [--registry <registry/catalog>] [--report <path>] [--format <text|json>]".to_string()
}

#[path = "../commands/gate/architecture_boundaries.rs"]
mod architecture_boundaries;

#[path = "../workspace_topology_gate.rs"]
mod workspace_topology_gate;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    // Accept: oya-gate gate validate <lane> [args…]
    // Also:   oya-gate validate <lane> [args…]  (without leading "gate")
    let rest: Vec<String> = args.into_iter().skip(1).collect();

    // Determine which lane and remaining args — avoid borrow-then-move by
    // copying the lane string before consuming `rest`.
    let (skip, lane_owned): (usize, String) = {
        match (
            rest.first().map(String::as_str),
            rest.get(1).map(String::as_str),
            rest.get(2).map(String::as_str),
        ) {
            (Some("gate"), Some("validate"), Some(lane)) => (3, lane.to_string()),
            (Some("validate"), Some(lane), _) => (2, lane.to_string()),
            _ => {
                eprintln!("{}", usage());
                return ExitCode::from(2);
            }
        }
    };
    let lane: &str = &lane_owned;
    let remaining: Vec<String> = rest.into_iter().skip(skip).collect();

    match lane {
        "architecture-boundaries" => architecture_boundaries::run(remaining),
        "workspace-topology" => {
            match workspace_topology_gate::parse_workspace_topology_validate_args(remaining) {
                Ok(parsed) => {
                    match workspace_topology_gate::validate_workspace_topology_gate(parsed) {
                        Ok(report) => {
                            for finding in &report.findings {
                                eprintln!(
                                    "workspace-topology {}: {}",
                                    finding.rule.as_str(),
                                    finding.detail
                                );
                            }
                            let count = report.findings.len();
                            println!(
                                "workspace-topology scan: {} members scanned, {} findings ({})",
                                report.members_scanned,
                                count,
                                if report.enforced {
                                    "enforce"
                                } else {
                                    "report-only"
                                }
                            );
                            if report.enforced && count > 0 {
                                eprintln!(
                                    "workspace-topology validation failed: {count} topology violations"
                                );
                                ExitCode::FAILURE
                            } else {
                                ExitCode::SUCCESS
                            }
                        }
                        Err(message) => {
                            eprintln!("workspace-topology validation failed: {message}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(message) => {
                    eprintln!("{message}");
                    ExitCode::from(2)
                }
            }
        }
        other => {
            eprintln!("unknown gate lane: {other}");
            eprintln!("{}", usage());
            ExitCode::from(2)
        }
    }
}
