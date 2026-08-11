#![forbid(unsafe_code)]
//! Differential-oracle harness skeleton for the owned OCI executor.
//!
//! Law (Round-2): the forever executor is an **owned** library of the per-sandbox
//! shim, built from the OCI runtime-spec. `youki` / `runc` / `crun` are pinned
//! **differential oracles** and CVE regression fixtures only — never shipped
//! product. Shipping an oracle to green a gate the owned executor did not pass
//! is **conformance laundering**.
//!
//! This crate is a hermetic scaffold: trait surface + fixture inventory + stub
//! oracle adapters. It does **not** invoke youki/runc/crun binaries, does **not**
//! PORT containerd, and does **not** claim W0/`w0_ready` readiness.
//!
//! data_class: PUBLIC

use serde::Deserialize;
use serde_json::Value;

/// Embedded CVE / adversarial obligation inventory (scaffold).
pub const CVE_OBLIGATIONS_JSON: &str =
    include_str!("../fixtures/cve-regression-obligations-v0.1.0.json");

/// Closed set of differential oracle identities (never product).
pub const ORACLE_IDS: [&str; 3] = ["youki", "runc", "crun"];

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
/// Scaffold: methods exist so differential harnesses can bind both sides; bodies
/// are stubbed and return [`DiffVerdict::Stubbed`] without process spawn.
pub trait OciExecutor {
    fn kind(&self) -> ExecutorKind;

    /// Scaffold create — no filesystem / namespace mutation.
    fn create_stub(&self, _bundle_id: &str) -> DiffVerdict {
        DiffVerdict::Stubbed
    }

    /// Scaffold start — no process spawn.
    fn start_stub(&self, _bundle_id: &str) -> DiffVerdict {
        DiffVerdict::Stubbed
    }

    /// Scaffold kill — no signals.
    fn kill_stub(&self, _bundle_id: &str) -> DiffVerdict {
        DiffVerdict::Stubbed
    }

    /// Scaffold delete — no cleanup side effects.
    fn delete_stub(&self, _bundle_id: &str) -> DiffVerdict {
        DiffVerdict::Stubbed
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

/// Oracle adapter stub — identity only; never ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OracleStub {
    pub id: &'static str,
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
    shipped: bool,
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
    for id in ORACLE_IDS {
        let row = root
            .oracles
            .iter()
            .find(|o| o.id == id)
            .ok_or_else(|| format!("missing oracle row for {id}"))?;
        if row.shipped {
            return Err(format!("oracle {id} must have shipped=false"));
        }
    }
    if root.cve_regression_obligations.is_empty() {
        return Err("cve_regression_obligations must be non-empty".into());
    }
    for cve in &root.cve_regression_obligations {
        if !cve.required {
            return Err(format!("obligation {} must be required=true", cve.id));
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
            assert_eq!(stub.create_stub("bundle"), DiffVerdict::Stubbed);
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
}
