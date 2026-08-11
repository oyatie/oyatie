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

use sha2::{Digest, Sha256};
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

/// Canonical adversarial fixture contract per [`REQUIRED_CVE_IDS`]:
/// `(cve_id, fixture_bytes, required_oci_operation)`.
///
/// Required ops are the vulnerability-triggering actions the matrix must exercise
/// (Delete / unrelated ops are rejected on the measured comparison path):
/// - CVE-2019-5736 (`proc_self_exe_reexec`): [`OciOperation::Start`] — malicious
///   `/proc/self/exe` overwrite payload runs after Start (create alone does not exec).
/// - CVE-2024-21626 (`fd_leak`): [`OciOperation::Start`] — leaked host FDs into the
///   container process across start/exec.
/// - CVE-MOUNT-SYMLINK-RACE (`mount_symlink_race`): [`OciOperation::Create`] — mount
///   setup is where symlink races land (Kill would not exercise the race).
pub const CANONICAL_CVE_FIXTURES: [(&str, &[u8], OciOperation); 3] = [
    (
        "CVE-2019-5736",
        b"canonical-adversarial:CVE-2019-5736:proc_self_exe_reexec",
        OciOperation::Start,
    ),
    (
        "CVE-2024-21626",
        b"canonical-adversarial:CVE-2024-21626:fd_leak",
        OciOperation::Start,
    ),
    (
        "CVE-MOUNT-SYMLINK-RACE",
        b"canonical-adversarial:CVE-MOUNT-SYMLINK-RACE:mount_symlink_race",
        OciOperation::Create,
    ),
];

/// Look up canonical fixture bytes for a required CVE id.
pub fn canonical_cve_fixture_bytes(cve_id: &str) -> Option<&'static [u8]> {
    CANONICAL_CVE_FIXTURES
        .iter()
        .find(|(id, _, _)| *id == cve_id)
        .map(|(_, bytes, _)| *bytes)
}

