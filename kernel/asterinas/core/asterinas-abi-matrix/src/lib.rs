#![forbid(unsafe_code)]
//! Versioned Asterinas ABI / kernel-service matrix (A1 / W0-entry evidence scaffold).
//!
//! Surfaces (v0.1.0 closed set): syscalls, procfs/sysfs/cgroupfs, netlink, mount semantics.
//! A1 is broader than syscalls alone — mandatory sub-surfaces (device/ioctl/eBPF/securityfs/
//! LSM/pidfd/unix-cred) and Round-4/5 row classes (io_uring, seccomp, device/driver) are
//! recorded in the matrix artifact; the container-critical sub-surface verdict table is an
//! A1 *exit* artifact, not claimed green by this scaffold.
//!
//! This crate embeds and validates the matrix; it does **not** claim Asterinas is the
//! canonical node kernel (blocked on founder ADR F1(a)). Scaffold ≠ green matrix.
//!
//! data_class: PUBLIC — measurement contract + pin identity only.

use kernel_asterinas_boundary as pin;
use serde_json::Value;
use std::fmt;

/// Embedded matrix artifact (v0.1.0). Compiled pin identity is cross-checked in tests.
/// data_class: PUBLIC
pub const MATRIX_JSON: &str = include_str!("../matrix/abi-matrix-v0.1.0.json");

/// Closed set of surface ids the matrix MUST declare (exactly these keys).
/// data_class: PUBLIC
pub const REQUIRED_SURFACES: [&str; 4] = [
    "syscalls",
    "proc_sys_cgroupfs",
    "netlink",
    "mount_semantics",
];

/// Closed set of node-stack consumer identifiers for `required_by_node_stack`.
/// data_class: PUBLIC
pub const NODE_STACK_CONSUMERS: [&str; 8] = [
    "kubelet",
    "cAdvisor",
    "eviction",
    "kube-proxy",
    "CNI",
    "runc",
    "youki",
    "containerd",
];

/// Closed set of severity values permitted on each row.
/// data_class: PUBLIC
pub const SEVERITY_VALUES: [&str; 4] = ["critical", "high", "medium", "low"];

/// Availability on the pinned Asterinas release (closed domain).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    /// Surface present and enforcing as claimed.
    /// data_class: PUBLIC
    Present,
    /// Measured absence or non-enforcing.
    /// data_class: PUBLIC
    Gap,
    /// Not yet measured (scaffold default).
    /// data_class: PUBLIC
    Unknown,
}

impl Availability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Gap => "gap",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(s: &str) -> Result<Self, MatrixError> {
        match s {
            "present" => Ok(Self::Present),
            "gap" => Ok(Self::Gap),
            "unknown" => Ok(Self::Unknown),
            other => Err(MatrixError::Row(format!(
                "available_on_asterinas_pin invalid: {other}"
            ))),
        }
    }
}

/// Severity if the surface is absent/non-enforcing (closed domain).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// data_class: PUBLIC
    Critical,
    /// data_class: PUBLIC
    High,
    /// data_class: PUBLIC
    Medium,
    /// data_class: PUBLIC
    Low,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    pub fn parse(s: &str) -> Result<Self, MatrixError> {
        match s {
            "critical" => Ok(Self::Critical),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            other => Err(MatrixError::Row(format!("severity invalid: {other}"))),
        }
    }
}
/// G5 trigger classes from the round-2 plan (cgroup v2 delegation / netlink / overlayfs).
/// data_class: PUBLIC
pub const G5_TRIGGER_CLASSES: [&str; 3] = ["cgroup_v2_delegation", "netlink", "overlayfs"];

/// Closed component ids the matrix `components_profiled` list MUST match exactly.
/// data_class: PUBLIC
pub const PROFILED_COMPONENTS: [&str; 4] = ["runc", "youki", "containerd", "kubelet"];

/// Exact v0.1.0 row-id census per surface (availability may mutate; identity set may not).
/// data_class: PUBLIC
pub const REQUIRED_ROWS_V0_1_0: &[(&str, &[&str])] = &[
    (
        "syscalls",
        &[
            "sc-clone-namespaces",
            "sc-mount-umount",
            "sc-pivot-root",
            "sc-seccomp",
            "sc-bpf-cgroup-devices",
        ],
    ),
    (
        "proc_sys_cgroupfs",
        &[
            "fs-proc-stat",
            "fs-proc-meminfo",
            "fs-proc-sys-net-core-somaxconn",
            "fs-sys-class-net",
            "fs-sys-fs-cgroup",
            "fs-cgroup-memory-current",
            "fs-cgroup-cpu-stat",
            "fs-cgroup-controllers",
            "fs-cgroup-subtree-control",
            "fs-cgroup-procs",
            "fs-cgroup-memory-max",
            "fs-cgroup-cpu-max",
            "fs-cgroup-pids-max",
            "fs-statfs-eviction",
        ],
    ),
    (
        "netlink",
        &["nl-route", "nl-link", "nl-addr", "nl-netfilter"],
    ),
    (
        "mount_semantics",
        &[
            "mnt-ms-shared",
            "mnt-ms-slave",
            "mnt-overlayfs-whiteouts",
            "mnt-asterinas-native-snapshotter",
            "mnt-pivot-root",
        ],
    ),
];

