#![forbid(unsafe_code)]
//! Differential-oracle harness skeleton for the owned OCI executor.
//!
//! Law (Round-2): the forever executor is an **owned** library of the per-sandbox
//! shim, built from the OCI runtime-spec. `youki` / `runc` / `crun` are pinned
//! **differential oracles** and CVE regression fixtures only — never shipped
//! product. Shipping an oracle to green a gate the owned executor did not pass
//! is **conformance laundering**.
//!
//! Bootstrap lock: K1-reference declared bootstrap (youki/Go-containerd, calendar
//! fail-closed expiry) → K1-owned gated on security-response process. Oracles
//! never ship to green a gate; CVE fixtures are the adversarial corpus.
//!
//! This crate is a hermetic scaffold: trait surface + fixture inventory + stub
//! oracle adapters. It does **not** invoke youki/runc/crun binaries, does **not**
//! PORT containerd, and does **not** claim W0/`w0_ready` readiness. Scaffolds ≠
//! production; no Accept claims.
//!
//! data_class: PUBLIC

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt;

/// Embedded CVE / adversarial obligation inventory (scaffold).
pub const CVE_OBLIGATIONS_JSON: &str =
    include_str!("../fixtures/cve-regression-obligations-v0.1.0.json");

/// Closed set of differential oracle identities (never product).
pub const ORACLE_IDS: [&str; 3] = ["youki", "runc", "crun"];

/// Closed set of mandatory CVE / adversarial obligation IDs (scaffold corpus).
pub const REQUIRED_CVE_IDS: [&str; 3] = [
    "CVE-2019-5736",
    "CVE-2024-21626",
    "CVE-MOUNT-SYMLINK-RACE",
];

/// Exact ID → regression-class mapping (adversarial corpus contract).
pub const REQUIRED_CVE_CLASSES: [(&str, &str); 3] = [
    ("CVE-2019-5736", "proc_self_exe_reexec"),
    ("CVE-2024-21626", "fd_leak"),
    ("CVE-MOUNT-SYMLINK-RACE", "mount_symlink_race"),
];

/// Exact readiness blocker set (Round-2 programme lock labels — not MPV2 IDs).
pub const REQUIRED_BLOCKERS: [&str; 3] = ["F1(b)", "W0", "port-engine-cri-oci"];

/// Required fixture role for every oracle row.
pub const ORACLE_ROLE: &str = "differential_oracle";

/// Scaffold pin marker — forbidden on measured observations.
pub const SCAFFOLD_PIN_REVISION: &str = "pin:scaffold-unresolved";

/// Matchable harness errors (scaffold; no thiserror dep).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessError {
    Parse(String),
    Schema(String),
    UnknownOracle(String),
    DuplicateOracle(String),
    MissingOracle(String),
    OracleShipped(String),
    OracleRole(String),
    OraclePin(String),
    UnknownCve(String),
    DuplicateCve(String),
    MissingCve(String),
    CveNotRequired(String),
    CveClassMismatch {
        id: String,
        expected: String,
        got: String,
    },
    MissingBlocker(String),
    UnknownBlocker(String),
    DuplicateBlocker(String),
    EmptyBundleDigest,
    ScaffoldBundleNotMeasured,
    ScaffoldPinNotMeasured,
    IncompleteMatrixCoverage(String),
    DuplicateMatrixCell(String),
    UnknownMatrixCve(String),
    FreeFormMatchForbidden,
    NotOracleObservation,
    /// Pairwise Match is not a conformance claim without full oracle×CVE coverage.
    ConformanceWithoutFullMatrix,
    ConformanceLaundering,
}

impl fmt::Display for HarnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(m)
            | Self::Schema(m)
            | Self::UnknownOracle(m)
            | Self::DuplicateOracle(m)
            | Self::MissingOracle(m)
            | Self::OracleShipped(m)
            | Self::OracleRole(m)
            | Self::OraclePin(m)
            | Self::UnknownCve(m)
            | Self::DuplicateCve(m)
            | Self::MissingCve(m)
            | Self::CveNotRequired(m)
            | Self::MissingBlocker(m)
            | Self::UnknownBlocker(m)
            | Self::DuplicateBlocker(m)
            | Self::IncompleteMatrixCoverage(m)
            | Self::DuplicateMatrixCell(m)
            | Self::UnknownMatrixCve(m) => write!(f, "{m}"),
            Self::EmptyBundleDigest => write!(f, "bundle content_digest must be non-empty"),
            Self::ScaffoldBundleNotMeasured => {
                write!(f, "measured observations cannot use scaffold bundle digests")
            }
            Self::ScaffoldPinNotMeasured => {
                write!(f, "measured observations cannot use scaffold oracle pins")
            }
            Self::FreeFormMatchForbidden => write!(
                f,
                "Match/Diverge matrix cells must be derived from ComparisonRecord"
            ),
            Self::NotOracleObservation => write!(f, "comparison oracle side must be ExecutorKind::Oracle"),
            Self::CveClassMismatch { id, expected, got } => {
                write!(f, "cve {id} class must be {expected} (got {got})")
            }
            Self::ConformanceWithoutFullMatrix => write!(
                f,
                "conformance verdict requires full oracle × CVE matrix coverage"
            ),
            Self::ConformanceLaundering => write!(
                f,
                "conformance-laundering ban: oracle executors must not be selected as shipped product"
            ),
        }
    }
}

