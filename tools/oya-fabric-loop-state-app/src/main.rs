//! `oya-fabric-loop-state` — read-only snapshot of the two-plane drive-loop
//! state through the plane ports. LOCAL BRIDGE FEEDBACK ONLY (retirement-marked
//! per the founder CLI directive 2026-06-09): this bin carries NO merge, CI, or
//! plan authority, mutates nothing, and reads exclusively through
//! `CoordinationPlanePort` / `ExecutionPlanePort`. The named owned cutover
//! target for both planes is the cloud/cloud-ci-owned loop-state service
//! recorded in `specs/fabric-drive-loop-state.json#cutover_target`.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use oya_fabric_loop_state_app::{
    CoordinationPlanePort, CutoverTarget, DEFAULT_DURABLE_PLANE_ROOT,
    DEFAULT_OPERATIONAL_PLANE_ROOT, ExecutionPlanePort, FsCoordinationStore, FsExecutionStore,
};

fn main() -> ExitCode {
    let mut repo_root = PathBuf::from(".");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = args.next() else {
                    eprintln!("--repo-root requires a path");
                    return ExitCode::FAILURE;
                };
                repo_root = PathBuf::from(value);
            }
            "--help" | "-h" => {
                println!(
                    "oya-fabric-loop-state [--repo-root PATH] — read-only two-plane loop-state snapshot (local bridge feedback; no authority)"
                );
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument: {other}");
                return ExitCode::FAILURE;
            }
        }
    }

    let coordination = FsCoordinationStore::open(repo_root.join(DEFAULT_DURABLE_PLANE_ROOT));
    let execution = FsExecutionStore::open(repo_root.join(DEFAULT_OPERATIONAL_PLANE_ROOT));

    let cards = match coordination.cards() {
        Ok(cards) => cards,
        Err(err) => {
            eprintln!("coordination plane read failed: {err}");
            return ExitCode::FAILURE;
        }
    };

    let target = CutoverTarget::canonical();
    println!("two-plane drive-loop state snapshot (read-only bridge view)");
    println!(
        "  coordination plane: {} ({} cards)",
        coordination.descriptor().store_root,
        cards.len()
    );
    println!(
        "  execution plane:    {}",
        execution.descriptor().store_root
    );
    for card in &cards {
        let claim = match execution.active_claim(&card.card_id) {
            Ok(claim) => claim,
            Err(err) => {
                eprintln!("execution plane read failed for {}: {err}", card.card_id);
                return ExitCode::FAILURE;
            }
        };
        let lane = claim
            .map(|c| format!(" [claimed by {}]", c.lane_id))
            .unwrap_or_default();
        println!(
            "  - {} status={} deps={:?}{}",
            card.card_id,
            card.status.as_str(),
            card.depends_on,
            lane
        );
    }
    println!(
        "  cutover target: {} (home {}, owner {})",
        target.service_name, target.destination_home, target.owner
    );
    ExitCode::SUCCESS
}