/// Closed set of row ids that MUST declare `g5_trigger: true` in v0.1.0.
/// data_class: PUBLIC
pub const REQUIRED_G5_TRIGGER_IDS: &[&str] = &[
    "sc-bpf-cgroup-devices",
    "fs-cgroup-memory-current",
    "fs-cgroup-cpu-stat",
    "fs-cgroup-controllers",
    "fs-cgroup-subtree-control",
    "fs-cgroup-procs",
    "fs-cgroup-memory-max",
    "fs-cgroup-cpu-max",
    "fs-cgroup-pids-max",
    "nl-route",
    "nl-link",
    "nl-addr",
    "nl-netfilter",
    "mnt-overlayfs-whiteouts",
];

/// Matchable library error for matrix parse / validation / evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MatrixError {
    /// JSON parse failure.
    /// data_class: PUBLIC
    Parse(String),
    /// Structural / schema invariant violation.
    /// data_class: PUBLIC
    Schema(String),
    /// Pin identity drifted from `kernel-asterinas-boundary`.
    /// data_class: PUBLIC
    PinDrift(String),
    /// Surface key missing or outside the closed set.
    /// data_class: PUBLIC
    Surface(String),
    /// Row-level value invalid (id/name/availability/consumers/…).
    /// data_class: PUBLIC
    Row(String),
}

impl fmt::Display for MatrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(m) | Self::Schema(m) | Self::PinDrift(m) | Self::Surface(m) | Self::Row(m) => {
                f.write_str(m)
            }
        }
    }
}

impl std::error::Error for MatrixError {}

/// Parse the embedded matrix as JSON. Returns the root object.
pub fn parse_matrix() -> Result<Value, MatrixError> {
    serde_json::from_str(MATRIX_JSON)
        .map_err(|e| MatrixError::Parse(format!("abi-matrix JSON parse error: {e}")))
}

/// Row view extracted from a surface for probe / evaluation consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixRow {
    /// Stable row id. data_class: PUBLIC
    pub id: String,
    /// Owning surface key. data_class: PUBLIC
    pub surface: String,
    /// Human-readable row name. data_class: PUBLIC
    pub name: String,
    /// Node-stack consumers that require this surface. data_class: PUBLIC
    pub required_by_node_stack: Vec<String>,
    /// Availability on the pinned Asterinas release. data_class: PUBLIC
    pub available_on_asterinas_pin: Availability,
    /// Severity if absent/non-enforcing. data_class: PUBLIC
    pub severity: Severity,
    /// Whether a measured gap fires G5. data_class: PUBLIC
    pub g5_trigger: bool,
}

/// Outcome of evaluating G5 triggers against current availability columns.
///
/// `Clear` is reserved for fully measured non-gap G5 rows. Scaffold matrices with
/// `unknown` G5 rows evaluate to `PendingMeasurement` so consumers cannot treat
/// `clear` as a completed verdict while `evaluation_status: pending_measurement`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum G5Evaluation {
    /// At least one G5-trigger row has a declared `gap`.
    Fired {
        /// data_class: PUBLIC
        gap_row_ids: Vec<String>,
    },
    /// No G5 gaps and no unknown G5 rows — fully measured clear.
    Clear,
    /// No measured gaps, but one or more G5 rows remain `unknown`.
    PendingMeasurement {
        /// data_class: PUBLIC
        unknown_g5_row_ids: Vec<String>,
    },
}