impl std::error::Error for HarnessError {}

fn expected_cve_class(id: &str) -> Option<&'static str> {
    REQUIRED_CVE_CLASSES
        .iter()
        .find(|(cid, _)| *cid == id)
        .map(|(_, class)| *class)
}

/// Closed oracle identity enum — the only values `ExecutorKind::Oracle` may carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OracleId {
    Youki,
    Runc,
    Crun,
}

impl OracleId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Youki => "youki",
            Self::Runc => "runc",
            Self::Crun => "crun",
        }
    }

    pub fn try_from_str(id: &str) -> Result<Self, HarnessError> {
        match id {
            "youki" => Ok(Self::Youki),
            "runc" => Ok(Self::Runc),
            "crun" => Ok(Self::Crun),
            other => Err(HarnessError::UnknownOracle(other.to_owned())),
        }
    }

    pub const fn all() -> [Self; 3] {
        [Self::Youki, Self::Runc, Self::Crun]
    }
}

/// Immutable oracle build pin (revision + platform).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OraclePin {
    revision: String,
    platform: String,
}

impl OraclePin {
    pub fn try_new(revision: &str, platform: &str) -> Result<Self, HarnessError> {
        if revision.trim().is_empty() {
            return Err(HarnessError::OraclePin(
                "oracle revision pin must be non-empty".into(),
            ));
        }
        if platform.trim().is_empty() {
            return Err(HarnessError::OraclePin(
                "oracle platform pin must be non-empty".into(),
            ));
        }
        Ok(Self {
            revision: revision.to_owned(),
            platform: platform.to_owned(),
        })
    }

    pub fn scaffold() -> Self {
        Self {
            revision: SCAFFOLD_PIN_REVISION.to_owned(),
            platform: "linux/amd64".to_owned(),
        }
    }

    pub fn is_scaffold(&self) -> bool {
        self.revision == SCAFFOLD_PIN_REVISION || self.revision.starts_with("scaffold:")
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn platform(&self) -> &str {
        &self.platform
    }
}

/// Outcome of a differential comparison (scaffold).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffVerdict {
    /// Not yet executed — default for scaffold.
    Stubbed,
    /// Owned executor matched oracle (measured path) with security postconditions held.
    Match,
    /// Owned executor diverged from oracle (measured path), or both unsafe.
    Diverge,
}

/// Kill signal seam aligned with `os_runtime` / containerd task Signal (scaffold).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillSignal {
    Term,
    Kill,
    Hup,
}

/// Closed OCI operation set — kill always carries its signal (invalid combos unrepresentable).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OciOperation {
    Create,
    Start,
    Kill(KillSignal),
    Delete,
}

impl OciOperation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Start => "start",
            Self::Kill(_) => "kill",
            Self::Delete => "delete",
        }
    }

    pub const fn kill_signal(self) -> Option<KillSignal> {
        match self {
            Self::Kill(signal) => Some(signal),
            _ => None,
        }
    }
}

/// Content-derived OCI bundle identity (id alone is insufficient). Fields private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleIdentity {
    bundle_id: String,
    content_digest: String,
}

impl BundleIdentity {
    pub fn try_new(bundle_id: &str, content_digest: &str) -> Result<Self, HarnessError> {
        if content_digest.trim().is_empty() {
            return Err(HarnessError::EmptyBundleDigest);
        }
        Ok(Self {
            bundle_id: bundle_id.to_owned(),
            content_digest: content_digest.to_owned(),
        })
    }

    /// Scaffold placeholder digest keyed by bundle id (not a live OCI digest).
    pub fn scaffold(bundle_id: &str) -> Self {
        Self {
            bundle_id: bundle_id.to_owned(),
            content_digest: format!("scaffold:unresolved:{bundle_id}"),
        }
    }

    pub fn is_scaffold(&self) -> bool {
        self.content_digest.starts_with("scaffold:unresolved:")
    }

    pub fn bundle_id(&self) -> &str {
        &self.bundle_id
    }

    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }
}

/// Typed CVE / OCI-state security postconditions (adversarial corpus contract).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityPostconditions {
    pub proc_self_exe_reexec_blocked: bool,
    pub fd_leak_absent: bool,
    pub mount_symlink_race_safe: bool,
}

impl SecurityPostconditions {
    pub const fn all_held() -> Self {
        Self {
            proc_self_exe_reexec_blocked: true,
            fd_leak_absent: true,
            mount_symlink_race_safe: true,
        }
    }

    pub const fn all_held_bool(&self) -> bool {
        self.proc_self_exe_reexec_blocked && self.fd_leak_absent && self.mount_symlink_race_safe
    }
}

/// Canonical measured outcome (shared contract — not adapter-defined digests).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasuredOutcome {
    pub exit_code: i32,
    pub status: String,
    pub stderr_fingerprint: String,
    pub security: SecurityPostconditions,
}

