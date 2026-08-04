//! cloud-ci-cache-wiring (ADR-0560): policy-as-data buck2 cache wiring for the
//! NativeLink CAS warm substrate, consuming the ADR-0556 classification.
//!
//! NEUTRAL engine — all repo-specifics live in DATA:
//! - the class posture comes from `/specs/cache-warmth-policy.json` (ADR-0556 D1;
//!   this crate re-decides nothing, it only enforces the policy fail-closed);
//! - the canary-licensed kill-switch comes from `/specs/cache-warm-license.json`
//!   (the mechanical carrier of the ADR-0556 D2 trust-invariant clause (b));
//! - the opt-in overlays live under `infra/ci/buckconfig/`; the controller
//!   materializes the selected effective config privately for one child daemon.
//!
//! Fail-closed everywhere: unknown class -> bypass, unlicensed -> bypass, the
//! canary class -> bypass unconditionally, warm emission without a keyed identity
//! -> hard error. Cache telemetry is read structurally from buck2's
//! `--unstable-write-invocation-record` JSON, never grepped from logs.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// Repo-relative path of the ADR-0556 classification policy (single source).
pub const POLICY_PATH: &str = "specs/cache-warmth-policy.json";
/// Repo-relative path of the ADR-0560 warm-license kill-switch.
pub const LICENSE_PATH: &str = "specs/cache-warm-license.json";
/// Repo-relative path of the warm read+write overlay (writer endpoint).
pub const OVERLAY_RW_PATH: &str = "infra/ci/buckconfig/warm-cache-rw.buckconfig";
/// Repo-relative path of the warm read-only overlay (reader endpoint).
pub const OVERLAY_RO_PATH: &str = "infra/ci/buckconfig/warm-cache-ro.buckconfig";
/// Env var carrying the path of the lane's mTLS client certificate (keyed identity).
pub const CLIENT_CERT_ENV: &str = "OYA_CACHE_TLS_CLIENT_CERT";
/// Env var carrying the path of the CA bundle that signed the CAS server cert.
pub const TLS_CA_CERTS_ENV: &str = "OYA_CACHE_TLS_CA_CERTS";
/// Schema id of the structured per-lane cache-hit report artifact.
pub const CACHE_HIT_REPORT_SCHEMA: &str = "oya-ci/cache-hit-report/v1";
/// Schema id of the canary digest manifest artifact.
pub const DIGEST_MANIFEST_SCHEMA: &str = "oya-ci/canary-digest-manifest/v1";
/// Schema id of the canary verdict artifact.
pub const CANARY_VERDICT_SCHEMA: &str = "oya-ci/canary-verdict/v1";

/// The resolved cache posture for one build invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    /// No cache participation at all: no RE client config is emitted, buck2 never
    /// dials the CAS (the ADR-0556 cold posture is bypass, not read-only).
    Bypass,
    /// Read hits allowed, uploads forbidden (reader endpoint; AC served read-only).
    WarmReadOnly,
    /// Read + write through the writer endpoint (trusted lanes only).
    WarmReadWrite,
}

impl CacheMode {
    pub fn as_str(self) -> &'static str {
        match self {
            CacheMode::Bypass => "bypass",
            CacheMode::WarmReadOnly => "warm-ro",
            CacheMode::WarmReadWrite => "warm-rw",
        }
    }
}

impl fmt::Display for CacheMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A resolution: the mode plus the reason chain that licensed (or refused) it.
#[derive(Debug, Clone)]
pub struct Resolution {
    pub build_class: String,
    pub mode: CacheMode,
    pub reasons: Vec<String>,
}

impl Resolution {
    pub fn to_json(&self) -> Value {
        json!({
            "build_class": self.build_class,
            "mode": self.mode.as_str(),
            "reasons": self.reasons,
        })
    }
}

fn bool_field(value: &Value, key: &str) -> Result<bool, String> {
    value
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("required boolean field `{key}` missing or non-boolean"))
}

/// Load + structurally validate the warmth policy.
pub fn load_policy(root: &Path) -> Result<Value, String> {
    let path = root.join(POLICY_PATH);
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let policy: Value =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    if !policy.get("build_classes").is_some_and(Value::is_object) {
        return Err("policy missing `build_classes` object".to_string());
    }
    Ok(policy)
}

/// Load + structurally validate the warm-license kill-switch. A malformed or
/// missing license is an ERROR (loud), never an implicit grant.
pub fn load_license(root: &Path) -> Result<Value, String> {
    let path = root.join(LICENSE_PATH);
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let license: Value =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    bool_field(&license, "warm_reads_licensed")?;
    Ok(license)
}

/// The name of the canary build class, read from the policy's own trust-invariant
/// pointer (no class name is hardcoded here).
pub fn canary_class(policy: &Value) -> Option<&str> {
    policy
        .get("trust_invariant")
        .and_then(|t| t.get("canary_build_class"))
        .and_then(Value::as_str)
}