/// Validate structural invariants of the matrix without requiring live Asterinas evidence.
/// Unknown availability is allowed and expected for scaffold.
pub fn validate_matrix(root: &Value) -> Result<(), MatrixError> {
    let obj = root
        .as_object()
        .ok_or_else(|| MatrixError::Schema("matrix root must be an object".into()))?;

    if obj.get("matrix_id").and_then(|v| v.as_str()) != Some("asterinas-abi-matrix") {
        return Err(MatrixError::Schema(
            "matrix_id must be asterinas-abi-matrix".into(),
        ));
    }
    if obj.get("schema_version").and_then(|v| v.as_str()) != Some("0.1.0") {
        return Err(MatrixError::Schema("schema_version must be 0.1.0".into()));
    }
    if obj.get("status").and_then(|v| v.as_str()) != Some("scaffold") {
        return Err(MatrixError::Schema(
            "scaffold matrix status must be scaffold".into(),
        ));
    }

    let claim = obj
        .get("claim_posture")
        .and_then(|v| v.as_object())
        .ok_or_else(|| MatrixError::Schema("claim_posture missing".into()))?;
    if claim
        .get("asterinas_is_canonical_node_kernel")
        .and_then(|v| v.as_bool())
        != Some(false)
    {
        return Err(MatrixError::Schema(
            "claim_posture.asterinas_is_canonical_node_kernel must be false (blocked on F1(a))"
                .into(),
        ));
    }
    if claim.get("blocked_on").and_then(|v| v.as_str()) != Some("F1(a)") {
        return Err(MatrixError::Schema(
            "claim_posture.blocked_on must be F1(a)".into(),
        ));
    }

    let a1_scope = obj
        .get("a1_scope")
        .and_then(|v| v.as_object())
        .ok_or_else(|| MatrixError::Schema("a1_scope missing".into()))?;
    if a1_scope.get("kind").and_then(|v| v.as_str()) != Some("abi_kernel_service_matrix") {
        return Err(MatrixError::Schema(
            "a1_scope.kind must be abi_kernel_service_matrix".into(),
        ));
    }
    let posture = a1_scope
        .get("pool_posture")
        .and_then(|v| v.as_object())
        .ok_or_else(|| MatrixError::Schema("a1_scope.pool_posture missing".into()))?;
    if posture.get("linux_pools").and_then(|v| v.as_str()) != Some("primary_production_path") {
        return Err(MatrixError::Schema(
            "a1_scope.pool_posture.linux_pools must be primary_production_path".into(),
        ));
    }
    if posture
        .get("asterinas_shared_kernel")
        .and_then(|v| v.as_str())
        != Some("soak_until_a1_green")
    {
        return Err(MatrixError::Schema(
            "a1_scope.pool_posture.asterinas_shared_kernel must be soak_until_a1_green".into(),
        ));
    }

    let pin_obj = obj
        .get("asterinas_pin")
        .and_then(|v| v.as_object())
        .ok_or_else(|| MatrixError::Schema("asterinas_pin missing".into()))?;
    if pin_obj.get("release_tag").and_then(|v| v.as_str()) != Some(pin::RELEASE_TAG) {
        return Err(MatrixError::PinDrift(format!(
            "asterinas_pin.release_tag must match boundary pin {}",
            pin::RELEASE_TAG
        )));
    }
    if pin_obj.get("release_commit").and_then(|v| v.as_str()) != Some(pin::RELEASE_COMMIT) {
        return Err(MatrixError::PinDrift(format!(
            "asterinas_pin.release_commit must match boundary pin {}",
            pin::RELEASE_COMMIT
        )));
    }
    if pin_obj.get("boot_iso_asset").and_then(|v| v.as_str()) != Some(pin::BOOT_ISO_ASSET) {
        return Err(MatrixError::PinDrift(format!(
            "asterinas_pin.boot_iso_asset must match boundary pin {}",
            pin::BOOT_ISO_ASSET
        )));
    }
    if pin_obj.get("boot_iso_sha256").and_then(|v| v.as_str()) != Some(pin::BOOT_ISO_SHA256) {
        return Err(MatrixError::PinDrift(format!(
            "asterinas_pin.boot_iso_sha256 must match boundary pin {}",
            pin::BOOT_ISO_SHA256
        )));
    }

    let pools = obj
        .get("pool_matrix_notes")
        .and_then(|v| v.as_object())
        .ok_or_else(|| MatrixError::Schema("pool_matrix_notes missing".into()))?;
    if pools
        .get("asterinas_exposes_dev_kvm")
        .and_then(|v| v.as_bool())
        != Some(false)
    {
        return Err(MatrixError::Schema(
            "pool_matrix_notes.asterinas_exposes_dev_kvm must be false".into(),
        ));
    }
    let ast_pools = pools
        .get("asterinas_pools")
        .and_then(|v| v.as_object())
        .ok_or_else(|| MatrixError::Schema("pool_matrix_notes.asterinas_pools missing".into()))?;
    let ast_tiers = ast_pools
        .get("serve_tiers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| MatrixError::Schema("asterinas_pools.serve_tiers missing".into()))?;
    if ast_tiers.len() != 1 || ast_tiers[0].as_str() != Some("shared-kernel") {
        return Err(MatrixError::Schema(
            "asterinas_pools.serve_tiers must be exactly [shared-kernel]".into(),
        ));
    }
    let linux_pools = pools
        .get("linux_kvm_pools")
        .and_then(|v| v.as_object())
        .ok_or_else(|| MatrixError::Schema("pool_matrix_notes.linux_kvm_pools missing".into()))?;
    let linux_tiers = linux_pools
        .get("serve_tiers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| MatrixError::Schema("linux_kvm_pools.serve_tiers missing".into()))?;
    const LINUX_KVM_TIERS: [&str; 2] = ["private-kernel", "private-kernel-attested"];
    if linux_tiers.len() != LINUX_KVM_TIERS.len() {
        return Err(MatrixError::Schema(format!(
            "linux_kvm_pools.serve_tiers must declare exactly {} tiers",
            LINUX_KVM_TIERS.len()
        )));
    }
    let mut seen_linux_tiers = Vec::new();
    for t in linux_tiers {
        let Some(name) = t.as_str().filter(|s| !s.is_empty()) else {
            return Err(MatrixError::Schema(
                "linux_kvm_pools.serve_tiers entries must be non-empty strings".into(),
            ));
        };
        if !LINUX_KVM_TIERS.contains(&name) {
            return Err(MatrixError::Schema(format!(
                "linux_kvm_pools.serve_tiers contains undeclared tier {name}"
            )));
        }
        if seen_linux_tiers.iter().any(|s| s == name) {
            return Err(MatrixError::Schema(format!(
                "linux_kvm_pools.serve_tiers duplicate {name}"
            )));
        }
        seen_linux_tiers.push(name.to_string());
    }
    for required in LINUX_KVM_TIERS {
        if !seen_linux_tiers.iter().any(|t| t == required) {
            return Err(MatrixError::Schema(format!(
                "linux_kvm_pools.serve_tiers missing {required}"
            )));
        }
    }
    if linux_pools.get("sku_status").and_then(|v| v.as_str()) != Some("permanent-co-selected") {
        return Err(MatrixError::Schema(
            "linux_kvm_pools.sku_status must be permanent-co-selected".into(),
        ));
    }
    let linux_sk = pools
        .get("linux_shared_kernel_pools")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            MatrixError::Schema("pool_matrix_notes.linux_shared_kernel_pools missing".into())
        })?;
    let linux_sk_tiers = linux_sk
        .get("serve_tiers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            MatrixError::Schema("linux_shared_kernel_pools.serve_tiers missing".into())
        })?;
    if linux_sk_tiers.len() != 1 || linux_sk_tiers[0].as_str() != Some("shared-kernel") {
        return Err(MatrixError::Schema(
            "linux_shared_kernel_pools.serve_tiers must be exactly [shared-kernel]".into(),
        ));
    }
    if linux_sk.get("sku_status").and_then(|v| v.as_str()) != Some("primary-production-path") {
        return Err(MatrixError::Schema(
            "linux_shared_kernel_pools.sku_status must be primary-production-path".into(),
        ));
    }
    let snap = pools
        .get("snapshotter_posture")
        .and_then(|v| v.as_object())
        .ok_or_else(|| MatrixError::Schema("snapshotter_posture missing".into()))?;
    if snap.get("asterinas_pools").and_then(|v| v.as_str()) != Some("native-snapshotter-first")
    {
        return Err(MatrixError::Schema(
            "Asterinas pools must declare native-snapshotter-first".into(),
        ));
    }
    if snap
        .get("linux_shared_kernel_pools")
        .and_then(|v| v.as_str())
        != Some("overlayfs-required-day-one")
    {
        return Err(MatrixError::Schema(
            "Linux shared-kernel pools must declare overlayfs-required-day-one".into(),
        ));
    }

    let g5 = obj
        .get("g5_trigger_definition")
        .and_then(|v| v.as_object())
        .ok_or_else(|| MatrixError::Schema("g5_trigger_definition missing".into()))?;
    let classes = g5
        .get("classes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| MatrixError::Schema("g5 classes missing".into()))?;
    if classes.len() != G5_TRIGGER_CLASSES.len() {
        return Err(MatrixError::Schema(format!(
            "g5_trigger_definition.classes must declare exactly {} entries",
            G5_TRIGGER_CLASSES.len()
        )));
    }
    let mut seen_classes = Vec::new();
    for c in classes {
        let Some(name) = c.as_str() else {
            return Err(MatrixError::Schema(
                "g5_trigger_definition.classes entries must be non-empty strings".into(),
            ));
        };
        if !G5_TRIGGER_CLASSES.contains(&name) {
            return Err(MatrixError::Schema(format!(
                "g5_trigger_definition.classes contains undeclared class {name}"
            )));
        }
        if seen_classes.iter().any(|s| s == name) {
            return Err(MatrixError::Schema(format!(
                "g5_trigger_definition.classes duplicate {name}"
            )));
        }
        seen_classes.push(name.to_string());
    }
    for expected in G5_TRIGGER_CLASSES {
        if !seen_classes.iter().any(|c| c == expected) {
            return Err(MatrixError::Schema(format!(
                "g5_trigger_definition.classes missing {expected}"
            )));
        }
    }
    let eval_status = g5
        .get("evaluation_status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            MatrixError::Schema("g5_trigger_definition.evaluation_status missing".into())
        })?;
    if !["pending_measurement", "clear", "fired"].contains(&eval_status) {
        return Err(MatrixError::Schema(format!(
            "g5_trigger_definition.evaluation_status invalid: {eval_status}"
        )));
    }

    validate_components_profiled(obj)?;

    let surfaces = obj
        .get("surfaces")
        .and_then(|v| v.as_object())
        .ok_or_else(|| MatrixError::Schema("surfaces missing".into()))?;
    if surfaces.len() != REQUIRED_SURFACES.len() {
        return Err(MatrixError::Surface(format!(
            "surfaces must declare exactly {} keys (closed set); found {}",
            REQUIRED_SURFACES.len(),
            surfaces.len()
        )));
    }
    for key in surfaces.keys() {
        if !REQUIRED_SURFACES.iter().any(|s| s == key) {
            return Err(MatrixError::Surface(format!(
                "surfaces contains undeclared/ignored key {key}; closed set is {:?}",
                REQUIRED_SURFACES
            )));
        }
    }
    let mut seen_ids: Vec<String> = Vec::new();
    for surface in REQUIRED_SURFACES {
        if !surfaces.contains_key(surface) {
            return Err(MatrixError::Surface(format!(
                "surfaces missing required surface {surface}"
            )));
        }
        validate_surface_rows(surface, &surfaces[surface], &mut seen_ids)?;
    }
    validate_row_census_and_g5_triggers(surfaces)?;
    validate_evaluation_status_matches_rows(obj, surfaces)?;
    validate_probe_harness(obj)?;

    Ok(())
}

