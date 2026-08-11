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
            | Self::CveNotRequired(m) => write!(f, "{m}"),
            Self::ConformanceLaundering => write!(
                f,
                "conformance-laundering ban: oracle executors must not be selected as shipped product"
            ),
        }
    }
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

/// Typed measured outcome carried by live adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasuredOutcome {
    /// Opaque digest of exit/status/stderr (adapter-defined).
    pub result_digest: String,
}

/// Per-side operation observation. Live adapters emit one of these; a separate
/// differential runner compares the pair into a [`DiffVerdict`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationObservation {
    pub kind: ExecutorKind,
    pub operation: &'static str,
    pub bundle_id: String,
    /// False for scaffold stubs; true once a live adapter executed the op.
    pub executed: bool,
    /// Present only when `executed` — compared for Match/Diverge.
    pub measured: Option<MeasuredOutcome>,
}

impl OperationObservation {
    pub fn stubbed(kind: ExecutorKind, operation: &'static str, bundle_id: &str) -> Self {
        Self {
            kind,
            operation,
            bundle_id: bundle_id.to_owned(),
            executed: false,
            measured: None,
        }
    }

    pub fn measured(
        kind: ExecutorKind,
        operation: &'static str,
        bundle_id: &str,
        result_digest: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            operation,
            bundle_id: bundle_id.to_owned(),
            executed: true,
            measured: Some(MeasuredOutcome {
                result_digest: result_digest.into(),
            }),
        }
    }
}

/// Compare two observations into a differential verdict.
pub fn compare_observations(
    owned: &OperationObservation,
    oracle: &OperationObservation,
) -> DiffVerdict {
    if !owned.kind.is_owned_product() || !oracle.kind.is_oracle_only() {
        return DiffVerdict::Diverge;
    }
    if owned.operation != oracle.operation || owned.bundle_id != oracle.bundle_id {
        return DiffVerdict::Diverge;
    }
    if !owned.executed || !oracle.executed {
        return DiffVerdict::Stubbed;
    }
    match (&owned.measured, &oracle.measured) {
        (Some(a), Some(b)) if a.result_digest == b.result_digest => DiffVerdict::Match,
        (Some(_), Some(_)) => DiffVerdict::Diverge,
        _ => DiffVerdict::Diverge,
    }
}

/// Kill signal seam aligned with `os_runtime` / containerd task Signal (scaffold).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillSignal {
    Term,
    Kill,
    Hup,
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
        OperationObservation::stubbed(self.kind(), "create", bundle_id)
    }

    /// Scaffold start — no process spawn.
    fn start_stub(&self, bundle_id: &str) -> OperationObservation {
        OperationObservation::stubbed(self.kind(), "start", bundle_id)
    }

    /// Scaffold kill — signal carried for future OCI kill semantics comparison.
    fn kill_stub(&self, bundle_id: &str, _signal: KillSignal) -> OperationObservation {
        OperationObservation::stubbed(self.kind(), "kill", bundle_id)
    }

    /// Scaffold delete — no cleanup side effects.
    fn delete_stub(&self, bundle_id: &str) -> OperationObservation {
        OperationObservation::stubbed(self.kind(), "delete", bundle_id)
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

#[derive(Debug, Deserialize)]
struct ObligationsRoot {
    schema_version: String,
    status: String,
    claim_posture: ClaimPosture,
    oracles: Vec<OracleRow>,
    cve_regression_obligations: Vec<CveRow>,
}

#[derive(Debug, Deserialize)]
struct ClaimPosture {
    oracles_are_shipped_product: bool,
    owned_executor_is_product_path: bool,
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
    required: bool,
}

fn validate_root(root: &ObligationsRoot) -> Result<(), HarnessError> {
    if root.schema_version != "0.1.0" {
        return Err(HarnessError::Schema("schema_version must be 0.1.0".into()));
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
        if !REQUIRED_CVE_IDS.contains(&cve.id.as_str()) {
            return Err(HarnessError::UnknownCve(cve.id.clone()));
        }
        if !cve_seen.insert(cve.id.as_str()) {
            return Err(HarnessError::DuplicateCve(cve.id.clone()));
        }
        if !cve.required {
            return Err(HarnessError::CveNotRequired(cve.id.clone()));
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
            assert_eq!(stub.create_stub("bundle").operation, "create");
            assert!(!stub.create_stub("bundle").executed);
            assert_eq!(
                stub.kill_stub("bundle", KillSignal::Term).operation,
                "kill"
            );
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
        let owned_ok = OperationObservation::measured(
            ExecutorKind::Owned,
            "start",
            "b1",
            "digest-a",
        );
        let oracle_ok = OperationObservation::measured(
            ExecutorKind::Oracle(OracleId::Runc),
            "start",
            "b1",
            "digest-a",
        );
        let oracle_bad = OperationObservation::measured(
            ExecutorKind::Oracle(OracleId::Runc),
            "start",
            "b1",
            "digest-b",
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
}
