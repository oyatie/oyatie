//! `oya gen <artifact>` — generative projections derived from canonical
//! sources (ADR-0364). The masterplan is a GENERATED projection of the ADR
//! decision log: `oya gen masterplan` reads accepted `planning_impact: true`
//! ADRs, topo-sorts by `depends_on`, groups by `milestone`, and emits each
//! deliverable as a roadmap line. Mirrors how `doc` / `catalog` dispatch from
//! the root CLI (crate::run_cli_from_env).

use std::process::ExitCode;

pub(crate) mod board_sync;
pub(crate) mod masterplan;

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("board-sync") => board_sync::run(args.collect(), usage),
        Some("masterplan") => masterplan::run(args.collect(), usage),
        _ => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}