fn validate_probe_harness(obj: &serde_json::Map<String, Value>) -> Result<(), MatrixError> {
    let harness = obj
        .get("probe_harness")
        .and_then(|v| v.as_object())
        .ok_or_else(|| MatrixError::Schema("probe_harness missing".into()))?;
    if harness.get("preferred_evidence_path").and_then(|v| v.as_str())
        != Some("qemu-tcg-against-pinned-iso")
    {
        return Err(MatrixError::Schema(
            "probe_harness.preferred_evidence_path must be qemu-tcg-against-pinned-iso".into(),
        ));
    }
    if harness.get("live_hardware_required").and_then(|v| v.as_bool()) != Some(false) {
        return Err(MatrixError::Schema(
            "probe_harness.live_hardware_required must be false".into(),
        ));
    }
    if harness.get("crate").and_then(|v| v.as_str()) != Some("kernel-asterinas-abi-probe") {
        return Err(MatrixError::Schema(
            "probe_harness.crate must be kernel-asterinas-abi-probe".into(),
        ));
    }
    Ok(())
}

fn validate_evaluation_status_matches_rows(
    obj: &serde_json::Map<String, Value>,
    surfaces: &serde_json::Map<String, Value>,
) -> Result<(), MatrixError> {
    let declared = obj["g5_trigger_definition"]["evaluation_status"]
        .as_str()
        .unwrap_or_default();
    let mut has_gap = false;
    let mut has_unknown = false;
    for surface in REQUIRED_SURFACES {
        let rows = surfaces[surface]["rows"].as_array().unwrap();
        for row in rows {
            if row.get("g5_trigger").and_then(|v| v.as_bool()) != Some(true) {
                continue;
            }
            match row
                .get("available_on_asterinas_pin")
                .and_then(|v| v.as_str())
            {
                Some("gap") => has_gap = true,
                Some("unknown") => has_unknown = true,
                _ => {}
            }
        }
    }
    let expected = if has_gap {
        "fired"
    } else if has_unknown {
        "pending_measurement"
    } else {
        "clear"
    };
    if declared != expected {
        return Err(MatrixError::Schema(format!(
            "g5_trigger_definition.evaluation_status={declared} disagrees with row availability ({expected})"
        )));
    }
    Ok(())
}

