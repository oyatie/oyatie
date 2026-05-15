//! oya-check-dependency-seam binary — runs the composite lane against the
//! current workspace and emits a JSON report to stdout.
//!
//! Day-1 mode: report-only. Exit code is always 0 unless a sub-check with
//! `error` severity fails (none do as of ADR-0092 D14 soak window).

use std::path::PathBuf;
use std::process::ExitCode;

use oya_check_dependency_seam::{WorkspaceContext, render_report_json, run_composite};

fn main() -> ExitCode {
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok()
        .and_then(|p| {
            // CARGO_MANIFEST_DIR points at this crate; walk up two dirs to
            // the workspace root. If the env is not set (running outside
            // cargo), fall back to PWD.
            p.parent().and_then(|p| p.parent()).map(PathBuf::from)
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let ctx = WorkspaceContext::new(workspace_root);
    let report = run_composite(&ctx);
    println!("{}", render_report_json(&report));
    if report.exit_code() == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
