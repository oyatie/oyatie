#![forbid(unsafe_code)]
//! Versioned four-surface Asterinas ABI matrix (A1 / W0-entry evidence scaffold).
//!
//! Surfaces: syscalls, procfs/sysfs/cgroupfs files, netlink families, mount semantics.
//! This crate embeds and validates the matrix; it does **not** claim Asterinas is the
//! canonical node kernel (blocked on founder ADR F1(a)).
//!
//! data_class: PUBLIC — measurement contract + pin identity only.

use kernel_asterinas_boundary as pin;
use serde_json::Value;

/// Embedded matrix artifact (v0.1.0). Compiled pin identity is cross-checked in tests.
pub const MATRIX_JSON: &str = include_str!("../matrix/abi-matrix-v0.1.0.json");

/// Closed set of surface ids the matrix MUST declare.
pub const REQUIRED_SURFACES: [&str; 4] = [
    "syscalls",
    "proc_sys_cgroupfs",
    "netlink",
    "mount_semantics",
];

/// Closed set of availability values permitted on each row.
pub const AVAILABILITY_VALUES: [&str; 3] = ["present", "gap", "unknown"];

/// Closed set of severity values permitted on each row.
pub const SEVERITY_VALUES: [&str; 4] = ["critical", "high", "medium", "low"];

/// G5 trigger classes from the round-2 plan (cgroup v2 delegation / netlink / overlayfs).
pub const G5_TRIGGER_CLASSES: [&str; 3] = ["cgroup_v2_delegation", "netlink", "overlayfs"];

/// Parse the embedded matrix as JSON. Returns the root object.
pub fn parse_matrix() -> Result<Value, String> {
    serde_json::from_str(MATRIX_JSON).map_err(|e| format!("abi-matrix JSON parse error: {e}"))
}

/// Row view extracted from a surface for probe / evaluation consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixRow {
    pub id: String,
    pub surface: String,
    pub name: String,
    pub required_by_node_stack: Vec<String>,
    pub available_on_asterinas_pin: String,
    pub severity: String,
    pub g5_trigger: bool,
}

/// Outcome of evaluating G5 triggers against current availability columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum G5Evaluation {
    /// At least one G5-trigger row has a measured `gap`.
    Fired { gap_row_ids: Vec<String> },
    /// No G5-trigger row is a measured gap; some may still be `unknown`.
    Clear { unknown_g5_row_ids: Vec<String> },
}

/// Validate structural invariants of the matrix without requiring live Asterinas evidence.
/// Unknown availability is allowed and expected for scaffold.
pub fn validate_matrix(root: &Value) -> Result<(), String> {
    let obj = root
        .as_object()
        .ok_or_else(|| "matrix root must be an object".to_string())?;

    if obj.get("matrix_id").and_then(|v| v.as_str()) != Some("asterinas-abi-matrix") {
        return Err("matrix_id must be asterinas-abi-matrix".into());
    }
    if obj.get("schema_version").and_then(|v| v.as_str()) != Some("0.1.0") {
        return Err("schema_version must be 0.1.0".into());
    }
    if obj.get("status").and_then(|v| v.as_str()) != Some("scaffold") {
        return Err("scaffold matrix status must be scaffold".into());
    }

    let claim = obj
        .get("claim_posture")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "claim_posture missing".to_string())?;
    if claim
        .get("asterinas_is_canonical_node_kernel")
        .and_then(|v| v.as_bool())
        != Some(false)
    {
        return Err(
            "claim_posture.asterinas_is_canonical_node_kernel must be false (blocked on F1(a))"
                .into(),
        );
    }
    if claim.get("blocked_on").and_then(|v| v.as_str()) != Some("F1(a)") {
        return Err("claim_posture.blocked_on must be F1(a)".into());
    }

    let pin_obj = obj
        .get("asterinas_pin")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "asterinas_pin missing".to_string())?;
    if pin_obj.get("release_tag").and_then(|v| v.as_str()) != Some(pin::RELEASE_TAG) {
        return Err(format!(
            "asterinas_pin.release_tag must match boundary pin {}",
            pin::RELEASE_TAG
        ));
    }
    if pin_obj.get("release_commit").and_then(|v| v.as_str()) != Some(pin::RELEASE_COMMIT) {
        return Err(format!(
            "asterinas_pin.release_commit must match boundary pin {}",
            pin::RELEASE_COMMIT
        ));
    }

    let pools = obj
        .get("pool_matrix_notes")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "pool_matrix_notes missing".to_string())?;
    if pools
        .get("asterinas_exposes_dev_kvm")
        .and_then(|v| v.as_bool())
        != Some(false)
    {
        return Err("pool_matrix_notes.asterinas_exposes_dev_kvm must be false".into());
    }
    let snap = pools
        .get("snapshotter_posture")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "snapshotter_posture missing".to_string())?;
    if snap.get("asterinas_pools").and_then(|v| v.as_str()) != Some("native-snapshotter-first")
    {
        return Err("Asterinas pools must declare native-snapshotter-first".into());
    }
    if snap
        .get("linux_shared_kernel_pools")
        .and_then(|v| v.as_str())
        != Some("overlayfs-required-day-one")
    {
        return Err("Linux shared-kernel pools must declare overlayfs-required-day-one".into());
    }

    let g5 = obj
        .get("g5_trigger_definition")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "g5_trigger_definition missing".to_string())?;
    let classes = g5
        .get("classes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "g5 classes missing".to_string())?;
    for expected in G5_TRIGGER_CLASSES {
        if !classes.iter().any(|c| c.as_str() == Some(expected)) {
            return Err(format!("g5_trigger_definition.classes missing {expected}"));
        }
    }

    let surfaces = obj
        .get("surfaces")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "surfaces missing".to_string())?;
    for surface in REQUIRED_SURFACES {
        if !surfaces.contains_key(surface) {
            return Err(format!("surfaces missing required surface {surface}"));
        }
        validate_surface_rows(surface, &surfaces[surface])?;
    }

    Ok(())
}