fn validate_row_census_and_g5_triggers(
    surfaces: &serde_json::Map<String, Value>,
) -> Result<(), MatrixError> {
    for (surface, expected_ids) in REQUIRED_ROWS_V0_1_0 {
        let rows = surfaces[*surface]["rows"]
            .as_array()
            .ok_or_else(|| MatrixError::Row(format!("{surface}.rows missing")))?;
        let actual: Vec<&str> = rows
            .iter()
            .filter_map(|r| r.get("id").and_then(|v| v.as_str()))
            .collect();
        if actual.len() != expected_ids.len() {
            return Err(MatrixError::Row(format!(
                "surface {surface} must declare exactly {} versioned row ids; found {}",
                expected_ids.len(),
                actual.len()
            )));
        }
        for expected in *expected_ids {
            if !actual.iter().any(|id| id == expected) {
                return Err(MatrixError::Row(format!(
                    "surface {surface} missing required versioned row id {expected}"
                )));
            }
        }
        for id in &actual {
            if !expected_ids.iter().any(|e| e == id) {
                return Err(MatrixError::Row(format!(
                    "surface {surface} contains undeclared row id {id}"
                )));
            }
        }
        for row in rows {
            let id = row.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            let g5 = row.get("g5_trigger").and_then(|v| v.as_bool()).unwrap_or(false);
            if REQUIRED_G5_TRIGGER_IDS.contains(&id) && !g5 {
                return Err(MatrixError::Row(format!(
                    "row {id} is in the closed G5-trigger set and must declare g5_trigger=true"
                )));
            }
            if g5 && !REQUIRED_G5_TRIGGER_IDS.contains(&id) {
                return Err(MatrixError::Row(format!(
                    "row {id} declares g5_trigger=true but is outside the closed G5-trigger set"
                )));
            }
        }
    }
    Ok(())
}

