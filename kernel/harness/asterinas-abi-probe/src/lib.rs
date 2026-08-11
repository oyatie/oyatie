#![forbid(unsafe_code)]
//! Probe harness skeleton for the Asterinas ABI / kernel-service matrix (A1).
//!
//! Today this crate:
//! - loads/validates the embedded matrix
//! - builds a deterministic probe plan from matrix rows + `components_profiled`
//! - emits stub outcomes (`Stubbed`) so hermetic tests pass without live QEMU
//!
//! Tomorrow (QEMU-provable path, no live hardware required):
//! - reuse `kernel-asterinas-real-boot` ISO fetch/verify + QEMU-TCG boot
//! - run guest-side surface probes and component strace/seccomp profiles
//! - write receipts that flip `available_on_asterinas_pin` from `unknown` → `present|gap`
//!
//! The optional `[[bin]]` is a local-bridge harness receipt emitter (same class as
//! `asterinas-real-boot`), not a product CLI capability surface. The library API is
//! the sanctioned automation surface; CLI surfaces remain retirement-marked.
//!
//! data_class: PUBLIC

use kernel_asterinas_abi_matrix::{
    self as matrix, G5Evaluation, MatrixError, MatrixRow, REQUIRED_SURFACES,
};
use kernel_asterinas_boundary as pin;
use serde_json::Value;

/// Preferred evidence path (plan law): QEMU-TCG against the pinned ISO.
/// data_class: PUBLIC
pub const PREFERRED_EVIDENCE_PATH: &str = "qemu-tcg-against-pinned-iso";

/// Re-export closed profiled-component set from the matrix crate (single authority).
/// data_class: PUBLIC
pub const PROFILE_TARGETS: [&str; 4] = matrix::PROFILED_COMPONENTS;

/// One planned probe against a matrix row (or a component footprint aggregate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbePlanItem {
    /// Probe kind. data_class: PUBLIC
    pub kind: ProbeKind,
    /// Matrix row id or component id. data_class: PUBLIC
    pub target_id: String,
    /// Surface key when kind is SurfaceAvailability. data_class: PUBLIC
    pub surface: Option<String>,
    /// Stubbed until measured receipts exist. data_class: PUBLIC
    pub status: ProbeItemStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeKind {
    /// Guest-side check that a surface row is present/enforcing on the Asterinas pin.
    SurfaceAvailability,
    /// Host/guest footprint capture for a profiled component (strace/seccomp).
    ComponentFootprint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeItemStatus {
    /// Not yet executed — scaffold default.
    Stubbed,
    /// Reserved for future measured receipts.
    Measured,
}

/// Deterministic probe plan derived from the matrix (no I/O).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbePlan {
    /// Pinned release tag. data_class: PUBLIC
    pub release_tag: String,
    /// Pinned boot ISO asset name. data_class: PUBLIC
    pub boot_iso_asset: String,
    /// Preferred evidence path id. data_class: PUBLIC
    pub evidence_path: String,
    /// Whether live hardware is required (always false for QEMU-TCG path). data_class: PUBLIC
    pub live_hardware_required: bool,
    /// Ordered probe items. data_class: PUBLIC
    pub items: Vec<ProbePlanItem>,
    /// Stable F1(a) consumption prose. data_class: PUBLIC
    pub f1a_consumption_note: String,
}

/// Build the scaffold probe plan: one SurfaceAvailability item per matrix row, plus one
/// ComponentFootprint item per `components_profiled` entry (closed set validated by matrix).
pub fn build_probe_plan(root: &Value) -> Result<ProbePlan, MatrixError> {
    matrix::validate_matrix(root)?;
    let rows = matrix::all_rows(root)?;
    let components = matrix::profiled_component_ids(root)?;
    let mut items = Vec::with_capacity(rows.len() + components.len());
    for row in &rows {
        items.push(ProbePlanItem {
            kind: ProbeKind::SurfaceAvailability,
            target_id: row.id.clone(),
            surface: Some(row.surface.clone()),
            status: ProbeItemStatus::Stubbed,
        });
    }
    for component in components {
        items.push(ProbePlanItem {
            kind: ProbeKind::ComponentFootprint,
            target_id: component,
            surface: None,
            status: ProbeItemStatus::Stubbed,
        });
    }
    Ok(ProbePlan {
        release_tag: pin::RELEASE_TAG.to_string(),
        boot_iso_asset: pin::BOOT_ISO_ASSET.to_string(),
        evidence_path: PREFERRED_EVIDENCE_PATH.to_string(),
        live_hardware_required: false,
        items,
        f1a_consumption_note: matrix::f1a_consumption_note().to_string(),
    })
}