/// Resolve the cache posture for `build_class`, fail-closed.
pub fn resolve(policy: &Value, license: &Value, build_class: &str) -> Result<Resolution, String> {
    let mut reasons = Vec::new();
    let bypass = |reasons: Vec<String>| Resolution {
        build_class: build_class.to_string(),
        mode: CacheMode::Bypass,
        reasons,
    };

    // 1. The trust anchor may NEVER participate, independent of any other data:
    //    any cache participation makes the proof circular (ADR-0556 D1).
    if canary_class(policy) == Some(build_class) {
        reasons.push(
            "integrity-canary trust anchor: cache participation would make the warm==cold \
             proof circular (ADR-0556 D1/D2) — bypass unconditionally"
                .to_string(),
        );
        return Ok(bypass(reasons));
    }

    // 2. Unlisted classes have no warm license — warmth is granted by reviewed
    //    classification, never by omission (policy default_for_unlisted_classes).
    let Some(entry) = policy
        .get("build_classes")
        .and_then(|c| c.get(build_class))
        .filter(|e| e.is_object())
    else {
        reasons.push(format!(
            "build class `{build_class}` is not listed in {POLICY_PATH}: fail-closed default \
             (cold, no read, no write) — warmth is never granted by omission (ADR-0556 D1)"
        ));
        return Ok(bypass(reasons));
    };

    let warmth = entry.get("warmth").and_then(Value::as_str).unwrap_or("");
    let cache_read = bool_field(entry, "cache_read")?;
    let cache_write = bool_field(entry, "cache_write")?;

    // 3. Cold classes BYPASS the CAS entirely (never read-only): a cold build does
    //    not dial the CAS at all (ADR-0556 D3).
    if warmth != "warm" || !cache_read {
        reasons.push(format!(
            "class `{build_class}` is classified {warmth:?} with cache_read={cache_read}: \
             cold classes bypass the CAS entirely (ADR-0556 D3)"
        ));
        return Ok(bypass(reasons));
    }

    // 4. The kill-switch: warm is admissible IFF the most recent cold canary run is
    //    GREEN (ADR-0556 D2 clause (b), carried by the license file).
    if !bool_field(license, "warm_reads_licensed")? {
        let why = license
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("license file carries warm_reads_licensed=false");
        reasons.push(format!(
            "warm-eligible class `{build_class}` refused: warm_reads_licensed=false ({why}) — \
             the ADR-0556 D2 IFF is unsatisfied, all warm reads stay suspended"
        ));
        return Ok(bypass(reasons));
    }

    reasons.push(format!(
        "class `{build_class}` is warm-eligible and the warm license is GREEN-backed \
         (licensed_by_canary_run={})",
        license
            .get("licensed_by_canary_run")
            .map(Value::to_string)
            .unwrap_or_else(|| "null".to_string())
    ));
    let mode = if cache_write {
        CacheMode::WarmReadWrite
    } else {
        CacheMode::WarmReadOnly
    };
    Ok(Resolution {
        build_class: build_class.to_string(),
        mode,
        reasons,
    })
}

/// Materialize the effective project configuration Buck2 actually reads at daemon
/// startup. `--config*` is deliberately not emitted: Buck2 does not apply those
/// flags to `buck2_re_client`, so doing so would produce an inert warm-cache claim.
pub fn effective_buckconfig(
    resolution: &Resolution,
    overlay: &str,
    client_cert: Option<&str>,
    tls_ca_certs: Option<&str>,
) -> Result<Option<String>, String> {
    match resolution.mode {
        CacheMode::Bypass => Ok(None),
        CacheMode::WarmReadOnly | CacheMode::WarmReadWrite => {
            let cert = client_cert.filter(|c| !c.trim().is_empty()).ok_or_else(|| {
                format!(
                    "warm mode `{}` requires the keyed mTLS client identity: set {CLIENT_CERT_ENV} \
                     to the secret-mounted client certificate path (founder 2026-05-30 keyed-auth \
                     posture; enforcement lives at the CAS service boundary)",
                    resolution.mode
                )
            })?;
            if !Path::new(cert).is_absolute()
                || tls_ca_certs
                    .filter(|path| !path.trim().is_empty())
                    .is_some_and(|path| !Path::new(path).is_absolute())
            {
                return Err("cache TLS certificate paths must be absolute".to_string());
            }
            if [cert, tls_ca_certs.unwrap_or_default()]
                .iter()
                .any(|value| value.contains(['\n', '\r']))
            {
                return Err("cache TLS paths must not contain newlines".to_string());
            }
            if !overlay
                .lines()
                .any(|line| line.trim() == "[buck2_re_client]")
            {
                return Err("warm overlay missing [buck2_re_client] section".to_string());
            }
            let mut identity = format!("[buck2_re_client]\n  tls_client_cert = {cert}\n");
            if let Some(ca) = tls_ca_certs.filter(|c| !c.trim().is_empty()) {
                identity.push_str(&format!("  tls_ca_certs = {ca}\n"));
            }
            let config = overlay.replacen("[buck2_re_client]", &identity, 1);
            Ok(Some(config))
        }
    }
}

