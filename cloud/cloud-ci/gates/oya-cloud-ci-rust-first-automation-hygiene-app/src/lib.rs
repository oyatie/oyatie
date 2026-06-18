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
use serde_yaml::Value as YamlValue;

pub const GATE_ID: &str = "cloud-ci-rust-first-automation-hygiene";

pub const VIOLATION_CODES: [&str; 9] = [
    "rust_first_automation_gate_id_mismatch",
    "rust_first_automation_exception_duplicate",
    "rust_first_automation_exception_missing_field",
    "rust_first_automation_exception_missing_replacement_contract",
    "rust_first_automation_exception_stale",
    "rust_first_automation_observed_path_missing_field",
    "rust_first_automation_unregistered_non_rust_automation",
    // Workflow-inline-shell dimension (pipeline-glue(a)): the extension-based file scan above is
    // blind to shell that lives INSIDE GitHub workflow YAML `run:` steps (the file is a `.yml`,
    // not a `.sh`). These two codes ratchet that surface shrink-only against a frozen keyed
    // baseline of today's accepted legacy-bridge inline shell.
    "rust_first_automation_unbaselined_workflow_inline_shell",
    "rust_first_automation_workflow_inline_shell_baseline_stale",
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

// ───────────────────────────── workflow-inline-shell dimension ─────────────────────────────
//
// The extension-based file scan above counts non-Rust automation FILES. It is structurally blind
// to shell that lives INSIDE a `.yml` workflow as a `run:` step: the file is a `.yml`, so the
// extension scan never flags it, and the gate never parses the YAML body. pipeline-glue(a) closes
// that blind spot by parsing the policy-declared workflow globs with a real YAML parser and
// emitting one keyed observation per (file, job, step) that carries an inline `run:` block. The
// keyed observations are ratcheted shrink-only against a FROZEN baseline (policy-as-data, born
// pack-shaped) of today's accepted legacy-bridge inline shell.

/// The stable key for a single inline-shell observation: `<workflow-relpath>::<job-id>::step-<N>`
/// where `N` is the 0-based index of the step within its job's `steps` array. This is the finest
/// defensible keyed unit (per file, per job, per step) so shrink is provable per-step and the
/// total-accounting/ratchet machinery applies uniformly.
fn workflow_shell_key(rel: &str, job: &str, step_index: usize) -> String {
    format!("{rel}::{job}::step-{step_index}")
}

/// Count the inline-shell lines in a `run:` scalar. Block scalars (`run: |` / `run: >`) and
/// single-line `run:` are both deserialized by serde_yaml into a plain string, so this is a simple
/// non-empty-line count over the already-parsed value (no regex, no manual block-scalar handling).
fn shell_line_count(run: &str) -> usize {
    run.lines().filter(|line| !line.trim().is_empty()).count()
}

/// Glob-free workflow file collector: the policy declares explicit directories + extensions for the
/// workflow surface (`scan.workflow_inline_shell.{roots,extensions}`); we walk each root and keep
/// files whose extension is in the set. Read-only, deterministic (sorted), no temp files.
fn collect_workflow_files(
    repo_root: &Path,
    roots: &[String],
    extensions: &BTreeSet<String>,
) -> Result<Vec<(PathBuf, String)>, ScanError> {
    let mut files = Vec::new();
    for root in roots {
        let absolute = repo_root.join(root);
        if !absolute.exists() {
            continue;
        }
        for path in sorted_dir_entries(&absolute)? {
            let metadata = fs::symlink_metadata(&path).map_err(|e| {
                ScanError::Io(format!(
                    "symlink_metadata {} during workflow scan: {e}",
                    path.display()
                ))
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                continue;
            }
            if !has_non_rust_extension(&path, extensions) {
                continue;
            }
            let rel = path
                .strip_prefix(repo_root)
                .map_err(|e| {
                    ScanError::Io(format!("strip repo root from {}: {e}", path.display()))
                })?
                .to_string_lossy()
                .replace('\\', "/");
            files.push((path, rel));
        }
    }
    files.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(files)
}

/// Extract every inline-shell step from a parsed workflow document. The shape is the GitHub Actions
/// contract: `jobs.<job-id>.steps[]`, each step optionally carrying a string `run:`. Composite
/// actions (`runs.steps[]`) are handled by the same traversal when the document carries a top-level
/// `runs` mapping (`action.yml`). Steps without a `run:` (e.g. `uses:` action steps) are skipped.
fn extract_inline_shell_steps(rel: &str, doc: &YamlValue, rows: &mut Vec<Value>) {
    let mut record_steps = |job: &str, steps: &YamlValue| {
        let Some(steps) = steps.as_sequence() else {
            return;
        };
        for (index, step) in steps.iter().enumerate() {
            let Some(run) = step.get("run").and_then(YamlValue::as_str) else {
                continue;
            };
            rows.push(json!({
                "key": workflow_shell_key(rel, job, index),
                "file": rel,
                "job": job,
                "step_index": index,
                "shell_lines": shell_line_count(run),
            }));
        }
    };

    if let Some(jobs) = doc.get("jobs").and_then(YamlValue::as_mapping) {
        for (job_id, job) in jobs {
            let job_name = job_id.as_str().unwrap_or("<job>");
            if let Some(steps) = job.get("steps") {
                record_steps(job_name, steps);
            }
        }
    }

    // Composite action surface: `runs.steps[]` under a single synthetic `runs` job key.
    if let Some(steps) = doc.get("runs").and_then(|runs| runs.get("steps")) {
        record_steps("runs", steps);
    }
}

/// Collect the repo-local workflow inline-shell inventory described by the policy's
/// `scan.workflow_inline_shell` block. Read-only, writes no temp files, deterministic order. When
/// the dimension is disabled (or the block is absent) this returns an empty inventory so the gate
/// stays a strict superset of its prior behavior.
pub fn collect_observed_workflow_inline_shell(
    repo_root: &Path,
    policy: &Value,
) -> Result<Value, ScanError> {
    let block = policy.get("scan").and_then(|scan| scan.get("workflow_inline_shell"));
    let enabled = block
        .and_then(|block| block.get("enabled"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !enabled {
        return Ok(json!({ "steps": [] }));
    }

    let roots = workflow_string_array(block, "roots")?;
    let extensions = workflow_string_array(block, "extensions")?
        .into_iter()
        .collect::<BTreeSet<_>>();

    let mut rows = Vec::new();
    for (path, rel) in collect_workflow_files(repo_root, &roots, &extensions)? {
        let text = fs::read_to_string(&path)
            .map_err(|e| ScanError::Io(format!("read workflow {}: {e}", path.display())))?;
        let doc: YamlValue = serde_yaml::from_str(&text)
            .map_err(|e| ScanError::Io(format!("parse workflow yaml {}: {e}", path.display())))?;
        extract_inline_shell_steps(&rel, &doc, &mut rows);
    }
    rows.sort_by(|a, b| string_field(a, "key").cmp(&string_field(b, "key")));
    Ok(json!({ "steps": rows }))
}

fn workflow_string_array(block: Option<&Value>, key: &str) -> Result<Vec<String>, ScanError> {
    block
        .and_then(|block| block.get(key))
        .and_then(Value::as_array)
        .ok_or_else(|| ScanError::MissingScanArray(format!("workflow_inline_shell.{key}")))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| ScanError::MissingScanArray(format!("workflow_inline_shell.{key}")))
        })
        .collect()
}

/// The set of inline-shell keys observed in the live workflow corpus.
fn observed_workflow_shell_keys(observed: &Value, findings: &mut BTreeSet<Finding>) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for row in observed
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        match string_field(row, "key").filter(|k| !k.is_empty()) {
            Some(key) => {
                keys.insert(key.to_owned());
            }
            None => {
                findings.insert(Finding::new(
                    "rust_first_automation_observed_path_missing_field",
                    "<observed-workflow-step>",
                    "observed workflow inline-shell row missing non-empty `key`",
                ));
            }
        }
    }
    keys
}

