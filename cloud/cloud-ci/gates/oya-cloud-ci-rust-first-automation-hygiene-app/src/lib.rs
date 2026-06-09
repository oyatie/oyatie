//! # cloud-ci-rust-first-automation-hygiene
//!
//! Buck2-native gate for the Rust-first automation ratchet. The gate is productized policy,
//! not Oyatie-only shell glue: repository-specific roots/exceptions live in DATA, while this
//! crate evaluates the portable contract that non-Rust automation is either absent or has a
//! documented Rust/Buck2/cloud-native replacement path.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

pub const GATE_ID: &str = "cloud-ci-rust-first-automation-hygiene";

pub const VIOLATION_CODES: [&str; 7] = [
    "rust_first_automation_gate_id_mismatch",
    "rust_first_automation_exception_duplicate",
    "rust_first_automation_exception_missing_field",
    "rust_first_automation_exception_missing_replacement_contract",
    "rust_first_automation_exception_stale",
    "rust_first_automation_observed_path_missing_field",
    "rust_first_automation_unregistered_non_rust_automation",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanError {
    MissingScanArray(String),
    Io(String),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::MissingScanArray(field) => write!(f, "policy scan.{field} must be an array"),
            ScanError::Io(message) => write!(f, "automation hygiene scan io: {message}"),
        }
    }
}

impl std::error::Error for ScanError {}

fn scan_string_array(policy: &Value, key: &str) -> Result<Vec<String>, ScanError> {
    policy
        .get("scan")
        .and_then(|scan| scan.get(key))
        .and_then(Value::as_array)
        .ok_or_else(|| ScanError::MissingScanArray(key.to_owned()))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| ScanError::MissingScanArray(key.to_owned()))
        })
        .collect()
}

fn path_is_excluded(path: &str, prefixes: &[String]) -> bool {
    prefixes
        .iter()
        .any(|prefix| path == prefix.trim_end_matches('/') || path.starts_with(prefix))
}

fn has_non_rust_extension(path: &Path, extensions: &BTreeSet<String>) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&format!(".{ext}")))
        .unwrap_or(false)
}

fn has_shebang(path: &Path) -> Result<bool, ScanError> {
    let mut file = fs::File::open(path).map_err(|e| {
        ScanError::Io(format!(
            "open {} for shebang detection: {e}",
            path.display()
        ))
    })?;
    let mut prefix = [0_u8; 2];
    let read = file.read(&mut prefix).map_err(|e| {
        ScanError::Io(format!(
            "read {} for shebang detection: {e}",
            path.display()
        ))
    })?;
    Ok(read == 2 && prefix == *b"#!")
}

fn sorted_dir_entries(dir: &Path) -> Result<Vec<PathBuf>, ScanError> {
    let mut entries = fs::read_dir(dir)
        .map_err(|e| ScanError::Io(format!("read_dir {}: {e}", dir.display())))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|e| ScanError::Io(format!("read_dir entry {}: {e}", dir.display())))
        })
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    Ok(entries)
}

fn visit_scan_root(
    repo_root: &Path,
    dir: &Path,
    exclude_prefixes: &[String],
    extensions: &BTreeSet<String>,
    rows: &mut Vec<Value>,
) -> Result<(), ScanError> {
    for path in sorted_dir_entries(dir)? {
        let rel = path
            .strip_prefix(repo_root)
            .map_err(|e| ScanError::Io(format!("strip repo root from {}: {e}", path.display())))?
            .to_string_lossy()
            .replace('\\', "/");
        if path_is_excluded(&rel, exclude_prefixes) {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|e| {
            ScanError::Io(format!(
                "symlink_metadata {} during automation scan: {e}",
                path.display()
            ))
        })?;
        if metadata.file_type().is_symlink() {
            // CI hygiene: do not double-count interop symlink surfaces. The target is
            // scanned once through its canonical path; symlink aliases add noise and
            // stale-policy churn.
            continue;
        }
        if metadata.is_dir() {
            visit_scan_root(repo_root, &path, exclude_prefixes, extensions, rows)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }

        let extension_hit = has_non_rust_extension(&path, extensions);
        let shebang_hit =
            has_shebang(&path)? && path.extension().and_then(|e| e.to_str()) != Some("rs");
        if extension_hit || shebang_hit {
            rows.push(json!({
                "path": rel,
                "detected_by": if extension_hit { "extension" } else { "shebang" }
            }));
        }
    }
    Ok(())
}