/// Create the ignored machine-local project config without clobbering a human or
/// sibling controller's file. Warm credentials make 0600 mandatory.
pub fn install_local_buckconfig(root: &Path, contents: &str) -> Result<PathBuf, String> {
    let path = root.join(".buckconfig.local");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(not(unix))]
    return Err("warm-cache controller requires Unix mode-0600 file semantics".to_string());

    #[cfg(unix)]
    {
        let mut file = options
            .open(&path)
            .map_err(|error| format!("create {} without clobbering: {error}", path.display()))?;
        file.write_all(contents.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|error| format!("write {}: {error}", path.display()))?;
        Ok(path)
    }
}

pub fn remove_local_buckconfig(path: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|error| format!("remove {}: {error}", path.display()))
}

/// Locate the InvocationRecord payload inside a buck2
/// `--unstable-write-invocation-record` JSON document.
pub fn invocation_record(doc: &Value) -> Result<&Value, String> {
    doc.pointer("/data/Record/data/InvocationRecord")
        .ok_or_else(|| "no /data/Record/data/InvocationRecord in record JSON".to_string())
}

fn record_u64(record: &Value, key: &str) -> u64 {
    record.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn required_u64(record: &Value, key: &str, findings: &mut Vec<String>) -> Option<u64> {
    match record.get(key).and_then(Value::as_u64) {
        Some(value) => Some(value),
        None => {
            findings.push(format!(
                "record-shape violation: counter `{key}` missing from the invocation record — \
                 cannot prove warm cache behavior from an unrecognized record shape (fail closed)"
            ));
            None
        }
    }
}

fn required_last_snapshot_u64(
    record: &Value,
    key: &str,
    findings: &mut Vec<String>,
) -> Option<u64> {
    let pointer = format!("/last_snapshot/{key}");
    match record.pointer(&pointer).and_then(Value::as_u64) {
        Some(value) => Some(value),
        None => {
            findings.push(format!(
                "record-shape violation: last_snapshot.{key} missing — cannot prove warm cache \
                 behavior from an unrecognized record shape (fail closed)"
            ));
            None
        }
    }
}

/// Build the structured per-lane cache-hit report (the audit's missing-SLO item):
/// action-cache hits, local/remote executions, upload counts, and buck2's own
/// cache_hit_rate, labeled with the lane's build class + resolved mode.
pub fn cache_hit_report(record: &Value, build_class: &str, mode: &str) -> Value {
    json!({
        "schema": CACHE_HIT_REPORT_SCHEMA,
        "adr": "ADR-0560",
        "build_class": build_class,
        "mode": mode,
        "cache_hit_rate": record.get("cache_hit_rate").and_then(Value::as_f64).unwrap_or(0.0),
        "run_action_cache_count": record_u64(record, "run_action_cache_count"),
        "run_local_count": record_u64(record, "run_local_count"),
        "run_remote_count": record_u64(record, "run_remote_count"),
        "run_skipped_count": record_u64(record, "run_skipped_count"),
        "cache_upload_attempt_count": record_u64(record, "cache_upload_attempt_count"),
        "cache_upload_count": record_u64(record, "cache_upload_count"),
        "exit_result_name": record.get("exit_result_name").and_then(Value::as_str).unwrap_or(""),
    })
}

/// Assert a build invocation succeeded and, when run in warm mode, actually
/// participated in the warm cache and observed at least one action-cache hit.
///
/// This is deliberately stricter than telemetry. A warm lane with a 0% hit rate
/// is usually an endpoint / credential / keying misconfiguration; allowing that
/// to stay green recreates the "cache exists but never hits" false-green class.
/// Bypass/cold modes still validate the recorded build result, because an error
/// record must never be reclassified as green telemetry.
pub fn assert_warm_cache_participation(
    record: &Value,
    build_class: &str,
    mode: &str,
) -> Result<(), Vec<String>> {
    let normalized = mode.trim().to_ascii_lowercase();
    let bypass_mode = matches!(normalized.as_str(), "bypass" | "cold" | "off" | "disabled");
    let warm_mode = matches!(
        normalized.as_str(),
        "warm-ro" | "warm-rw" | "warm-read-only" | "warm-read-write"
    );
    let mut findings = Vec::new();
    if !bypass_mode && !warm_mode {
        findings.push(format!(
            "cache mode `{mode}` for class `{build_class}` is not recognized — refusing to \
             infer warm-cache correctness from an unknown mode (fail closed)"
        ));
    }

    match record.get("exit_result_name").and_then(Value::as_str) {
        Some("SUCCESS") => {}
        Some(result) => findings.push(format!(
            "warm cache guard saw non-success buck2 exit_result_name={result:?} for \
             class `{build_class}` — refusing false-green telemetry"
        )),
        None => findings.push(
            "record-shape violation: exit_result_name missing from the invocation record — \
             cannot prove the guarded build succeeded (fail closed)"
                .to_string(),
        ),
    }

    let cache_hit_rate = match record.get("cache_hit_rate").and_then(Value::as_f64) {
        Some(value) if value.is_finite() && value >= 0.0 => Some(value),
        Some(value) => {
            findings.push(format!(
                "record-shape violation: cache_hit_rate={value:?} is not a finite non-negative \
                 number (fail closed)"
            ));
            None
        }
        None => {
            findings.push(
                "record-shape violation: cache_hit_rate missing from the invocation record — \
                 cannot prove warm cache hit rate (fail closed)"
                    .to_string(),
            );
            None
        }
    };
    let action_hits = required_u64(record, "run_action_cache_count", &mut findings);
    let _local_actions = required_u64(record, "run_local_count", &mut findings);
    let _remote_actions = required_u64(record, "run_remote_count", &mut findings);
    let action_cache_started =
        required_last_snapshot_u64(record, "re_action_cache_started", &mut findings);

    if warm_mode {
        if action_cache_started == Some(0) {
            findings.push(format!(
                "warm cache guard for class `{build_class}` ran in mode `{mode}` but \
                 last_snapshot.re_action_cache_started=0 — the action cache did not start"
            ));
        }
        if matches!(cache_hit_rate, Some(0.0)) {
            findings.push(format!(
                "warm cache guard for class `{build_class}` ran in mode `{mode}` with 0% hit rate \
                 — this is an obvious warm-cache misconfiguration, not a green presubmit"
            ));
        }
        if action_hits == Some(0) {
            findings.push(format!(
                "warm cache guard for class `{build_class}` ran in mode `{mode}` with \
                 run_action_cache_count=0 — no action-cache hit was observed"
            ));
        }
    }

    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings)
    }
}

