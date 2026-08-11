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
    CveClassMismatch { id: String, expected: String, got: String },
    MissingBlocker(String),
    UnknownBlocker(String),
    DuplicateBlocker(String),
    EmptyBundleDigest,
    IncompleteMatrixCoverage(String),
    DuplicateMatrixCell(String),
    UnknownMatrixCve(String),
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
            Self::CveClassMismatch { id, expected, got } => write!(
                f,
                "cve {id} class must be {expected} (got {got})"
            ),
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

/// Outcome of a differential comparison (scaffold).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffVerdict {
    /// Not yet executed — default for scaffold.
    Stubbed,
    /// Owned executor matched oracle (measured path).
    Match,
    /// Owned executor diverged from oracle (measured path).
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

/// Content-derived OCI bundle identity (id alone is insufficient).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleIdentity {
    pub bundle_id: String,
    /// Digest of config/rootfs/mounts — empty rejected at construction.
    pub content_digest: String,
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
    pub fn kind(&self) -> ExecutorKind {
        self.kind
    }
    pub fn operation(&self) -> OciOperation {
        self.operation
    }
    pub fn bundle(&self) -> &BundleIdentity {
        &self.bundle
    }
    pub fn bundle_id(&self) -> &str {
        &self.bundle.bundle_id
    }
    pub fn content_digest(&self) -> &str {
        &self.bundle.content_digest
    }
    /// Kill signal when operation is [`OciOperation::Kill`]; otherwise `None`.
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

    pub fn measured(
        kind: ExecutorKind,
        operation: OciOperation,
        bundle: BundleIdentity,
        outcome: MeasuredOutcome,
    ) -> Self {
        Self {
            kind,
            operation,
            bundle,
            execution: ExecutionState::Measured(outcome),
        }
    }
}

/// Compare two observations into a differential verdict.
///
/// Pairwise `Match` is **not** a conformance / Accept claim — callers must run
/// [`aggregate_oracle_cve_matrix`] over the full oracle × CVE set first.
pub fn compare_observations(
    owned: &OperationObservation,
    oracle: &OperationObservation,
) -> DiffVerdict {
    if !owned.kind.is_owned_product() || !oracle.kind.is_oracle_only() {
        return DiffVerdict::Diverge;
    }
    // Operation equality includes Kill(signal); bundle compares id + content digest.
    if owned.operation != oracle.operation || owned.bundle != oracle.bundle {
        return DiffVerdict::Diverge;
    }
    match (&owned.execution, &oracle.execution) {
        (ExecutionState::Stubbed, ExecutionState::Stubbed) => DiffVerdict::Stubbed,
        // Equality includes SecurityPostconditions — exit-only Match is unreachable.
        (ExecutionState::Measured(a), ExecutionState::Measured(b)) if a == b => DiffVerdict::Match,
        (ExecutionState::Measured(_), ExecutionState::Measured(_)) => DiffVerdict::Diverge,
        // Partial wiring / failed differential run — not the all-scaffold case.
        _ => DiffVerdict::Diverge,
    }
}

/// One cell of the required oracle × CVE differential matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixCell {
    pub oracle: OracleId,
    pub cve_id: String,
    pub verdict: DiffVerdict,
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

/// Aggregate the exact oracle × CVE matrix. Never yields Accept.
pub fn aggregate_oracle_cve_matrix(cells: &[MatrixCell]) -> Result<MatrixAggregate, HarnessError> {
    validate_matrix_coverage(cells)?;
    if cells.iter().any(|c| c.verdict == DiffVerdict::Stubbed) {
        Ok(MatrixAggregate::ScaffoldCoverageComplete)
    } else {
        Ok(MatrixAggregate::MeasuredCoverageComplete)
    }
}

