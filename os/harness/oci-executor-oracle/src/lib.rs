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
use std::collections::{BTreeMap, BTreeSet};
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

/// Closed platform allowlist for non-empty pin identities (exact strings).
pub const ALLOWED_PIN_PLATFORMS: [&str; 2] = ["linux/amd64", "linux/arm64"];

/// Canonical adversarial fixture bytes per [`REQUIRED_CVE_IDS`] (crate-local SSOT).
pub const CANONICAL_CVE_FIXTURES: [(&str, &[u8]); 3] = [
    (
        "CVE-2019-5736",
        b"canonical-adversarial:CVE-2019-5736:proc_self_exe_reexec",
    ),
    (
        "CVE-2024-21626",
        b"canonical-adversarial:CVE-2024-21626:fd_leak",
    ),
    (
        "CVE-MOUNT-SYMLINK-RACE",
        b"canonical-adversarial:CVE-MOUNT-SYMLINK-RACE:mount_symlink_race",
    ),
];

/// Look up canonical fixture bytes for a required CVE id.
pub fn canonical_cve_fixture_bytes(cve_id: &str) -> Option<&'static [u8]> {
    CANONICAL_CVE_FIXTURES
        .iter()
        .find(|(id, _)| *id == cve_id)
        .map(|(_, bytes)| *bytes)
}

/// Build canonical [`CveFixtureMaterial`] for a required CVE id.
pub fn canonical_cve_fixture_material(cve_id: &str) -> Result<CveFixtureMaterial<'static>, HarnessError> {
    let Some(fixture_bytes) = canonical_cve_fixture_bytes(cve_id) else {
        return Err(HarnessError::UnknownCve(cve_id.to_owned()));
    };
    debug_assert!(!fixture_bytes.is_empty());
    Ok(CveFixtureMaterial {
        cve_id: CANONICAL_CVE_FIXTURES
            .iter()
            .find(|(id, _)| *id == cve_id)
            .map(|(id, _)| *id)
            .expect("canonical bytes imply id"),
        fixture_bytes,
    })
}

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
    OwnedPin(String),
    UnknownCve(String),
    DuplicateCve(String),
    MissingCve(String),
    CveNotRequired(String),
    CveClassMismatch {
        id: String,
        expected: String,
        got: String,
    },
    /// Fixture bytes empty or not exactly the canonical material for the CVE.
    NonCanonicalCveFixture(String),
    MissingBlocker(String),
    UnknownBlocker(String),
    DuplicateBlocker(String),
    EmptyBundleDigest,
    ScaffoldBundleNotMeasured,
    ScaffoldPinNotMeasured,
    WeakBundleDigestNotMeasured,
    MissingFixtureReceipt,
    IncompleteMatrixCoverage(String),
    DuplicateMatrixCell(String),
    UnknownMatrixCve(String),
    InconsistentOraclePin(String),
    InconsistentOwnedPin(String),
    MatrixDivergence(String),
    MissingCveExecution,
    CveExecutionMismatch {
        expected: String,
        owned: Option<String>,
        oracle: Option<String>,
    },
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
            | Self::OwnedPin(m)
            | Self::UnknownCve(m)
            | Self::DuplicateCve(m)
            | Self::MissingCve(m)
            | Self::CveNotRequired(m)
            | Self::NonCanonicalCveFixture(m)
            | Self::MissingBlocker(m)
            | Self::UnknownBlocker(m)
            | Self::DuplicateBlocker(m)
            | Self::IncompleteMatrixCoverage(m)
            | Self::DuplicateMatrixCell(m)
            | Self::UnknownMatrixCve(m)
            | Self::InconsistentOraclePin(m)
            | Self::InconsistentOwnedPin(m)
            | Self::MatrixDivergence(m) => write!(f, "{m}"),
            Self::EmptyBundleDigest => write!(f, "bundle content_digest must be non-empty"),
            Self::ScaffoldBundleNotMeasured => {
                write!(f, "measured observations cannot use scaffold bundle digests")
            }
            Self::WeakBundleDigestNotMeasured => {
                write!(
                    f,
                    "measured observations require sha256: + 64 lowercase hex digests (oya1:/scaffold: rejected)"
                )
            }
            Self::ScaffoldPinNotMeasured => {
                write!(f, "measured observations cannot use scaffold oracle pins")
            }
            Self::MissingFixtureReceipt => {
                write!(
                    f,
                    "measured CVE execution requires CveExecutionId::from_fixture_material receipt"
                )
            }
            Self::MissingCveExecution => {
                write!(f, "comparison requires CveExecutionId on both observations")
            }
            Self::CveExecutionMismatch {
                expected,
                owned,
                oracle,
            } => write!(
                f,
                "cve execution mismatch: expected {expected}, owned={owned:?}, oracle={oracle:?}"
            ),
            Self::FreeFormMatchForbidden => write!(
                f,
                "Match/Diverge matrix cells must be derived from ComparisonRecord"
            ),
            Self::NotOracleObservation => {
                write!(f, "comparison oracle side must be ExecutorKind::Oracle")
            }
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