/// Assert a build had ZERO cache participation (the canary's from-empty proof):
/// no action-cache hits, no remote executions, no upload attempts.
///
/// FAIL-CLOSED on shape drift: a counter that is MISSING from the record is a
/// finding, not a zero — otherwise a buck2 upgrade that renames a field would
/// silently turn the cold proof vacuous (assert-everything-absent == assert
/// nothing).
pub fn assert_cold(record: &Value) -> Result<(), Vec<String>> {
    let mut findings = Vec::new();
    for key in [
        "run_action_cache_count",
        "run_remote_count",
        "cache_upload_attempt_count",
    ] {
        match record.get(key).and_then(Value::as_u64) {
            Some(0) => {}
            Some(v) => findings.push(format!(
                "cold violation: {key}={v} (expected 0 — the canary build must not touch any cache)"
            )),
            None => findings.push(format!(
                "record-shape violation: counter `{key}` missing from the invocation record — \
                 cannot prove coldness from an unrecognized record shape (fail closed)"
            )),
        }
    }
    match record
        .pointer("/last_snapshot/re_action_cache_started")
        .and_then(Value::as_u64)
    {
        Some(0) => {}
        Some(started) => findings.push(format!(
            "cold violation: last_snapshot.re_action_cache_started={started} (expected 0)"
        )),
        None => findings.push(
            "record-shape violation: last_snapshot.re_action_cache_started missing — \
             cannot prove coldness from an unrecognized record shape (fail closed)"
                .to_string(),
        ),
    }
    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings)
    }
}

fn hash_file(hasher: &mut Sha256, path: &Path) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_le_bytes());
    hasher.update(&bytes);
    Ok(())
}