/// Required vulnerability-triggering [`OciOperation`] for a required CVE id.
pub fn required_cve_operation(cve_id: &str) -> Option<OciOperation> {
    CANONICAL_CVE_FIXTURES
        .iter()
        .find(|(id, _, _)| *id == cve_id)
        .map(|(_, _, op)| *op)
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
            .find(|(id, _, _)| *id == cve_id)
            .map(|(id, _, _)| *id)
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
    MissingSecurityEvidence,
    CveReceiptBundleMismatch(String),
    MissingHostEnvironment,
    HostEnvironmentMismatch(String),
    IncompleteMatrixCoverage(String),
    DuplicateMatrixCell(String),
    UnknownMatrixCve(String),
    InconsistentOraclePin(String),
    InconsistentOwnedPin(String),
    /// Record oracle pin does not match the validated obligations inventory pin.
    InventoryOraclePinMismatch(String),
    MatrixDivergence(String),
    MissingCveExecution,
    CveExecutionMismatch {
        expected: String,
        owned: Option<String>,
        oracle: Option<String>,
    },
    /// Observation OCI op must match the CVE's required vulnerability-triggering op.
    CveOperationMismatch {
        cve_id: String,
        expected: String,
        got_owned: String,
        got_oracle: String,
    },
    /// All oracle rows for one CVE must share an identical [`BundleIdentity`].
    InconsistentCveBundle(String),
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
            | Self::InventoryOraclePinMismatch(m)
            | Self::InconsistentCveBundle(m)
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
            Self::MissingSecurityEvidence => {
                write!(
                    f,
                    "measured security postconditions require from_runner_evidence receipt"
                )
            }
            Self::CveReceiptBundleMismatch(m) => {
                write!(f, "CVE fixture receipt does not bind to measured bundle: {m}")
            }
            Self::MissingHostEnvironment => {
                write!(f, "measured observations require MeasuredHostEnvironment")
            }
            Self::HostEnvironmentMismatch(m) => {
                write!(f, "measured host environment mismatch: {m}")
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
            Self::CveOperationMismatch {
                cve_id,
                expected,
                got_owned,
                got_oracle,
            } => write!(
                f,
                "cve {cve_id} requires OCI op {expected} (owned={got_owned}, oracle={got_oracle})"
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

/// Whether the security postcondition identified by a CVE class is held.
fn postcondition_held_for_class(security: &SecurityPostconditions, class: &str) -> bool {
    match class {
        "proc_self_exe_reexec" => security.proc_self_exe_reexec_blocked,
        "fd_leak" => security.fd_leak_absent,
        "mount_symlink_race" => security.mount_symlink_race_safe,
        _ => false,
    }
}

/// FNV-1a 64-bit (local; no crate dep) for hermetic digests / fixture receipts.

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let dig = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in dig {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

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
        // Store canonicalized (trim'd) values — validation already trims for checks.
        Ok(Self {
            revision: revision.trim().to_owned(),
            platform: platform.trim().to_owned(),
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
        // Store canonicalized (trim'd) values — validation already trims for checks.
        Ok(Self {
            revision: revision.trim().to_owned(),
            platform: platform.trim().to_owned(),
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
    /// Collision-resistant receipt over runner evidence bytes; empty = unauthenticated.
    evidence_receipt: String,
}

impl SecurityPostconditions {
    /// Unauthenticated scaffold defaults (stub path only — measured requires evidence).
    pub fn all_held() -> Self {
        Self {
            proc_self_exe_reexec_blocked: true,
            fd_leak_absent: true,
            mount_symlink_race_safe: true,
            evidence_receipt: String::new(),
        }
    }

    /// Measured-path constructor: postconditions must bind to runner evidence bytes.
    pub fn from_runner_evidence(
        proc_self_exe_reexec_blocked: bool,
        fd_leak_absent: bool,
        mount_symlink_race_safe: bool,
        evidence_bytes: &[u8],
    ) -> Result<Self, HarnessError> {
        if evidence_bytes.is_empty() {
            return Err(HarnessError::MissingSecurityEvidence);
        }
        Ok(Self {
            proc_self_exe_reexec_blocked,
            fd_leak_absent,
            mount_symlink_race_safe,
            evidence_receipt: format!("sha256:{}", sha256_hex(evidence_bytes)),
        })
    }

    pub const fn all_held_bool(&self) -> bool {
        self.proc_self_exe_reexec_blocked && self.fd_leak_absent && self.mount_symlink_race_safe
    }

    pub fn evidence_receipt(&self) -> &str {
        &self.evidence_receipt
    }

    pub fn has_authenticated_evidence(&self) -> bool {
        !self.evidence_receipt.is_empty()
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
    /// `sha256:{hex}` over len-prefixed `(cve_id || fixture_bytes || bundle digest)`; empty = bare.
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
    /// len-prefixed `(cve_id || fixture_bytes || bundle.content_digest)`.
    ///
    /// The receipt is bound to the measured bundle identity so a single benign
    /// SHA-256 bundle cannot be reused across CVEs with swapped receipts.
    pub fn from_fixture_material(
        material: &CveFixtureMaterial<'_>,
        bundle: &BundleIdentity,
    ) -> Result<Self, HarnessError> {
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
        if bundle.is_scaffold() || !bundle.is_sha256_digest() {
            return Err(HarnessError::WeakBundleDigestNotMeasured);
        }
        let mut buf = Vec::new();
        push_len_prefixed(&mut buf, material.cve_id.as_bytes());
        push_len_prefixed(&mut buf, material.fixture_bytes);
        push_len_prefixed(&mut buf, bundle.content_digest().as_bytes());
        let fixture_receipt = format!("sha256:{}", sha256_hex(&buf));
        Ok(Self {
            cve_id: material.cve_id.to_owned(),
            class: class.to_owned(),
            fixture_receipt,
        })
    }

    /// Recompute the expected receipt for `(cve, canonical fixture, bundle)` and
    /// compare to this id's embedded receipt.
    pub fn receipt_matches_bundle(&self, bundle: &BundleIdentity) -> Result<(), HarnessError> {
        let material = canonical_cve_fixture_material(&self.cve_id)?;
        let expected = Self::from_fixture_material(&material, bundle)?;
        if expected.fixture_receipt != self.fixture_receipt {
            return Err(HarnessError::CveReceiptBundleMismatch(format!(
                "cve={} bundle={} receipt={}",
                self.cve_id,
                bundle.content_digest(),
                self.fixture_receipt
            )));
        }
        Ok(())
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
/// `Owned` is sealed: only [`OwnedExecutorStub::kind`] constructs it. Public API
/// yields scaffold pins only; measured Owned minting is `pub(crate)` (harness tests).
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


/// Immutable measured host-environment identity (CVE outcomes are host-sensitive).
///
/// Digest covers platform + kernel build/config + cgroup mode + runner image +
/// filesystem + mount-namespace labels. Scaffold tests use fixed labels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasuredHostEnvironment {
    digest: String,
    platform: String,
}

impl MeasuredHostEnvironment {
    /// Build an immutable environment digest from required host facets.
    pub fn try_new(
        platform: &str,
        kernel_build: &str,
        cgroup_mode: &str,
        runner_image: &str,
        filesystem: &str,
        mount_namespace: &str,
    ) -> Result<Self, HarnessError> {
        validate_pin_platform(platform, "host")?;
        for (name, value) in [
            ("kernel_build", kernel_build),
            ("cgroup_mode", cgroup_mode),
            ("runner_image", runner_image),
            ("filesystem", filesystem),
            ("mount_namespace", mount_namespace),
        ] {
            let t = value.trim();
            if t.is_empty() || t != value {
                return Err(HarnessError::HostEnvironmentMismatch(format!(
                    "{name} must be non-empty exact label (no outer whitespace)"
                )));
            }
        }
        let mut buf = Vec::new();
        for part in [
            platform,
            kernel_build,
            cgroup_mode,
            runner_image,
            filesystem,
            mount_namespace,
        ] {
            push_len_prefixed(&mut buf, part.as_bytes());
        }
        Ok(Self {
            digest: format!("env1:sha256:{}", sha256_hex(&buf)),
            platform: platform.to_owned(),
        })
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn platform(&self) -> &str {
        &self.platform
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
    host: Option<MeasuredHostEnvironment>,
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

    pub fn host_environment(&self) -> Option<&MeasuredHostEnvironment> {
        self.host.as_ref()
    }

    pub fn stubbed(kind: ExecutorKind, operation: OciOperation, bundle: BundleIdentity) -> Self {
        Self {
            kind,
            operation,
            bundle,
            execution: ExecutionState::Stubbed,
            cve_execution: None,
            host: None,
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
        host: MeasuredHostEnvironment,
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
            if pin.platform() != host.platform() {
                return Err(HarnessError::HostEnvironmentMismatch(format!(
                    "oracle pin platform {} != host {}",
                    pin.platform(),
                    host.platform()
                )));
            }
        }
        if let Some(pin) = kind.owned_pin() {
            if pin.is_scaffold() {
                return Err(HarnessError::ScaffoldPinNotMeasured);
            }
            if pin.platform() != host.platform() {
                return Err(HarnessError::HostEnvironmentMismatch(format!(
                    "owned pin platform {} != host {}",
                    pin.platform(),
                    host.platform()
                )));
            }
        }
        if !outcome.security.has_authenticated_evidence() {
            return Err(HarnessError::MissingSecurityEvidence);
        }
        if let Some(ref cve_id) = cve {
            if !cve_id.has_fixture_receipt() {
                return Err(HarnessError::MissingFixtureReceipt);
            }
            cve_id.receipt_matches_bundle(&bundle)?;
            let Some(required_op) = required_cve_operation(cve_id.cve_id()) else {
                return Err(HarnessError::UnknownCve(cve_id.cve_id().to_owned()));
            };
            if operation != required_op {
                return Err(HarnessError::CveOperationMismatch {
                    cve_id: cve_id.cve_id().to_owned(),
                    expected: required_op.as_str().to_owned(),
                    got_owned: operation.as_str().to_owned(),
                    got_oracle: operation.as_str().to_owned(),
                });
            }
        }
        Ok(Self {
            kind,
            operation,
            bundle,
            execution: ExecutionState::Measured(outcome),
            cve_execution: cve,
            host: Some(host),
        })
    }
}

/// Compare two observations into a differential verdict for a required CVE cell.
///
/// Pairwise `Match` is **not** a conformance / Accept claim — callers must run
/// [`aggregate_comparison_records`] over the full oracle × CVE set first.
///
/// Measured path (active CVE class only — unrelated security flags do not count):
/// - owned holds the CVE postcondition + oracle does **not** → [`DiffVerdict::Match`]
///   even when exit_code / status / stderr_fingerprint differ (owned blocked the
///   exploit with a different core outcome)
/// - owned holds CVE postcondition + oracle holds + equal core → Match
/// - owned holds CVE postcondition + oracle holds + unequal core → Diverge
/// - owned does **not** hold the CVE postcondition → [`DiffVerdict::Diverge`]
pub fn compare_observations(
    cve_id: &str,
    owned: &OperationObservation,
    oracle: &OperationObservation,
) -> DiffVerdict {
    let Some(class) = expected_cve_class(cve_id) else {
        return DiffVerdict::Diverge;
    };
    if !owned.kind.is_owned_product() || !oracle.kind.is_oracle_only() {
        return DiffVerdict::Diverge;
    }
    // Owned and oracle pins must share the same platform (architecture environment).
    let owned_platform = owned.kind.owned_pin().map(OwnedPin::platform);
    let oracle_platform = oracle.kind.oracle_pin().map(OraclePin::platform);
    match (owned_platform, oracle_platform) {
        (Some(a), Some(b)) if a == b => {}
        _ => return DiffVerdict::Diverge,
    }
    // Measured host environment must be identical (kernel/cgroup/runner/fs/ns).
    match (owned.host.as_ref(), oracle.host.as_ref()) {
        (Some(a), Some(b)) if a.digest() == b.digest() => {}
        (None, None) => {} // stubbed path
        _ => return DiffVerdict::Diverge,
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
            let owned_holds = postcondition_held_for_class(&a.security, class);
            let oracle_holds = postcondition_held_for_class(&b.security, class);
            // Owned hardened vs vulnerable oracle on the *active* CVE postcondition:
            // Match even when core outcomes differ. Unrelated flag failures alone do not qualify.
            if owned_holds && !oracle_holds {
                return DiffVerdict::Match;
            }
            if !measured_core_equal {
                return DiffVerdict::Diverge;
            }
            if owned_holds {
                DiffVerdict::Match
            } else {
                // Owned unsafe on this CVE (including both-equally-unsafe) → Diverge.
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
        let Some(required_op) = required_cve_operation(cve_id) else {
            return Err(HarnessError::UnknownMatrixCve(cve_id.to_owned()));
        };
        if owned.operation != required_op || oracle.operation != required_op {
            return Err(HarnessError::CveOperationMismatch {
                cve_id: cve_id.to_owned(),
                expected: required_op.as_str().to_owned(),
                got_owned: owned.operation.as_str().to_owned(),
                got_oracle: oracle.operation.as_str().to_owned(),
            });
        }
        let verdict = compare_observations(cve_id, &owned, &oracle);
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


fn validate_host_environments_consistent(records: &[ComparisonRecord]) -> Result<(), HarnessError> {
    let mut host: Option<&MeasuredHostEnvironment> = None;
    for record in records {
        let Some(h) = record.owned().host_environment() else {
            if record.owned().executed() || record.oracle().executed() {
                return Err(HarnessError::MissingHostEnvironment);
            }
            continue;
        };
        let Some(oh) = record.oracle().host_environment() else {
            return Err(HarnessError::MissingHostEnvironment);
        };
        if h.digest() != oh.digest() {
            return Err(HarnessError::HostEnvironmentMismatch(
                "owned/oracle host digests differ within a comparison record".into(),
            ));
        }
        match host {
            None => host = Some(h),
            Some(existing) if existing.digest() == h.digest() => {}
            Some(_) => {
                return Err(HarnessError::HostEnvironmentMismatch(
                    "measured host environment must be identical across matrix records".into(),
                ));
            }
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

/// For each CVE id, every matrix record must share one identical [`BundleIdentity`].
fn validate_cve_bundles_consistent(records: &[ComparisonRecord]) -> Result<(), HarnessError> {
    let mut by_cve: BTreeMap<&str, &BundleIdentity> = BTreeMap::new();
    for record in records {
        if record.owned().bundle() != record.oracle().bundle() {
            return Err(HarnessError::InconsistentCveBundle(format!(
                "CVE {} owned/oracle BundleIdentity mismatch within comparison record",
                record.cve_id()
            )));
        }
        let bundle = record.owned().bundle();
        match by_cve.get(record.cve_id()) {
            None => {
                by_cve.insert(record.cve_id(), bundle);
            }
            Some(existing) if *existing != bundle => {
                return Err(HarnessError::InconsistentCveBundle(format!(
                    "CVE {} must use one BundleIdentity across all oracle matrix rows",
                    record.cve_id()
                )));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

fn validate_oracle_pins_against_expected(
    records: &[ComparisonRecord],
    expected: &BTreeMap<OracleId, OraclePin>,
) -> Result<(), HarnessError> {
    for record in records {
        let Some(id) = record.oracle().kind().oracle_id() else {
            return Err(HarnessError::NotOracleObservation);
        };
        let Some(pin) = record.oracle().kind().oracle_pin() else {
            return Err(HarnessError::InventoryOraclePinMismatch(format!(
                "oracle {} comparison record missing OraclePin",
                id.as_str()
            )));
        };
        match expected.get(&id) {
            Some(exp) if exp == pin => {}
            Some(exp) => {
                return Err(HarnessError::InventoryOraclePinMismatch(format!(
                    "oracle {} pin revision={} platform={} does not match inventory revision={} platform={}",
                    id.as_str(),
                    pin.revision(),
                    pin.platform(),
                    exp.revision(),
                    exp.platform()
                )));
            }
            None => {
                return Err(HarnessError::InventoryOraclePinMismatch(format!(
                    "oracle {} has no expected inventory pin",
                    id.as_str()
                )));
            }
        }
    }
    Ok(())
}

fn aggregate_comparison_records_inner(
    records: &[ComparisonRecord],
    expected_pins: Option<&BTreeMap<OracleId, OraclePin>>,
) -> Result<MatrixAggregate, HarnessError> {
    let cells: Vec<MatrixCell> = records.iter().map(ComparisonRecord::to_matrix_cell).collect();
    validate_matrix_coverage(&cells)?;
    validate_owned_pins_consistent(records)?;
    validate_host_environments_consistent(records)?;
    validate_cve_bundles_consistent(records)?;
    if records.iter().any(|r| r.verdict == DiffVerdict::Stubbed) {
        return Ok(MatrixAggregate::ScaffoldCoverageComplete);
    }
    // Measured path: require oracle pins match the expected / inventory map.
    let owned_expected;
    let expected = match expected_pins {
        Some(map) => map,
        None => {
            owned_expected = obligation_oracle_pins()?;
            &owned_expected
        }
    };
    validate_oracle_pins_against_expected(records, expected)?;
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
///
/// Measured path requires each oracle pin to match the validated obligations inventory
/// ([`obligation_oracle_pins`]) when aggregating via
/// [`aggregate_comparison_records_against_inventory`].
pub fn aggregate_comparison_records(
    records: &[ComparisonRecord],
) -> Result<MatrixAggregate, HarnessError> {
    aggregate_comparison_records_inner(records, None)
}

/// Crate/test-only pin map override — still must equal the validated inventory.
///
/// Public callers must use [`aggregate_comparison_records_against_inventory`].
pub(crate) fn aggregate_comparison_records_with_expected_pins(
    records: &[ComparisonRecord],
    expected_pins: &BTreeMap<OracleId, OraclePin>,
) -> Result<MatrixAggregate, HarnessError> {
    let inventory = obligation_oracle_pins()?;
    if expected_pins != &inventory {
        return Err(HarnessError::InventoryOraclePinMismatch(
            "expected_pins must equal validated obligations inventory (no public bypass)".into(),
        ));
    }
    aggregate_comparison_records_inner(records, Some(expected_pins))
}

/// Aggregate measured comparison records against pins loaded from the validated
/// embedded obligations fixture ([`CVE_OBLIGATIONS_JSON`]).
pub fn aggregate_comparison_records_against_inventory(
    records: &[ComparisonRecord],
) -> Result<MatrixAggregate, HarnessError> {
    let pins = obligation_oracle_pins()?;
    aggregate_comparison_records_inner(records, Some(&pins))
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
    /// Public harness owned stub (scaffold pin only).
    ///
    /// Measured Owned identity is **not** publicly issuable: oracle adapters and
    /// other downstream crates can only hold the scaffold product path. Live
    /// measured minting stays `pub(crate)` until an in-tree owned adapter provides
    /// a sealed integration (oracles must never emit Owned [`ExecutorKind`]).
    pub fn scaffold() -> Self {
        Self {
            pin: OwnedPin::scaffold(),
        }
    }

    /// Crate-local measured Owned identity (immutable pin only).
    ///
    /// Not part of the public API — prevents runc/youki/crun adapters from minting
    /// a measured Owned kind that would pass [`refuse_oracle_as_product`].
    pub(crate) fn try_measured_identity(pin: OwnedPin) -> Result<Self, HarnessError> {
        if pin.is_scaffold() || !is_immutable_revision(pin.revision()) {
            return Err(HarnessError::ScaffoldPinNotMeasured);
        }
        Ok(Self { pin })
    }

    /// Crate-local pin constructor (includes scaffold for stub tests).
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

    /// Inventory-bound oracle stub (sealed adapter identity + declared pin).
    pub fn from_inventory(id: OracleId) -> Result<Self, HarnessError> {
        let pins = obligation_oracle_pins()?;
        let Some(pin) = pins.get(&id) else {
            return Err(HarnessError::MissingOracle(id.as_str().to_owned()));
        };
        Self::try_new_pinned(id.as_str(), pin.clone())
    }

    pub fn try_new(id: &str) -> Result<Self, HarnessError> {
        Ok(Self {
            id: OracleId::try_from_str(id)?,
            pin: OraclePin::scaffold(),
        })
    }

    /// Crate-local pin binding — public surface is inventory-bound factories only.
    pub(crate) fn try_new_pinned(id: &str, pin: OraclePin) -> Result<Self, HarnessError> {
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

/// Load oracle id → pin map from validated obligations JSON.
pub fn obligation_oracle_pins_from_json(
    json: &str,
) -> Result<BTreeMap<OracleId, OraclePin>, HarnessError> {
    let root: ObligationsRoot =
        serde_json::from_str(json).map_err(|e| HarnessError::Parse(e.to_string()))?;
    validate_root(&root)?;
    let mut map = BTreeMap::new();
    for row in &root.oracles {
        let id = OracleId::try_from_str(&row.id)?;
        let pin = OraclePin::try_new(&row.revision, &row.platform)?;
        map.insert(id, pin);
    }
    Ok(map)
}

/// Load oracle id → pin map from the embedded validated obligations fixture.
pub fn obligation_oracle_pins() -> Result<BTreeMap<OracleId, OraclePin>, HarnessError> {
    obligation_oracle_pins_from_json(CVE_OBLIGATIONS_JSON)
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

    const CVE_FD: &str = "CVE-2024-21626";
    const CVE_PROC: &str = "CVE-2019-5736";

    fn live_owned_pin() -> OwnedPin {
        OwnedPin::try_new(&format!("sha256:{HEX_OWNED}"), "linux/amd64").unwrap()
    }

    fn live_owned() -> OwnedExecutorStub {
        OwnedExecutorStub::try_measured_identity(live_owned_pin()).unwrap()
    }

    fn live_host() -> MeasuredHostEnvironment {
        MeasuredHostEnvironment::try_new(
            "linux/amd64",
            "scaffold-kernel-build",
            "cgroup2",
            "scaffold-runner:0",
            "ext4",
            "mntns-scaffold",
        )
        .unwrap()
    }

    fn inventory_pins() -> BTreeMap<OracleId, OraclePin> {
        obligation_oracle_pins().unwrap()
    }

    fn pinned_oracle(id: OracleId) -> OracleStub {
        let pin = inventory_pins()[&id].clone();
        OracleStub::try_new_pinned(id.as_str(), pin).unwrap()
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

    fn fixture_for(cve_id: &str, bundle: &BundleIdentity) -> CveExecutionId {
        CveExecutionId::from_fixture_material(
            &canonical_cve_fixture_material(cve_id).unwrap(),
            bundle,
        )
        .unwrap()
    }

    fn required_op(cve_id: &str) -> OciOperation {
        required_cve_operation(cve_id).expect("required CVE")
    }

    fn unsafe_outcome(exit: i32, fp: &str) -> MeasuredOutcome {
        unsafe_outcome_for_cve(CVE_FD, exit, fp)
    }

    fn unsafe_outcome_for_cve(cve_id: &str, exit: i32, fp: &str) -> MeasuredOutcome {
        let mut out = safe_outcome(exit, fp);
        match expected_cve_class(cve_id).expect("required CVE") {
            "proc_self_exe_reexec" => out.security.proc_self_exe_reexec_blocked = false,
            "fd_leak" => out.security.fd_leak_absent = false,
            "mount_symlink_race" => out.security.mount_symlink_race_safe = false,
            other => panic!("unexpected class {other}"),
        }
        out
    }

    fn safe_outcome(exit: i32, fp: &str) -> MeasuredOutcome {
        MeasuredOutcome {
            exit_code: exit,
            status: "exited".into(),
            stderr_fingerprint: fp.into(),
            security: SecurityPostconditions::from_runner_evidence(
                true,
                true,
                true,
                format!("safe-evidence:{fp}").as_bytes(),
            )
            .unwrap(),
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
        assert_eq!(compare_observations(CVE_FD, &owned, &oracle), DiffVerdict::Stubbed);
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
            live_host()
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
            live_host()
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
            live_host()
        )
        .unwrap();
        assert_eq!(
            compare_observations(CVE_FD, &owned_ok, &oracle_ok),
            DiffVerdict::Match
        );
        assert_eq!(
            compare_observations(CVE_FD, &owned_ok, &oracle_bad),
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
            live_host()
        )
        .unwrap();
        let oracle = OracleStub::runc().start_stub("b1");
        assert_eq!(compare_observations(CVE_FD, &owned, &oracle), DiffVerdict::Diverge);
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
        assert_eq!(compare_observations(CVE_FD, &owned, &oracle), DiffVerdict::Diverge);
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
        assert_eq!(compare_observations(CVE_FD, &owned, &oracle), DiffVerdict::Diverge);
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
            live_host()
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
            live_host()
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
            live_host()
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
            live_host()
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
            live_host()
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
            live_host()
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
            live_host()
        )
        .unwrap();
        assert_eq!(compare_observations(CVE_FD, &owned, &oracle), DiffVerdict::Match);
    }

    #[test]
    fn owned_safe_nonzero_vs_oracle_unsafe_zero_matches() {
        // Owned blocks exploit (nonzero exit / different stderr) while oracle
        // reproduces CVE successfully — still Match when owned security holds.
        let bundle = measured_bundle("b1", HEX_A);
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            bundle.clone(),
            None,
            safe_outcome(1, "blocked-exploit"),
            live_host()
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("runc", live_oracle_pin())
                .unwrap()
                .kind(),
            OciOperation::Start,
            bundle,
            None,
            unsafe_outcome(0, "exploit-ran"),
            live_host()
        )
        .unwrap();
        assert_eq!(compare_observations(CVE_FD, &owned, &oracle), DiffVerdict::Match);
    }

    #[test]
    fn unrelated_security_flag_failure_does_not_match_on_core_diff() {
        // CVE-2019-5736 cares about proc_self_exe_reexec_blocked. Oracle fails only
        // fd_leak_absent (unrelated) while holding the active postcondition — core
        // divergence must not become Match via the hardening exception.
        let bundle = measured_bundle("b1", HEX_A);
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            bundle.clone(),
            None,
            safe_outcome(1, "blocked-exploit"),
            live_host()
        )
        .unwrap();
        let mut oracle_out = safe_outcome(0, "exploit-ran");
        oracle_out.security.fd_leak_absent = false; // unrelated flag only
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("crun", live_oracle_pin())
                .unwrap()
                .kind(),
            OciOperation::Start,
            bundle,
            None,
            oracle_out,
            live_host()
        )
        .unwrap();
        assert_eq!(
            compare_observations(CVE_PROC, &owned, &oracle),
            DiffVerdict::Diverge
        );
    }

    #[test]
    fn owned_unsafe_core_diff_still_diverges() {
        let bundle = measured_bundle("b1", HEX_A);
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Start,
            bundle.clone(),
            None,
            unsafe_outcome(1, "owned-unsafe"),
            live_host()
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("youki", live_oracle_pin())
                .unwrap()
                .kind(),
            OciOperation::Start,
            bundle,
            None,
            unsafe_outcome(0, "oracle-unsafe"),
            live_host()
        )
        .unwrap();
        assert_eq!(compare_observations(CVE_FD, &owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn crate_local_measured_identity_supports_measured_observations() {
        let owned = OwnedExecutorStub::try_measured_identity(live_owned_pin()).unwrap();
        assert!(!owned.pin().is_scaffold());
        assert!(owned.kind().is_owned_product());
        let obs = OperationObservation::try_measured(
            owned.kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            None,
            safe_outcome(0, "fp"),
            live_host()
        )
        .unwrap();
        assert!(obs.executed());
        assert_eq!(
            OwnedExecutorStub::try_measured_identity(OwnedPin::scaffold()).unwrap_err(),
            HarnessError::ScaffoldPinNotMeasured
        );
        // Public surface cannot elevate scaffold → measured Owned.
        assert!(OwnedExecutorStub::scaffold().pin().is_scaffold());
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
            live_host()
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
            live_host()
        )
        .unwrap();
        assert_eq!(compare_observations(CVE_FD, &owned, &oracle), DiffVerdict::Diverge);
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
            live_host()
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
            live_host()
        )
        .unwrap();
        assert_eq!(compare_observations(CVE_FD, &owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn owned_hardened_matrix_still_measured_coverage_complete() {
        let owned = live_owned();
        let mut records = Vec::new();
        for (oracle_id, cve_id) in required_matrix_pairs() {
            let oracle = pinned_oracle(oracle_id);
            let hex = match cve_id {
                "CVE-2019-5736" => HEX_A,
                "CVE-2024-21626" => HEX_B,
                "CVE-MOUNT-SYMLINK-RACE" => HEX_C,
                other => panic!("unexpected {other}"),
            };
            let bundle = measured_bundle("b1", hex);
            let exec = fixture_for(cve_id, &bundle);
            // Oracle reproduces vulnerable CVE surface; owned holds all postconditions.
            let op = required_op(cve_id);
            let owned_obs = OperationObservation::try_measured(
                owned.kind(),
                op,
                bundle.clone(),
                Some(exec.clone()),
                // Nonzero exit: owned blocked exploit; oracle still reproduces CVE.
                safe_outcome(1, "owned-blocked"),
            live_host()
        )
            .unwrap();
            let oracle_obs = OperationObservation::try_measured(
                oracle.kind(),
                op,
                bundle,
                Some(exec),
                unsafe_outcome_for_cve(cve_id, 0, "oracle-reproduced"),
            live_host()
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
        assert_eq!(
            aggregate_comparison_records_against_inventory(&records).unwrap(),
            MatrixAggregate::MeasuredCoverageComplete
        );
    }

    #[test]
    fn matrix_requires_full_oracle_cve_coverage() {
        let cve_id = "CVE-2019-5736";
        let bundle = measured_bundle("b1", HEX_A);
        let exec = fixture_for(cve_id, &bundle);
        let op = required_op(cve_id);
        let incomplete = [ComparisonRecord::try_from_observations(
            cve_id,
            OperationObservation::try_measured(
                live_owned().kind(),
                op,
                bundle.clone(),
                Some(exec.clone()),
                safe_outcome(0, "fp"),
            live_host()
        )
            .unwrap(),
            OperationObservation::try_measured(
                OracleStub::try_new_pinned("runc", live_oracle_pin())
                    .unwrap()
                    .kind(),
                op,
                measured_bundle("b1", HEX_A),
                Some(exec),
                safe_outcome(0, "fp"),
            live_host()
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
            let oracle = pinned_oracle(oracle_id);
            let hex = match cve_id {
                "CVE-2019-5736" => HEX_A,
                "CVE-2024-21626" => HEX_B,
                "CVE-MOUNT-SYMLINK-RACE" => HEX_C,
                other => panic!("unexpected {other}"),
            };
            let bundle = measured_bundle("b1", hex);
            let exec = fixture_for(cve_id, &bundle);
            let op = required_op(cve_id);
            let owned_obs = OperationObservation::try_measured(
                owned.kind(),
                op,
                bundle.clone(),
                Some(exec.clone()),
                safe_outcome(0, "fp"),
            live_host()
        )
            .unwrap();
            let oracle_obs = OperationObservation::try_measured(
                oracle.kind(),
                op,
                bundle,
                Some(exec),
                safe_outcome(0, "fp"),
            live_host()
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
            let oracle = pinned_oracle(oracle_id);
            let bundle = measured_bundle("b1", HEX_A);
            let exec = fixture_for(cve_id, &bundle);
            let diverge = oracle_id == OracleId::Runc && cve_id == "CVE-2019-5736";
            let oracle_outcome = if diverge {
                safe_outcome(1, "fp-diverge")
            } else {
                safe_outcome(0, "fp")
            };
            let op = required_op(cve_id);
            let owned_obs = OperationObservation::try_measured(
                owned.kind(),
                op,
                bundle.clone(),
                Some(exec.clone()),
                safe_outcome(0, "fp"),
            live_host()
        )
            .unwrap();
            let oracle_obs = OperationObservation::try_measured(
                oracle.kind(),
                op,
                bundle,
                Some(exec),
                oracle_outcome,
            live_host()
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
        let op = required_op("CVE-2019-5736");
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            op,
            measured_bundle("b1", HEX_A),
            None,
            safe_outcome(0, "fp"),
            live_host()
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("runc", live_oracle_pin())
                .unwrap()
                .kind(),
            op,
            measured_bundle("b1", HEX_A),
            None,
            safe_outcome(0, "fp"),
            live_host()
        )
        .unwrap();
        assert_eq!(
            ComparisonRecord::try_from_observations("CVE-2019-5736", owned, oracle).unwrap_err(),
            HarnessError::MissingCveExecution
        );
    }

    #[test]
    fn comparison_record_rejects_mismatched_cve_execution() {
        // Construct with each CVE's required op so try_measured accepts; mismatch is on CVE ids.
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            required_op("CVE-2019-5736"),
            measured_bundle("b1", HEX_A),
            Some(fixture_for("CVE-2019-5736", &measured_bundle("b1", HEX_A))),
            safe_outcome(0, "fp"),
            live_host(),
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("runc", live_oracle_pin())
                .unwrap()
                .kind(),
            required_op("CVE-2024-21626"),
            measured_bundle("b1", HEX_A),
            Some(fixture_for("CVE-2024-21626", &measured_bundle("b1", HEX_A))),
            safe_outcome(0, "fp"),
            live_host(),
        )
        .unwrap();
        assert!(matches!(
            ComparisonRecord::try_from_observations("CVE-2019-5736", owned, oracle).unwrap_err(),
            HarnessError::CveExecutionMismatch { .. }
        ));
    }

    #[test]
    fn comparison_record_rejects_cve_id_param_mismatch() {
        let bundle = measured_bundle("b1", HEX_A);
        let exec = fixture_for("CVE-2024-21626", &bundle);
        let op = required_op("CVE-2024-21626");
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            op,
            bundle.clone(),
            Some(exec.clone()),
            safe_outcome(0, "fp"),
            live_host()
        )
        .unwrap();
        let oracle = OperationObservation::try_measured(
            OracleStub::try_new_pinned("runc", live_oracle_pin())
                .unwrap()
                .kind(),
            op,
            measured_bundle("b1", HEX_A),
            Some(exec),
            safe_outcome(0, "fp"),
            live_host()
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
        let bundle = measured_bundle("b1", HEX_A);
        assert!(matches!(
            CveExecutionId::from_fixture_material(
                &CveFixtureMaterial {
                    cve_id: "CVE-2019-5736",
                    fixture_bytes: b"",
                },
                &bundle,
            ),
            Err(HarnessError::NonCanonicalCveFixture(_))
        ));
        assert!(matches!(
            CveExecutionId::from_fixture_material(
                &CveFixtureMaterial {
                    cve_id: "CVE-2019-5736",
                    fixture_bytes: b"wrong-bytes-not-canonical",
                },
                &bundle,
            ),
            Err(HarnessError::NonCanonicalCveFixture(_))
        ));
        let ok = CveExecutionId::from_fixture_material(
            &canonical_cve_fixture_material("CVE-2019-5736").unwrap(),
            &bundle,
        )
        .unwrap();
        assert!(ok.has_fixture_receipt());
        // Receipts differ across CVE ids even for the same bundle.
        let mut receipts = BTreeSet::new();
        for id in REQUIRED_CVE_IDS {
            let exec = fixture_for(id, &bundle);
            assert!(receipts.insert(exec.fixture_receipt().to_owned()));
        }
        assert_eq!(receipts.len(), REQUIRED_CVE_IDS.len());
        // Same CVE against a different bundle yields a different receipt.
        let other = fixture_for("CVE-2019-5736", &measured_bundle("b1", HEX_B));
        assert_ne!(ok.fixture_receipt(), other.fixture_receipt());
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
            let oracle = pinned_oracle(oracle_id);
            let bundle = measured_bundle("b1", HEX_A);
            let exec = fixture_for(cve_id, &bundle);
            let op = required_op(cve_id);
            let owned_obs = OperationObservation::try_measured(
                owned.kind(),
                op,
                bundle.clone(),
                Some(exec.clone()),
                safe_outcome(0, "fp"),
            live_host()
        )
            .unwrap();
            let oracle_obs = OperationObservation::try_measured(
                oracle.kind(),
                op,
                bundle,
                Some(exec),
                safe_outcome(0, "fp"),
            live_host()
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
        let measured = fixture_for("CVE-2019-5736", &measured_bundle("b1", HEX_A));
        assert!(measured.has_fixture_receipt());
        assert!(measured.fixture_receipt().starts_with("sha256:"));
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

    #[test]
    fn pin_try_new_stores_trimmed_revision_and_platform() {
        let rev = format!("  sha256:{HEX_A}  ");
        let pin = OraclePin::try_new(&rev, "  linux/amd64  ").unwrap();
        assert_eq!(pin.revision(), format!("sha256:{HEX_A}"));
        assert_eq!(pin.platform(), "linux/amd64");
        let owned = OwnedPin::try_new(&rev, "\tlinux/arm64\t").unwrap();
        assert_eq!(owned.revision(), format!("sha256:{HEX_A}"));
        assert_eq!(owned.platform(), "linux/arm64");
    }

    #[test]
    fn compare_observations_diverges_on_platform_mismatch() {
        let arm_host = MeasuredHostEnvironment::try_new(
            "linux/arm64",
            "scaffold-kernel-build",
            "cgroup2",
            "scaffold-runner:0",
            "ext4",
            "mntns-scaffold",
        )
        .unwrap();
        let owned = OperationObservation::try_measured(
            OwnedExecutorStub::try_pinned(
                OwnedPin::try_new(&format!("sha256:{HEX_OWNED}"), "linux/arm64").unwrap(),
            )
            .unwrap()
            .kind(),
            OciOperation::Start,
            measured_bundle("b1", HEX_A),
            None,
            safe_outcome(0, "fp"),
            arm_host,
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
            live_host(),
        )
        .unwrap();
        assert_eq!(compare_observations(CVE_FD, &owned, &oracle), DiffVerdict::Diverge);
    }

    #[test]
    fn comparison_record_rejects_wrong_cve_operation() {
        let cve_id = "CVE-2019-5736";
        let bundle = measured_bundle("b1", HEX_A);
        let exec = fixture_for(cve_id, &bundle);
        // Create alone does not exec the /proc/self/exe overwrite — Start is required.
        let owned = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Create,
            bundle.clone(),
            Some(exec.clone()),
            safe_outcome(0, "fp"),
            live_host()
        );
        assert!(matches!(owned, Err(HarnessError::CveOperationMismatch { .. })));
        let delete = OperationObservation::try_measured(
            live_owned().kind(),
            OciOperation::Delete,
            measured_bundle("b1", HEX_A),
            Some(exec),
            safe_outcome(0, "fp"),
            live_host()
        );
        assert!(matches!(delete, Err(HarnessError::CveOperationMismatch { .. })));
        assert_eq!(required_op(cve_id), OciOperation::Start);
        assert_eq!(required_op("CVE-2024-21626"), OciOperation::Start);
        assert_eq!(required_op("CVE-MOUNT-SYMLINK-RACE"), OciOperation::Create);
    }

    #[test]
    fn aggregate_rejects_inconsistent_cve_bundle_across_oracles() {
        let owned = live_owned();
        let mut records = Vec::new();
        for (oracle_id, cve_id) in required_matrix_pairs() {
            let oracle = pinned_oracle(oracle_id);
            // Same CVE across oracles gets different digests for the first CVE only.
            let hex = if cve_id == "CVE-2019-5736" && oracle_id == OracleId::Runc {
                HEX_B
            } else if cve_id == "CVE-2019-5736" {
                HEX_A
            } else if cve_id == "CVE-2024-21626" {
                HEX_B
            } else {
                HEX_C
            };
            let bundle = measured_bundle("b1", hex);
            let exec = fixture_for(cve_id, &bundle);
            let op = required_op(cve_id);
            let owned_obs = OperationObservation::try_measured(
                owned.kind(),
                op,
                bundle.clone(),
                Some(exec.clone()),
                safe_outcome(0, "fp"),
            live_host()
        )
            .unwrap();
            let oracle_obs = OperationObservation::try_measured(
                oracle.kind(),
                op,
                bundle,
                Some(exec),
                safe_outcome(0, "fp"),
            live_host()
        )
            .unwrap();
            records
                .push(ComparisonRecord::try_from_observations(cve_id, owned_obs, oracle_obs).unwrap());
        }
        assert!(matches!(
            aggregate_comparison_records(&records).unwrap_err(),
            HarnessError::InconsistentCveBundle(_)
        ));
    }

    #[test]
    fn obligation_oracle_pins_are_distinct_immutable() {
        let pins = obligation_oracle_pins().unwrap();
        assert_eq!(pins.len(), ORACLE_IDS.len());
        let mut revs = BTreeSet::new();
        for id in OracleId::all() {
            let pin = &pins[&id];
            assert!(!pin.is_scaffold());
            assert!(pin.revision().starts_with("sha256:"));
            assert!(revs.insert(pin.revision().to_owned()));
        }
    }

    #[test]
    fn aggregate_rejects_oracle_pin_not_in_inventory() {
        let owned = live_owned();
        let mut records = Vec::new();
        for (oracle_id, cve_id) in required_matrix_pairs() {
            // Same live pin for every oracle — valid sha256 but not inventory.
            let oracle =
                OracleStub::try_new_pinned(oracle_id.as_str(), live_oracle_pin()).unwrap();
            let hex = match cve_id {
                "CVE-2019-5736" => HEX_A,
                "CVE-2024-21626" => HEX_B,
                "CVE-MOUNT-SYMLINK-RACE" => HEX_C,
                other => panic!("unexpected {other}"),
            };
            let bundle = measured_bundle("b1", hex);
            let exec = fixture_for(cve_id, &bundle);
            let op = required_op(cve_id);
            let owned_obs = OperationObservation::try_measured(
                owned.kind(),
                op,
                bundle.clone(),
                Some(exec.clone()),
                safe_outcome(0, "fp"),
            live_host()
        )
            .unwrap();
            let oracle_obs = OperationObservation::try_measured(
                oracle.kind(),
                op,
                bundle,
                Some(exec),
                safe_outcome(0, "fp"),
            live_host()
        )
            .unwrap();
            records
                .push(ComparisonRecord::try_from_observations(cve_id, owned_obs, oracle_obs).unwrap());
        }
        assert!(matches!(
            aggregate_comparison_records(&records).unwrap_err(),
            HarnessError::InventoryOraclePinMismatch(_)
        ));
        assert!(matches!(
            aggregate_comparison_records_against_inventory(&records).unwrap_err(),
            HarnessError::InventoryOraclePinMismatch(_)
        ));
        // Explicit expected map with undeclared pins is rejected (must equal inventory).
        let mut expected = BTreeMap::new();
        for id in OracleId::all() {
            expected.insert(id, live_oracle_pin());
        }
        assert!(matches!(
            aggregate_comparison_records_with_expected_pins(&records, &expected).unwrap_err(),
            HarnessError::InventoryOraclePinMismatch(_)
        ));
    }
}