/// Skeleton receipt for a single stubbed probe — honest about not measuring yet.
pub fn stub_probe_receipt(item: &ProbePlanItem) -> Value {
    serde_json::json!({
        "$schema": "https://docs.oyatie.com/schemas/asterinas-abi-probe-receipt.v0.1.0.json",
        "receipt_type": "abi-probe",
        "status": "stubbed",
        "kind": match item.kind {
            ProbeKind::SurfaceAvailability => "surface_availability",
            ProbeKind::ComponentFootprint => "component_footprint",
        },
        "target_id": item.target_id,
        "surface": item.surface,
        "measured": false,
        "available_on_asterinas_pin": Value::Null,
        "evidence_path": PREFERRED_EVIDENCE_PATH,
        "live_hardware_required": false,
        "notes": "Scaffold stub — QEMU-TCG probe not yet executed; unknown availability remains valid. Scaffold ≠ green matrix.",
    })
}

/// Aggregate scaffold run: validate matrix, build plan, emit stub receipts, evaluate G5.
#[derive(Debug, Clone)]
pub struct ScaffoldRun {
    /// Deterministic probe plan. data_class: PUBLIC
    pub plan: ProbePlan,
    /// Stub receipts (not measured). data_class: PUBLIC
    pub stub_receipts: Vec<Value>,
    /// G5 evaluation (PendingMeasurement on scaffold unknowns). data_class: PUBLIC
    pub g5: G5Evaluation,
    /// Flattened matrix rows. data_class: PUBLIC
    pub rows: Vec<MatrixRow>,
}

pub fn run_scaffold() -> Result<ScaffoldRun, MatrixError> {
    let root = matrix::parse_matrix()?;
    let plan = build_probe_plan(&root)?;
    let stub_receipts = plan.items.iter().map(stub_probe_receipt).collect();
    let g5 = matrix::evaluate_g5(&root)?;
    let rows = matrix::all_rows(&root)?;
    Ok(ScaffoldRun {
        plan,
        stub_receipts,
        g5,
        rows,
    })
}

/// Summarize scaffold run as a JSON receipt (by-reference; no large artifacts).
pub fn scaffold_summary_receipt(run: &ScaffoldRun) -> Value {
    let surfaces_covered: Vec<&str> = REQUIRED_SURFACES.to_vec();
    let stubbed = run
        .stub_receipts
        .iter()
        .filter(|r| r["status"] == "stubbed")
        .count();
    let g5_json = match &run.g5 {
        G5Evaluation::PendingMeasurement {
            unknown_g5_row_ids,
        } => serde_json::json!({
            "status": "pending_measurement",
            "unknown_g5_row_ids": unknown_g5_row_ids,
            "note": "unknown does not fire G5 and must not serialize as clear; F1(a) remains blocked on measurement",
        }),
        G5Evaluation::Clear => serde_json::json!({
            "status": "clear",
            "note": "all G5 rows measured present with no gaps",
        }),
        G5Evaluation::Fired { gap_row_ids } => serde_json::json!({
            "status": "fired",
            "gap_row_ids": gap_row_ids,
        }),
    };
    serde_json::json!({
        "$schema": "https://docs.oyatie.com/schemas/asterinas-abi-probe-scaffold-receipt.v0.1.0.json",
        "receipt_type": "abi-probe-scaffold",
        "wave": "A1-W0-entry",
        "release_tag": run.plan.release_tag,
        "boot_iso_asset": run.plan.boot_iso_asset,
        "evidence_path": run.plan.evidence_path,
        "live_hardware_required": run.plan.live_hardware_required,
        "surfaces_covered": surfaces_covered,
        "probe_item_count": run.plan.items.len(),
        "stubbed_receipt_count": stubbed,
        "measured_receipt_count": 0,
        "g5_evaluation": g5_json,
        "claim_posture": {
            "asterinas_is_canonical_node_kernel": false,
            "blocked_on": "F1(a)",
        },
        "pool_matrix_notes_recorded": true,
        "f1a_consumption_note": run.plan.f1a_consumption_note,
        "measured_today": [
            "matrix schema + column contract",
            "four surfaces present",
            "G5-trigger flags including cgroup delegation/enforcement rows",
            "pool-matrix notes",
            "probe plan enumeration",
        ],
        "stubbed_today": [
            "live strace/seccomp footprints",
            "guest-side QEMU surface probes",
            "available_on_asterinas_pin transitions",
        ],
        "scaffold_is_not": "green_matrix",
    })
}