/// Pairwise Match alone must not be treated as product conformance.
pub fn refuse_pairwise_match_as_conformance(
    cells: &[MatrixCell],
) -> Result<MatrixAggregate, HarnessError> {
    if cells.is_empty() {
        return Err(HarnessError::ConformanceWithoutFullMatrix);
    }
    aggregate_oracle_cve_matrix(cells)
}

/// Identity of an OCI executor implementation behind the shared trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorKind {
    /// Forever product path (owned-from-spec shim library). Bodies arrive later.
    Owned,
    /// Differential oracle only — must never be selected as shipped runtime.
    Oracle(OracleId),
}

impl ExecutorKind {
    /// True iff this kind is forbidden as a shipped product runtime.
    pub const fn is_oracle_only(self) -> bool {
        matches!(self, Self::Oracle(_))
    }

    /// True iff this kind is the forever product executor.
    pub const fn is_owned_product(self) -> bool {
        matches!(self, Self::Owned)
    }
}

/// Minimal OCI create/start/kill/delete surface shared by owned executor + oracles.
///
/// Scaffold: methods return typed observations so a differential runner can
/// compare sides later; bodies do not spawn processes.
pub trait OciExecutor {
    fn kind(&self) -> ExecutorKind;

    /// Scaffold create — no filesystem / namespace mutation.
    fn create_stub(&self, bundle_id: &str) -> OperationObservation {
        OperationObservation::stubbed_scaffold(self.kind(), OciOperation::Create, bundle_id)
    }

    /// Scaffold start — no process spawn.
    fn start_stub(&self, bundle_id: &str) -> OperationObservation {
        OperationObservation::stubbed_scaffold(self.kind(), OciOperation::Start, bundle_id)
    }

    /// Scaffold kill — signal is part of the closed operation type.
    fn kill_stub(&self, bundle_id: &str, signal: KillSignal) -> OperationObservation {
        OperationObservation::stubbed_scaffold(
            self.kind(),
            OciOperation::Kill(signal),
            bundle_id,
        )
    }

    /// Scaffold delete — no cleanup side effects.
    fn delete_stub(&self, bundle_id: &str) -> OperationObservation {
        OperationObservation::stubbed_scaffold(self.kind(), OciOperation::Delete, bundle_id)
    }

    /// Create with an explicit content-derived bundle identity.
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

/// Oracle adapter stub — identity only; never ship. Construction is allowlisted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OracleStub {
    id: OracleId,
}

impl OracleStub {
    pub const fn youki() -> Self {
        Self { id: OracleId::Youki }
    }
    pub const fn runc() -> Self {
        Self { id: OracleId::Runc }
    }
    pub const fn crun() -> Self {
        Self { id: OracleId::Crun }
    }

    pub fn try_new(id: &str) -> Result<Self, HarnessError> {
        Ok(Self {
            id: OracleId::try_from_str(id)?,
        })
    }

    pub const fn id(self) -> OracleId {
        self.id
    }
}

impl OciExecutor for OracleStub {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind::Oracle(self.id)
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