fn walk_sorted(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("read_dir {}: {e}", dir.display()))?
        .collect::<Result<_, _>>()
        .map_err(|e| format!("read_dir entry under {}: {e}", dir.display()))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            walk_sorted(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

/// SHA-256 over an output artifact: a file hashes (len || bytes); a directory
/// hashes every file in sorted relative order as (relpath || 0 || len || bytes),
/// so the digest is deterministic across hosts.
pub fn hash_output(path: &Path) -> Result<String, String> {
    let mut hasher = Sha256::new();
    if path.is_dir() {
        let mut files = Vec::new();
        walk_sorted(path, &mut files)?;
        for file in files {
            let rel = file
                .strip_prefix(path)
                .map_err(|e| format!("strip_prefix {}: {e}", file.display()))?;
            hasher.update(rel.to_string_lossy().as_bytes());
            hasher.update([0u8]);
            hash_file(&mut hasher, &file)?;
        }
    } else {
        hash_file(&mut hasher, path)?;
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Parse `buck2 build --show-full-output` lines (`<target> <path>`) into a
/// target -> output-digest manifest.
pub fn digest_manifest_from_show_output(text: &str) -> Result<BTreeMap<String, String>, String> {
    let mut entries = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((target, path)) = line.split_once(' ') else {
            return Err(format!(
                "malformed --show-full-output line (no space): {line}"
            ));
        };
        let digest = hash_output(Path::new(path.trim()))?;
        entries.insert(target.to_string(), digest);
    }
    Ok(entries)
}

pub fn manifest_to_json(entries: &BTreeMap<String, String>) -> Value {
    json!({
        "schema": DIGEST_MANIFEST_SCHEMA,
        "adr": "ADR-0560",
        "entries": entries,
    })
}

pub fn manifest_from_json(doc: &Value) -> Result<BTreeMap<String, String>, String> {
    let entries = doc
        .get("entries")
        .and_then(Value::as_object)
        .ok_or_else(|| "manifest missing `entries` object".to_string())?;
    let mut out = BTreeMap::new();
    for (k, v) in entries {
        let digest = v
            .as_str()
            .ok_or_else(|| format!("manifest entry `{k}` is not a string digest"))?;
        out.insert(k.clone(), digest.to_string());
    }
    Ok(out)
}

/// Canary verdict states. Anything other than `Green` licenses NOTHING.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanaryStatus {
    /// Every overlapping key is byte-identical and at least one key was compared.
    Green,
    /// At least one overlapping key DIVERGES: hermeticity/non-determinism defect,
    /// fail closed (ADR-0556 D2 RED response: suspend all warm reads fleet-wide).
    Red,
    /// No warm substrate endpoint is live: nothing to verify, warmth stays
    /// unlicensed (the slice-1 dark state) — explicitly NOT green.
    InactiveNoEndpoint,
    /// A warm manifest exists but shares zero keys with the cold build: nothing
    /// was verified, so GREEN is refused (an empty comparison must never license).
    UnverifiedEmptyOverlap,
}

impl CanaryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            CanaryStatus::Green => "GREEN",
            CanaryStatus::Red => "RED",
            CanaryStatus::InactiveNoEndpoint => "INACTIVE_NO_ENDPOINT",
            CanaryStatus::UnverifiedEmptyOverlap => "UNVERIFIED_EMPTY_OVERLAP",
        }
    }

    /// Process exit semantics: ONLY `Green` exits zero.
    ///
    /// The canary's exit code is what becomes the scheduled job's check
    /// conclusion, and that conclusion is a declared result channel of the
    /// workflow (ADR-0556 D4.3: "result via API/artifact, never a log someone
    /// shells in to read"). So a non-Green verdict that exits zero publishes a
    /// GREEN badge for a run that verified nothing about cache integrity —
    /// measured on run 30690156857 (2026-08-01): warm-probe SKIPPED, verdict
    /// INACTIVE_NO_ENDPOINT, job conclusion `success`.
    ///
    /// `InactiveNoEndpoint` and `UnverifiedEmptyOverlap` are the SAME condition —
    /// zero keys compared — so they get the same exit semantics. This makes the
    /// type's own contract ("anything other than `Green` licenses NOTHING")
    /// mechanically true instead of merely documented.
    pub fn is_failure(self) -> bool {
        !matches!(self, CanaryStatus::Green)
    }
}

/// Compare the cold build's digests against the warm substrate's digests for the
/// same keys (ADR-0556 D2). `warm: None` = no live endpoint (INACTIVE).
pub fn canary_verdict(
    cold: &BTreeMap<String, String>,
    warm: Option<&BTreeMap<String, String>>,
) -> (CanaryStatus, Value) {
    let Some(warm) = warm else {
        let status = CanaryStatus::InactiveNoEndpoint;
        return (
            status,
            json!({
                "schema": CANARY_VERDICT_SCHEMA,
                "adr": "ADR-0560",
                "status": status.as_str(),
                "cold_keys": cold.len(),
                "compared_keys": 0,
                "divergent_keys": [],
                "note": "no live CAS endpoint — nothing to verify; warm stays unlicensed \
                         (ADR-0556 D2 clause (b) unsatisfied); this is the slice-1 dark state",
            }),
        );
    };

    let mut divergent = Vec::new();
    let mut compared = 0usize;
    for (key, cold_digest) in cold {
        if let Some(warm_digest) = warm.get(key) {
            compared += 1;
            if warm_digest != cold_digest {
                divergent.push(json!({
                    "key": key,
                    "cold": cold_digest,
                    "warm": warm_digest,
                }));
            }
        }
    }
    let uncovered = cold.len() - compared;
    let status = if !divergent.is_empty() {
        CanaryStatus::Red
    } else if compared == 0 {
        CanaryStatus::UnverifiedEmptyOverlap
    } else {
        CanaryStatus::Green
    };
    let verdict = json!({
        "schema": CANARY_VERDICT_SCHEMA,
        "adr": "ADR-0560",
        "status": status.as_str(),
        "cold_keys": cold.len(),
        "compared_keys": compared,
        "uncovered_cold_keys": uncovered,
        "divergent_keys": divergent,
        "red_response": if status == CanaryStatus::Red {
            "ADR-0556 D2: suspend ALL warm reads fleet-wide — flip \
             specs/cache-warm-license.json warm_reads_licensed to false before any \
             remediation; evict divergent keys; open a friction-ledger row"
        } else { "" },
    });
    (status, verdict)
}

/// Minimal buckconfig (INI) reader for the conformance gate: sections of
/// `key = value` lines, `#` comments, no continuations (our overlays carry none).
pub fn parse_buckconfig(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut current = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            current = name.trim().to_string();
            sections.entry(current.clone()).or_default();
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            sections
                .entry(current.clone())
                .or_default()
                .insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    sections
}

/// Bundled canary policy DATA (pinned target set + invariants). Embedded per the
/// gate-app pattern; the file is part of the crate's srcs.
pub const CANARY_POLICY: &str = include_str!("canary-policy.json");

pub fn canary_policy() -> Result<Value, String> {
    serde_json::from_str(CANARY_POLICY)
        .map_err(|e| format!("parse bundled canary-policy.json: {e}"))
}