/// Execution state — closed so measured outcomes cannot coexist with "unexecuted".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionState {
    Stubbed,
    Measured(MeasuredOutcome),
}

impl ExecutionState {
    pub const fn is_executed(&self) -> bool {
        matches!(self, Self::Measured(_))
    }

    pub fn measured(&self) -> Option<&MeasuredOutcome> {
        match self {
            Self::Measured(m) => Some(m),
            Self::Stubbed => None,
        }
    }
}

/// Identity of an OCI executor implementation behind the shared trait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutorKind {
    /// Forever product path (owned-from-spec shim library). Bodies arrive later.
    Owned,
    /// Differential oracle only — must never be selected as shipped runtime.
    Oracle { id: OracleId, pin: OraclePin },
}

impl ExecutorKind {
    pub const fn is_oracle_only(&self) -> bool {
        matches!(self, Self::Oracle { .. })
    }

    pub const fn is_owned_product(&self) -> bool {
        matches!(self, Self::Owned)
    }

    pub fn oracle_id(&self) -> Option<OracleId> {
        match self {
            Self::Oracle { id, .. } => Some(*id),
            Self::Owned => None,
        }
    }

    pub fn oracle_pin(&self) -> Option<&OraclePin> {
        match self {
            Self::Oracle { pin, .. } => Some(pin),
            Self::Owned => None,
        }
    }
}

/// Per-side operation observation. Live adapters emit one of these; a separate
/// differential runner compares the pair into a [`DiffVerdict`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationObservation {
    kind: ExecutorKind,
    operation: OciOperation,
    bundle: BundleIdentity,
    execution: ExecutionState,
}

impl OperationObservation {
    pub fn kind(&self) -> &ExecutorKind {
        &self.kind
    }
    pub fn operation(&self) -> OciOperation {
        self.operation
    }
    pub fn bundle(&self) -> &BundleIdentity {
        &self.bundle
    }
    pub fn bundle_id(&self) -> &str {
        self.bundle.bundle_id()
    }
    pub fn content_digest(&self) -> &str {
        self.bundle.content_digest()
    }
    pub fn kill_signal(&self) -> Option<KillSignal> {
        self.operation.kill_signal()
    }
    pub fn execution(&self) -> &ExecutionState {
        &self.execution
    }
    pub fn executed(&self) -> bool {
        self.execution.is_executed()
    }

    pub fn stubbed(kind: ExecutorKind, operation: OciOperation, bundle: BundleIdentity) -> Self {
        Self {
            kind,
            operation,
            bundle,
            execution: ExecutionState::Stubbed,
        }
    }

    pub fn stubbed_scaffold(
        kind: ExecutorKind,
        operation: OciOperation,
        bundle_id: &str,
    ) -> Self {
        Self::stubbed(kind, operation, BundleIdentity::scaffold(bundle_id))
    }

    /// Measured construction rejects scaffold bundle digests and scaffold oracle pins.
    pub fn try_measured(
        kind: ExecutorKind,
        operation: OciOperation,
        bundle: BundleIdentity,
        outcome: MeasuredOutcome,
    ) -> Result<Self, HarnessError> {
        if bundle.is_scaffold() {
            return Err(HarnessError::ScaffoldBundleNotMeasured);
        }
        if let Some(pin) = kind.oracle_pin() {
            if pin.is_scaffold() {
                return Err(HarnessError::ScaffoldPinNotMeasured);
            }
        }
        Ok(Self {
            kind,
            operation,
            bundle,
            execution: ExecutionState::Measured(outcome),
        })
    }
}

/// Compare two observations into a differential verdict.
///
/// Pairwise `Match` is **not** a conformance / Accept claim — callers must run
/// [`aggregate_comparison_records`] over the full oracle × CVE set first.
/// Equal-but-unsafe security postconditions diverge (never Match).
pub fn compare_observations(
    owned: &OperationObservation,
    oracle: &OperationObservation,
) -> DiffVerdict {
    if !owned.kind.is_owned_product() || !oracle.kind.is_oracle_only() {
        return DiffVerdict::Diverge;
    }
    if owned.operation != oracle.operation || owned.bundle != oracle.bundle {
        return DiffVerdict::Diverge;
    }
    match (&owned.execution, &oracle.execution) {
        (ExecutionState::Stubbed, ExecutionState::Stubbed) => DiffVerdict::Stubbed,
        (ExecutionState::Measured(a), ExecutionState::Measured(b)) if a == b => {
            if a.security.all_held_bool() {
                DiffVerdict::Match
            } else {
                // Both sides equally unsafe — not a safe Match.
                DiffVerdict::Diverge
            }
        }
        (ExecutionState::Measured(_), ExecutionState::Measured(_)) => DiffVerdict::Diverge,
        _ => DiffVerdict::Diverge,
    }
}

/// One cell of the required oracle × CVE differential matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixCell {
    pub oracle: OracleId,
    pub pin: OraclePin,
    pub cve_id: String,
    pub verdict: DiffVerdict,
}

