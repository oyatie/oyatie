//! `oya-check-doc-coverage` — LEAN-A5 fitness lane.
//!
//! Implements the documentation suite coverage algorithm declared in
//! `docs/decisions/ADR-0063-documentation-suite-coverage.md` §5. Verifies
//! every µservice registered in `[workspace.metadata.oya.microservices]` ships
//! the canonical artifact suite (PRD + Microservice record + Naming ADR + BC
//! registrations + Phase-Spec + Impl-Plan) plus per-pack overlays for every
//! (pack × µservice) pair in `docs/localization-packs/<pack>/pack.yaml`.
//!
//! Runs in `--report-only` mode until M02-P22 exit gate (per
//! `registry/quality/lanes.yaml` lane `lean-a5-doc-coverage`).

pub mod algorithm;
pub mod manifest;
pub mod registry;
pub mod types;

pub use types::{Report, Violation, ViolationKind};

/// Top-level entry: run the full ADR-0063 §5 algorithm against a repo root.
///
/// Returns a `Report`. The caller decides exit code based on report-only vs
/// blocker mode and whether `report.violations` is empty.
pub fn run(repo_root: &std::path::Path) -> anyhow::Result<Report> {
    let mut report = Report::new();
    let registered = registry::read_workspace_microservices(repo_root)?;
    let planned = registry::read_masterplan_catalog(repo_root)?;
    let packs = manifest::read_pack_catalog(repo_root)?;

    algorithm::reconcile_registered_vs_planned(repo_root, &registered, &planned, &mut report);
    algorithm::verify_canonical_suite(repo_root, &registered, &mut report);
    algorithm::verify_pack_overlays(repo_root, &packs, &mut report);
    algorithm::verify_milestone_artifacts(repo_root, &mut report);
    algorithm::verify_section_completeness(repo_root, &mut report);
    algorithm::orphan_scan(repo_root, &registered, &packs, &mut report);

    Ok(report)
}
