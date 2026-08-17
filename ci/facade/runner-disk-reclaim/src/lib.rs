//! # cloud-ci-runner-disk-reclaim (FRIC-017 productization)
//!
//! Data-driven, Rust-first CI-runner disk preflight. Replaces the two duplicated inline
//! `sudo rm -rf /usr/share/dotnet …` blocks in `.github/workflows/oya-ci-required.yml` (the
//! `buck2` + `gate-affected-set` jobs) with one neutral engine + a single source-of-truth
//! policy file (`runner-disk-reclaim-policy.json`): founder doctrine "new automation never
//! ships as shell" (ADR-0548 pipeline-as-product; ADR-0556/0560 cache-integrity-unaffected).
//!
//! The engine is pure + fs-injected so the reclaim plan, the policy parse, and the
//! threshold/INFRA-RED logic are unit-testable against a temp dir — never the real system dirs.
//! `src/main.rs` wires the real fs (`std::fs::remove_dir_all` + `libc::statvfs`) and runs it as
//! a runner preflight step BEFORE the buck-out warm restore (so it must build/run with ZERO
//! dependency on the buck-out cache).

use serde_json::{Value, json};
use std::path::Path;

/// Bytes in one GiB.
pub const GIB: u64 = 1024 * 1024 * 1024;

/// The reclaim policy for a single runner profile, parsed from the policy DATA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReclaimProfile {
    /// Vendor preinstall dirs to remove (best-effort, like the old `|| true`).
    pub reclaim_dirs: Vec<String>,
    /// Post-reclaim free-disk floor on `/` (GiB). Below it ⇒ INFRA-RED.
    pub min_free_gib_after: u64,
}

/// Why a policy parse failed. Fail-loud: a malformed/absent profile is a hard error naming the
/// defect, never a silent "reclaim nothing" degrade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    /// The policy text is not valid JSON.
    Json(String),
    /// `profiles` is absent or not an object.
    MissingProfiles,
    /// The requested profile id is not present in `profiles`.
    UnknownProfile(String),
    /// A profile field is absent or the wrong shape.
    MalformedProfile(String),
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyError::Json(e) => write!(f, "policy json parse error: {e}"),
            PolicyError::MissingProfiles => {
                write!(f, "policy is missing the `profiles` object")
            }
            PolicyError::UnknownProfile(id) => {
                write!(
                    f,
                    "unknown runner profile `{id}` (not declared in policy `profiles`)"
                )
            }
            PolicyError::MalformedProfile(why) => write!(f, "malformed profile: {why}"),
        }
    }
}

impl std::error::Error for PolicyError {}

/// Parse the reclaim profile for `profile_id` from the policy DATA text. Pure + fail-loud.
pub fn parse_profile(policy_text: &str, profile_id: &str) -> Result<ReclaimProfile, PolicyError> {
    let doc: Value =
        serde_json::from_str(policy_text).map_err(|e| PolicyError::Json(e.to_string()))?;
    let profiles = doc
        .get("profiles")
        .and_then(Value::as_object)
        .ok_or(PolicyError::MissingProfiles)?;
    let profile = profiles
        .get(profile_id)
        .ok_or_else(|| PolicyError::UnknownProfile(profile_id.to_owned()))?;

    let reclaim_dirs = profile
        .get("reclaim_dirs")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            PolicyError::MalformedProfile(format!(
                "profile `{profile_id}`: `reclaim_dirs` must be an array of strings"
            ))
        })?
        .iter()
        .map(|v| {
            v.as_str().map(str::to_owned).ok_or_else(|| {
                PolicyError::MalformedProfile(format!(
                    "profile `{profile_id}`: every `reclaim_dirs` entry must be a string"
                ))
            })
        })
        .collect::<Result<Vec<String>, PolicyError>>()?;

    let min_free_gib_after = profile
        .get("min_free_gib_after")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            PolicyError::MalformedProfile(format!(
                "profile `{profile_id}`: `min_free_gib_after` must be a non-negative integer (GiB)"
            ))
        })?;

    Ok(ReclaimProfile {
        reclaim_dirs,
        min_free_gib_after,
    })
}