    // Exact blocker set — programme lock labels, not MPV2 IDs.
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
        if row.revision.trim().is_empty() {
            return Err(HarnessError::OraclePin(format!(
                "oracle {} must pin a non-empty revision",
                row.id
            )));
        }
        if row.platform.trim().is_empty() {
            return Err(HarnessError::OraclePin(format!(
                "oracle {} must pin a non-empty platform",
                row.id
            )));
        }
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
pub fn refuse_oracle_as_product(kind: ExecutorKind) -> Result<(), HarnessError> {
    if kind.is_oracle_only() {
        return Err(HarnessError::ConformanceLaundering);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obligations_fixture_validates() {
        validate_obligations().expect("scaffold obligations must validate");
    }

    #[test]
    fn oracles_are_oracle_only() {
        for stub in [OracleStub::youki(), OracleStub::runc(), OracleStub::crun()] {
            assert!(stub.kind().is_oracle_only());
            assert!(!stub.kind().is_owned_product());
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
        refuse_oracle_as_product(owned.kind()).expect("owned ok");
    }

    #[test]
    fn refuse_shipping_youki() {
        let err = refuse_oracle_as_product(OracleStub::youki().kind()).unwrap_err();
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
        let outcome_a = MeasuredOutcome {
            exit_code: 0,
            status: "exited".into(),
            stderr_fingerprint: "fp-a".into(),
            security: SecurityPostconditions::all_held(),
        };
        let outcome_b = MeasuredOutcome {
            exit_code: 1,
            status: "exited".into(),
            stderr_fingerprint: "fp-b".into(),
            security: SecurityPostconditions::all_held(),
        };
        let bundle = BundleIdentity::scaffold("b1");
        let owned_ok = OperationObservation::measured(
            ExecutorKind::Owned,
            OciOperation::Start,
            bundle.clone(),
            outcome_a.clone(),
        );
        let oracle_ok = OperationObservation::measured(
            ExecutorKind::Oracle(OracleId::Runc),
            OciOperation::Start,
            bundle.clone(),
            outcome_a,
        );
        let oracle_bad = OperationObservation::measured(
            ExecutorKind::Oracle(OracleId::Runc),
            OciOperation::Start,
            bundle,
            outcome_b,
        );
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
        let owned = OperationObservation::measured(
            ExecutorKind::Owned,
            OciOperation::Start,
            BundleIdentity::scaffold("b1"),
            MeasuredOutcome {
                exit_code: 0,
                status: "exited".into(),
                stderr_fingerprint: "fp".into(),
                security: SecurityPostconditions::all_held(),
            },
        );
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
            ExecutorKind::Oracle(OracleId::Youki),
            OciOperation::Kill(KillSignal::Kill),
            "b1",
        );
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn bundle_content_digest_mismatch_diverges() {
        let owned = OwnedExecutorStub.create_with_bundle(
            BundleIdentity::try_new("b1", "digest-a").unwrap(),
        );
        let oracle = OracleStub::runc().create_with_bundle(
            BundleIdentity::try_new("b1", "digest-b").unwrap(),
        );
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
    fn security_postcondition_mismatch_diverges() {
        let base = MeasuredOutcome {
            exit_code: 0,
            status: "exited".into(),
            stderr_fingerprint: "fp".into(),
            security: SecurityPostconditions::all_held(),
        };
        let mut leaky = base.clone();
        leaky.security.fd_leak_absent = false;
        let bundle = BundleIdentity::scaffold("b1");
        let owned = OperationObservation::measured(
            ExecutorKind::Owned,
            OciOperation::Start,
            bundle.clone(),
            base,
        );
        let oracle = OperationObservation::measured(
            ExecutorKind::Oracle(OracleId::Crun),
            OciOperation::Start,
            bundle,
            leaky,
        );
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn matrix_requires_full_oracle_cve_coverage() {
        // Single pairwise Match is not enough.
        let incomplete = [MatrixCell {
            oracle: OracleId::Runc,
            cve_id: "CVE-2019-5736".into(),
            verdict: DiffVerdict::Match,
        }];
        assert!(matches!(
            refuse_pairwise_match_as_conformance(&incomplete),
            Err(HarnessError::IncompleteMatrixCoverage(_))
        ));

        let mut cells = Vec::new();
        for (oracle, cve) in required_matrix_pairs() {
            cells.push(MatrixCell {
                oracle,
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
    fn kill_operation_always_carries_signal() {
        let kill = OwnedExecutorStub.kill_stub("b1", KillSignal::Hup);
        assert!(matches!(
            kill.operation(),
            OciOperation::Kill(KillSignal::Hup)
        ));
        assert_eq!(kill.kill_signal(), Some(KillSignal::Hup));
        assert_eq!(OciOperation::Create.kill_signal(), None);
        assert_eq!(OciOperation::Start.kill_signal(), None);
        assert_eq!(OciOperation::Delete.kill_signal(), None);
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