/// Walk up from `start` to the repo root (the standing live-corpus pattern).
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

    fn policy_fixture() -> Value {
        json!({
            "trust_invariant": { "canary_build_class": "integrity-canary" },
            "default_for_unlisted_classes": {
                "warmth": "cold", "cache_read": false, "cache_write": false
            },
            "build_classes": {
                "integrity-canary": { "warmth": "cold", "cache_read": false, "cache_write": false },
                "release-production-image": { "warmth": "cold", "cache_read": false, "cache_write": false },
                "dev-agentic-iteration": { "warmth": "warm", "cache_read": true, "cache_write": true },
                "future-read-only-class": { "warmth": "warm", "cache_read": true, "cache_write": false }
            }
        })
    }

    fn license(licensed: bool) -> Value {
        json!({ "warm_reads_licensed": licensed, "reason": "fixture" })
    }

    #[test]
    fn canary_class_always_bypasses_even_under_a_green_license() {
        let r = resolve(&policy_fixture(), &license(true), "integrity-canary").unwrap();
        assert_eq!(r.mode, CacheMode::Bypass);
    }

    #[test]
    fn canary_class_bypasses_even_if_the_data_were_tampered_warm() {
        let mut policy = policy_fixture();
        policy["build_classes"]["integrity-canary"] =
            json!({ "warmth": "warm", "cache_read": true, "cache_write": true });
        let r = resolve(&policy, &license(true), "integrity-canary").unwrap();
        assert_eq!(
            r.mode,
            CacheMode::Bypass,
            "the trust anchor must win over tampered data"
        );
    }

    #[test]
    fn unlisted_class_is_fail_closed_bypass() {
        let r = resolve(&policy_fixture(), &license(true), "no-such-class").unwrap();
        assert_eq!(r.mode, CacheMode::Bypass);
    }

    #[test]
    fn cold_class_bypasses_under_a_green_license() {
        let r = resolve(
            &policy_fixture(),
            &license(true),
            "release-production-image",
        )
        .unwrap();
        assert_eq!(r.mode, CacheMode::Bypass);
    }

    #[test]
    fn warm_class_is_suspended_while_the_kill_switch_is_false() {
        let r = resolve(&policy_fixture(), &license(false), "dev-agentic-iteration").unwrap();
        assert_eq!(r.mode, CacheMode::Bypass);
        assert!(
            r.reasons
                .iter()
                .any(|m| m.contains("warm_reads_licensed=false"))
        );
    }

    #[test]
    fn warm_class_resolves_rw_only_when_licensed() {
        let r = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        assert_eq!(r.mode, CacheMode::WarmReadWrite);
    }

    #[test]
    fn warm_read_only_class_resolves_ro() {
        let r = resolve(&policy_fixture(), &license(true), "future-read-only-class").unwrap();
        assert_eq!(r.mode, CacheMode::WarmReadOnly);
    }

    #[test]
    fn malformed_license_is_a_loud_error_not_a_grant() {
        let err = resolve(
            &policy_fixture(),
            &json!({ "warm_reads_licensed": "yes" }),
            "dev-agentic-iteration",
        )
        .unwrap_err();
        assert!(err.contains("warm_reads_licensed"));
    }

    #[test]
    fn bypass_materializes_no_local_config() {
        let r = resolve(&policy_fixture(), &license(false), "dev-agentic-iteration").unwrap();
        assert_eq!(effective_buckconfig(&r, "", None, None).unwrap(), None);
    }

    #[test]
    fn warm_emission_without_a_keyed_identity_is_a_hard_error() {
        let r = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        let err = effective_buckconfig(&r, "", None, None).unwrap_err();
        assert!(err.contains(CLIENT_CERT_ENV));
    }

    #[test]
    fn warm_identity_paths_must_be_absolute() {
        let r = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        let err = effective_buckconfig(
            &r,
            "[buck2_re_client]\ntls = true\n",
            Some("relative/client.pem"),
            None,
        )
        .unwrap_err();
        assert!(err.contains("absolute"));
    }

    #[test]
    fn warm_rw_effective_config_selects_the_rw_overlay_and_carries_the_identity() {
        let r = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        let config = effective_buckconfig(
            &r,
            "[buck2_re_client]\ntls = true\n",
            Some("/secrets/writer.pem"),
            Some("/secrets/ca.pem"),
        )
        .unwrap()
        .expect("warm config");
        assert!(config.contains("tls_client_cert = /secrets/writer.pem"));
        assert!(config.contains("tls_ca_certs = /secrets/ca.pem"));
        assert!(!config.contains("--config"));
    }

    #[test]
    fn private_local_config_is_mode_0600_and_removed() {
        let root = std::env::temp_dir().join(format!("oya-cache-config-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = install_local_buckconfig(&root, "[buck2_re_client]\ntls = true\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        remove_local_buckconfig(&path).unwrap();
        assert!(!path.exists());
        fs::remove_dir_all(root).unwrap();
    }

    fn record_fixture(action_cache: u64, remote: u64, uploads: u64) -> Value {
        let cache_hit_rate = if action_cache == 0 { 0.0 } else { 0.5 };
        json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "cache_hit_rate": cache_hit_rate,
                "run_action_cache_count": action_cache,
                "run_local_count": 7,
                "run_remote_count": remote,
                "run_skipped_count": 1,
                "cache_upload_attempt_count": uploads,
                "cache_upload_count": uploads,
                "exit_result_name": "SUCCESS",
                "last_snapshot": { "re_action_cache_started": action_cache }
            } } } }
        })
    }

    #[test]
    fn cache_hit_report_extracts_structured_counters() {
        let doc = record_fixture(3, 0, 2);
        let record = invocation_record(&doc).unwrap();
        let report = cache_hit_report(record, "dev-agentic-iteration", "warm-rw");
        assert_eq!(report["schema"], CACHE_HIT_REPORT_SCHEMA);
        assert_eq!(report["run_action_cache_count"], 3);
        assert_eq!(report["cache_upload_count"], 2);
        assert_eq!(report["build_class"], "dev-agentic-iteration");
    }

    #[test]
    fn assert_cold_is_green_on_a_zero_participation_record() {
        let doc = record_fixture(0, 0, 0);
        assert!(assert_cold(invocation_record(&doc).unwrap()).is_ok());
    }

    #[test]
    fn assert_warm_allows_explicit_bypass_mode() {
        let doc = record_fixture(0, 0, 0);
        assert!(
            assert_warm_cache_participation(
                invocation_record(&doc).unwrap(),
                "gate-fleet-shared-graph",
                "bypass"
            )
            .is_ok()
        );
    }

    #[test]
    fn assert_warm_still_fails_on_bypass_error_record() {
        let doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "exit_result_name": "FAILURE"
            } } } }
        });
        let findings = assert_warm_cache_participation(
            invocation_record(&doc).unwrap(),
            "gate-fleet-shared-graph",
            "bypass",
        )
        .unwrap_err();
        assert!(findings.iter().any(|f| f.contains("non-success")));
    }

    #[test]
    fn assert_warm_accepts_positive_cache_hits() {
        let doc = record_fixture(3, 0, 0);
        assert!(
            assert_warm_cache_participation(
                invocation_record(&doc).unwrap(),
                "gate-fleet-shared-graph",
                "warm-rw"
            )
            .is_ok()
        );
    }

    #[test]
    fn assert_warm_fails_on_zero_hit_warm_mode() {
        let doc = record_fixture(0, 0, 0);
        let findings = assert_warm_cache_participation(
            invocation_record(&doc).unwrap(),
            "gate-fleet-shared-graph",
            "warm-rw",
        )
        .unwrap_err();
        assert!(findings.iter().any(|f| f.contains("0% hit rate")));
        assert!(findings.iter().any(|f| f.contains("did not start")));
    }

    #[test]
    fn assert_warm_rejects_zero_rate_even_with_positive_cache_count() {
        let doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "cache_hit_rate": 0.0,
                "run_action_cache_count": 3,
                "run_local_count": 4,
                "run_remote_count": 0,
                "run_skipped_count": 1,
                "cache_upload_attempt_count": 0,
                "cache_upload_count": 0,
                "exit_result_name": "SUCCESS",
                "last_snapshot": { "re_action_cache_started": 3 }
            } } } }
        });
        let findings = assert_warm_cache_participation(
            invocation_record(&doc).unwrap(),
            "gate-fleet-shared-graph",
            "warm-rw",
        )
        .unwrap_err();
        assert!(findings.iter().any(|f| f.contains("0% hit rate")));
    }

    #[test]
    fn assert_warm_rejects_positive_rate_with_zero_cache_count() {
        let doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "cache_hit_rate": 0.5,
                "run_action_cache_count": 0,
                "run_local_count": 4,
                "run_remote_count": 0,
                "run_skipped_count": 1,
                "cache_upload_attempt_count": 0,
                "cache_upload_count": 0,
                "exit_result_name": "SUCCESS",
                "last_snapshot": { "re_action_cache_started": 1 }
            } } } }
        });
        let findings = assert_warm_cache_participation(
            invocation_record(&doc).unwrap(),
            "gate-fleet-shared-graph",
            "warm-rw",
        )
        .unwrap_err();
        assert!(
            findings
                .iter()
                .any(|f| f.contains("run_action_cache_count=0"))
        );
    }

    #[test]
    fn assert_warm_fails_closed_on_error_or_missing_shape() {
        let error_doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "cache_hit_rate": 1.0,
                "run_action_cache_count": 1,
                "run_local_count": 0,
                "run_remote_count": 0,
                "exit_result_name": "FAILURE",
                "last_snapshot": { "re_action_cache_started": 1 }
            } } } }
        });
        let findings = assert_warm_cache_participation(
            invocation_record(&error_doc).unwrap(),
            "gate-fleet-shared-graph",
            "warm-rw",
        )
        .unwrap_err();
        assert!(findings.iter().any(|f| f.contains("non-success")));

        let missing_doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "exit_result_name": "SUCCESS"
            } } } }
        });
        let findings = assert_warm_cache_participation(
            invocation_record(&missing_doc).unwrap(),
            "gate-fleet-shared-graph",
            "warm-rw",
        )
        .unwrap_err();
        assert!(
            findings
                .iter()
                .any(|f| f.contains("record-shape violation"))
        );
    }

    #[test]
    fn assert_cold_fails_closed_on_any_cache_participation() {
        for doc in [
            record_fixture(1, 0, 0),
            record_fixture(0, 1, 0),
            record_fixture(0, 0, 1),
        ] {
            let findings = assert_cold(invocation_record(&doc).unwrap()).unwrap_err();
            assert!(!findings.is_empty());
        }
    }

    #[test]
    fn assert_cold_fails_closed_on_an_unrecognized_record_shape() {
        // A record with the counters RENAMED/missing must not pass: absence is a
        // shape violation, never an implicit zero.
        let doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "exit_result_name": "SUCCESS"
            } } } }
        });
        let findings = assert_cold(invocation_record(&doc).unwrap()).unwrap_err();
        assert!(
            findings
                .iter()
                .any(|f| f.contains("record-shape violation"))
        );
    }

    fn manifest(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn verdict_inactive_when_no_endpoint() {
        let (status, v) = canary_verdict(&manifest(&[("//a:a", "1")]), None);
        assert_eq!(status, CanaryStatus::InactiveNoEndpoint);
        assert_eq!(v["compared_keys"], 0);
        // RED fixture for the false-green channel: INACTIVE compared ZERO keys, so it must
        // NOT exit zero. Exit code is the scheduled job's check conclusion, and a `success`
        // conclusion on a run that verified nothing publishes a daily green badge for an
        // unestablished trust anchor (observed: run 30690156857, 2026-08-01 — warm-probe
        // skipped, verdict INACTIVE_NO_ENDPOINT, conclusion `success`). Reverting
        // `is_failure` to exclude InactiveNoEndpoint fails here.
        assert!(
            status.is_failure(),
            "INACTIVE_NO_ENDPOINT compared 0 keys; exiting zero would publish a green check \
             conclusion for a run that verified no cache integrity"
        );
    }

    #[test]
    fn only_green_exits_zero() {
        // The type's contract is "anything other than Green licenses NOTHING". Assert it over
        // EVERY variant so a new status added later cannot default into the passing arm.
        for status in [
            CanaryStatus::Red,
            CanaryStatus::InactiveNoEndpoint,
            CanaryStatus::UnverifiedEmptyOverlap,
        ] {
            assert!(
                status.is_failure(),
                "{} licenses nothing, so it must not exit zero",
                status.as_str()
            );
        }
        assert!(!CanaryStatus::Green.is_failure());
    }

    #[test]
    fn verdict_green_requires_at_least_one_compared_identical_key() {
        let cold = manifest(&[("//a:a", "1"), ("//b:b", "2")]);
        let warm = manifest(&[("//a:a", "1")]);
        let (status, v) = canary_verdict(&cold, Some(&warm));
        assert_eq!(status, CanaryStatus::Green);
        assert_eq!(v["compared_keys"], 1);
        assert_eq!(v["uncovered_cold_keys"], 1);
    }

    #[test]
    fn verdict_red_on_any_divergent_key() {
        let cold = manifest(&[("//a:a", "1")]);
        let warm = manifest(&[("//a:a", "DIFFERENT")]);
        let (status, v) = canary_verdict(&cold, Some(&warm));
        assert_eq!(status, CanaryStatus::Red);
        assert!(status.is_failure());
        assert_eq!(v["divergent_keys"].as_array().unwrap().len(), 1);
        assert!(
            v["red_response"]
                .as_str()
                .unwrap()
                .contains("suspend ALL warm reads")
        );
    }

    #[test]
    fn verdict_refuses_green_on_empty_overlap() {
        let cold = manifest(&[("//a:a", "1")]);
        let warm = manifest(&[("//z:z", "9")]);
        let (status, _) = canary_verdict(&cold, Some(&warm));
        assert_eq!(status, CanaryStatus::UnverifiedEmptyOverlap);
        assert!(status.is_failure());
    }

    #[test]
    fn digest_manifest_hashes_files_deterministically() {
        let dir =
            std::env::temp_dir().join(format!("oya-cache-wiring-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("out.bin");
        std::fs::write(&file, b"payload").unwrap();
        let text = format!("//x:x {}\n", file.display());
        let a = digest_manifest_from_show_output(&text).unwrap();
        let b = digest_manifest_from_show_output(&text).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn parse_buckconfig_reads_sections_and_keys() {
        let cfg = parse_buckconfig("# c\n[a]\n  k = v\n[b]\nx=y\n");
        assert_eq!(cfg["a"]["k"], "v");
        assert_eq!(cfg["b"]["x"], "y");
    }

    #[test]
    fn bundled_canary_policy_parses_and_pins_targets() {
        let policy = canary_policy().unwrap();
        assert_eq!(policy["build_class"], "integrity-canary");
        assert!(!policy["pinned_targets"].as_array().unwrap().is_empty());
    }
}