/// Filesystem effects, injected so the reclaim logic is testable against a temp dir (never the
/// real system dirs). `main.rs` provides the real impl; tests provide an in-memory/temp fake.
pub trait DiskOps {
    /// Free bytes on the filesystem containing `path`. Errors propagate (fail-loud on a
    /// runner whose `/` cannot be stat'd).
    fn free_bytes(&self, path: &Path) -> std::io::Result<u64>;
    /// Best-effort recursive remove of `dir`. An absent dir is NOT an error (mirrors the old
    /// `rm -rf … || true`); a present-but-undeletable dir surfaces its io error to the log but
    /// never aborts the reclaim of the remaining dirs.
    fn remove_dir_all(&self, dir: &Path) -> std::io::Result<()>;
}

/// The outcome of one dir's best-effort reclaim, for structured logging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirOutcome {
    /// The dir was present and removed.
    Removed,
    /// The dir was already absent (no-op, like `|| true`).
    Absent,
    /// The dir was present but removal failed; the io error string is carried for the log.
    Failed(String),
    /// The path was rejected by the safety guard before any fs operation (not absolute,
    /// equals a filesystem root, or contains `..`). Never reaches `DiskOps::remove_dir_all`.
    Rejected(String),
}

/// Safety guard: reject a reclaim path if it is (a) not absolute, (b) a filesystem root (`/`,
/// `//`, etc.), or (c) contains any `..` component. Returns `Err(reason)` on a violation.
///
/// This hardens the engine against a future malformed policy edit — the committed policy's
/// 5 vendor dirs are all safe absolute paths, so this guard only fires on policy bugs.
pub fn validate_reclaim_dir(path: &Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!(
            "path `{}` is not absolute — refusing to reclaim a relative path",
            path.display()
        ));
    }
    // Reject pure filesystem roots: paths whose only component is the root itself.
    // `components()` on `/` yields exactly one `RootDir` component with no `Normal` parts.
    let has_normal = path
        .components()
        .any(|c| matches!(c, std::path::Component::Normal(_)));
    if !has_normal {
        return Err(format!(
            "path `{}` is a filesystem root — refusing to reclaim a root dir",
            path.display()
        ));
    }
    // Reject any path that contains a `..` component (even after parsing).
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(format!(
            "path `{}` contains `..` — refusing to reclaim a path with parent-dir traversal",
            path.display()
        ));
    }
    Ok(())
}

/// The result of a full reclaim run: per-dir outcomes + the before/after free-byte snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReclaimReport {
    pub free_before: u64,
    pub free_after: u64,
    pub outcomes: Vec<(String, DirOutcome)>,
    pub min_free_gib_after: u64,
}

impl ReclaimReport {
    /// Bytes freed (saturating: a concurrent writer could grow usage, so this never underflows).
    pub fn freed_bytes(&self) -> u64 {
        self.free_after.saturating_sub(self.free_before)
    }

    /// Free GiB after reclaim (floor division — a partial GiB does not satisfy an integer floor).
    pub fn free_gib_after(&self) -> u64 {
        self.free_after / GIB
    }

    /// INFRA-RED iff the post-reclaim free disk is strictly below the policy floor. When true a
    /// downstream disk-exhaustion is attributable to INFRA (insufficient runner capacity), not
    /// to CODE.
    pub fn is_infra_red(&self) -> bool {
        self.free_gib_after() < self.min_free_gib_after
    }
}

/// Required-context handling for an INFRA-RED disk-capacity result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfraRedPolicy {
    /// Capacity miss is an infrastructure failure and exits non-zero.
    FailClosed,
    /// Capacity miss may exit zero only with a typed waiver and a durable operator artifact.
    FailOpenWithWaiver,
}

impl InfraRedPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            InfraRedPolicy::FailClosed => "fail-closed",
            InfraRedPolicy::FailOpenWithWaiver => "fail-open-with-waiver",
        }
    }
}

/// Typed justification required before an INFRA-RED result can fail open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfraRedWaiver {
    /// data_class: INTERNAL_ONLY — stable policy/ADR/task identifier authorizing fail-open.
    pub waiver_id: String,
    /// data_class: INTERNAL_ONLY — operator-readable justification; must not contain secrets.
    pub reason: String,
}

impl InfraRedWaiver {
    pub fn new(waiver_id: String, reason: String) -> Result<Self, String> {
        if waiver_id.trim().is_empty() {
            return Err("INFRA-RED fail-open waiver requires a non-empty waiver id".to_owned());
        }
        if reason.trim().is_empty() {
            return Err("INFRA-RED fail-open waiver requires a non-empty reason".to_owned());
        }
        Ok(Self { waiver_id, reason })
    }
}