/// Typed comparison bound to observations + CVE (not a free-form verdict row).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonRecord {
    cve_id: String,
    owned: OperationObservation,
    oracle: OperationObservation,
    verdict: DiffVerdict,
}

impl ComparisonRecord {
    pub fn try_from_observations(
        cve_id: &str,
        owned: OperationObservation,
        oracle: OperationObservation,
    ) -> Result<Self, HarnessError> {
        if !REQUIRED_CVE_IDS.contains(&cve_id) {
            return Err(HarnessError::UnknownMatrixCve(cve_id.to_owned()));
        }
        if !oracle.kind.is_oracle_only() {
            return Err(HarnessError::NotOracleObservation);
        }
        let verdict = compare_observations(&owned, &oracle);
        Ok(Self {
            cve_id: cve_id.to_owned(),
            owned,
            oracle,
            verdict,
        })
    }

    pub fn cve_id(&self) -> &str {
        &self.cve_id
    }
    pub fn verdict(&self) -> DiffVerdict {
        self.verdict
    }
    pub fn owned(&self) -> &OperationObservation {
        &self.owned
    }
    pub fn oracle(&self) -> &OperationObservation {
        &self.oracle
    }

    pub fn to_matrix_cell(&self) -> MatrixCell {
        let (id, pin) = match self.oracle.kind() {
            ExecutorKind::Oracle { id, pin } => (*id, pin.clone()),
            ExecutorKind::Owned => unreachable!("validated in try_from_observations"),
        };
        MatrixCell {
            oracle: id,
            pin,
            cve_id: self.cve_id.clone(),
            verdict: self.verdict,
        }
    }
}

/// Scaffold-only aggregate — never an Accept / product conformance claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixAggregate {
    /// Exact oracle × CVE coverage present; measurements still scaffold/stubbed.
    ScaffoldCoverageComplete,
    /// Exact coverage with measured cells (still ≠ Accept — no product ship).
    MeasuredCoverageComplete,
}

/// Exact closed set of (oracle, cve) pairs that must be covered.
pub fn required_matrix_pairs() -> Vec<(OracleId, &'static str)> {
    let mut out = Vec::with_capacity(ORACLE_IDS.len() * REQUIRED_CVE_IDS.len());
    for oracle in OracleId::all() {
        for cve in REQUIRED_CVE_IDS {
            out.push((oracle, cve));
        }
    }
    out
}

/// Reject incomplete oracle × CVE coverage before any conformance-shaped claim.
pub fn validate_matrix_coverage(cells: &[MatrixCell]) -> Result<(), HarnessError> {
    let required = required_matrix_pairs();
    let mut seen = BTreeSet::new();
    for cell in cells {
        if !REQUIRED_CVE_IDS.contains(&cell.cve_id.as_str()) {
            return Err(HarnessError::UnknownMatrixCve(cell.cve_id.clone()));
        }
        let key = (cell.oracle.as_str(), cell.cve_id.as_str());
        if !seen.insert(key) {
            return Err(HarnessError::DuplicateMatrixCell(format!(
                "{}×{}",
                cell.oracle.as_str(),
                cell.cve_id
            )));
        }
    }
    for (oracle, cve) in &required {
        if !seen.contains(&(oracle.as_str(), *cve)) {
            return Err(HarnessError::IncompleteMatrixCoverage(format!(
                "missing {}×{cve}",
                oracle.as_str()
            )));
        }
    }
    Ok(())
}

/// Aggregate free-form stubbed cells only. Match/Diverge must use ComparisonRecord.
pub fn aggregate_oracle_cve_matrix(cells: &[MatrixCell]) -> Result<MatrixAggregate, HarnessError> {
    validate_matrix_coverage(cells)?;
    if cells
        .iter()
        .any(|c| matches!(c.verdict, DiffVerdict::Match | DiffVerdict::Diverge))
    {
        return Err(HarnessError::FreeFormMatchForbidden);
    }
    Ok(MatrixAggregate::ScaffoldCoverageComplete)
}

/// Aggregate typed comparison records (verdict derived from observations + CVE).
pub fn aggregate_comparison_records(
    records: &[ComparisonRecord],
) -> Result<MatrixAggregate, HarnessError> {
    let cells: Vec<MatrixCell> = records.iter().map(ComparisonRecord::to_matrix_cell).collect();
    validate_matrix_coverage(&cells)?;
    if records.iter().any(|r| r.verdict == DiffVerdict::Stubbed) {
        Ok(MatrixAggregate::ScaffoldCoverageComplete)
    } else {
        Ok(MatrixAggregate::MeasuredCoverageComplete)
    }
}

/// Pairwise Match alone must not be treated as product conformance.
pub fn refuse_pairwise_match_as_conformance(
    records: &[ComparisonRecord],
) -> Result<MatrixAggregate, HarnessError> {
    if records.is_empty() {
        return Err(HarnessError::ConformanceWithoutFullMatrix);
    }
    aggregate_comparison_records(records)
}