/// Collect the repo-local non-Rust automation inventory described by the policy's `scan` block.
/// This helper is intentionally reusable by CI, tests, and a future cloud-native controller. It
/// is read-only and writes no temporary files, so each run cleans up after itself by construction.
pub fn collect_observed_non_rust_automation(
    repo_root: &Path,
    policy: &Value,
) -> Result<Value, ScanError> {
    let scan_roots = scan_string_array(policy, "roots")?;
    let exclude_prefixes = scan_string_array(policy, "exclude_prefixes")?;
    let extensions = scan_string_array(policy, "non_rust_extensions")?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    for scan_root in scan_roots {
        let absolute = repo_root.join(&scan_root);
        if absolute.exists() {
            visit_scan_root(
                repo_root,
                &absolute,
                &exclude_prefixes,
                &extensions,
                &mut rows,
            )?;
        }
    }
    rows.sort_by(|a, b| string_field(a, "path").cmp(&string_field(b, "path")));
    Ok(json!({ "rows": rows }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect::<BTreeSet<_>>();
        Self {
            verdict: if violations.is_empty() {
                Verdict::Green
            } else {
                Verdict::Red
            },
            violations,
        }
    }
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).map(str::trim)
}

fn required_nonblank<'a>(
    object: &'a Value,
    field: &str,
    exception_path: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<&'a str> {
    match string_field(object, field).filter(|v| !v.is_empty()) {
        Some(value) => Some(value),
        None => {
            findings.insert(Finding::new(
                "rust_first_automation_exception_missing_field",
                exception_path,
                format!("exception missing non-empty `{field}`"),
            ));
            None
        }
    }
}

fn replacement_is_cloud_native_rust_contract(replacement: &str) -> bool {
    let normalized = replacement.to_ascii_lowercase();
    let names_rust = normalized.contains("rust");
    let names_execution_contract = ["buck2", "cloud", "kubernetes", "gitops", "controller"]
        .iter()
        .any(|needle| normalized.contains(needle));
    names_rust && names_execution_contract
}

fn exception_map(policy: &Value, findings: &mut BTreeSet<Finding>) -> BTreeMap<String, Value> {
    let mut exceptions = BTreeMap::new();
    for exception in policy
        .get("exceptions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(path) = string_field(exception, "path").filter(|p| !p.is_empty()) else {
            findings.insert(Finding::new(
                "rust_first_automation_exception_missing_field",
                "<missing-path>",
                "exception missing non-empty `path`",
            ));
            continue;
        };

        if exceptions.contains_key(path) {
            findings.insert(Finding::new(
                "rust_first_automation_exception_duplicate",
                path,
                "duplicate exception path",
            ));
        }

        let reason = required_nonblank(exception, "reason", path, findings);
        let replacement = required_nonblank(exception, "replacement", path, findings);
        let status = required_nonblank(exception, "status", path, findings);

        if let Some(replacement) = replacement {
            if !replacement_is_cloud_native_rust_contract(replacement) {
                findings.insert(Finding::new(
                    "rust_first_automation_exception_missing_replacement_contract",
                    path,
                    "replacement must name a Rust plus Buck2/cloud/Kubernetes/GitOps/controller path",
                ));
            }
        }
        if let Some(reason) = reason {
            if reason.len() < 24 {
                findings.insert(Finding::new(
                    "rust_first_automation_exception_missing_field",
                    path,
                    "exception reason is too short to justify a non-Rust automation surface",
                ));
            }
        }
        if let Some(status) = status {
            if !matches!(
                status,
                "temporary_legacy_bridge" | "portable_declarative_bridge"
            ) {
                findings.insert(Finding::new(
                    "rust_first_automation_exception_missing_field",
                    path,
                    "status must be temporary_legacy_bridge or portable_declarative_bridge",
                ));
            }
        }

        exceptions.insert(path.to_owned(), exception.clone());
    }
    exceptions
}