/// Validate the required-context contract for an INFRA-RED result.
///
/// A fail-open INFRA-RED can exit 0 only when it has both a typed waiver and a durable operator
/// artifact output path. Without the artifact, the waiver would exist only in logs and the required
/// context could silently green without machine-readable evidence.
pub fn validate_infra_red_exit_contract(
    report: &ReclaimReport,
    policy: InfraRedPolicy,
    waiver: Option<&InfraRedWaiver>,
    artifact_output_requested: bool,
) -> Result<(), String> {
    if !report.is_infra_red() || policy != InfraRedPolicy::FailOpenWithWaiver {
        return Ok(());
    }
    if waiver.is_none() {
        return Err(
            "INFRA-RED fail-open requires a typed waiver before the required context may stay green"
                .to_owned(),
        );
    }
    if !artifact_output_requested {
        return Err(
            "INFRA-RED fail-open requires --artifact-out so the typed waiver is durable".to_owned(),
        );
    }
    Ok(())
}

/// Machine-readable operator artifact for the runner disk-reclaim preflight.
///
/// This is intentionally deterministic (no wall-clock field) so fixtures can compare it directly.
/// The workflow uploads the file with GitHub artifact retention metadata.
pub fn runner_disk_reclaim_operator_artifact(
    profile_id: &str,
    report: &ReclaimReport,
    policy: InfraRedPolicy,
    waiver: Option<&InfraRedWaiver>,
) -> Result<Value, String> {
    if report.is_infra_red() && policy == InfraRedPolicy::FailOpenWithWaiver && waiver.is_none() {
        return Err(
            "INFRA-RED fail-open requires a typed waiver before the required context may stay green"
                .to_owned(),
        );
    }

    let outcomes = report
        .outcomes
        .iter()
        .map(|(dir, outcome)| {
            let (status, detail) = match outcome {
                DirOutcome::Removed => ("removed", None),
                DirOutcome::Absent => ("absent", None),
                DirOutcome::Failed(error) => ("failed", Some(error.as_str())),
                DirOutcome::Rejected(reason) => ("rejected", Some(reason.as_str())),
            };
            json!({
                "path": dir,
                "status": status,
                "detail": detail,
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "schema_version": 1,
        "artifact_type": "cloud_ci_operator_artifact",
        "artifact_id": "runner-disk-reclaim",
        "gate_id": "cloud-ci-runner-disk-reclaim",
        "runner_profile": profile_id,
        "infra_red": report.is_infra_red(),
        "infra_red_policy": policy.as_str(),
        "typed_waiver": waiver.map(|w| json!({
            "waiver_id": w.waiver_id.as_str(),
            "reason": w.reason.as_str(),
        })),
        "disk": {
            "free_before_bytes": report.free_before,
            "free_after_bytes": report.free_after,
            "freed_bytes": report.freed_bytes(),
            "free_after_gib": report.free_gib_after(),
            "min_free_gib_after": report.min_free_gib_after,
        },
        "reclaim_outcomes": outcomes,
        "retention_and_pii": {
            "retention_days": 30,
            "pii": "none; runner profile, filesystem paths, byte counts, and typed waiver only",
            "secret_redaction": "no tenant, idempotency, DSN, token, or password material is emitted"
        }
    }))
}

/// Run the reclaim plan for `profile` using the injected `DiskOps`. Pure orchestration: it
/// snapshots free-before, best-effort removes each dir in declared order, snapshots free-after,
/// and returns the structured report (the caller decides exit code / logging). Errors only on a
/// `free_bytes` stat failure (fail-loud); per-dir removal failures are captured as outcomes, not
/// aborts (the old `|| true` semantics).
pub fn run_reclaim(
    ops: &dyn DiskOps,
    root: &Path,
    profile: &ReclaimProfile,
) -> std::io::Result<ReclaimReport> {
    let free_before = ops.free_bytes(root)?;
    let mut outcomes = Vec::with_capacity(profile.reclaim_dirs.len());
    for dir in &profile.reclaim_dirs {
        let path = Path::new(dir);
        let outcome = match validate_reclaim_dir(path) {
            Err(reason) => DirOutcome::Rejected(reason),
            Ok(()) => {
                if !path.exists() {
                    DirOutcome::Absent
                } else {
                    match ops.remove_dir_all(path) {
                        Ok(()) => DirOutcome::Removed,
                        // A NotFound race ⇒ Absent (lost to a concurrent reaper, still a no-op).
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => DirOutcome::Absent,
                        Err(e) => DirOutcome::Failed(e.to_string()),
                    }
                }
            }
        };
        outcomes.push((dir.clone(), outcome));
    }
    let free_after = ops.free_bytes(root)?;
    Ok(ReclaimReport {
        free_before,
        free_after,
        outcomes,
        min_free_gib_after: profile.min_free_gib_after,
    })
}

/// The committed policy's repo-relative path (single source of truth for both the gate test and
/// the binary's default).
pub const POLICY_REL_PATH: &str = "ci/facade/runner-disk-reclaim/runner-disk-reclaim-policy.json";

/// Walk up from `start` to the repo root (the dir holding `specs/root-hub-pointers.json`). Mirrors
/// the cache-wiring gate's root discovery so the live-corpus test works under both buck2 (cwd =
/// project root) and cargo (cwd = crate dir) without `CARGO_MANIFEST_DIR`.
pub fn repo_root_from(start: &Path) -> Option<std::path::PathBuf> {
    let mut dir = start.to_path_buf();
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    const POLICY: &str = r#"{
        "profiles": {
            "github-hosted-ubuntu-latest": {
                "reclaim_dirs": ["/usr/share/dotnet", "/opt/ghc"],
                "min_free_gib_after": 20
            }
        }
    }"#;

    #[test]
    fn parses_seeded_profile() {
        let p = parse_profile(POLICY, "github-hosted-ubuntu-latest").expect("parse");
        assert_eq!(p.reclaim_dirs, vec!["/usr/share/dotnet", "/opt/ghc"]);
        assert_eq!(p.min_free_gib_after, 20);
    }

    #[test]
    fn unknown_profile_is_fail_loud() {
        let err = parse_profile(POLICY, "self-hosted").unwrap_err();
        assert!(matches!(err, PolicyError::UnknownProfile(id) if id == "self-hosted"));
    }

    #[test]
    fn malformed_json_is_fail_loud() {
        assert!(matches!(
            parse_profile("{not json", "x"),
            Err(PolicyError::Json(_))
        ));
    }

    #[test]
    fn missing_min_free_is_malformed() {
        let text = r#"{"profiles":{"p":{"reclaim_dirs":[]}}}"#;
        assert!(matches!(
            parse_profile(text, "p"),
            Err(PolicyError::MalformedProfile(_))
        ));
    }

    #[test]
    fn non_string_reclaim_dir_is_malformed() {
        let text = r#"{"profiles":{"p":{"reclaim_dirs":[1],"min_free_gib_after":1}}}"#;
        assert!(matches!(
            parse_profile(text, "p"),
            Err(PolicyError::MalformedProfile(_))
        ));
    }

    /// A fake fs: a configurable free-bytes sequence (before, after) + a set of dirs that
    /// "exist" within a temp root. Never touches real system dirs.
    struct FakeOps {
        free: RefCell<Vec<u64>>,
        removed: RefCell<Vec<String>>,
        fail_on: BTreeSet<String>,
    }

    impl DiskOps for FakeOps {
        fn free_bytes(&self, _path: &Path) -> std::io::Result<u64> {
            Ok(self.free.borrow_mut().remove(0))
        }
        fn remove_dir_all(&self, dir: &Path) -> std::io::Result<()> {
            let key = dir.to_string_lossy().into_owned();
            if self.fail_on.contains(&key) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "denied",
                ));
            }
            self.removed.borrow_mut().push(key);
            Ok(())
        }
    }

    fn profile(dirs: &[&str], min: u64) -> ReclaimProfile {
        ReclaimProfile {
            reclaim_dirs: dirs.iter().map(|s| (*s).to_owned()).collect(),
            min_free_gib_after: min,
        }
    }

    #[test]
    fn reclaim_plan_removes_present_skips_absent() {
        // Use temp dirs so `path.exists()` is real but scoped to the temp root.
        let tmp = std::env::temp_dir().join(format!("fric017-{}", std::process::id()));
        let present = tmp.join("present");
        let absent = tmp.join("absent");
        std::fs::create_dir_all(&present).expect("mkdir");
        let prof = profile(&[present.to_str().unwrap(), absent.to_str().unwrap()], 20);
        let ops = FakeOps {
            free: RefCell::new(vec![10 * GIB, 25 * GIB]),
            removed: RefCell::new(Vec::new()),
            fail_on: BTreeSet::new(),
        };
        let report = run_reclaim(&ops, Path::new("/"), &prof).expect("reclaim");
        assert_eq!(report.outcomes[0].1, DirOutcome::Removed);
        assert_eq!(report.outcomes[1].1, DirOutcome::Absent);
        // Only the present dir reached the (faked) remover; the absent dir was skipped pre-remove.
        assert_eq!(
            ops.removed.into_inner(),
            vec![present.to_string_lossy().into_owned()]
        );
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn removal_failure_is_captured_not_aborted() {
        let tmp = std::env::temp_dir().join(format!("fric017-fail-{}", std::process::id()));
        let d1 = tmp.join("d1");
        let d2 = tmp.join("d2");
        std::fs::create_dir_all(&d1).expect("mkdir");
        std::fs::create_dir_all(&d2).expect("mkdir");
        let prof = profile(&[d1.to_str().unwrap(), d2.to_str().unwrap()], 20);
        let mut fail_on = BTreeSet::new();
        fail_on.insert(d1.to_string_lossy().into_owned());
        let ops = FakeOps {
            free: RefCell::new(vec![10 * GIB, 25 * GIB]),
            removed: RefCell::new(Vec::new()),
            fail_on,
        };
        let report = run_reclaim(&ops, Path::new("/"), &prof).expect("reclaim");
        assert!(matches!(report.outcomes[0].1, DirOutcome::Failed(_)));
        assert_eq!(report.outcomes[1].1, DirOutcome::Removed);
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn infra_red_when_below_floor() {
        let report = ReclaimReport {
            free_before: 5 * GIB,
            free_after: 18 * GIB,
            outcomes: vec![],
            min_free_gib_after: 20,
        };
        assert!(report.is_infra_red());
        assert_eq!(report.free_gib_after(), 18);
        assert_eq!(report.freed_bytes(), 13 * GIB);
    }

    #[test]
    fn ok_when_at_or_above_floor() {
        let report = ReclaimReport {
            free_before: 5 * GIB,
            free_after: 20 * GIB,
            outcomes: vec![],
            min_free_gib_after: 20,
        };
        assert!(!report.is_infra_red());
        assert_eq!(report.free_gib_after(), 20);
    }

    #[test]
    fn operator_artifact_records_fail_closed_infra_red_without_secrets() {
        let report = ReclaimReport {
            free_before: 5 * GIB,
            free_after: 18 * GIB,
            outcomes: vec![
                ("/usr/share/dotnet".to_owned(), DirOutcome::Removed),
                ("/opt/ghc".to_owned(), DirOutcome::Absent),
            ],
            min_free_gib_after: 20,
        };

        let artifact = runner_disk_reclaim_operator_artifact(
            "github-hosted-ubuntu-latest",
            &report,
            InfraRedPolicy::FailClosed,
            None,
        )
        .expect("fail-closed artifact should not need a waiver");

        assert_eq!(artifact["artifact_type"], "cloud_ci_operator_artifact");
        assert_eq!(artifact["artifact_id"], "runner-disk-reclaim");
        assert_eq!(artifact["infra_red"], true);
        assert_eq!(artifact["infra_red_policy"], "fail-closed");
        assert!(artifact["typed_waiver"].is_null());
        assert_eq!(artifact["disk"]["free_after_gib"], 18);
        assert_eq!(artifact["reclaim_outcomes"].as_array().unwrap().len(), 2);
        let rendered = artifact.to_string();
        assert!(
            !rendered.contains("postgres://")
                && !rendered.contains("postgres:postgres")
                && !rendered.contains("oya_app:app"),
            "operator artifact must not leak DSNs or credentials: {rendered}"
        );
    }

    #[test]
    fn fail_open_infra_red_requires_typed_waiver() {
        let report = ReclaimReport {
            free_before: 5 * GIB,
            free_after: 18 * GIB,
            outcomes: vec![],
            min_free_gib_after: 20,
        };

        let err = runner_disk_reclaim_operator_artifact(
            "github-hosted-ubuntu-latest",
            &report,
            InfraRedPolicy::FailOpenWithWaiver,
            None,
        )
        .unwrap_err();
        assert!(
            err.contains("typed waiver"),
            "missing fail-open waiver must fail closed: {err}"
        );

        let waiver = InfraRedWaiver::new(
            "INFRA-RED-WAIVER-001".to_owned(),
            "temporary GitHub hosted runner capacity incident".to_owned(),
        )
        .expect("valid waiver");
        let artifact = runner_disk_reclaim_operator_artifact(
            "github-hosted-ubuntu-latest",
            &report,
            InfraRedPolicy::FailOpenWithWaiver,
            Some(&waiver),
        )
        .expect("typed waiver allows artifact generation");
        assert_eq!(artifact["infra_red_policy"], "fail-open-with-waiver");
        assert_eq!(
            artifact["typed_waiver"]["waiver_id"],
            "INFRA-RED-WAIVER-001"
        );
    }

    #[test]
    fn fail_open_infra_red_requires_durable_artifact_output() {
        let report = ReclaimReport {
            free_before: 5 * GIB,
            free_after: 18 * GIB,
            outcomes: vec![],
            min_free_gib_after: 20,
        };
        let waiver = InfraRedWaiver::new(
            "INFRA-RED-WAIVER-001".to_owned(),
            "temporary GitHub hosted runner capacity incident".to_owned(),
        )
        .expect("valid waiver");

        let err = validate_infra_red_exit_contract(
            &report,
            InfraRedPolicy::FailOpenWithWaiver,
            Some(&waiver),
            false,
        )
        .unwrap_err();
        assert!(
            err.contains("--artifact-out"),
            "fail-open without artifact output must fail closed: {err}"
        );

        validate_infra_red_exit_contract(
            &report,
            InfraRedPolicy::FailOpenWithWaiver,
            Some(&waiver),
            true,
        )
        .expect("typed waiver plus artifact output is acceptable");
    }

    #[test]
    fn freed_bytes_saturates_on_concurrent_growth() {
        let report = ReclaimReport {
            free_before: 20 * GIB,
            free_after: 19 * GIB,
            outcomes: vec![],
            min_free_gib_after: 10,
        };
        assert_eq!(report.freed_bytes(), 0);
    }

    // --- path-safety guard tests ---

    #[test]
    fn guard_accepts_safe_absolute_paths() {
        assert!(validate_reclaim_dir(Path::new("/usr/share/dotnet")).is_ok());
        assert!(validate_reclaim_dir(Path::new("/opt/hostedtoolcache/CodeQL")).is_ok());
        assert!(validate_reclaim_dir(Path::new("/usr/local/lib/android")).is_ok());
    }

    #[test]
    fn guard_rejects_relative_path() {
        let err = validate_reclaim_dir(Path::new("usr/share/dotnet")).unwrap_err();
        assert!(err.contains("not absolute"), "got: {err}");
    }

    #[test]
    fn guard_rejects_filesystem_root() {
        let err = validate_reclaim_dir(Path::new("/")).unwrap_err();
        assert!(err.contains("filesystem root"), "got: {err}");
        // Double-slash root variant.
        let err2 = validate_reclaim_dir(Path::new("//")).unwrap_err();
        assert!(err2.contains("filesystem root"), "got: {err2}");
    }

    #[test]
    fn guard_rejects_dotdot_traversal() {
        let err = validate_reclaim_dir(Path::new("/usr/share/../dotnet")).unwrap_err();
        assert!(err.contains(".."), "got: {err}");
        let err2 = validate_reclaim_dir(Path::new("/usr/share/dotnet/..")).unwrap_err();
        assert!(err2.contains(".."), "got: {err2}");
    }

    #[test]
    fn run_reclaim_rejects_bad_path_without_calling_remove() {
        // A Rejected outcome must never reach DiskOps::remove_dir_all.
        let prof = profile(&["/", "usr/share/dotnet", "/opt/ghc/../real"], 1);
        let ops = FakeOps {
            free: RefCell::new(vec![10 * GIB, 10 * GIB]),
            removed: RefCell::new(Vec::new()),
            fail_on: BTreeSet::new(),
        };
        let report = run_reclaim(&ops, Path::new("/"), &prof).expect("reclaim");
        // All three are rejected.
        assert!(
            matches!(report.outcomes[0].1, DirOutcome::Rejected(_)),
            "/ must be Rejected"
        );
        assert!(
            matches!(report.outcomes[1].1, DirOutcome::Rejected(_)),
            "relative must be Rejected"
        );
        assert!(
            matches!(report.outcomes[2].1, DirOutcome::Rejected(_)),
            "dotdot must be Rejected"
        );
        // remove_dir_all was never called.
        assert!(
            ops.removed.borrow().is_empty(),
            "remove must not be called for rejected paths"
        );
    }
}