/// Minimal OCI create/start/kill/delete surface shared by owned executor + oracles.
pub trait OciExecutor {
    fn kind(&self) -> ExecutorKind;

    fn create_stub(&self, bundle_id: &str) -> OperationObservation {
        OperationObservation::stubbed_scaffold(self.kind(), OciOperation::Create, bundle_id)
    }

    fn start_stub(&self, bundle_id: &str) -> OperationObservation {
        OperationObservation::stubbed_scaffold(self.kind(), OciOperation::Start, bundle_id)
    }

    fn kill_stub(&self, bundle_id: &str, signal: KillSignal) -> OperationObservation {
        OperationObservation::stubbed_scaffold(self.kind(), OciOperation::Kill(signal), bundle_id)
    }

    fn delete_stub(&self, bundle_id: &str) -> OperationObservation {
        OperationObservation::stubbed_scaffold(self.kind(), OciOperation::Delete, bundle_id)
    }

    fn create_with_bundle(&self, bundle: BundleIdentity) -> OperationObservation {
        OperationObservation::stubbed(self.kind(), OciOperation::Create, bundle)
    }
}

/// Owned executor placeholder (product path). Not an oracle.
#[derive(Debug, Default, Clone, Copy)]
pub struct OwnedExecutorStub;

impl OciExecutor for OwnedExecutorStub {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind::Owned
    }
}

/// Oracle adapter stub — identity + pin; never ship. Construction is allowlisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleStub {
    id: OracleId,
    pin: OraclePin,
}

impl OracleStub {
    pub fn youki() -> Self {
        Self {
            id: OracleId::Youki,
            pin: OraclePin::scaffold(),
        }
    }
    pub fn runc() -> Self {
        Self {
            id: OracleId::Runc,
            pin: OraclePin::scaffold(),
        }
    }
    pub fn crun() -> Self {
        Self {
            id: OracleId::Crun,
            pin: OraclePin::scaffold(),
        }
    }

    pub fn try_new(id: &str) -> Result<Self, HarnessError> {
        Ok(Self {
            id: OracleId::try_from_str(id)?,
            pin: OraclePin::scaffold(),
        })
    }

    pub fn try_new_pinned(id: &str, pin: OraclePin) -> Result<Self, HarnessError> {
        Ok(Self {
            id: OracleId::try_from_str(id)?,
            pin,
        })
    }

    pub const fn id(&self) -> OracleId {
        self.id
    }

    pub fn pin(&self) -> &OraclePin {
        &self.pin
    }
}

impl OciExecutor for OracleStub {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind::Oracle {
            id: self.id,
            pin: self.pin.clone(),
        }
    }
}

/// Expected fixture_set_id for the embedded obligations document.
pub const EXPECTED_FIXTURE_SET_ID: &str = "oci-executor-cve-regression-obligations";

#[derive(Debug, Deserialize)]
struct ObligationsRoot {
    schema_version: String,
    fixture_set_id: String,
    status: String,
    claim_posture: ClaimPosture,
    oracles: Vec<OracleRow>,
    cve_regression_obligations: Vec<CveRow>,
}