fn validate_surface_rows(surface: &str, surface_val: &Value) -> Result<(), String> {
    let obj = surface_val
        .as_object()
        .ok_or_else(|| format!("surface {surface} must be an object"))?;
    let rows = obj
        .get("rows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("surface {surface} must have rows[]"))?;
    if rows.is_empty() {
        return Err(format!("surface {surface} rows must be non-empty"));
    }
    for (i, row) in rows.iter().enumerate() {
        let r = row
            .as_object()
            .ok_or_else(|| format!("{surface}[{i}] must be an object"))?;
        for key in [
            "id",
            "name",
            "required_by_node_stack",
            "available_on_asterinas_pin",
            "severity",
            "g5_trigger",
        ] {
            if !r.contains_key(key) {
                return Err(format!("{surface}[{i}] missing column {key}"));
            }
        }
        let avail = r
            .get("available_on_asterinas_pin")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("{surface}[{i}].available_on_asterinas_pin must be string"))?;
        if !AVAILABILITY_VALUES.contains(&avail) {
            return Err(format!(
                "{surface}[{i}].available_on_asterinas_pin invalid: {avail}"
            ));
        }
        let sev = r
            .get("severity")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("{surface}[{i}].severity must be string"))?;
        if !SEVERITY_VALUES.contains(&sev) {
            return Err(format!("{surface}[{i}].severity invalid: {sev}"));
        }
        let req = r
            .get("required_by_node_stack")
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("{surface}[{i}].required_by_node_stack must be array"))?;
        if req.is_empty() {
            return Err(format!(
                "{surface}[{i}].required_by_node_stack must be non-empty"
            ));
        }
        if !r
            .get("g5_trigger")
            .and_then(|v| v.as_bool())
            .is_some()
        {
            return Err(format!("{surface}[{i}].g5_trigger must be bool"));
        }
    }
    Ok(())
}