fn validate_components_profiled(
    obj: &serde_json::Map<String, Value>,
) -> Result<(), MatrixError> {
    let comps = obj
        .get("components_profiled")
        .and_then(|v| v.as_array())
        .ok_or_else(|| MatrixError::Schema("components_profiled missing".into()))?;
    if comps.len() != PROFILED_COMPONENTS.len() {
        return Err(MatrixError::Schema(format!(
            "components_profiled must list exactly {} components",
            PROFILED_COMPONENTS.len()
        )));
    }
    let mut ids = Vec::with_capacity(comps.len());
    for (i, c) in comps.iter().enumerate() {
        let id = c
            .as_object()
            .and_then(|o| o.get("id"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                MatrixError::Schema(format!(
                    "components_profiled[{i}].id must be a non-empty string"
                ))
            })?;
        ids.push(id.to_string());
    }
    for expected in PROFILED_COMPONENTS {
        if !ids.iter().any(|id| id == expected) {
            return Err(MatrixError::Schema(format!(
                "components_profiled missing required id {expected}"
            )));
        }
    }
    for id in &ids {
        if !PROFILED_COMPONENTS.iter().any(|e| e == id) {
            return Err(MatrixError::Schema(format!(
                "components_profiled contains unsupported id {id}; closed set is {:?}",
                PROFILED_COMPONENTS
            )));
        }
    }
    Ok(())
}

fn validate_surface_rows(
    surface: &str,
    surface_val: &Value,
    seen_ids: &mut Vec<String>,
) -> Result<(), MatrixError> {
    let obj = surface_val
        .as_object()
        .ok_or_else(|| MatrixError::Surface(format!("surface {surface} must be an object")))?;
    if obj.get("surface_id").and_then(|v| v.as_str()) != Some(surface) {
        return Err(MatrixError::Surface(format!(
            "surfaces.{surface}.surface_id must equal enclosing key {surface}"
        )));
    }
    let rows = obj
        .get("rows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| MatrixError::Surface(format!("surface {surface} must have rows[]")))?;
    if rows.is_empty() {
        return Err(MatrixError::Row(format!(
            "surface {surface} rows must be non-empty"
        )));
    }
    for (i, row) in rows.iter().enumerate() {
        let r = row
            .as_object()
            .ok_or_else(|| MatrixError::Row(format!("{surface}[{i}] must be an object")))?;
        for key in [
            "id",
            "name",
            "required_by_node_stack",
            "available_on_asterinas_pin",
            "severity",
            "g5_trigger",
        ] {
            if !r.contains_key(key) {
                return Err(MatrixError::Row(format!(
                    "{surface}[{i}] missing column {key}"
                )));
            }
        }
        let id = r
            .get("id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                MatrixError::Row(format!("{surface}[{i}].id must be a non-empty string"))
            })?;
        if seen_ids.iter().any(|s| s == id) {
            return Err(MatrixError::Row(format!(
                "duplicate matrix row id {id} (must be globally unique)"
            )));
        }
        seen_ids.push(id.to_string());
        let _name = r
            .get("name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                MatrixError::Row(format!("{surface}[{i}].name must be a non-empty string"))
            })?;

        let avail = r
            .get("available_on_asterinas_pin")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MatrixError::Row(format!(
                    "{surface}[{i}].available_on_asterinas_pin must be string"
                ))
            })?;
        Availability::parse(avail)?;
        let sev = r
            .get("severity")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                MatrixError::Row(format!("{surface}[{i}].severity must be string"))
            })?;
        Severity::parse(sev)?;
        let req = r
            .get("required_by_node_stack")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                MatrixError::Row(format!(
                    "{surface}[{i}].required_by_node_stack must be array"
                ))
            })?;
        if req.is_empty() {
            return Err(MatrixError::Row(format!(
                "{surface}[{i}].required_by_node_stack must be non-empty"
            )));
        }
        for (j, entry) in req.iter().enumerate() {
            match entry.as_str() {
                Some(s) if !s.is_empty() => {
                    if !NODE_STACK_CONSUMERS.contains(&s) {
                        return Err(MatrixError::Row(format!(
                            "{surface}[{i}].required_by_node_stack[{j}] unsupported consumer {s}; closed set is {:?}",
                            NODE_STACK_CONSUMERS
                        )));
                    }
                }
                _ => {
                    return Err(MatrixError::Row(format!(
                        "{surface}[{i}].required_by_node_stack[{j}] must be a non-empty string"
                    )));
                }
            }
        }
        if r.get("g5_trigger").and_then(|v| v.as_bool()).is_none() {
            return Err(MatrixError::Row(format!(
                "{surface}[{i}].g5_trigger must be bool"
            )));
        }
    }
    Ok(())
}

/// Profiled component ids in matrix order (validated closed set).
pub fn profiled_component_ids(root: &Value) -> Result<Vec<String>, MatrixError> {
    validate_matrix(root)?;
    let comps = root["components_profiled"]
        .as_array()
        .ok_or_else(|| MatrixError::Schema("components_profiled missing".into()))?;
    Ok(comps
        .iter()
        .filter_map(|c| c.get("id").and_then(|v| v.as_str()).map(str::to_string))
        .collect())
}