#[derive(Debug, Deserialize)]
struct ClaimPosture {
    oracles_are_shipped_product: bool,
    owned_executor_is_product_path: bool,
    blocked_on: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OracleRow {
    id: String,
    role: String,
    shipped: bool,
    revision: String,
    platform: String,
}

#[derive(Debug, Deserialize)]
struct CveRow {
    id: String,
    class: String,
    required: bool,
}

fn validate_root(root: &ObligationsRoot) -> Result<(), HarnessError> {
    if root.schema_version != "0.1.0" {
        return Err(HarnessError::Schema("schema_version must be 0.1.0".into()));
    }
    if root.fixture_set_id != EXPECTED_FIXTURE_SET_ID {
        return Err(HarnessError::Schema(format!(
            "fixture_set_id must be {EXPECTED_FIXTURE_SET_ID} (got {})",
            root.fixture_set_id
        )));
    }
    if root.status != "scaffold" {
        return Err(HarnessError::Schema("status must be scaffold".into()));
    }
    if root.claim_posture.oracles_are_shipped_product {
        return Err(HarnessError::Schema(
            "claim_posture.oracles_are_shipped_product must be false".into(),
        ));
    }
    if !root.claim_posture.owned_executor_is_product_path {
        return Err(HarnessError::Schema(
            "claim_posture.owned_executor_is_product_path must be true".into(),
        ));
    }

    let mut blockers_seen = BTreeSet::new();
    for blocker in &root.claim_posture.blocked_on {
        if !REQUIRED_BLOCKERS.contains(&blocker.as_str()) {
            return Err(HarnessError::UnknownBlocker(blocker.clone()));
        }
        if !blockers_seen.insert(blocker.as_str()) {
            return Err(HarnessError::DuplicateBlocker(blocker.clone()));
        }
    }
    for required in REQUIRED_BLOCKERS {
        if !blockers_seen.contains(required) {
            return Err(HarnessError::MissingBlocker(required.to_owned()));
        }
    }

    if root.oracles.len() != ORACLE_IDS.len() {
        return Err(HarnessError::Schema(format!(
            "oracles must be exactly {} rows (got {})",
            ORACLE_IDS.len(),
            root.oracles.len()
        )));
    }
    let mut seen = BTreeSet::new();
    for row in &root.oracles {
        let oid = OracleId::try_from_str(&row.id)?;
        if !seen.insert(oid.as_str()) {
            return Err(HarnessError::DuplicateOracle(row.id.clone()));
        }
        if row.shipped {
            return Err(HarnessError::OracleShipped(row.id.clone()));
        }
        if row.role != ORACLE_ROLE {
            return Err(HarnessError::OracleRole(format!(
                "oracle {} role must be {ORACLE_ROLE} (got {})",
                row.id, row.role
            )));
        }
        // Validates pin shape (non-empty); live digests replace scaffold later.
        OraclePin::try_new(&row.revision, &row.platform)?;
    }
    for id in OracleId::all() {
        if !seen.contains(id.as_str()) {
            return Err(HarnessError::MissingOracle(id.as_str().to_owned()));
        }
    }

    let mut cve_seen = BTreeSet::new();
    for cve in &root.cve_regression_obligations {
        let Some(expected_class) = expected_cve_class(&cve.id) else {
            return Err(HarnessError::UnknownCve(cve.id.clone()));
        };
        if !cve_seen.insert(cve.id.as_str()) {
            return Err(HarnessError::DuplicateCve(cve.id.clone()));
        }
        if !cve.required {
            return Err(HarnessError::CveNotRequired(cve.id.clone()));
        }
        if cve.class != expected_class {
            return Err(HarnessError::CveClassMismatch {
                id: cve.id.clone(),
                expected: expected_class.to_owned(),
                got: cve.class.clone(),
            });
        }
    }
    for id in REQUIRED_CVE_IDS {
        if !cve_seen.contains(id) {
            return Err(HarnessError::MissingCve(id.to_owned()));
        }
    }
    Ok(())
}

/// Validate obligations JSON text (used by embedded fixture + negative tests).
pub fn validate_obligations_json(json: &str) -> Result<Value, HarnessError> {
    let root: ObligationsRoot =
        serde_json::from_str(json).map_err(|e| HarnessError::Parse(e.to_string()))?;
    validate_root(&root)?;
    serde_json::from_str(json).map_err(|e| HarnessError::Parse(e.to_string()))
}

/// Parse and structurally validate the embedded obligations fixture.
pub fn validate_obligations() -> Result<Value, HarnessError> {
    validate_obligations_json(CVE_OBLIGATIONS_JSON)
}

/// Pair owned stub with one oracle for a future differential run.
pub fn differential_pair(oracle: OracleStub) -> (OwnedExecutorStub, OracleStub) {
    (OwnedExecutorStub, oracle)
}

/// Conformance-laundering guard: refuse selecting an oracle as the product runtime.
pub fn refuse_oracle_as_product(kind: &ExecutorKind) -> Result<(), HarnessError> {
    if kind.is_oracle_only() {
        return Err(HarnessError::ConformanceLaundering);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live_pin() -> OraclePin {
        OraclePin::try_new("sha256:deadbeef", "linux/amd64").unwrap()
    }

    fn live_bundle(id: &str) -> BundleIdentity {
        BundleIdentity::try_new(id, "sha256:bundle-config-rootfs").unwrap()
    }

    fn safe_outcome(exit: i32, fp: &str) -> MeasuredOutcome {
        MeasuredOutcome {
            exit_code: exit,
            status: "exited".into(),
            stderr_fingerprint: fp.into(),
            security: SecurityPostconditions::all_held(),
        }
    }

    #[test]
    fn obligations_fixture_validates() {
        validate_obligations().expect("scaffold obligations must validate");
    }

    #[test]
    fn oracles_are_oracle_only() {
        for stub in [OracleStub::youki(), OracleStub::runc(), OracleStub::crun()] {
            assert!(stub.kind().is_oracle_only());
            assert!(!stub.kind().is_owned_product());
            assert!(stub.pin().is_scaffold());
            assert_eq!(stub.create_stub("bundle").operation(), OciOperation::Create);
            assert!(!stub.create_stub("bundle").executed());
            let kill = stub.kill_stub("bundle", KillSignal::Term);
            assert_eq!(kill.operation(), OciOperation::Kill(KillSignal::Term));
            assert_eq!(kill.kill_signal(), Some(KillSignal::Term));
        }
    }

    #[test]
    fn owned_stub_is_product_path() {
        let owned = OwnedExecutorStub;
        assert!(owned.kind().is_owned_product());
        assert!(!owned.kind().is_oracle_only());
        refuse_oracle_as_product(&owned.kind()).expect("owned ok");
    }

    #[test]
    fn refuse_shipping_youki() {
        let err = refuse_oracle_as_product(&OracleStub::youki().kind()).unwrap_err();
        assert_eq!(err, HarnessError::ConformanceLaundering);
    }

    #[test]
    fn oracle_try_new_rejects_unknown() {
        assert!(matches!(
            OracleStub::try_new("containerd"),
            Err(HarnessError::UnknownOracle(_))
        ));
        assert_eq!(OracleStub::try_new("youki").unwrap().id(), OracleId::Youki);
    }

    #[test]
    fn compare_stubbed_pair_is_stubbed() {
        let owned = OwnedExecutorStub.create_stub("b1");
        let oracle = OracleStub::runc().create_stub("b1");
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Stubbed);
    }

    #[test]
    fn compare_measured_match_and_diverge() {
        let bundle = live_bundle("b1");
        let pin = live_pin();
        let owned_ok = OperationObservation::try_measured(
            ExecutorKind::Owned,
            OciOperation::Start,
            bundle.clone(),
            safe_outcome(0, "fp-a"),
        )
        .unwrap();
        let oracle_ok = OperationObservation::try_measured(
            ExecutorKind::Oracle {
                id: OracleId::Runc,
                pin: pin.clone(),
            },
            OciOperation::Start,
            bundle.clone(),
            safe_outcome(0, "fp-a"),
        )
        .unwrap();
        let oracle_bad = OperationObservation::try_measured(
            ExecutorKind::Oracle {
                id: OracleId::Runc,
                pin,
            },
            OciOperation::Start,
            bundle,
            safe_outcome(1, "fp-b"),
        )
        .unwrap();
        assert_eq!(
            compare_observations(&owned_ok, &oracle_ok),
            DiffVerdict::Match
        );
        assert_eq!(
            compare_observations(&owned_ok, &oracle_bad),
            DiffVerdict::Diverge
        );
    }

    #[test]
    fn asymmetric_execution_diverges() {
        let owned = OperationObservation::try_measured(
            ExecutorKind::Owned,
            OciOperation::Start,
            live_bundle("b1"),
            safe_outcome(0, "fp"),
        )
        .unwrap();
        let oracle = OracleStub::runc().start_stub("b1");
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn kill_signal_mismatch_diverges() {
        let owned = OperationObservation::stubbed_scaffold(
            ExecutorKind::Owned,
            OciOperation::Kill(KillSignal::Term),
            "b1",
        );
        let oracle = OperationObservation::stubbed_scaffold(
            ExecutorKind::Oracle {
                id: OracleId::Youki,
                pin: OraclePin::scaffold(),
            },
            OciOperation::Kill(KillSignal::Kill),
            "b1",
        );
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn kill_operation_always_carries_signal() {
        let kill = OwnedExecutorStub.kill_stub("b1", KillSignal::Hup);
        assert!(matches!(
            kill.operation(),
            OciOperation::Kill(KillSignal::Hup)
        ));
        assert_eq!(kill.kill_signal(), Some(KillSignal::Hup));
    }

    #[test]
    fn bundle_content_digest_mismatch_diverges() {
        let owned = OwnedExecutorStub
            .create_with_bundle(BundleIdentity::try_new("b1", "digest-a").unwrap());
        let oracle = OracleStub::runc()
            .create_with_bundle(BundleIdentity::try_new("b1", "digest-b").unwrap());
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn empty_bundle_digest_rejected() {
        assert_eq!(
            BundleIdentity::try_new("b1", "  ").unwrap_err(),
            HarnessError::EmptyBundleDigest
        );
    }

    #[test]
    fn measured_rejects_scaffold_bundle() {
        let err = OperationObservation::try_measured(
            ExecutorKind::Owned,
            OciOperation::Start,
            BundleIdentity::scaffold("b1"),
            safe_outcome(0, "fp"),
        )
        .unwrap_err();
        assert_eq!(err, HarnessError::ScaffoldBundleNotMeasured);
    }

    #[test]
    fn measured_rejects_scaffold_oracle_pin() {
        let err = OperationObservation::try_measured(
            ExecutorKind::Oracle {
                id: OracleId::Runc,
                pin: OraclePin::scaffold(),
            },
            OciOperation::Start,
            live_bundle("b1"),
            safe_outcome(0, "fp"),
        )
        .unwrap_err();
        assert_eq!(err, HarnessError::ScaffoldPinNotMeasured);
    }

    #[test]
    fn security_postcondition_mismatch_diverges() {
        let mut leaky = safe_outcome(0, "fp");
        leaky.security.fd_leak_absent = false;
        let bundle = live_bundle("b1");
        let pin = live_pin();
        let owned = OperationObservation::try_measured(
            ExecutorKind::Owned,
            OciOperation::Start,
            bundle.clone(),
            safe_outcome(0, "fp"),
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            ExecutorKind::Oracle {
                id: OracleId::Crun,
                pin,
            },
            OciOperation::Start,
            bundle,
            leaky,
        )
        .unwrap();
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn both_unsafe_equal_outcomes_diverge() {
        let mut unsafe_out = safe_outcome(0, "fp");
        unsafe_out.security.fd_leak_absent = false;
        let bundle = live_bundle("b1");
        let pin = live_pin();
        let owned = OperationObservation::try_measured(
            ExecutorKind::Owned,
            OciOperation::Start,
            bundle.clone(),
            unsafe_out.clone(),
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            ExecutorKind::Oracle {
                id: OracleId::Youki,
                pin,
            },
            OciOperation::Start,
            bundle,
            unsafe_out,
        )
        .unwrap();
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn matrix_requires_full_oracle_cve_coverage() {
        let incomplete = [ComparisonRecord::try_from_observations(
            "CVE-2019-5736",
            OwnedExecutorStub.create_stub("b1"),
            OracleStub::runc().create_stub("b1"),
        )
        .unwrap()];
        assert!(matches!(
            refuse_pairwise_match_as_conformance(&incomplete),
            Err(HarnessError::IncompleteMatrixCoverage(_))
        ));

        let mut cells = Vec::new();
        for (oracle, cve) in required_matrix_pairs() {
            cells.push(MatrixCell {
                oracle,
                pin: OraclePin::scaffold(),
                cve_id: cve.to_owned(),
                verdict: DiffVerdict::Stubbed,
            });
        }
        assert_eq!(
            aggregate_oracle_cve_matrix(&cells).unwrap(),
            MatrixAggregate::ScaffoldCoverageComplete
        );
    }

    #[test]
    fn free_form_match_cells_rejected() {
        let mut cells = Vec::new();
        for (oracle, cve) in required_matrix_pairs() {
            cells.push(MatrixCell {
                oracle,
                pin: OraclePin::scaffold(),
                cve_id: cve.to_owned(),
                verdict: DiffVerdict::Match,
            });
        }
        assert_eq!(
            aggregate_oracle_cve_matrix(&cells).unwrap_err(),
            HarnessError::FreeFormMatchForbidden
        );
    }

    #[test]
    fn comparison_records_bind_verdict_to_observations() {
        let mut records = Vec::new();
        for (oracle_id, cve) in required_matrix_pairs() {
            let oracle = OracleStub::try_new_pinned(oracle_id.as_str(), live_pin()).unwrap();
            let owned = OperationObservation::try_measured(
                ExecutorKind::Owned,
                OciOperation::Start,
                live_bundle("b1"),
                safe_outcome(0, "fp"),
            )
            .unwrap();
            let oracle_obs = OperationObservation::try_measured(
                oracle.kind(),
                OciOperation::Start,
                live_bundle("b1"),
                safe_outcome(0, "fp"),
            )
            .unwrap();
            records.push(ComparisonRecord::try_from_observations(cve, owned, oracle_obs).unwrap());
        }
        assert_eq!(
            aggregate_comparison_records(&records).unwrap(),
            MatrixAggregate::MeasuredCoverageComplete
        );
        assert!(records.iter().all(|r| r.verdict() == DiffVerdict::Match));
        assert!(records
            .iter()
            .all(|r| !r.oracle().kind().oracle_pin().unwrap().is_scaffold()));
    }

    #[test]
    fn missing_required_cve_fails_validation() {
        let mut root: Value = serde_json::from_str(CVE_OBLIGATIONS_JSON).unwrap();
        root["cve_regression_obligations"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row["id"] != "CVE-2019-5736");
        let json = serde_json::to_string(&root).unwrap();
        let err = validate_obligations_json(&json).unwrap_err();
        assert_eq!(err, HarnessError::MissingCve("CVE-2019-5736".into()));
    }

    #[test]
    fn swapped_cve_class_fails_validation() {
        let mut root: Value = serde_json::from_str(CVE_OBLIGATIONS_JSON).unwrap();
        for row in root["cve_regression_obligations"].as_array_mut().unwrap() {
            if row["id"] == "CVE-2019-5736" {
                row["class"] = Value::String("fd_leak".into());
            }
        }
        let json = serde_json::to_string(&root).unwrap();
        let err = validate_obligations_json(&json).unwrap_err();
        assert_eq!(
            err,
            HarnessError::CveClassMismatch {
                id: "CVE-2019-5736".into(),
                expected: "proc_self_exe_reexec".into(),
                got: "fd_leak".into(),
            }
        );
    }

    #[test]
    fn missing_blocker_fails_validation() {
        for required in REQUIRED_BLOCKERS {
            let mut root: Value = serde_json::from_str(CVE_OBLIGATIONS_JSON).unwrap();
            root["claim_posture"]["blocked_on"]
                .as_array_mut()
                .unwrap()
                .retain(|b| b.as_str() != Some(required));
            let json = serde_json::to_string(&root).unwrap();
            let err = validate_obligations_json(&json).unwrap_err();
            assert_eq!(err, HarnessError::MissingBlocker(required.to_owned()));
        }
    }

    #[test]
    fn unknown_blocker_fails_validation() {
        let mut root: Value = serde_json::from_str(CVE_OBLIGATIONS_JSON).unwrap();
        root["claim_posture"]["blocked_on"]
            .as_array_mut()
            .unwrap()
            .push(Value::String("NOT-A-BLOCKER".into()));
        let json = serde_json::to_string(&root).unwrap();
        let err = validate_obligations_json(&json).unwrap_err();
        assert_eq!(err, HarnessError::UnknownBlocker("NOT-A-BLOCKER".into()));
    }
}