/// Flatten all surface rows for probe enumeration.
pub fn all_rows(root: &Value) -> Result<Vec<MatrixRow>, String> {
    validate_matrix(root)?;
    let surfaces = root["surfaces"]
        .as_object()
        .ok_or_else(|| "surfaces missing".to_string())?;
    let mut out = Vec::new();
    for surface in REQUIRED_SURFACES {
        let rows = surfaces[surface]["rows"]
            .as_array()
            .ok_or_else(|| format!("{surface}.rows missing"))?;
        for row in rows {
            let id = row["id"].as_str().unwrap_or_default().to_string();
            let name = row["name"].as_str().unwrap_or_default().to_string();
            let available_on_asterinas_pin = row["available_on_asterinas_pin"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let severity = row["severity"].as_str().unwrap_or_default().to_string();
            let g5_trigger = row["g5_trigger"].as_bool().unwrap_or(false);
            let required_by_node_stack = row["required_by_node_stack"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            out.push(MatrixRow {
                id,
                surface: surface.to_string(),
                name,
                required_by_node_stack,
                available_on_asterinas_pin,
                severity,
                g5_trigger,
            });
        }
    }
    Ok(out)
}

/// Evaluate G5: measured `gap` on a g5_trigger row fires; `unknown` does not.
pub fn evaluate_g5(root: &Value) -> Result<G5Evaluation, String> {
    let rows = all_rows(root)?;
    let mut gap_row_ids = Vec::new();
    let mut unknown_g5_row_ids = Vec::new();
    for row in rows.into_iter().filter(|r| r.g5_trigger) {
        match row.available_on_asterinas_pin.as_str() {
            "gap" => gap_row_ids.push(row.id),
            "unknown" => unknown_g5_row_ids.push(row.id),
            _ => {}
        }
    }
    if gap_row_ids.is_empty() {
        Ok(G5Evaluation::Clear {
            unknown_g5_row_ids,
        })
    } else {
        Ok(G5Evaluation::Fired { gap_row_ids })
    }
}

/// How F1(a) consumes this artifact (stable prose for receipts / PR bodies).
pub fn f1a_consumption_note() -> &'static str {
    "F1(a) consumes this matrix as W0-entry evidence for the kernel + pool-matrix ruling: \
     (1) four-surface delta vs Asterinas pin, (2) G5 trigger evaluation on measured gaps \
     (cgroup v2 delegation / netlink / overlayfs), (3) pool physics (no /dev/kvm on Asterinas ⇒ \
     private-kernel* on Linux KVM pools; Asterinas serves shared-kernel; native snapshotter \
     first on Asterinas). Scaffold availability=unknown keeps F1(a) blocked on measurement — \
     it does not authorize Asterinas as canonical node kernel."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_matrix_parses_and_validates() {
        let root = parse_matrix().expect("parse");
        validate_matrix(&root).expect("validate");
    }

    #[test]
    fn four_surfaces_present_with_rows() {
        let root = parse_matrix().unwrap();
        let rows = all_rows(&root).unwrap();
        assert!(rows.len() >= 4);
        for surface in REQUIRED_SURFACES {
            assert!(
                rows.iter().any(|r| r.surface == surface),
                "missing surface {surface}"
            );
        }
    }

    #[test]
    fn scaffold_defaults_availability_to_unknown() {
        let root = parse_matrix().unwrap();
        let rows = all_rows(&root).unwrap();
        assert!(rows.iter().all(|r| r.available_on_asterinas_pin == "unknown"));
    }

    #[test]
    fn g5_flags_cover_netlink_cgroup_overlay() {
        let root = parse_matrix().unwrap();
        let rows = all_rows(&root).unwrap();
        assert!(rows.iter().any(|r| r.id == "nl-route" && r.g5_trigger));
        assert!(rows
            .iter()
            .any(|r| r.id == "fs-cgroup-memory-current" && r.g5_trigger));
        assert!(rows
            .iter()
            .any(|r| r.id == "mnt-overlayfs-whiteouts" && r.g5_trigger));
        assert!(rows.iter().any(|r| r.id == "nl-netfilter" && r.severity == "critical"));
    }

    #[test]
    fn unknown_availability_does_not_fire_g5() {
        let root = parse_matrix().unwrap();
        match evaluate_g5(&root).unwrap() {
            G5Evaluation::Clear { unknown_g5_row_ids } => {
                assert!(!unknown_g5_row_ids.is_empty());
            }
            other => panic!("expected Clear on scaffold, got {other:?}"),
        }
    }

    #[test]
    fn measured_gap_on_g5_row_fires() {
        let mut root = parse_matrix().unwrap();
        // Flip one netlink G5 row to gap — simulates a QEMU-proven absence.
        let rows = root["surfaces"]["netlink"]["rows"].as_array_mut().unwrap();
        for row in rows.iter_mut() {
            if row["id"] == "nl-route" {
                row["available_on_asterinas_pin"] = Value::String("gap".into());
            }
        }
        match evaluate_g5(&root).unwrap() {
            G5Evaluation::Fired { gap_row_ids } => {
                assert!(gap_row_ids.iter().any(|id| id == "nl-route"));
            }
            other => panic!("expected Fired, got {other:?}"),
        }
    }

    #[test]
    fn refuses_canonical_kernel_claim() {
        let mut root = parse_matrix().unwrap();
        root["claim_posture"]["asterinas_is_canonical_node_kernel"] = Value::Bool(true);
        let err = validate_matrix(&root).expect_err("must refuse canonical claim");
        assert!(err.contains("canonical"));
    }

    #[test]
    fn pin_identity_matches_boundary() {
        let root = parse_matrix().unwrap();
        assert_eq!(
            root["asterinas_pin"]["release_tag"],
            pin::RELEASE_TAG
        );
        assert_eq!(
            root["asterinas_pin"]["boot_iso_asset"],
            pin::BOOT_ISO_ASSET
        );
    }

    #[test]
    fn f1a_note_is_nonempty() {
        assert!(f1a_consumption_note().contains("F1(a)"));
        assert!(f1a_consumption_note().contains("pool"));
    }

    #[test]
    fn cadvisor_and_mount_load_bearing_rows_exist() {
        let root = parse_matrix().unwrap();
        let rows = all_rows(&root).unwrap();
        for id in [
            "fs-proc-stat",
            "fs-proc-meminfo",
            "fs-cgroup-memory-current",
            "fs-cgroup-cpu-stat",
            "fs-statfs-eviction",
            "mnt-ms-shared",
            "mnt-ms-slave",
            "mnt-pivot-root",
        ] {
            assert!(rows.iter().any(|r| r.id == id), "missing row {id}");
        }
    }
}
