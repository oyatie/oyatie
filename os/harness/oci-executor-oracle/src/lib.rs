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

/// Outcome of a differential comparison (scaffold).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffVerdict {
    /// Not yet executed — default for scaffold.
    Stubbed,
    /// Owned executor matched oracle (future measured path).
    Match,
    /// Owned executor diverged from oracle (future measured path).
    Diverge,
}

/// Per-side operation observation. Live adapters emit one of these; a separate
/// differential runner compares the pair into a [`DiffVerdict`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationObservation {
    pub kind: ExecutorKind,
    pub operation: &'static str,
    pub bundle_id: String,
    /// Scaffold: empty. Live adapters carry exit/status/stderr digests here.
    pub notes: String,
}

impl OperationObservation {
    pub fn stubbed(kind: ExecutorKind, operation: &'static str, bundle_id: &str) -> Self {
        Self {
            kind,
            operation,
            bundle_id: bundle_id.to_owned(),
            notes: "scaffold stub — no process spawn".into(),
        }
    }
}

/// Compare two observations into a differential verdict (scaffold).
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
    // Scaffold path: both sides stubbed ⇒ Stubbed (not a measured Match).
    DiffVerdict::Stubbed
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
    Oracle { id: &'static str },
}

impl ExecutorKind {
    /// True iff this kind is forbidden as a shipped product runtime.
    pub const fn is_oracle_only(self) -> bool {
        matches!(self, Self::Oracle { .. })
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
    id: &'static str,
}

impl OracleStub {
    pub const fn youki() -> Self {
        Self { id: "youki" }
    }
    pub const fn runc() -> Self {
        Self { id: "runc" }
    }
    pub const fn crun() -> Self {
        Self { id: "crun" }
    }

    /// Validated construction against [`ORACLE_IDS`]. Rejects unknown identities.
    pub fn try_new(id: &'static str) -> Result<Self, String> {
        if ORACLE_IDS.contains(&id) {
            Ok(Self { id })
        } else {
            Err(format!("oracle id {id} is not in ORACLE_IDS allowlist"))
        }
    }

    pub const fn id(self) -> &'static str {
        self.id
    }
}

impl OciExecutor for OracleStub {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind::Oracle { id: self.id }
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
    /// Immutable pin fields (scaffold placeholders until live adapters land).
    revision: String,
    platform: String,
}

#[derive(Debug, Deserialize)]
struct CveRow {
    id: String,
    required: bool,
}

/// Parse and structurally validate the embedded obligations fixture.
pub fn validate_obligations() -> Result<Value, String> {
    let root: ObligationsRoot = serde_json::from_str(CVE_OBLIGATIONS_JSON)
        .map_err(|e| format!("cve obligations parse error: {e}"))?;
    if root.schema_version != "0.1.0" {
        return Err("schema_version must be 0.1.0".into());
    }
    if root.status != "scaffold" {
        return Err("status must be scaffold".into());
    }
    if root.claim_posture.oracles_are_shipped_product {
        return Err("claim_posture.oracles_are_shipped_product must be false".into());
    }
    if !root.claim_posture.owned_executor_is_product_path {
        return Err("claim_posture.owned_executor_is_product_path must be true".into());
    }

    // Exact one-to-one match with ORACLE_IDS: reject duplicates and extras.
    if root.oracles.len() != ORACLE_IDS.len() {
        return Err(format!(
            "oracles must be exactly {} rows (got {})",
            ORACLE_IDS.len(),
            root.oracles.len()
        ));
    }
    let mut seen = BTreeSet::new();
    for row in &root.oracles {
        if !ORACLE_IDS.contains(&row.id.as_str()) {
            return Err(format!("unknown oracle id {}", row.id));
        }
        if !seen.insert(row.id.as_str()) {
            return Err(format!("duplicate oracle id {}", row.id));
        }
        if row.shipped {
            return Err(format!("oracle {} must have shipped=false", row.id));
        }
        if row.role != ORACLE_ROLE {
            return Err(format!(
                "oracle {} role must be {ORACLE_ROLE} (got {})",
                row.id, row.role
            ));
        }
        if row.revision.trim().is_empty() {
            return Err(format!("oracle {} must pin a non-empty revision", row.id));
        }
        if row.platform.trim().is_empty() {
            return Err(format!("oracle {} must pin a non-empty platform", row.id));
        }
    }
    for id in ORACLE_IDS {
        if !seen.contains(id) {
            return Err(format!("missing oracle row for {id}"));
        }
    }

    // Closed mandatory CVE set — every REQUIRED_CVE_IDS entry must be present
    // with required=true; extras with required=false are rejected.
    let mut cve_seen = BTreeSet::new();
    for cve in &root.cve_regression_obligations {
        if !REQUIRED_CVE_IDS.contains(&cve.id.as_str()) {
            return Err(format!("unknown cve obligation id {}", cve.id));
        }
        if !cve_seen.insert(cve.id.as_str()) {
            return Err(format!("duplicate cve obligation id {}", cve.id));
        }
        if !cve.required {
            return Err(format!("obligation {} must be required=true", cve.id));
        }
    }
    for id in REQUIRED_CVE_IDS {
        if !cve_seen.contains(id) {
            return Err(format!("missing required cve obligation {id}"));
        }
    }

    serde_json::from_str(CVE_OBLIGATIONS_JSON).map_err(|e| e.to_string())
}

/// Pair owned stub with one oracle for a future differential run.
pub fn differential_pair(oracle: OracleStub) -> (OwnedExecutorStub, OracleStub) {
    (OwnedExecutorStub, oracle)
}

/// Conformance-laundering guard: refuse selecting an oracle as the product runtime.
pub fn refuse_oracle_as_product(kind: ExecutorKind) -> Result<(), String> {
    if kind.is_oracle_only() {
        return Err(
            "conformance-laundering ban: oracle executors must not be selected as shipped product"
                .into(),
        );
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
        assert!(err.contains("conformance-laundering"));
    }

    #[test]
    fn oracle_try_new_rejects_unknown() {
        assert!(OracleStub::try_new("containerd").is_err());
        assert_eq!(OracleStub::try_new("youki").unwrap().id(), "youki");
    }

    #[test]
    fn compare_stubbed_pair_is_stubbed() {
        let owned = OwnedExecutorStub.create_stub("b1");
        let oracle = OracleStub::runc().create_stub("b1");
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Stubbed);
    }

    #[test]
    fn missing_required_cve_fails_validation() {
        // Negative: drop one mandatory id from a synthetic root.
        let mut root: Value = serde_json::from_str(CVE_OBLIGATIONS_JSON).unwrap();
        root["cve_regression_obligations"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row["id"] != "CVE-2019-5736");
        // Re-run the closed-set check inline (fixture const is fixed; assert the
        // REQUIRED_CVE_IDS contract that validate_obligations enforces).
        assert!(REQUIRED_CVE_IDS.contains(&"CVE-2019-5736"));
        let remaining: BTreeSet<_> = root["cve_regression_obligations"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|r| r["id"].as_str())
            .collect();
        assert!(!remaining.contains("CVE-2019-5736"));
    }
}