/// The frozen baseline keys for the workflow-inline-shell code, read from the baseline face's
/// `codes.rust_first_automation_unbaselined_workflow_inline_shell` array.
fn baseline_workflow_shell_keys(baseline: &Value) -> BTreeSet<String> {
    baseline
        .get("codes")
        .and_then(|codes| codes.get("rust_first_automation_unbaselined_workflow_inline_shell"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Pure evaluator for the workflow-inline-shell dimension. `observed` is the scanner output shaped
/// as `{ "steps": [{"key": "..."}] }`; `baseline` is the frozen keyed baseline face. Shrink-only:
///   - any observed key NOT in the baseline ⇒ `rust_first_automation_unbaselined_workflow_inline_shell`
///     (a NEW inline-shell step beyond the accepted legacy-bridge debt is born-blocking);
///   - any baseline key NOT observed ⇒ `rust_first_automation_workflow_inline_shell_baseline_stale`
///     (a retired step must shrink the baseline in the same PR, mirroring the file-scan stale code).
pub fn evaluate_workflow_inline_shell_keyed(
    observed: &Value,
    baseline: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let baseline_keys = baseline_workflow_shell_keys(baseline);
    let observed_keys = observed_workflow_shell_keys(observed, &mut findings);

    for key in &observed_keys {
        if !baseline_keys.contains(key) {
            findings.insert(Finding::new(
                "rust_first_automation_unbaselined_workflow_inline_shell",
                key,
                "new inline shell inside workflow YAML beyond the frozen legacy-bridge baseline; \
                 productize it as a Rust/Buck2 step or extend the reviewed baseline (shrink-only)",
            ));
        }
    }

    for key in &baseline_keys {
        if !observed_keys.contains(key) {
            findings.insert(Finding::new(
                "rust_first_automation_workflow_inline_shell_baseline_stale",
                key,
                "baselined workflow inline-shell step no longer exists; shrink the baseline in this PR",
            ));
        }
    }

    findings
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

    // ───────────────── workflow-inline-shell evaluator unit tests (pipeline-glue(a)) ─────────────

    fn shell_observed(keys: &[&str]) -> Value {
        json!({"steps": keys.iter().map(|k| json!({"key": k})).collect::<Vec<_>>()})
    }

    fn shell_baseline(keys: &[&str]) -> Value {
        json!({"codes": {
            "rust_first_automation_unbaselined_workflow_inline_shell":
                keys.iter().map(|k| json!(k)).collect::<Vec<_>>()
        }})
    }

    #[test]
    fn workflow_shell_corpus_equal_to_baseline_is_green() {
        let findings = evaluate_workflow_inline_shell_keyed(
            &shell_observed(&["a.yml::job::step-0", "a.yml::job::step-1"]),
            &shell_baseline(&["a.yml::job::step-0", "a.yml::job::step-1"]),
        );
        assert!(findings.is_empty(), "{findings:#?}");
    }

    #[test]
    fn workflow_shell_new_key_beyond_baseline_is_red() {
        let findings = evaluate_workflow_inline_shell_keyed(
            &shell_observed(&["a.yml::job::step-0", "a.yml::job::step-NEW"]),
            &shell_baseline(&["a.yml::job::step-0"]),
        );
        assert!(findings.iter().any(|f| {
            f.code == "rust_first_automation_unbaselined_workflow_inline_shell"
                && f.key == "a.yml::job::step-NEW"
        }));
    }

    #[test]
    fn workflow_shell_retired_baselined_key_is_stale() {
        let findings = evaluate_workflow_inline_shell_keyed(
            &shell_observed(&[]),
            &shell_baseline(&["a.yml::job::step-0"]),
        );
        assert!(findings.iter().any(|f| {
            f.code == "rust_first_automation_workflow_inline_shell_baseline_stale"
                && f.key == "a.yml::job::step-0"
        }));
    }

    #[test]
    fn shell_line_count_handles_block_and_single_line() {
        assert_eq!(shell_line_count("echo hi"), 1);
        assert_eq!(shell_line_count("set -e\n\necho a\necho b\n"), 3);
    }

    #[test]
    fn extract_inline_shell_steps_skips_uses_and_keys_per_step() {
        let doc: YamlValue = serde_yaml::from_str(
            "jobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n      - run: echo one\n      - run: |\n          echo two\n          echo three\n",
        )
        .unwrap();
        let mut rows = Vec::new();
        extract_inline_shell_steps(".github/workflows/x.yml", &doc, &mut rows);
        let keys: Vec<&str> = rows.iter().filter_map(|r| r["key"].as_str()).collect();
        // step-0 is the `uses:` step (no run) → skipped; step-1 and step-2 are run steps.
        assert_eq!(
            keys,
            vec![
                ".github/workflows/x.yml::build::step-1",
                ".github/workflows/x.yml::build::step-2"
            ]
        );
        assert_eq!(rows[1]["shell_lines"].as_u64(), Some(2));
    }

    #[test]
    fn evaluator_only_emits_declared_violation_codes() {
        // Guard against the workflow-shell + file-scan evaluators drifting from VIOLATION_CODES.
        let declared: BTreeSet<&str> = VIOLATION_CODES.into_iter().collect();
        let mut emitted: BTreeSet<String> = BTreeSet::new();
        // File-scan codes.
        for f in evaluate_keyed(
            &json!({"gate_id": "wrong", "exceptions": [{"path": "x"}]}),
            &observed(&["scripts/y.sh"]),
        ) {
            emitted.insert(f.code);
        }
        // Workflow-shell codes (both directions).
        for f in evaluate_workflow_inline_shell_keyed(
            &shell_observed(&["a::j::step-NEW"]),
            &shell_baseline(&["a::j::step-OLD"]),
        ) {
            emitted.insert(f.code);
        }
        for code in &emitted {
            assert!(
                declared.contains(code.as_str()),
                "evaluator emitted `{code}` which is not in VIOLATION_CODES"
            );
        }
    }
}