/// Future impure entry: document the QEMU argv surface without spawning (skeleton).
/// Real boots stay in `kernel-asterinas-real-boot`; this only records the intended coupling.
pub fn qemu_probe_coupling_note() -> Value {
    serde_json::json!({
        "preferred_evidence_path": PREFERRED_EVIDENCE_PATH,
        "iso_pin": {
            "release_tag": pin::RELEASE_TAG,
            "boot_iso_asset": pin::BOOT_ISO_ASSET,
            "download_url": pin::BOOT_ISO_DOWNLOAD_URL,
        },
        "accel": "tcg",
        "live_hardware_required": false,
        "depends_on_harness": "kernel-asterinas-real-boot",
        "status": "skeleton-not-spawned",
        "next_step": "After ISO fetch-verify + boot-ready marker, run guest surface probes and write measured receipts that update available_on_asterinas_pin.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_run_is_hermetic_and_stubbed() {
        let run = run_scaffold().expect("scaffold");
        assert!(!run.plan.items.is_empty());
        assert!(run
            .plan
            .items
            .iter()
            .all(|i| i.status == ProbeItemStatus::Stubbed));
        assert_eq!(run.plan.evidence_path, PREFERRED_EVIDENCE_PATH);
        assert!(!run.plan.live_hardware_required);
        assert_eq!(run.stub_receipts.len(), run.plan.items.len());
        assert!(matches!(run.g5, G5Evaluation::PendingMeasurement { .. }));
    }

    #[test]
    fn plan_covers_four_surfaces_and_profiled_components() {
        let root = matrix::parse_matrix().unwrap();
        let plan = build_probe_plan(&root).unwrap();
        for surface in REQUIRED_SURFACES {
            assert!(
                plan.items.iter().any(|i| {
                    i.kind == ProbeKind::SurfaceAvailability
                        && i.surface.as_deref() == Some(surface)
                }),
                "missing surface probes for {surface}"
            );
        }
        for component in PROFILE_TARGETS {
            assert!(
                plan.items.iter().any(|i| {
                    i.kind == ProbeKind::ComponentFootprint && i.target_id == component
                }),
                "missing footprint probe for {component}"
            );
        }
    }

    #[test]
    fn summary_receipt_refuses_canonical_claim_and_clear_on_unknowns() {
        let run = run_scaffold().unwrap();
        let summary = scaffold_summary_receipt(&run);
        assert_eq!(
            summary["claim_posture"]["asterinas_is_canonical_node_kernel"],
            false
        );
        assert_eq!(summary["measured_receipt_count"], 0);
        assert!(summary["stubbed_receipt_count"].as_u64().unwrap() > 0);
        assert_eq!(summary["g5_evaluation"]["status"], "pending_measurement");
        assert_ne!(summary["g5_evaluation"]["status"], "clear");
    }

    #[test]
    fn qemu_coupling_is_documented_not_spawned() {
        let note = qemu_probe_coupling_note();
        assert_eq!(note["status"], "skeleton-not-spawned");
        assert_eq!(note["accel"], "tcg");
        assert_eq!(note["live_hardware_required"], false);
    }
}
