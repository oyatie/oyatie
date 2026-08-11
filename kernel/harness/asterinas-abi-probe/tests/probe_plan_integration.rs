//! Integration: scaffold probe plan covers matrix rows hermetically (no QEMU/network).

use kernel_asterinas_abi_matrix::{self as matrix, G5Evaluation};
use kernel_asterinas_abi_probe::{
    build_probe_plan, run_scaffold, scaffold_summary_receipt, ProbeItemStatus, ProbeKind,
    PROFILE_TARGETS,
};

#[test]
fn integration_scaffold_plan_matches_matrix_row_count() {
    let root = matrix::parse_matrix().expect("parse matrix");
    let rows = matrix::all_rows(&root).expect("rows");
    let plan = build_probe_plan(&root).expect("plan");
    let surface_items = plan
        .items
        .iter()
        .filter(|i| i.kind == ProbeKind::SurfaceAvailability)
        .count();
    let footprint_items = plan
        .items
        .iter()
        .filter(|i| i.kind == ProbeKind::ComponentFootprint)
        .count();
    assert_eq!(surface_items, rows.len());
    assert_eq!(footprint_items, PROFILE_TARGETS.len());
    assert!(plan.items.iter().all(|i| i.status == ProbeItemStatus::Stubbed));
}

#[test]
fn integration_summary_documents_f1a_consumption() {
    let run = run_scaffold().expect("scaffold");
    let summary = scaffold_summary_receipt(&run);
    let note = summary["f1a_consumption_note"].as_str().unwrap_or_default();
    assert!(note.contains("F1(a)"));
    assert!(note.contains("G5"));
    assert!(matches!(run.g5, G5Evaluation::PendingMeasurement { .. }));
}