/// Flatten all surface rows for probe enumeration.
pub fn all_rows(root: &Value) -> Result<Vec<MatrixRow>, MatrixError> {
    validate_matrix(root)?;
    let surfaces = root["surfaces"]
        .as_object()
        .ok_or_else(|| MatrixError::Schema("surfaces missing".into()))?;
    let mut out = Vec::new();
    for surface in REQUIRED_SURFACES {
        let rows = surfaces[surface]["rows"]
            .as_array()
            .ok_or_else(|| MatrixError::Row(format!("{surface}.rows missing")))?;
        for row in rows {
            let id = row["id"].as_str().unwrap_or_default().to_string();
            let name = row["name"].as_str().unwrap_or_default().to_string();
            let available_on_asterinas_pin = Availability::parse(
                row["available_on_asterinas_pin"]
                    .as_str()
                    .unwrap_or_default(),
            )?;
            let severity = Severity::parse(row["severity"].as_str().unwrap_or_default())?;
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

/// Evaluate G5: measured `gap` on a g5_trigger row fires; `unknown` yields pending, not clear.
///
/// This is a pure function over the declared availability column (scaffold wiring). Binding
/// `present`/`gap` to probe receipts is a post-scaffold measurement obligation — scaffold
/// tests may mutate the column to exercise Fired wiring without live receipts.
pub fn evaluate_g5(root: &Value) -> Result<G5Evaluation, MatrixError> {
    let rows = all_rows(root)?;
    let mut gap_row_ids = Vec::new();
    let mut unknown_g5_row_ids = Vec::new();
    for row in rows.into_iter().filter(|r| r.g5_trigger) {
        match row.available_on_asterinas_pin {
            Availability::Gap => gap_row_ids.push(row.id),
            Availability::Unknown => unknown_g5_row_ids.push(row.id),
            Availability::Present => {}
        }
    }
    if !gap_row_ids.is_empty() {
        Ok(G5Evaluation::Fired { gap_row_ids })
    } else if !unknown_g5_row_ids.is_empty() {
        Ok(G5Evaluation::PendingMeasurement {
            unknown_g5_row_ids,
        })
    } else {
        Ok(G5Evaluation::Clear)
    }
}

/// How F1(a) consumes this artifact (stable prose for receipts / PR bodies).
pub fn f1a_consumption_note() -> &'static str {
    "F1(a) consumes this matrix as W0-entry evidence for the kernel + pool-matrix ruling: \
     (1) ABI/kernel-service surface delta vs Asterinas pin (not syscalls alone), (2) G5 \
     trigger evaluation on measured gaps (cgroup v2 delegation / netlink / overlayfs), \
     (3) pool physics (Linux pools = primary production path; Asterinas shared-kernel = \
     soak until A1 goes green; no /dev/kvm on Asterinas ⇒ private-kernel* on Linux KVM \
     pools; native snapshotter first on Asterinas). Scaffold availability=unknown keeps \
     F1(a) blocked on measurement — scaffold ≠ green matrix and does not authorize \
     Asterinas as canonical node kernel. CRI/PID1/runtime-controller are node-stack law \
     outside this artifact's code scope."
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
        assert!(rows
            .iter()
            .all(|r| r.available_on_asterinas_pin == Availability::Unknown));
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
        assert!(rows
            .iter()
            .any(|r| r.id == "nl-netfilter" && r.severity == Severity::Critical));
    }

    #[test]
    fn unknown_availability_is_pending_not_clear() {
        let root = parse_matrix().unwrap();
        match evaluate_g5(&root).unwrap() {
            G5Evaluation::PendingMeasurement {
                unknown_g5_row_ids,
            } => {
                assert!(!unknown_g5_row_ids.is_empty());
            }
            other => panic!("expected PendingMeasurement on scaffold, got {other:?}"),
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
        root["g5_trigger_definition"]["evaluation_status"] = Value::String("fired".into());
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
        assert!(err.to_string().contains("canonical"));
    }

    #[test]
    fn pin_identity_matches_boundary() {
        let root = parse_matrix().unwrap();
        assert_eq!(root["asterinas_pin"]["release_tag"], pin::RELEASE_TAG);
        assert_eq!(
            root["asterinas_pin"]["boot_iso_asset"],
            pin::BOOT_ISO_ASSET
        );
    }

    #[test]
    fn rejects_boot_iso_asset_drift() {
        let mut root = parse_matrix().unwrap();
        root["asterinas_pin"]["boot_iso_asset"] = Value::String("other.iso".into());
        let err = validate_matrix(&root).expect_err("boot iso must match pin");
        assert!(err.to_string().contains("boot_iso_asset"));
    }

    #[test]
    fn rejects_malformed_row_id() {
        let mut root = parse_matrix().unwrap();
        root["surfaces"]["syscalls"]["rows"][0]["id"] = Value::Null;
        let err = validate_matrix(&root).expect_err("null id");
        assert!(err.to_string().contains("id"));
    }

    #[test]
    fn rejects_null_node_stack_consumer() {
        let mut root = parse_matrix().unwrap();
        root["surfaces"]["syscalls"]["rows"][0]["required_by_node_stack"] =
            Value::Array(vec![Value::Null]);
        let err = validate_matrix(&root).expect_err("null consumer");
        assert!(err.to_string().contains("required_by_node_stack"));
    }

    #[test]
    fn rejects_unknown_node_stack_consumer() {
        let mut root = parse_matrix().unwrap();
        root["surfaces"]["syscalls"]["rows"][0]["required_by_node_stack"] =
            Value::Array(vec![Value::String("kubelet-typo".into())]);
        let err = validate_matrix(&root).expect_err("typo consumer");
        assert!(err.to_string().contains("unsupported consumer"));
    }

    #[test]
    fn rejects_extra_linux_kvm_tier() {
        let mut root = parse_matrix().unwrap();
        root["pool_matrix_notes"]["linux_kvm_pools"]["serve_tiers"] = serde_json::json!([
            "private-kernel",
            "private-kernel-attested",
            "shared-kernel"
        ]);
        let err = validate_matrix(&root).expect_err("extra linux kvm tier");
        assert!(
            err.to_string().contains("exactly")
                || err.to_string().contains("undeclared")
                || err.to_string().contains("shared-kernel")
        );
    }

    #[test]
    fn rejects_extra_surface_key() {
        let mut root = parse_matrix().unwrap();
        root["surfaces"]["extra_surface"] = serde_json::json!({"rows": []});
        let err = validate_matrix(&root).expect_err("extra surface");
        assert!(err.to_string().contains("extra_surface") || err.to_string().contains("exactly"));
    }

    #[test]
    fn rejects_surface_id_mismatch() {
        let mut root = parse_matrix().unwrap();
        root["surfaces"]["netlink"]["surface_id"] = Value::String("other".into());
        let err = validate_matrix(&root).expect_err("surface_id mismatch");
        assert!(err.to_string().contains("surface_id"));
    }

    #[test]
    fn rejects_duplicate_row_ids() {
        let mut root = parse_matrix().unwrap();
        // Cross-surface id collision (mount row renamed to an existing netlink id).
        root["surfaces"]["mount_semantics"]["rows"][0]["id"] =
            Value::String("nl-route".into());
        let err = validate_matrix(&root).expect_err("duplicate id");
        assert!(
            err.to_string().contains("duplicate")
                || err.to_string().contains("undeclared")
                || err.to_string().contains("missing required")
        );
    }

    #[test]
    fn rejects_false_g5_trigger_on_closed_set_row() {
        let mut root = parse_matrix().unwrap();
        let rows = root["surfaces"]["netlink"]["rows"].as_array_mut().unwrap();
        for row in rows.iter_mut() {
            if row["id"] == "nl-route" {
                row["g5_trigger"] = Value::Bool(false);
                row["available_on_asterinas_pin"] = Value::String("gap".into());
            }
        }
        let err = validate_matrix(&root).expect_err("must refuse false-clear G5 bypass");
        assert!(err.to_string().contains("g5_trigger"));
    }

    #[test]
    fn rejects_missing_versioned_row_id() {
        let mut root = parse_matrix().unwrap();
        let rows = root["surfaces"]["netlink"]["rows"].as_array_mut().unwrap();
        rows.retain(|r| r["id"] != "nl-netfilter");
        let err = validate_matrix(&root).expect_err("missing census row");
        assert!(
            err.to_string().contains("nl-netfilter") || err.to_string().contains("exactly")
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
            "fs-proc-sys-net-core-somaxconn",
            "fs-sys-class-net",
            "fs-sys-fs-cgroup",
            "fs-cgroup-memory-current",
            "fs-cgroup-cpu-stat",
            "fs-cgroup-subtree-control",
            "fs-cgroup-procs",
            "fs-cgroup-memory-max",
            "fs-cgroup-cpu-max",
            "fs-statfs-eviction",
            "mnt-ms-shared",
            "mnt-ms-slave",
            "mnt-pivot-root",
        ] {
            assert!(rows.iter().any(|r| r.id == id), "missing row {id}");
        }
    }

    #[test]
    fn a1_scope_records_kernel_service_breadth() {
        let root = parse_matrix().unwrap();
        assert_eq!(
            root["a1_scope"]["kind"],
            "abi_kernel_service_matrix"
        );
        assert_eq!(
            root["a1_scope"]["pool_posture"]["linux_pools"],
            "primary_production_path"
        );
        assert_eq!(
            root["a1_scope"]["pool_posture"]["asterinas_shared_kernel"],
            "soak_until_a1_green"
        );
        assert!(
            root["a1_scope"]["netfilter_nftables_placement"]
                .as_str()
                .unwrap_or_default()
                .contains("nl-netfilter")
        );
    }
}