/// FNV-1a 64-bit (local; no crate dep) for hermetic digests / fixture receipts.
fn fnv1a64(data: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn push_len_prefixed(buf: &mut Vec<u8>, part: &[u8]) {
    buf.extend_from_slice(&(part.len() as u64).to_le_bytes());
    buf.extend_from_slice(part);
}

fn is_lowercase_hex(s: &str, expected_len: usize) -> bool {
    s.len() == expected_len && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Immutable pin form (measured path).
///
/// Accepted revisions:
/// - `sha256:` + exactly 64 lowercase hex chars
/// - `git:` + exactly 40 lowercase hex chars
///
/// Rejected (mutable / unbound): empty, `main`, `master`, `latest`, `HEAD`, and
/// any other non-immutable label. Scaffold pin [`SCAFFOLD_PIN_REVISION`] is
/// accepted by constructors for inventory/stub use only — measured path rejects it.
fn is_immutable_revision(revision: &str) -> bool {
    if let Some(hex) = revision.strip_prefix("sha256:") {
        return is_lowercase_hex(hex, 64);
    }
    if let Some(hex) = revision.strip_prefix("git:") {
        return is_lowercase_hex(hex, 40);
    }
    false
}

fn is_mutable_revision_label(revision: &str) -> bool {
    matches!(
        revision,
        "" | "main" | "master" | "latest" | "HEAD"
    )
}

fn is_scaffold_revision(revision: &str) -> bool {
    revision == SCAFFOLD_PIN_REVISION || revision.starts_with("scaffold:")
}

fn validate_pin_revision(revision: &str, pin_kind: &str) -> Result<(), HarnessError> {
    let rev = revision.trim();
    if is_mutable_revision_label(rev) {
        let msg = format!("{pin_kind} revision rejects mutable/empty label {rev:?}");
        return Err(if pin_kind == "owned" {
            HarnessError::OwnedPin(msg)
        } else {
            HarnessError::OraclePin(msg)
        });
    }
    if is_scaffold_revision(rev) || is_immutable_revision(rev) {
        return Ok(());
    }
    let msg = format!(
        "{pin_kind} revision must be sha256:<64 lowercase hex>, git:<40 lowercase hex>, or scaffold pin (got {rev:?})"
    );
    Err(if pin_kind == "owned" {
        HarnessError::OwnedPin(msg)
    } else {
        HarnessError::OraclePin(msg)
    })
}

fn validate_pin_platform(platform: &str, pin_kind: &str) -> Result<(), HarnessError> {
    let platform = platform.trim();
    if !ALLOWED_PIN_PLATFORMS.contains(&platform) {
        let msg = format!(
            "{pin_kind} platform must be one of {ALLOWED_PIN_PLATFORMS:?} (got {platform:?})"
        );
        return Err(if pin_kind == "owned" {
            HarnessError::OwnedPin(msg)
        } else {
            HarnessError::OraclePin(msg)
        });
    }
    Ok(())
}

/// Closed oracle identity enum — the only values `ExecutorKind::Oracle` may carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    /// Construct a pin. Accepts scaffold inventory pins or immutable `sha256:`/`git:` forms.
    /// Rejects mutable labels (`main`/`master`/`latest`/`HEAD`/empty) and other unbound strings.
    pub fn try_new(revision: &str, platform: &str) -> Result<Self, HarnessError> {
        validate_pin_platform(platform, "oracle")?;
        validate_pin_revision(revision, "oracle")?;
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
        is_scaffold_revision(&self.revision)
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn platform(&self) -> &str {
        &self.platform
    }
}

/// Owned executor build pin (revision + platform) — same immutability rules as [`OraclePin`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedPin {
    revision: String,
    platform: String,
}

impl OwnedPin {
    pub fn try_new(revision: &str, platform: &str) -> Result<Self, HarnessError> {
        validate_pin_platform(platform, "owned")?;
        validate_pin_revision(revision, "owned")?;
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
        is_scaffold_revision(&self.revision)
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

/// Byte inputs that uniquely determine a measured bundle content digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundleInputs<'a> {
    pub config: &'a [u8],
    pub rootfs_fingerprint: &'a [u8],
    pub mounts_fingerprint: &'a [u8],
}

/// Content-derived OCI bundle identity (id alone is insufficient). Fields private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleIdentity {
    bundle_id: String,
    content_digest: String,
}

impl BundleIdentity {
    /// Derive a deterministic `oya1:{fnv1a64:016x}` digest from bundle byte inputs.
    ///
    /// Stub/scaffold hermetic tests only — **rejected** on the measured path.
    pub fn from_inputs(bundle_id: &str, inputs: &BundleInputs<'_>) -> Self {
        let mut buf = Vec::new();
        push_len_prefixed(&mut buf, inputs.config);
        push_len_prefixed(&mut buf, inputs.rootfs_fingerprint);
        push_len_prefixed(&mut buf, inputs.mounts_fingerprint);
        let digest = format!("oya1:{:016x}", fnv1a64(&buf));
        Self {
            bundle_id: bundle_id.to_owned(),
            content_digest: digest,
        }
    }

    /// Measured-path constructor: `content_digest = sha256:` + 64 lowercase hex.
    pub fn try_sha256_hex(bundle_id: &str, hex64: &str) -> Result<Self, HarnessError> {
        if bundle_id.is_empty() {
            return Err(HarnessError::EmptyBundleDigest);
        }
        if !is_lowercase_hex(hex64, 64) {
            return Err(HarnessError::WeakBundleDigestNotMeasured);
        }
        Ok(Self {
            bundle_id: bundle_id.to_owned(),
            content_digest: format!("sha256:{hex64}"),
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
        self.content_digest.starts_with("scaffold:")
    }

    /// True iff digest is the measured form `sha256:` + 64 lowercase hex.
    pub fn is_sha256_digest(&self) -> bool {
        self.content_digest
            .strip_prefix("sha256:")
            .is_some_and(|hex| is_lowercase_hex(hex, 64))
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

/// CVE fixture bytes bound into a measured execution receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CveFixtureMaterial<'a> {
    pub cve_id: &'a str,
    pub fixture_bytes: &'a [u8],
}

/// CVE fixture binding — only REQUIRED_CVE_IDS; class from REQUIRED_CVE_CLASSES.
///
/// Measured path IDs must carry a fixture content receipt from
/// [`CveExecutionId::from_fixture_material`]. Bare [`try_new`] alone is insufficient.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CveExecutionId {
    cve_id: String,
    class: String,
    /// `fnv1a64:{016x}` over len-prefixed `(cve_id || fixture_bytes)`; empty = bare/non-measured.
    fixture_receipt: String,
}

impl CveExecutionId {
    /// Allowlist + class only — **not** measured-ready (no fixture receipt).
    pub fn try_new(cve_id: &str) -> Result<Self, HarnessError> {
        let Some(class) = expected_cve_class(cve_id) else {
            return Err(HarnessError::UnknownCve(cve_id.to_owned()));
        };
        Ok(Self {
            cve_id: cve_id.to_owned(),
            class: class.to_owned(),
            fixture_receipt: String::new(),
        })
    }

    /// Measured-path constructor: validates CVE allowlist, requires exact
    /// canonical fixture bytes for that CVE, embeds FNV-1a receipt over
    /// len-prefixed `(cve_id || fixture_bytes)`.
    pub fn from_fixture_material(material: &CveFixtureMaterial<'_>) -> Result<Self, HarnessError> {
        let Some(class) = expected_cve_class(material.cve_id) else {
            return Err(HarnessError::UnknownCve(material.cve_id.to_owned()));
        };
        let Some(canonical) = canonical_cve_fixture_bytes(material.cve_id) else {
            return Err(HarnessError::UnknownCve(material.cve_id.to_owned()));
        };
        if material.fixture_bytes.is_empty() || material.fixture_bytes != canonical {
            return Err(HarnessError::NonCanonicalCveFixture(format!(
                "fixture bytes for {} must be the non-empty canonical adversarial material",
                material.cve_id
            )));
        }
        let mut buf = Vec::new();
        push_len_prefixed(&mut buf, material.cve_id.as_bytes());
        push_len_prefixed(&mut buf, material.fixture_bytes);
        let fixture_receipt = format!("fnv1a64:{:016x}", fnv1a64(&buf));
        Ok(Self {
            cve_id: material.cve_id.to_owned(),
            class: class.to_owned(),
            fixture_receipt,
        })
    }

    pub fn cve_id(&self) -> &str {
        &self.cve_id
    }

    pub fn class(&self) -> &str {
        &self.class
    }

    pub fn fixture_receipt(&self) -> &str {
        &self.fixture_receipt
    }

    /// True when constructed via [`from_fixture_material`] (non-empty receipt).
    pub fn has_fixture_receipt(&self) -> bool {
        !self.fixture_receipt.is_empty()
    }
}

/// Private identity variants — `Owned` is not constructible outside this module.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ExecutorKindInner {
    Owned { pin: OwnedPin },
    Oracle { id: OracleId, pin: OraclePin },
}

/// Identity of an OCI executor implementation behind the shared trait.
///
/// `Owned` is sealed: only [`OwnedExecutorStub::kind`] can construct it.
/// Oracle kinds come from [`OracleStub`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutorKind(ExecutorKindInner);

impl ExecutorKind {
    pub const fn is_oracle_only(&self) -> bool {
        matches!(self.0, ExecutorKindInner::Oracle { .. })
    }

    pub const fn is_owned_product(&self) -> bool {
        matches!(self.0, ExecutorKindInner::Owned { .. })
    }

    pub fn oracle_id(&self) -> Option<OracleId> {
        match &self.0 {
            ExecutorKindInner::Oracle { id, .. } => Some(*id),
            ExecutorKindInner::Owned { .. } => None,
        }
    }

    pub fn oracle_pin(&self) -> Option<&OraclePin> {
        match &self.0 {
            ExecutorKindInner::Oracle { pin, .. } => Some(pin),
            ExecutorKindInner::Owned { .. } => None,
        }
    }

    pub fn owned_pin(&self) -> Option<&OwnedPin> {
        match &self.0 {
            ExecutorKindInner::Owned { pin } => Some(pin),
            ExecutorKindInner::Oracle { .. } => None,
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
    cve_execution: Option<CveExecutionId>,
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
    pub fn cve_execution(&self) -> Option<&CveExecutionId> {
        self.cve_execution.as_ref()
    }

    pub fn stubbed(kind: ExecutorKind, operation: OciOperation, bundle: BundleIdentity) -> Self {
        Self {
            kind,
            operation,
            bundle,
            execution: ExecutionState::Stubbed,
            cve_execution: None,
        }
    }

    pub fn stubbed_scaffold(
        kind: ExecutorKind,
        operation: OciOperation,
        bundle_id: &str,
    ) -> Self {
        Self::stubbed(kind, operation, BundleIdentity::scaffold(bundle_id))
    }

    /// Measured construction rejects scaffold/`oya1:` bundles and scaffold pins.
    ///
    /// When a CVE binding is present it must carry a fixture receipt from
    /// [`CveExecutionId::from_fixture_material`]. Matrix measured comparisons
    /// require [`Some`] via [`ComparisonRecord::try_from_observations`].
    pub fn try_measured(
        kind: ExecutorKind,
        operation: OciOperation,
        bundle: BundleIdentity,
        cve: Option<CveExecutionId>,
        outcome: MeasuredOutcome,
    ) -> Result<Self, HarnessError> {
        if bundle.is_scaffold() {
            return Err(HarnessError::ScaffoldBundleNotMeasured);
        }
        if !bundle.is_sha256_digest() {
            return Err(HarnessError::WeakBundleDigestNotMeasured);
        }
        if let Some(pin) = kind.oracle_pin() {
            if pin.is_scaffold() {
                return Err(HarnessError::ScaffoldPinNotMeasured);
            }
        }
        if let Some(pin) = kind.owned_pin() {
            if pin.is_scaffold() {
                return Err(HarnessError::ScaffoldPinNotMeasured);
            }
        }
        if let Some(ref cve_id) = cve {
            if !cve_id.has_fixture_receipt() {
                return Err(HarnessError::MissingFixtureReceipt);
            }
        }
        Ok(Self {
            kind,
            operation,
            bundle,
            execution: ExecutionState::Measured(outcome),
            cve_execution: cve,
        })
    }
}

/// Compare two observations into a differential verdict.
///
/// Pairwise `Match` is **not** a conformance / Accept claim — callers must run
/// [`aggregate_comparison_records`] over the full oracle × CVE set first.
///
/// When exit/status/stderr_fingerprint match:
/// - both security all-held → [`DiffVerdict::Match`]
/// - owned all-held and oracle not (hardened vs vulnerable CVE reproduction) → Match
/// - owned unsafe (including both-equally-unsafe) → [`DiffVerdict::Diverge`]
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
        (ExecutionState::Measured(a), ExecutionState::Measured(b)) => {
            let measured_core_equal = a.exit_code == b.exit_code
                && a.status == b.status
                && a.stderr_fingerprint == b.stderr_fingerprint;
            if !measured_core_equal {
                return DiffVerdict::Diverge;
            }
            // Preserve owned hardening over vulnerable oracle CVE reproduction.
            if a.security.all_held_bool() {
                DiffVerdict::Match
            } else {
                DiffVerdict::Diverge
            }
        }
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
        let owned_cve = owned.cve_execution.as_ref();
        let oracle_cve = oracle.cve_execution.as_ref();
        match (owned_cve, oracle_cve) {
            (None, _) | (_, None) => {
                return Err(HarnessError::MissingCveExecution);
            }
            (Some(a), Some(b)) => {
                // Both sides must share identical CveExecutionId (incl. receipt)
                // and match the matrix cell CVE label.
                if a != b || a.cve_id() != cve_id || !a.has_fixture_receipt() {
                    return Err(HarnessError::CveExecutionMismatch {
                        expected: cve_id.to_owned(),
                        owned: Some(a.cve_id().to_owned()),
                        oracle: Some(b.cve_id().to_owned()),
                    });
                }
            }
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
        let id = self
            .oracle
            .kind()
            .oracle_id()
            .expect("validated in try_from_observations");
        let pin = self
            .oracle
            .kind()
            .oracle_pin()
            .expect("validated in try_from_observations")
            .clone();
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
    /// Exact coverage with measured cells all Match (still ≠ Accept — no product ship).
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
/// Also requires one declared pin per [`OracleId`] across all cells.
pub fn validate_matrix_coverage(cells: &[MatrixCell]) -> Result<(), HarnessError> {
    let required = required_matrix_pairs();
    let mut seen = BTreeSet::new();
    let mut pins_by_oracle: BTreeMap<OracleId, &OraclePin> = BTreeMap::new();
    for cell in cells {
        if !REQUIRED_CVE_IDS.contains(&cell.cve_id.as_str()) {
            return Err(HarnessError::UnknownMatrixCve(cell.cve_id.clone()));
        }
        if let Some(existing) = pins_by_oracle.get(&cell.oracle) {
            if *existing != &cell.pin {
                return Err(HarnessError::InconsistentOraclePin(format!(
                    "oracle {} has inconsistent pins across matrix cells",
                    cell.oracle.as_str()
                )));
            }
        } else {
            pins_by_oracle.insert(cell.oracle, &cell.pin);
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

fn validate_owned_pins_consistent(records: &[ComparisonRecord]) -> Result<(), HarnessError> {
    let mut owned_pin: Option<&OwnedPin> = None;
    for record in records {
        let Some(pin) = record.owned().kind().owned_pin() else {
            return Err(HarnessError::InconsistentOwnedPin(
                "comparison owned side missing OwnedPin".into(),
            ));
        };
        match owned_pin {
            None => owned_pin = Some(pin),
            Some(existing) if existing != pin => {
                return Err(HarnessError::InconsistentOwnedPin(
                    "owned executor pin must be identical across matrix records".into(),
                ));
            }
            Some(_) => {}
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
///
/// - Any stubbed cell → [`MatrixAggregate::ScaffoldCoverageComplete`]
/// - Full measured coverage with any [`DiffVerdict::Diverge`] → [`HarnessError::MatrixDivergence`]
/// - Full measured coverage all Match → [`MatrixAggregate::MeasuredCoverageComplete`]
pub fn aggregate_comparison_records(
    records: &[ComparisonRecord],
) -> Result<MatrixAggregate, HarnessError> {
    let cells: Vec<MatrixCell> = records.iter().map(ComparisonRecord::to_matrix_cell).collect();
    validate_matrix_coverage(&cells)?;
    validate_owned_pins_consistent(records)?;
    if records.iter().any(|r| r.verdict == DiffVerdict::Stubbed) {
        return Ok(MatrixAggregate::ScaffoldCoverageComplete);
    }
    if records.iter().any(|r| r.verdict == DiffVerdict::Diverge) {
        let diverged: Vec<String> = records
            .iter()
            .filter(|r| r.verdict == DiffVerdict::Diverge)
            .map(|r| {
                let oid = r
                    .oracle()
                    .kind()
                    .oracle_id()
                    .map(|o| o.as_str())
                    .unwrap_or("?");
                format!("{}×{}", oid, r.cve_id())
            })
            .collect();
        return Err(HarnessError::MatrixDivergence(format!(
            "matrix contains Diverge verdicts: {}",
            diverged.join(", ")
        )));
    }
    Ok(MatrixAggregate::MeasuredCoverageComplete)
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedExecutorStub {
    pin: OwnedPin,
}

impl Default for OwnedExecutorStub {
    fn default() -> Self {
        Self::scaffold()
    }
}

impl OwnedExecutorStub {
    /// Public harness owned stub (scaffold pin only — adapters cannot forge measured pins).
    pub fn scaffold() -> Self {
        Self {
            pin: OwnedPin::scaffold(),
        }
    }

    /// Crate-local measured-path constructor; not public so external crates cannot mint Owned.
    pub(crate) fn try_pinned(pin: OwnedPin) -> Result<Self, HarnessError> {
        Ok(Self { pin })
    }

    pub fn pin(&self) -> &OwnedPin {
        &self.pin
    }
}

/// Public owned stub for harness use after inventory validation (scaffold product path only).
pub fn owned_stub_from_validated_inventory() -> Result<OwnedExecutorStub, HarnessError> {
    validate_obligations()?;
    Ok(OwnedExecutorStub::scaffold())
}

impl OciExecutor for OwnedExecutorStub {
    fn kind(&self) -> ExecutorKind {
        ExecutorKind(ExecutorKindInner::Owned {
            pin: self.pin.clone(),
        })
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
        ExecutorKind(ExecutorKindInner::Oracle {
            id: self.id,
            pin: self.pin.clone(),
        })
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
        // Accepts scaffold inventory pin or immutable sha256:/git: forms.
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
    (OwnedExecutorStub::scaffold(), oracle)
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

    const HEX_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HEX_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HEX_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const HEX_OWNED: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const HEX_ORACLE: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
    const HEX_ORACLE_ALT: &str = "1111111111111111111111111111111111111111111111111111111111111111";

    fn live_oracle_pin() -> OraclePin {
        OraclePin::try_new(&format!("sha256:{HEX_ORACLE}"), "linux/amd64").unwrap()
    }

    fn live_owned_pin() -> OwnedPin {
        OwnedPin::try_new(&format!("sha256:{HEX_OWNED}"), "linux/amd64").unwrap()
    }

    fn live_owned() -> OwnedExecutorStub {
        OwnedExecutorStub::try_pinned(live_owned_pin()).unwrap()
    }

    /// Stub/hermetic oya1 digest (rejected on measured path).
    fn oya1_bundle(id: &str) -> BundleIdentity {
        BundleIdentity::from_inputs(
            id,
            &BundleInputs {
                config: b"config-v1",
                rootfs_fingerprint: b"rootfs-a",
                mounts_fingerprint: b"mounts-a",
            },
        )
    }

    fn alt_oya1_bundle(id: &str) -> BundleIdentity {
        BundleIdentity::from_inputs(
            id,
            &BundleInputs {
                config: b"config-v1",
                rootfs_fingerprint: b"rootfs-b",
                mounts_fingerprint: b"mounts-a",
            },
        )
    }

    fn measured_bundle(id: &str, hex64: &str) -> BundleIdentity {
        BundleIdentity::try_sha256_hex(id, hex64).unwrap()
    }

    fn fixture_for(cve_id: &str) -> CveExecutionId {
        CveExecutionId::from_fixture_material(&canonical_cve_fixture_material(cve_id).unwrap())
            .unwrap()
    }

    fn unsafe_outcome(exit: i32, fp: &str) -> MeasuredOutcome {
        let mut out = safe_outcome(exit, fp);
        out.security.fd_leak_absent = false;
        out
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
        let owned = OwnedExecutorStub::scaffold();
        assert!(owned.kind().is_owned_product());
        assert!(!owned.kind().is_oracle_only());
        assert!(owned.pin().is_scaffold());
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
        let owned = OwnedExecutorStub::scaffold().create_stub("b1");
        let oracle = OracleStub::runc().create_stub("b1");
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Stubbed);
        assert!(owned.cve_execution().is_none());
        assert!(oracle.cve_execution().is_none());
    }

    #[test]
    fn compare_measured_match_and_diverge() {
        let bundle = measured_bundle("b1", HEX_A);
        let owned_ok = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            bundle.clone(),
            None,
            safe_outcome(0, "fp-a"),
        )
        .unwrap();
        let oracle_ok = OperationObservation::try_measured(
            OracleStub::try_new_pinned("runc", live_oracle_pin())
                .unwrap()
                .kind(),
            OciOperation::Start,
            bundle.clone(),
            None,
            safe_outcome(0, "fp-a"),
        )
        .unwrap();
        let oracle_bad = OperationObservation::try_measured(
            OracleStub::try_new_pinned("runc", live_oracle_pin())
                .unwrap()
                .kind(),
            OciOperation::Start,
            bundle,
            None,
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
            live_owned().kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            None,
            safe_outcome(0, "fp"),
        )
        .unwrap();
        let oracle = OracleStub::runc().start_stub("b1");
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn kill_signal_mismatch_diverges() {
        let owned = OperationObservation::stubbed_scaffold(
            OwnedExecutorStub::scaffold().kind(),
            OciOperation::Kill(KillSignal::Term),
            "b1",
        );
        let oracle = OperationObservation::stubbed_scaffold(
            OracleStub::youki().kind(),
            OciOperation::Kill(KillSignal::Kill),
            "b1",
        );
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn kill_operation_always_carries_signal() {
        let kill = OwnedExecutorStub::scaffold().kill_stub("b1", KillSignal::Hup);
        assert!(matches!(
            kill.operation(),
            OciOperation::Kill(KillSignal::Hup)
        ));
        assert_eq!(kill.kill_signal(), Some(KillSignal::Hup));
    }

    #[test]
    fn bundle_content_digest_mismatch_diverges() {
        let owned =
            OwnedExecutorStub::scaffold().create_with_bundle(oya1_bundle("b1"));
        let oracle = OracleStub::runc().create_with_bundle(alt_oya1_bundle("b1"));
        assert_ne!(owned.content_digest(), oracle.content_digest());
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn from_inputs_digest_is_oya1_fnv() {
        let a = oya1_bundle("b1");
        assert!(a.content_digest().starts_with("oya1:"));
        assert_eq!(a.content_digest().len(), "oya1:".len() + 16);
        let same = oya1_bundle("b1");
        assert_eq!(a.content_digest(), same.content_digest());
        let different = alt_oya1_bundle("b1");
        assert_ne!(a.content_digest(), different.content_digest());
    }

    #[test]
    fn measured_rejects_scaffold_bundle() {
        let err = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            BundleIdentity::scaffold("b1"),
            None,
            safe_outcome(0, "fp"),
        )
        .unwrap_err();
        assert_eq!(err, HarnessError::ScaffoldBundleNotMeasured);
    }

    #[test]
    fn measured_rejects_oya1_bundle() {
        let err = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            oya1_bundle("b1"),
            None,
            safe_outcome(0, "fp"),
        )
        .unwrap_err();
        assert_eq!(err, HarnessError::WeakBundleDigestNotMeasured);
    }

    #[test]
    fn measured_rejects_scaffold_oracle_pin() {
        let err = OperationObservation::try_measured(
            OracleStub::runc().kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            None,
            safe_outcome(0, "fp"),
        )
        .unwrap_err();
        assert_eq!(err, HarnessError::ScaffoldPinNotMeasured);
    }

    #[test]
    fn measured_rejects_scaffold_owned_pin() {
        let err = OperationObservation::try_measured(
            OwnedExecutorStub::scaffold().kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            None,
            safe_outcome(0, "fp"),
        )
        .unwrap_err();
        assert_eq!(err, HarnessError::ScaffoldPinNotMeasured);
    }

    #[test]
    fn measured_rejects_bare_cve_execution_id() {
        let bare = CveExecutionId::try_new("CVE-2019-5736").unwrap();
        assert!(!bare.has_fixture_receipt());
        let err = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            Some(bare),
            safe_outcome(0, "fp"),
        )
        .unwrap_err();
        assert_eq!(err, HarnessError::MissingFixtureReceipt);
    }

    #[test]
    fn owned_safe_vs_oracle_unsafe_matches() {
        let bundle = measured_bundle("b1", HEX_A);
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            bundle.clone(),
            None,
            safe_outcome(0, "fp"),
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("crun", live_oracle_pin())
                .unwrap()
                .kind(),
            OciOperation::Start,
            bundle,
            None,
            unsafe_outcome(0, "fp"),
        )
        .unwrap();
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Match);
    }

    #[test]
    fn both_unsafe_equal_outcomes_diverge() {
        let unsafe_out = unsafe_outcome(0, "fp");
        let bundle = measured_bundle("b1", HEX_B);
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            bundle.clone(),
            None,
            unsafe_out.clone(),
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("youki", live_oracle_pin())
                .unwrap()
                .kind(),
            OciOperation::Start,
            bundle,
            None,
            unsafe_out,
        )
        .unwrap();
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn owned_unsafe_vs_oracle_safe_diverges() {
        let bundle = measured_bundle("b1", HEX_A);
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            bundle.clone(),
            None,
            unsafe_outcome(0, "fp"),
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("runc", live_oracle_pin())
                .unwrap()
                .kind(),
            OciOperation::Start,
            bundle,
            None,
            safe_outcome(0, "fp"),
        )
        .unwrap();
        assert_eq!(compare_observations(&owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn owned_hardened_matrix_still_measured_coverage_complete() {
        let owned = live_owned();
        let mut records = Vec::new();
        for (oracle_id, cve_id) in required_matrix_pairs() {
            let oracle = OracleStub::try_new_pinned(oracle_id.as_str(), live_oracle_pin()).unwrap();
            let exec = fixture_for(cve_id);
            let hex = match cve_id {
                "CVE-2019-5736" => HEX_A,
                "CVE-2024-21626" => HEX_B,
                "CVE-MOUNT-SYMLINK-RACE" => HEX_C,
                other => panic!("unexpected {other}"),
            };
            let bundle = measured_bundle("b1", hex);
            // Oracle reproduces vulnerable CVE surface; owned holds all postconditions.
            let owned_obs = OperationObservation::try_measured(
                owned.kind(),
                OciOperation::Start,
                bundle.clone(),
                Some(exec.clone()),
                safe_outcome(0, "fp"),
            )
            .unwrap();
            let oracle_obs = OperationObservation::try_measured(
                oracle.kind(),
                OciOperation::Start,
                bundle,
                Some(exec),
                unsafe_outcome(0, "fp"),
            )
            .unwrap();
            records
                .push(ComparisonRecord::try_from_observations(cve_id, owned_obs, oracle_obs).unwrap());
        }
        assert!(records.iter().all(|r| r.verdict() == DiffVerdict::Match));
        assert_eq!(
            aggregate_comparison_records(&records).unwrap(),
            MatrixAggregate::MeasuredCoverageComplete
        );
    }

    #[test]
    fn matrix_requires_full_oracle_cve_coverage() {
        let cve_id = "CVE-2019-5736";
        let exec = fixture_for(cve_id);
        let incomplete = [ComparisonRecord::try_from_observations(
            cve_id,
            OperationObservation::try_measured(
                live_owned().kind(),
                OciOperation::Start,
                measured_bundle("b1", HEX_A),
                Some(exec.clone()),
                safe_outcome(0, "fp"),
            )
            .unwrap(),
            OperationObservation::try_measured(
                OracleStub::try_new_pinned("runc", live_oracle_pin())
                    .unwrap()
                    .kind(),
                OciOperation::Start,
                measured_bundle("b1", HEX_A),
                Some(exec),
                safe_outcome(0, "fp"),
            )
            .unwrap(),
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
        let owned = live_owned();
        let mut records = Vec::new();
        for (oracle_id, cve_id) in required_matrix_pairs() {
            let oracle = OracleStub::try_new_pinned(oracle_id.as_str(), live_oracle_pin()).unwrap();
            let exec = fixture_for(cve_id);
            // Distinct sha256 digests across CVE rows (not one reused Start blob).
            let hex = match cve_id {
                "CVE-2019-5736" => HEX_A,
                "CVE-2024-21626" => HEX_B,
                "CVE-MOUNT-SYMLINK-RACE" => HEX_C,
                other => panic!("unexpected {other}"),
            };
            let bundle = measured_bundle("b1", hex);
            let owned_obs = OperationObservation::try_measured(
                owned.kind(),
                OciOperation::Start,
                bundle.clone(),
                Some(exec.clone()),
                safe_outcome(0, "fp"),
            )
            .unwrap();
            let oracle_obs = OperationObservation::try_measured(
                oracle.kind(),
                OciOperation::Start,
                bundle,
                Some(exec),
                safe_outcome(0, "fp"),
            )
            .unwrap();
            records
                .push(ComparisonRecord::try_from_observations(cve_id, owned_obs, oracle_obs).unwrap());
        }
        // Distinct fixture receipts per CVE.
        let receipts: BTreeSet<_> = records
            .iter()
            .map(|r| r.owned().cve_execution().unwrap().fixture_receipt().to_owned())
            .collect();
        assert_eq!(receipts.len(), REQUIRED_CVE_IDS.len());
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
    fn aggregate_rejects_matrix_divergence() {
        let owned = live_owned();
        let mut records = Vec::new();
        for (oracle_id, cve_id) in required_matrix_pairs() {
            let oracle = OracleStub::try_new_pinned(oracle_id.as_str(), live_oracle_pin()).unwrap();
            let exec = fixture_for(cve_id);
            let bundle = measured_bundle("b1", HEX_A);
            let diverge = oracle_id == OracleId::Runc && cve_id == "CVE-2019-5736";
            let oracle_outcome = if diverge {
                safe_outcome(1, "fp-diverge")
            } else {
                safe_outcome(0, "fp")
            };
            let owned_obs = OperationObservation::try_measured(
                owned.kind(),
                OciOperation::Start,
                bundle.clone(),
                Some(exec.clone()),
                safe_outcome(0, "fp"),
            )
            .unwrap();
            let oracle_obs = OperationObservation::try_measured(
                oracle.kind(),
                OciOperation::Start,
                bundle,
                Some(exec),
                oracle_outcome,
            )
            .unwrap();
            records
                .push(ComparisonRecord::try_from_observations(cve_id, owned_obs, oracle_obs).unwrap());
        }
        assert!(matches!(
            aggregate_comparison_records(&records),
            Err(HarnessError::MatrixDivergence(_))
        ));
        assert!(matches!(
            refuse_pairwise_match_as_conformance(&records),
            Err(HarnessError::MatrixDivergence(_))
        ));
    }

    #[test]
    fn comparison_record_rejects_missing_cve_execution() {
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            None,
            safe_outcome(0, "fp"),
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("runc", live_oracle_pin())
                .unwrap()
                .kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            None,
            safe_outcome(0, "fp"),
        )
        .unwrap();
        assert_eq!(
            ComparisonRecord::try_from_observations("CVE-2019-5736", owned, oracle).unwrap_err(),
            HarnessError::MissingCveExecution
        );
    }

    #[test]
    fn comparison_record_rejects_mismatched_cve_execution() {
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            Some(fixture_for("CVE-2019-5736")),
            safe_outcome(0, "fp"),
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("runc", live_oracle_pin())
                .unwrap()
                .kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            Some(fixture_for("CVE-2024-21626")),
            safe_outcome(0, "fp"),
        )
        .unwrap();
        assert!(matches!(
            ComparisonRecord::try_from_observations("CVE-2019-5736", owned, oracle).unwrap_err(),
            HarnessError::CveExecutionMismatch { .. }
        ));
    }

    #[test]
    fn comparison_record_rejects_cve_id_param_mismatch() {
        let exec = fixture_for("CVE-2024-21626");
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            Some(exec.clone()),
            safe_outcome(0, "fp"),
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("runc", live_oracle_pin())
                .unwrap()
                .kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            Some(exec),
            safe_outcome(0, "fp"),
        )
        .unwrap();
        // Same CveExecutionId on both sides, but wrong vs matrix cell label.
        assert!(matches!(
            ComparisonRecord::try_from_observations("CVE-2019-5736", owned, oracle).unwrap_err(),
            HarnessError::CveExecutionMismatch { .. }
        ));
    }

    #[test]
    fn from_fixture_material_rejects_empty_and_non_canonical_bytes() {
        assert!(matches!(
            CveExecutionId::from_fixture_material(&CveFixtureMaterial {
                cve_id: "CVE-2019-5736",
                fixture_bytes: b"",
            }),
            Err(HarnessError::NonCanonicalCveFixture(_))
        ));
        assert!(matches!(
            CveExecutionId::from_fixture_material(&CveFixtureMaterial {
                cve_id: "CVE-2019-5736",
                fixture_bytes: b"wrong-bytes-not-canonical",
            }),
            Err(HarnessError::NonCanonicalCveFixture(_))
        ));
        let ok = CveExecutionId::from_fixture_material(
            &canonical_cve_fixture_material("CVE-2019-5736").unwrap(),
        )
        .unwrap();
        assert!(ok.has_fixture_receipt());
        // Canonical bytes are distinct across REQUIRED_CVE_IDS.
        let mut receipts = BTreeSet::new();
        for id in REQUIRED_CVE_IDS {
            let exec = fixture_for(id);
            assert!(receipts.insert(exec.fixture_receipt().to_owned()));
        }
        assert_eq!(receipts.len(), REQUIRED_CVE_IDS.len());
    }

    #[test]
    fn pin_platform_rejects_typos_outside_allowlist() {
        assert!(matches!(
            OraclePin::try_new(&format!("sha256:{HEX_A}"), "linux/am64"),
            Err(HarnessError::OraclePin(_))
        ));
        assert!(matches!(
            OwnedPin::try_new(&format!("sha256:{HEX_A}"), "linux/am64"),
            Err(HarnessError::OwnedPin(_))
        ));
        assert!(matches!(
            OraclePin::try_new(&format!("sha256:{HEX_A}"), ""),
            Err(HarnessError::OraclePin(_))
        ));
        assert!(OraclePin::try_new(&format!("sha256:{HEX_A}"), "linux/amd64").is_ok());
        assert!(OraclePin::try_new(&format!("sha256:{HEX_A}"), "linux/arm64").is_ok());
        assert_eq!(OraclePin::scaffold().platform(), "linux/amd64");
    }

    #[test]
    fn owned_stub_from_validated_inventory_is_scaffold_product_path() {
        let owned = owned_stub_from_validated_inventory().unwrap();
        assert!(owned.kind().is_owned_product());
        assert!(owned.pin().is_scaffold());
        refuse_oracle_as_product(&owned.kind()).unwrap();
    }

    #[test]
    fn inconsistent_oracle_pin_across_matrix_rejected() {
        let mut cells = Vec::new();
        for (oracle, cve_id) in required_matrix_pairs() {
            let pin = if oracle == OracleId::Runc && cve_id == "CVE-2024-21626" {
                OraclePin::try_new(&format!("sha256:{HEX_ORACLE_ALT}"), "linux/amd64").unwrap()
            } else {
                live_oracle_pin()
            };
            cells.push(MatrixCell {
                oracle,
                pin,
                cve_id: cve_id.to_owned(),
                verdict: DiffVerdict::Stubbed,
            });
        }
        assert!(matches!(
            validate_matrix_coverage(&cells).unwrap_err(),
            HarnessError::InconsistentOraclePin(_)
        ));
    }

    #[test]
    fn inconsistent_owned_pin_across_matrix_rejected() {
        let owned_a = live_owned();
        let owned_b = OwnedExecutorStub::try_pinned(
            OwnedPin::try_new(&format!("sha256:{HEX_ORACLE_ALT}"), "linux/amd64").unwrap(),
        )
        .unwrap();
        let mut records = Vec::new();
        for (i, (oracle_id, cve_id)) in required_matrix_pairs().into_iter().enumerate() {
            let owned = if i == 0 { &owned_b } else { &owned_a };
            let oracle = OracleStub::try_new_pinned(oracle_id.as_str(), live_oracle_pin()).unwrap();
            let exec = fixture_for(cve_id);
            let bundle = measured_bundle("b1", HEX_A);
            let owned_obs = OperationObservation::try_measured(
                owned.kind(),
                OciOperation::Start,
                bundle.clone(),
                Some(exec.clone()),
                safe_outcome(0, "fp"),
            )
            .unwrap();
            let oracle_obs = OperationObservation::try_measured(
                oracle.kind(),
                OciOperation::Start,
                bundle,
                Some(exec),
                safe_outcome(0, "fp"),
            )
            .unwrap();
            records
                .push(ComparisonRecord::try_from_observations(cve_id, owned_obs, oracle_obs).unwrap());
        }
        assert!(matches!(
            aggregate_comparison_records(&records).unwrap_err(),
            HarnessError::InconsistentOwnedPin(_)
        ));
    }

    #[test]
    fn oracle_pin_rejects_mutable_labels() {
        for label in ["main", "master", "latest", "HEAD", "", "v1.0.0", "sha256:deadbeef"] {
            assert!(
                matches!(
                    OraclePin::try_new(label, "linux/amd64"),
                    Err(HarnessError::OraclePin(_))
                ),
                "expected reject for {label:?}"
            );
        }
        assert!(OraclePin::try_new(SCAFFOLD_PIN_REVISION, "linux/amd64").is_ok());
        assert!(OraclePin::try_new(&format!("sha256:{HEX_A}"), "linux/amd64").is_ok());
        assert!(OraclePin::try_new(
            "git:0123456789abcdef0123456789abcdef01234567",
            "linux/amd64"
        )
        .is_ok());
    }

    #[test]
    fn cve_execution_id_rejects_unknown() {
        assert!(matches!(
            CveExecutionId::try_new("CVE-NOT-IN-CORPUS"),
            Err(HarnessError::UnknownCve(_))
        ));
        let ok = CveExecutionId::try_new("CVE-2019-5736").unwrap();
        assert_eq!(ok.class(), "proc_self_exe_reexec");
        assert!(!ok.has_fixture_receipt());
        let measured = fixture_for("CVE-2019-5736");
        assert!(measured.has_fixture_receipt());
        assert!(measured.fixture_receipt().starts_with("fnv1a64:"));
    }

    #[test]
    fn try_sha256_hex_validates_charset_length() {
        assert!(BundleIdentity::try_sha256_hex("b1", HEX_A).is_ok());
        assert!(matches!(
            BundleIdentity::try_sha256_hex("b1", "deadbeef"),
            Err(HarnessError::WeakBundleDigestNotMeasured)
        ));
        assert!(matches!(
            BundleIdentity::try_sha256_hex("b1", &HEX_A.to_uppercase()),
            Err(HarnessError::WeakBundleDigestNotMeasured)
        ));
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