fn observed_paths(observed: &Value, findings: &mut BTreeSet<Finding>) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    for row in observed
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        match string_field(row, "path").filter(|p| !p.is_empty()) {
            Some(path) => {
                paths.insert(path.to_owned());
            }
            None => {
                findings.insert(Finding::new(
                    "rust_first_automation_observed_path_missing_field",
                    "<observed-row>",
                    "observed non-Rust automation row missing non-empty `path`",
                ));
            }
        }
    }
    paths
}

/// Pure evaluator. `policy` is DATA (`rust-first-automation-policy.json`); `observed` is the scanner
/// output shaped as `{ "rows": [{"path": "..."}] }`.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if string_field(policy, "gate_id") != Some(GATE_ID) {
        findings.insert(Finding::new(
            "rust_first_automation_gate_id_mismatch",
            "<policy>",
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let exceptions = exception_map(policy, &mut findings);
    let observed = observed_paths(observed, &mut findings);

    for path in &observed {
        if !exceptions.contains_key(path) {
            findings.insert(Finding::new(
                "rust_first_automation_unregistered_non_rust_automation",
                path,
                "non-Rust automation path needs an explicit exception or Rust migration",
            ));
        }
    }

    for path in exceptions.keys() {
        if !observed.contains(path) {
            findings.insert(Finding::new(
                "rust_first_automation_exception_stale",
                path,
                "exception path no longer exists as non-Rust automation; remove the exception",
            ));
        }
    }

    findings
}

pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy(paths: &[&str]) -> Value {
        json!({
            "gate_id": GATE_ID,
            "exceptions": paths.iter().map(|path| json!({
                "path": path,
                "status": "temporary_legacy_bridge",
                "reason": "legacy non-Rust automation is temporarily admitted only as explicit shrinkable debt",
                "replacement": "replace with Rust Buck2 cloud-ci gate or Kubernetes-native controller contract"
            })).collect::<Vec<_>>()
        })
    }

    fn observed(paths: &[&str]) -> Value {
        json!({"rows": paths.iter().map(|path| json!({"path": path})).collect::<Vec<_>>()})
    }

    #[test]
    fn registered_legacy_paths_are_green() {
        let report = evaluate(
            &policy(&["scripts/legacy.sh"]),
            &observed(&["scripts/legacy.sh"]),
        );
        assert_eq!(report.verdict, Verdict::Green);
    }

    #[test]
    fn new_unregistered_non_rust_automation_is_red() {
        let findings = evaluate_keyed(&policy(&[]), &observed(&["scripts/new.sh"]));
        assert!(findings.iter().any(|finding| {
            finding.code == "rust_first_automation_unregistered_non_rust_automation"
                && finding.key == "scripts/new.sh"
        }));
    }

    #[test]
    fn stale_exception_is_red_to_force_shrinkage() {
        let findings = evaluate_keyed(&policy(&["scripts/retired.sh"]), &observed(&[]));
        assert!(findings.iter().any(|finding| {
            finding.code == "rust_first_automation_exception_stale"
                && finding.key == "scripts/retired.sh"
        }));
    }

    #[test]
    fn weak_exception_without_rust_cloud_native_replacement_is_red() {
        let weak = json!({
            "gate_id": GATE_ID,
            "exceptions": [{
                "path": "scripts/legacy.sh",
                "status": "temporary_legacy_bridge",
                "reason": "legacy non-Rust automation is temporarily admitted only as explicit shrinkable debt",
                "replacement": "keep shell"
            }]
        });
        let findings = evaluate_keyed(&weak, &observed(&["scripts/legacy.sh"]));
        assert!(findings.iter().any(|finding| {
            finding.code == "rust_first_automation_exception_missing_replacement_contract"
                && finding.key == "scripts/legacy.sh"
        }));
    }
}
