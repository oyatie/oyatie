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
use std::process::Command;

use serde_json::{Value, json};
use serde_yaml::Value as YamlValue;
use toml::Value as TomlValue;

pub const GATE_ID: &str = "cloud-ci-rust-first-automation-hygiene";

/// The protected branch used to freeze exception debt. This is deliberately not policy data:
/// candidate policy must not be able to repoint the frozen reference it is checked against.
const PROTECTED_BASE_REF: &str = "origin/dev";
const POLICY_REPO_PATH: &str =
    "ci/facade/automation-language-policy/rust-first-automation-policy.json";

pub const VIOLATION_CODES: [&str; 18] = [
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
    "rust_first_automation_workflow_inline_shell_missing_step_name",
    "rust_first_automation_workflow_inline_shell_duplicate_step_name",
    "rust_first_automation_workflow_inline_shell_line_count_growth",
    "rust_first_automation_workflow_inline_shell_baseline_malformed",
    // Non-Rust-exception SHRINK-ONLY dimension: the file scan permits a non-Rust file iff it has an
    // exceptions[] entry (any-allowlisted-ok). These two codes additionally freeze the EXCEPTION SET
    // shrink-only against a review-visible baseline, so a NEW .py/.sh bridge cannot be admitted by
    // silently adding an allowlist row — growth requires a reviewed baseline edit. This forces new
    // contract-slice validators onto //ci/facade/contract-slice-conformance instead of scripts/tests/*.py.
    "rust_first_automation_unbaselined_non_rust_exception",
    "rust_first_automation_non_rust_exception_baseline_stale",
    // Workflow-uses dimension: the Buck2 setup action is not an allowed bridge. The repo-owned
    // installer may download the official facebook/buck2 release asset, but CI must not outsource
    // that policy boundary to a marketplace action.
    "rust_first_automation_forbidden_workflow_action",
    // Interpreter-command dimension (G006): Rust source must not re-authorize Python/Node/MJS by
    // spawning interpreter commands directly. This inventory closes the gap where the extension scan
    // is green but a Rust test/helper still shells out to a retired interpreter.
    "rust_first_automation_interpreter_command_authority",
    // CLI-package dimension: infrastructure/cloud automation must not add a new package whose
    // canonical role is a CLI. Gate binaries and controllers stay `*-app`; local CLIs are retired
    // bridge surfaces, not the new cloud-native path.
    "rust_first_automation_cli_package_authority",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanError {
    MissingScanArray(String),
    Io(String),
    Parse(String),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::MissingScanArray(field) => write!(f, "policy scan.{field} must be an array"),
            ScanError::Io(message) => write!(f, "automation hygiene scan io: {message}"),
            ScanError::Parse(message) => write!(f, "automation hygiene scan parse: {message}"),
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
// emitting one keyed observation per (file, job, required unique step name) that carries an inline
// `run:` block and its non-empty `shell_lines`. Schema-v2 ratchets both values exactly against the
// embedded FROZEN baseline (policy-as-data): a new/grown entry blocks, while a removed/shrunk entry
// makes the baseline stale until the same reviewed change shrinks it.

/// The stable key for a single inline-shell observation: `<workflow-relpath>::<job-id>::<name>`.
/// A workflow `run:` step is required to have a unique non-empty `name`, so inserting an unrelated
/// step cannot renumber accepted debt or silently transfer its baseline allowance.
fn workflow_shell_key(rel: &str, job: &str, name: &str) -> String {
    format!("{rel}::{job}::{name}")
}

/// Count the inline-shell lines in a `run:` scalar. Block scalars (`run: |` / `run: >`) and
/// single-line `run:` are both deserialized by serde_yaml into a plain string, so this is a simple
/// non-empty-line count over the already-parsed value (no regex, no manual block-scalar handling).
fn shell_line_count(run: &str) -> usize {
    run.lines().filter(|line| !line.trim().is_empty()).count()
}

/// Glob-free file collector: the policy declares explicit directories + extensions for a scan
/// surface; we walk each root and keep files whose extension is in the set. Read-only,
/// deterministic (sorted), no temp files.
fn collect_files_with_extensions(
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
        visit_files_with_extensions(repo_root, &absolute, extensions, &mut files)?;
    }
    files.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(files)
}

fn visit_files_with_extensions(
    repo_root: &Path,
    dir: &Path,
    extensions: &BTreeSet<String>,
    files: &mut Vec<(PathBuf, String)>,
) -> Result<(), ScanError> {
    for path in sorted_dir_entries(dir)? {
        let metadata = fs::symlink_metadata(&path).map_err(|e| {
            ScanError::Io(format!(
                "symlink_metadata {} during workflow scan: {e}",
                path.display()
            ))
        })?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            visit_files_with_extensions(repo_root, &path, extensions, files)?;
            continue;
        }
        if !metadata.is_file() || !has_non_rust_extension(&path, extensions) {
            continue;
        }
        let rel = path
            .strip_prefix(repo_root)
            .map_err(|e| ScanError::Io(format!("strip repo root from {}: {e}", path.display())))?
            .to_string_lossy()
            .replace('\\', "/");
        files.push((path, rel));
    }
    Ok(())
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
        let mut names = BTreeSet::new();
        for step in steps {
            let Some(run) = step.get("run").and_then(YamlValue::as_str) else {
                continue;
            };
            let Some(name) = step
                .get("name")
                .and_then(YamlValue::as_str)
                .map(str::trim)
                .filter(|name| !name.is_empty())
            else {
                rows.push(json!({
                    "identity_error": "missing_name",
                    "file": rel,
                    "job": job,
                }));
                continue;
            };
            if !names.insert(name.to_owned()) {
                rows.push(json!({
                    "identity_error": "duplicate_name",
                    "key": workflow_shell_key(rel, job, name),
                    "file": rel,
                    "job": job,
                    "name": name,
                }));
                continue;
            }
            rows.push(json!({
                "key": workflow_shell_key(rel, job, name),
                "file": rel,
                "job": job,
                "name": name,
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
    let block = policy
        .get("scan")
        .and_then(|scan| scan.get("workflow_inline_shell"));
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
    for (path, rel) in collect_files_with_extensions(repo_root, &roots, &extensions)? {
        let text = fs::read_to_string(&path)
            .map_err(|e| ScanError::Io(format!("read workflow {}: {e}", path.display())))?;
        let doc: YamlValue = serde_yaml::from_str(&text)
            .map_err(|e| ScanError::Io(format!("parse workflow yaml {}: {e}", path.display())))?;
        extract_inline_shell_steps(&rel, &doc, &mut rows);
    }
    rows.sort_by(|a, b| string_field(a, "key").cmp(&string_field(b, "key")));
    Ok(json!({ "steps": rows }))
}

fn workflow_block_string_array(
    block: Option<&Value>,
    block_name: &str,
    key: &str,
) -> Result<Vec<String>, ScanError> {
    block
        .and_then(|block| block.get(key))
        .and_then(Value::as_array)
        .ok_or_else(|| ScanError::MissingScanArray(format!("{block_name}.{key}")))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| ScanError::MissingScanArray(format!("{block_name}.{key}")))
        })
        .collect()
}

fn workflow_string_array(block: Option<&Value>, key: &str) -> Result<Vec<String>, ScanError> {
    workflow_block_string_array(block, "workflow_inline_shell", key)
}

/// The set of inline-shell keys observed in the live workflow corpus.
fn observed_workflow_shell_entries(
    observed: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, usize> {
    let mut entries = BTreeMap::new();
    for row in observed
        .get("steps")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        match row.get("identity_error").and_then(Value::as_str) {
            Some("missing_name") => {
                let key = format!(
                    "{}::{}",
                    string_field(row, "file").unwrap_or("<unknown-file>"),
                    string_field(row, "job").unwrap_or("<unknown-job>")
                );
                findings.insert(Finding::new(
                    "rust_first_automation_workflow_inline_shell_missing_step_name",
                    &key,
                    "workflow run step missing required non-empty unique `name`",
                ));
            }
            Some("duplicate_name") => {
                let key = string_field(row, "key")
                    .unwrap_or("<observed-workflow-step>")
                    .to_owned();
                findings.insert(Finding::new(
                    "rust_first_automation_workflow_inline_shell_duplicate_step_name",
                    &key,
                    "workflow run step name is duplicated within its job",
                ));
            }
            Some(_) => {
                findings.insert(Finding::new(
                    "rust_first_automation_observed_path_missing_field",
                    "<observed-workflow-step>",
                    "observed workflow inline-shell row has an unknown identity error",
                ));
            }
            None => match (
                string_field(row, "key").filter(|k| !k.is_empty()),
                row.get("shell_lines").and_then(Value::as_u64),
            ) {
                (Some(key), Some(lines)) => {
                    entries.insert(key.to_owned(), lines as usize);
                }
                _ => {
                    findings.insert(Finding::new(
                        "rust_first_automation_observed_path_missing_field",
                        "<observed-workflow-step>",
                        "observed workflow inline-shell row missing non-empty `key` or `shell_lines`",
                    ));
                }
            },
        }
    }
    entries
}

/// The frozen named-step entries for the workflow-inline-shell code, read from the baseline face's
/// `codes.rust_first_automation_unbaselined_workflow_inline_shell` array. Every entry must carry
/// an exact non-empty key and measured `shell_lines`; malformed or duplicate rows fail closed.
fn baseline_workflow_shell_entries(
    baseline: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, usize> {
    let mut entries = BTreeMap::new();
    let Some(values) = baseline
        .get("codes")
        .and_then(|codes| codes.get("rust_first_automation_unbaselined_workflow_inline_shell"))
        .and_then(Value::as_array)
    else {
        findings.insert(Finding::new(
            "rust_first_automation_workflow_inline_shell_baseline_malformed",
            "<workflow-inline-shell-baseline>",
            "workflow inline-shell baseline must contain a codes array of named entries",
        ));
        return entries;
    };
    for value in values {
        let Some(key) = value
            .get("key")
            .and_then(Value::as_str)
            .filter(|key| !key.is_empty())
        else {
            findings.insert(Finding::new(
                "rust_first_automation_workflow_inline_shell_baseline_malformed",
                "<workflow-inline-shell-baseline>",
                "workflow inline-shell baseline entry missing non-empty `key`",
            ));
            continue;
        };
        let Some(lines) = value.get("shell_lines").and_then(Value::as_u64) else {
            findings.insert(Finding::new(
                "rust_first_automation_workflow_inline_shell_baseline_malformed",
                key,
                "workflow inline-shell baseline entry missing unsigned `shell_lines`",
            ));
            continue;
        };
        if entries.insert(key.to_owned(), lines as usize).is_some() {
            findings.insert(Finding::new(
                "rust_first_automation_workflow_inline_shell_baseline_malformed",
                key,
                "workflow inline-shell baseline contains duplicate named-step key",
            ));
        }
    }
    entries
}

/// Pure evaluator for the workflow-inline-shell dimension. `observed` is the scanner output shaped
/// as `{ "steps": [{"key": "...", "shell_lines": N}] }`; `baseline` is the frozen named-step
/// baseline face. Exact shrink-only:
///   - any observed key NOT in the baseline ⇒ `rust_first_automation_unbaselined_workflow_inline_shell`
///     (a NEW inline-shell step beyond the accepted legacy-bridge debt is born-blocking);
///   - any baseline key NOT observed ⇒ `rust_first_automation_workflow_inline_shell_baseline_stale`
///     (a retired step must shrink the baseline in the same PR, mirroring the file-scan stale code);
///   - line-count growth is born-blocking and line-count shrink makes the frozen baseline stale,
///     preventing a prior ceiling from permitting later regrowth.
pub fn evaluate_workflow_inline_shell_keyed(
    observed: &Value,
    baseline: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let baseline_entries = baseline_workflow_shell_entries(baseline, &mut findings);
    let observed_entries = observed_workflow_shell_entries(observed, &mut findings);

    for (key, observed_lines) in &observed_entries {
        match baseline_entries.get(key) {
            None => {
                findings.insert(Finding::new(
                "rust_first_automation_unbaselined_workflow_inline_shell",
                key,
                "new inline shell inside workflow YAML beyond the frozen legacy-bridge baseline; \
                 productize it as a Rust/Buck2 step or extend the reviewed baseline (shrink-only)",
            ));
            }
            Some(baseline_lines) if observed_lines > baseline_lines => {
                findings.insert(Finding::new(
                    "rust_first_automation_workflow_inline_shell_line_count_growth",
                    key,
                    format!("workflow inline-shell line count grew from frozen baseline {baseline_lines} to {observed_lines}; only shrink is allowed"),
                ));
            }
            Some(baseline_lines) if observed_lines < baseline_lines => {
                findings.insert(Finding::new(
                    "rust_first_automation_workflow_inline_shell_baseline_stale",
                    key,
                    format!("workflow inline-shell line count shrank from frozen baseline {baseline_lines} to {observed_lines}; shrink the baseline in this PR"),
                ));
            }
            Some(_) => {}
        }
    }

    for key in baseline_entries.keys() {
        if !observed_entries.contains_key(key) {
            findings.insert(Finding::new(
                "rust_first_automation_workflow_inline_shell_baseline_stale",
                key,
                "baselined workflow inline-shell step no longer exists; shrink the baseline in this PR",
            ));
        }
    }

    findings
}

/// Enforce the immutable merge-base workflow baseline as an anti-expansion ceiling. A candidate
/// may remove an accepted shell step or reduce its line count, but it may not add a baseline key
/// or raise a line-count ceiling to waive newly introduced workflow shell debt.
pub fn validate_workflow_inline_shell_baseline_ceiling(
    candidate_baseline: &Value,
    protected_baseline: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let candidate_entries = baseline_workflow_shell_entries(candidate_baseline, &mut findings);
    let protected_entries = baseline_workflow_shell_entries(protected_baseline, &mut findings);

    for (key, candidate_lines) in candidate_entries {
        match protected_entries.get(&key) {
            None => {
                findings.insert(Finding::new(
                    "rust_first_automation_unbaselined_workflow_inline_shell",
                    &key,
                    "candidate workflow inline-shell baseline adds a key beyond the immutable \
                     merge-base ceiling",
                ));
            }
            Some(protected_lines) if candidate_lines > *protected_lines => {
                findings.insert(Finding::new(
                    "rust_first_automation_workflow_inline_shell_line_count_growth",
                    &key,
                    format!(
                        "candidate workflow inline-shell baseline raises the immutable merge-base \
                         ceiling from {protected_lines} to {candidate_lines}"
                    ),
                ));
            }
            Some(_) => {}
        }
    }

    findings
}

/// The frozen baseline exception paths, read from the baseline face's
/// `codes.rust_first_automation_unbaselined_non_rust_exception` array.
fn baseline_non_rust_exception_keys(baseline: &Value) -> BTreeSet<String> {
    baseline
        .get("codes")
        .and_then(|codes| codes.get("rust_first_automation_unbaselined_non_rust_exception"))
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

/// The exception paths currently declared in the policy's `exceptions[]` allowlist.
fn observed_non_rust_exception_keys(policy: &Value) -> BTreeSet<String> {
    policy
        .get("exceptions")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row.get("path").and_then(Value::as_str))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Pure evaluator for the non-Rust-exception SHRINK-ONLY dimension. The file scan already permits a
/// non-Rust file iff it has an `exceptions[]` entry; this dimension additionally freezes the
/// EXCEPTION SET shrink-only against the review-visible baseline. Shrink-only:
///   - any current exception NOT in the baseline ⇒ `rust_first_automation_unbaselined_non_rust_exception`
///     (a NEW non-Rust automation bridge is born-blocking; convert it to owned Rust — e.g. a
///     contract-slice entry on //ci/facade/contract-slice-conformance — or extend the reviewed
///     baseline shrink-only);
///   - any baseline path NOT currently declared ⇒ `rust_first_automation_non_rust_exception_baseline_stale`
///     (a removed bridge must shrink the baseline in the same PR, mirroring the workflow-shell code).
pub fn evaluate_non_rust_exception_baseline_keyed(
    policy: &Value,
    baseline: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let baseline_keys = baseline_non_rust_exception_keys(baseline);
    let observed_keys = observed_non_rust_exception_keys(policy);

    for key in &observed_keys {
        if !baseline_keys.contains(key) {
            findings.insert(Finding::new(
                "rust_first_automation_unbaselined_non_rust_exception",
                key,
                "new non-Rust automation exception beyond the frozen review-visible baseline; \
                 convert it to owned Rust (e.g. a contract-slice policy entry on \
                 //ci/facade/contract-slice-conformance) or extend the reviewed baseline (shrink-only)",
            ));
        }
    }

    for key in &baseline_keys {
        if !observed_keys.contains(key) {
            findings.insert(Finding::new(
                "rust_first_automation_non_rust_exception_baseline_stale",
                key,
                "baselined non-Rust exception no longer declared; shrink the baseline in this PR",
            ));
        }
    }

    findings
}

/// Enforce the immutable merge-base non-Rust exception baseline as an anti-expansion ceiling. A
/// candidate may remove an exception and shrink its matching baseline in the same PR, but may not
/// add a baseline path to waive a new exception.
pub fn validate_non_rust_exception_baseline_ceiling(
    candidate_baseline: &Value,
    protected_baseline: &Value,
) -> BTreeSet<Finding> {
    let candidate_keys = baseline_non_rust_exception_keys(candidate_baseline);
    let protected_keys = baseline_non_rust_exception_keys(protected_baseline);
    candidate_keys
        .difference(&protected_keys)
        .map(|key| {
            Finding::new(
                "rust_first_automation_unbaselined_non_rust_exception",
                key,
                "candidate non-Rust exception baseline adds a path beyond the immutable merge-base \
                 ceiling",
            )
        })
        .collect()
}

// ───────────────────────── merge-base frozen exception baseline ─────────────────────────────

/// Narrow Git-object seam for the immutable baseline. Mirroring the repository's other frozen
/// reference gates keeps candidate filesystem reads out of the allowance source.
trait FrozenPolicySource {
    fn merge_base(&self, base_ref: &str) -> Result<String, String>;
    fn show_file(&self, revision: &str, path: &str) -> Result<String, String>;
}

struct GitCliFrozenPolicySource<'a> {
    repo_root: &'a Path,
}

impl FrozenPolicySource for GitCliFrozenPolicySource<'_> {
    fn merge_base(&self, base_ref: &str) -> Result<String, String> {
        git_stdout(self.repo_root, &["merge-base", base_ref, "HEAD"])
    }

    fn show_file(&self, revision: &str, path: &str) -> Result<String, String> {
        let output = Command::new("git")
            .args(["show", &format!("{revision}:{path}")])
            .current_dir(self.repo_root)
            .output()
            .map_err(|error| format!("run git show for frozen automation policy: {error}"))?;
        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|error| format!("frozen automation policy is not UTF-8: {error}"))
        } else {
            Err(format!(
                "read frozen automation policy {revision}:{path}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }
}

fn git_stdout(repo_root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("run git for frozen automation policy: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "frozen automation policy Git command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("frozen automation policy Git output is not UTF-8: {error}"))
}

fn frozen_policy_baseline(
    source: &impl FrozenPolicySource,
    baseline_field: &str,
) -> Result<Value, String> {
    let merge_base = source.merge_base(PROTECTED_BASE_REF)?;
    let policy = source.show_file(&merge_base, POLICY_REPO_PATH)?;
    let policy: Value = serde_json::from_str(&policy)
        .map_err(|error| format!("parse frozen automation policy: {error}"))?;
    policy
        .get(baseline_field)
        .cloned()
        .ok_or_else(|| format!("frozen automation policy missing {baseline_field}"))
}

/// Load the non-Rust exception allowlist baseline from the immutable merge-base tree.
///
/// The candidate policy remains the observed exception set, but its accompanying baseline is
/// intentionally ignored. A new exception can therefore only become accepted after a distinct
/// protected-base change carries that baseline forward.
pub fn load_non_rust_exception_baseline_from_merge_base(repo_root: &Path) -> Result<Value, String> {
    frozen_policy_baseline(
        &GitCliFrozenPolicySource { repo_root },
        "non_rust_exception_baseline",
    )
}

/// Load the workflow inline-shell baseline from the immutable merge-base tree.
///
/// Candidate workflow contents remain the observed corpus, but the candidate's matching baseline
/// is intentionally ignored. A new inline shell can therefore only become accepted after a
/// distinct protected-base change carries that baseline forward.
pub fn load_workflow_inline_shell_baseline_from_merge_base(
    repo_root: &Path,
) -> Result<Value, String> {
    frozen_policy_baseline(
        &GitCliFrozenPolicySource { repo_root },
        "workflow_inline_shell_baseline",
    )
}

// ─────────────────────────── forbidden workflow `uses:` dimension ───────────────────────────
//
// `workflow_inline_shell` owns inline `run:` debt. It intentionally skips `uses:` steps, so a
// marketplace action can re-enter without adding any shell. This dimension is a tiny, data-driven
// negative-space guard for that shape. It is deliberately narrow today: forbid Buck2 setup actions
// while still allowing the repo-owned installer to download the official facebook/buck2 release
// asset and verify its pinned SHA-256.

fn workflow_forbidden_uses_block(policy: &Value) -> Option<&Value> {
    policy
        .get("scan")
        .and_then(|scan| scan.get("workflow_forbidden_uses"))
}

fn forbidden_workflow_uses_key(rel: &str, job: &str, step_index: usize, uses: &str) -> String {
    format!("{rel}::{job}::step-{step_index}::{uses}")
}

fn extract_forbidden_workflow_uses(
    rel: &str,
    doc: &YamlValue,
    forbidden_substrings: &[String],
    rows: &mut Vec<Value>,
) {
    let mut record_steps = |job: &str, steps: &YamlValue| {
        let Some(steps) = steps.as_sequence() else {
            return;
        };
        for (index, step) in steps.iter().enumerate() {
            let Some(uses) = step.get("uses").and_then(YamlValue::as_str) else {
                continue;
            };
            let normalized_uses = uses.to_ascii_lowercase();
            for forbidden in forbidden_substrings {
                if normalized_uses.contains(forbidden) {
                    rows.push(json!({
                        "key": forbidden_workflow_uses_key(rel, job, index, uses),
                        "file": rel,
                        "job": job,
                        "step_index": index,
                        "uses": uses,
                        "forbidden_substring": forbidden,
                    }));
                }
            }
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

    if let Some(steps) = doc.get("runs").and_then(|runs| runs.get("steps")) {
        record_steps("runs", steps);
    }
}

pub fn collect_observed_forbidden_workflow_uses(
    repo_root: &Path,
    policy: &Value,
) -> Result<Value, ScanError> {
    let block = workflow_forbidden_uses_block(policy);
    let enabled = block
        .and_then(|block| block.get("enabled"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !enabled {
        return Ok(json!({ "uses": [] }));
    }

    let roots = workflow_block_string_array(block, "workflow_forbidden_uses", "roots")?;
    let extensions = workflow_block_string_array(block, "workflow_forbidden_uses", "extensions")?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let forbidden_substrings = workflow_block_string_array(
        block,
        "workflow_forbidden_uses",
        "forbidden_uses_substrings",
    )?
    .into_iter()
    .map(|pattern| pattern.to_ascii_lowercase())
    .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for (path, rel) in collect_files_with_extensions(repo_root, &roots, &extensions)? {
        let text = fs::read_to_string(&path)
            .map_err(|e| ScanError::Io(format!("read workflow {}: {e}", path.display())))?;
        let doc: YamlValue = serde_yaml::from_str(&text)
            .map_err(|e| ScanError::Io(format!("parse workflow yaml {}: {e}", path.display())))?;
        extract_forbidden_workflow_uses(&rel, &doc, &forbidden_substrings, &mut rows);
    }
    rows.sort_by(|a, b| string_field(a, "key").cmp(&string_field(b, "key")));
    Ok(json!({ "uses": rows }))
}

pub fn evaluate_forbidden_workflow_uses(observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    for row in observed
        .get("uses")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        match string_field(row, "key").filter(|k| !k.is_empty()) {
            Some(key) => {
                let uses = string_field(row, "uses").unwrap_or("<unknown>");
                findings.insert(Finding::new(
                    "rust_first_automation_forbidden_workflow_action",
                    key,
                    format!(
                        "forbidden workflow action `{uses}`; use repo-owned infra/ci/install-buck2.sh \
                         to download the official facebook/buck2 release asset with pinned SHA-256"
                    ),
                ));
            }
            None => {
                findings.insert(Finding::new(
                    "rust_first_automation_observed_path_missing_field",
                    "<observed-workflow-uses>",
                    "observed workflow uses row missing non-empty `key`",
                ));
            }
        }
    }
    findings
}

// ───────────────────── interpreter command authority dimension (G006) ─────────────────────
//
// The file-extension scan inventories non-Rust automation files. It does not catch Rust source that
// re-introduces a retired interpreter by executing `Command::new("python3")` or
// `Command::new("/path/to/tool.mjs")`. This dimension is intentionally inventory-first and narrow:
// policy DATA declares Rust roots, exact forbidden interpreter command names, and forbidden command
// suffixes; the scanner records direct `Command::new("<literal>")` authority sites with file:line
// keys. Porting the two live python3 test bridges to Rust std keeps the live corpus green.

fn interpreter_command_block(policy: &Value) -> Option<&Value> {
    policy
        .get("scan")
        .and_then(|scan| scan.get("interpreter_command_authority"))
}

fn command_literal_after_new(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") || trimmed.starts_with('*') {
        return None;
    }
    let marker = "Command::new";
    let marker_index = trimmed.find(marker)?;
    let after_marker = &trimmed[marker_index + marker.len()..];
    let open_index = after_marker.find('(')?;
    let after_open = after_marker[open_index + 1..].trim_start();
    let after_quote = after_open.strip_prefix('"')?;
    let close_index = after_quote.find('"')?;
    Some(&after_quote[..close_index])
}

fn command_basename(command: &str) -> &str {
    command
        .rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(command)
}

fn command_matches_interpreter_policy(
    command: &str,
    forbidden_commands: &BTreeSet<String>,
    forbidden_suffixes: &[String],
) -> bool {
    let normalized = command.to_ascii_lowercase();
    let basename = command_basename(&normalized);
    forbidden_commands.contains(basename)
        || forbidden_suffixes
            .iter()
            .any(|suffix| normalized.ends_with(suffix))
}

fn interpreter_command_key(rel: &str, line_number: usize, command: &str) -> String {
    format!("{rel}:{line_number}::{command}")
}

fn extract_interpreter_command_authority(
    rel: &str,
    text: &str,
    forbidden_commands: &BTreeSet<String>,
    forbidden_suffixes: &[String],
    rows: &mut Vec<Value>,
) {
    for (line_index, line) in text.lines().enumerate() {
        let Some(command) = command_literal_after_new(line) else {
            continue;
        };
        if command_matches_interpreter_policy(command, forbidden_commands, forbidden_suffixes) {
            let line_number = line_index + 1;
            rows.push(json!({
                "key": interpreter_command_key(rel, line_number, command),
                "file": rel,
                "line": line_number,
                "command": command,
            }));
        }
    }
}

pub fn collect_observed_interpreter_command_authority(
    repo_root: &Path,
    policy: &Value,
) -> Result<Value, ScanError> {
    let block = interpreter_command_block(policy);
    let enabled = block
        .and_then(|block| block.get("enabled"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !enabled {
        return Ok(json!({ "commands": [] }));
    }

    let roots = workflow_block_string_array(block, "interpreter_command_authority", "roots")?;
    let exclude_prefixes =
        workflow_block_string_array(block, "interpreter_command_authority", "exclude_prefixes")?;
    let extensions =
        workflow_block_string_array(block, "interpreter_command_authority", "extensions")?
            .into_iter()
            .collect::<BTreeSet<_>>();
    let forbidden_commands = workflow_block_string_array(
        block,
        "interpreter_command_authority",
        "forbidden_command_literals",
    )?
    .into_iter()
    .map(|command| command.to_ascii_lowercase())
    .collect::<BTreeSet<_>>();
    let forbidden_suffixes = workflow_block_string_array(
        block,
        "interpreter_command_authority",
        "forbidden_command_suffixes",
    )?
    .into_iter()
    .map(|suffix| suffix.to_ascii_lowercase())
    .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for (path, rel) in collect_files_with_extensions(repo_root, &roots, &extensions)? {
        if path_is_excluded(&rel, &exclude_prefixes) {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|e| {
            ScanError::Io(format!(
                "read source {} for interpreter-command scan: {e}",
                path.display()
            ))
        })?;
        extract_interpreter_command_authority(
            &rel,
            &text,
            &forbidden_commands,
            &forbidden_suffixes,
            &mut rows,
        );
    }
    rows.sort_by(|a, b| string_field(a, "key").cmp(&string_field(b, "key")));
    Ok(json!({ "commands": rows }))
}

pub fn evaluate_interpreter_command_authority(observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    for row in observed
        .get("commands")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        match string_field(row, "key").filter(|k| !k.is_empty()) {
            Some(key) => {
                let command = string_field(row, "command").unwrap_or("<unknown>");
                findings.insert(Finding::new(
                    "rust_first_automation_interpreter_command_authority",
                    key,
                    format!(
                        "direct interpreter command `{command}` is retired authority; port the \
                         behavior to Rust/Buck2/cloud-ci or record a narrower reviewed exception"
                    ),
                ));
            }
            None => {
                findings.insert(Finding::new(
                    "rust_first_automation_observed_path_missing_field",
                    "<observed-interpreter-command>",
                    "observed interpreter command row missing non-empty `key`",
                ));
            }
        }
    }
    findings
}

// ───────────────────────────── CLI package authority dimension ─────────────────────────────
//
// The file-extension and workflow scans block shell/Python-style automation. This dimension closes
// the neighboring regression where a new infrastructure package is born as a CLI-first workflow
// even though the cloud-native standard requires API-shaped Rust apps, gates, controllers, or
// declarative config. The policy is intentionally scoped to infrastructure/cloud/tooling roots and
// exact package-name suffixes, so normal gate binaries (`*-app`) and non-infra product crates stay out
// of the blast radius.

fn cli_package_authority_block(policy: &Value) -> Option<&Value> {
    policy
        .get("scan")
        .and_then(|scan| scan.get("cli_package_authority"))
}

fn cargo_package_name(rel: &str, text: &str) -> Result<Option<String>, ScanError> {
    let document: TomlValue =
        toml::from_str(text).map_err(|error| ScanError::Parse(format!("parse {rel}: {error}")))?;
    let Some(package) = document.get("package").and_then(TomlValue::as_table) else {
        if document.get("workspace").is_some() {
            return Ok(None);
        }
        return Err(ScanError::Parse(format!(
            "{rel} is a Cargo.toml without [package] or [workspace]"
        )));
    };
    package
        .get("name")
        .and_then(TomlValue::as_str)
        .map(str::to_owned)
        .map(Some)
        .ok_or_else(|| ScanError::Parse(format!("{rel} missing string [package].name")))
}

fn package_name_matches_suffix(package_name: &str, suffixes: &[String]) -> bool {
    let normalized = package_name.to_ascii_lowercase();
    suffixes
        .iter()
        .any(|suffix| normalized.ends_with(&suffix.to_ascii_lowercase()))
}

fn cli_package_key(rel: &str, package_name: &str) -> String {
    format!("{rel}::{package_name}")
}

pub fn collect_observed_cli_package_authority(
    repo_root: &Path,
    policy: &Value,
) -> Result<Value, ScanError> {
    let block = cli_package_authority_block(policy);
    let enabled = block
        .and_then(|block| block.get("enabled"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !enabled {
        return Ok(json!({ "packages": [] }));
    }

    let roots = workflow_block_string_array(block, "cli_package_authority", "roots")?;
    let exclude_prefixes =
        workflow_block_string_array(block, "cli_package_authority", "exclude_prefixes")?;
    let extensions = workflow_block_string_array(block, "cli_package_authority", "extensions")?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let forbidden_suffixes = workflow_block_string_array(
        block,
        "cli_package_authority",
        "forbidden_package_name_suffixes",
    )?;

    let mut rows = Vec::new();
    for (path, rel) in collect_files_with_extensions(repo_root, &roots, &extensions)? {
        if path_is_excluded(&rel, &exclude_prefixes) || !rel.ends_with("/Cargo.toml") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|e| {
            ScanError::Io(format!(
                "read Cargo manifest {} for CLI package scan: {e}",
                path.display()
            ))
        })?;
        let Some(package_name) = cargo_package_name(&rel, &text)? else {
            continue;
        };
        if package_name_matches_suffix(&package_name, &forbidden_suffixes) {
            rows.push(json!({
                "key": cli_package_key(&rel, &package_name),
                "path": rel,
                "package_name": package_name,
            }));
        }
    }
    rows.sort_by(|a, b| string_field(a, "key").cmp(&string_field(b, "key")));
    Ok(json!({ "packages": rows }))
}

pub fn evaluate_cli_package_authority(observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    for row in observed
        .get("packages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        match string_field(row, "key").filter(|k| !k.is_empty()) {
            Some(key) => {
                let package_name = string_field(row, "package_name").unwrap_or("<unknown>");
                findings.insert(Finding::new(
                    "rust_first_automation_cli_package_authority",
                    key,
                    format!(
                        "infrastructure package `{package_name}` is CLI-shaped; use an API-shaped \
                         Rust app/gate/controller plus declarative config instead"
                    ),
                ));
            }
            None => {
                findings.insert(Finding::new(
                    "rust_first_automation_observed_path_missing_field",
                    "<observed-cli-package>",
                    "observed CLI package row missing non-empty `key`",
                ));
            }
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

    #[test]
    fn cli_package_authority_flags_infra_cli_suffix() {
        let findings = evaluate_cli_package_authority(&json!({"packages": [{
            "key": "infra/example/Cargo.toml::infra-fix-cli",
            "path": "infra/example/Cargo.toml",
            "package_name": "infra-fix-cli"
        }]}));
        assert!(findings.iter().any(|finding| {
            finding.code == "rust_first_automation_cli_package_authority"
                && finding.key == "infra/example/Cargo.toml::infra-fix-cli"
        }));
    }

    #[test]
    fn cargo_package_name_reads_only_package_table() {
        let text = "[package]\nname = 'infra-fix-cli'\n\n[[bin]]\nname = \"ignored-bin-cli\"\n";
        assert_eq!(
            cargo_package_name("infra/example/Cargo.toml", text)
                .unwrap()
                .as_deref(),
            Some("infra-fix-cli")
        );
    }

    #[test]
    fn cargo_package_name_skips_workspace_manifests() {
        let text = "[workspace]\nmembers = [\"crates/example\"]\n";
        assert_eq!(
            cargo_package_name("cloud/cloud-kernel/Cargo.toml", text).unwrap(),
            None
        );
    }

    #[test]
    fn cargo_package_name_fails_closed_on_unparseable_manifest() {
        let err = cargo_package_name("infra/bad/Cargo.toml", "[package]\nname = ").unwrap_err();
        assert!(matches!(err, ScanError::Parse(_)));
    }

    #[test]
    fn package_name_suffix_matching_is_case_insensitive() {
        assert!(package_name_matches_suffix(
            "Infra-Fix-CLI",
            &["-cli".to_owned()]
        ));
        assert!(!package_name_matches_suffix(
            "oya-cloud-ci-firewall-app",
            &["-cli".to_owned()]
        ));
    }

    // ───────────────── workflow-inline-shell evaluator unit tests (pipeline-glue(a)) ─────────────

    fn shell_observed(keys: &[&str]) -> Value {
        json!({"steps": keys.iter().map(|k| json!({"key": k, "shell_lines": 1})).collect::<Vec<_>>()})
    }

    fn shell_baseline(keys: &[&str]) -> Value {
        json!({"codes": {
            "rust_first_automation_unbaselined_workflow_inline_shell":
                keys.iter().map(|k| json!({"key": k, "shell_lines": 1})).collect::<Vec<_>>()
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
    fn extract_inline_shell_steps_skips_uses_and_keys_per_unique_name() {
        let doc: YamlValue = serde_yaml::from_str(
            "jobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n      - name: First shell\n        run: echo one\n      - name: Second shell\n        run: |\n          echo two\n          echo three\n",
        )
        .unwrap();
        let mut rows = Vec::new();
        extract_inline_shell_steps(".github/workflows/x.yml", &doc, &mut rows);
        let keys: Vec<&str> = rows.iter().filter_map(|r| r["key"].as_str()).collect();
        // The `uses:` step has no run block; run steps are keyed by their required names.
        assert_eq!(
            keys,
            vec![
                ".github/workflows/x.yml::build::First shell",
                ".github/workflows/x.yml::build::Second shell"
            ]
        );
        assert_eq!(rows[1]["shell_lines"].as_u64(), Some(2));
    }

    #[test]
    fn workflow_shell_missing_or_duplicate_named_run_step_fails_closed() {
        let doc: YamlValue = serde_yaml::from_str(
            "jobs:\n  build:\n    steps:\n      - run: echo unnamed\n      - name: Duplicate\n        run: echo one\n      - name: Duplicate\n        run: echo two\n",
        )
        .unwrap();
        let mut rows = Vec::new();
        extract_inline_shell_steps(".github/workflows/x.yml", &doc, &mut rows);
        let findings = evaluate_workflow_inline_shell_keyed(
            &json!({"steps": rows}),
            &shell_baseline(&[".github/workflows/x.yml::build::Duplicate"]),
        );
        assert!(
            findings
                .iter()
                .any(|f| f.code == "rust_first_automation_workflow_inline_shell_missing_step_name")
        );
        assert!(
            findings.iter().any(
                |f| f.code == "rust_first_automation_workflow_inline_shell_duplicate_step_name"
            )
        );
    }

    #[test]
    fn workflow_shell_line_count_growth_beyond_frozen_baseline_is_red() {
        let findings = evaluate_workflow_inline_shell_keyed(
            &json!({"steps": [{"key": "a.yml::job::Named shell", "shell_lines": 2}]}),
            &json!({"codes": {"rust_first_automation_unbaselined_workflow_inline_shell": [
                {"key": "a.yml::job::Named shell", "shell_lines": 1}
            ]}}),
        );
        assert!(findings.iter().any(|f| {
            f.code == "rust_first_automation_workflow_inline_shell_line_count_growth"
                && f.key == "a.yml::job::Named shell"
        }));
    }

    #[test]
    fn workflow_shell_line_count_shrink_requires_baseline_shrink() {
        let findings = evaluate_workflow_inline_shell_keyed(
            &json!({"steps": [{"key": "a.yml::job::Named shell", "shell_lines": 1}]}),
            &json!({"codes": {"rust_first_automation_unbaselined_workflow_inline_shell": [
                {"key": "a.yml::job::Named shell", "shell_lines": 2}
            ]}}),
        );
        assert!(findings.iter().any(|f| {
            f.code == "rust_first_automation_workflow_inline_shell_baseline_stale"
                && f.key == "a.yml::job::Named shell"
        }));
    }

    #[test]
    fn workflow_shell_malformed_or_duplicate_baseline_entries_fail_closed() {
        let findings = evaluate_workflow_inline_shell_keyed(
            &shell_observed(&["a.yml::job::Named shell"]),
            &json!({"codes": {"rust_first_automation_unbaselined_workflow_inline_shell": [
                {"key": "a.yml::job::Named shell", "shell_lines": 1},
                {"key": "a.yml::job::Named shell", "shell_lines": 1},
                {"key": "a.yml::job::Broken"}
            ]}}),
        );
        assert!(findings.iter().any(|f| {
            f.code == "rust_first_automation_workflow_inline_shell_baseline_malformed"
                && f.key == "a.yml::job::Named shell"
        }));
        assert!(findings.iter().any(|f| {
            f.code == "rust_first_automation_workflow_inline_shell_baseline_malformed"
                && f.key == "a.yml::job::Broken"
        }));
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
        for f in evaluate_forbidden_workflow_uses(
            &json!({"uses": [{"key": "a.yml::j::step-0::x/setup-buck2@v1", "uses": "x/setup-buck2@v1"}]}),
        ) {
            emitted.insert(f.code);
        }
        for f in evaluate_interpreter_command_authority(
            &json!({"commands": [{"key": "tools/x/src/main.rs:1::python3", "command": "python3"}]}),
        ) {
            emitted.insert(f.code);
        }
        for f in evaluate_cli_package_authority(&json!({"packages": [{
            "key": "infra/example/Cargo.toml::infra-fix-cli",
            "package_name": "infra-fix-cli"
        }]})) {
            emitted.insert(f.code);
        }
        for code in &emitted {
            assert!(
                declared.contains(code.as_str()),
                "evaluator emitted `{code}` which is not in VIOLATION_CODES"
            );
        }
    }

    #[test]
    fn forbidden_workflow_uses_flags_setup_buck2_action() {
        let findings = evaluate_forbidden_workflow_uses(&json!({"uses": [{
            "key": ".github/workflows/ci.yml::build::step-0::example/setup-buck2@v1",
            "uses": "example/setup-buck2@v1"
        }]}));
        assert!(findings.iter().any(|f| {
            f.code == "rust_first_automation_forbidden_workflow_action"
                && f.key == ".github/workflows/ci.yml::build::step-0::example/setup-buck2@v1"
        }));
    }

    #[test]
    fn extract_forbidden_workflow_uses_scans_uses_not_run_steps() {
        let doc: YamlValue = serde_yaml::from_str(
            "jobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n      - uses: example/setup-buck2@v1\n      - run: infra/ci/install-buck2.sh\n",
        )
        .unwrap();
        let mut rows = Vec::new();
        extract_forbidden_workflow_uses(
            ".github/workflows/ci.yml",
            &doc,
            &["setup-buck2".to_owned()],
            &mut rows,
        );
        let keys: Vec<&str> = rows.iter().filter_map(|r| r["key"].as_str()).collect();
        assert_eq!(
            keys,
            vec![".github/workflows/ci.yml::build::step-1::example/setup-buck2@v1"]
        );
    }

    #[test]
    fn forbidden_uses_scan_recurses_into_nested_composite_actions() {
        let mut root = std::env::temp_dir();
        root.push(format!(
            "oya-rust-first-action-scan-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let action_dir = root.join(".github/actions/buck");
        std::fs::create_dir_all(&action_dir).unwrap();
        std::fs::write(
            action_dir.join("action.yml"),
            "name: nested\nruns:\n  using: composite\n  steps:\n    - uses: example/setup-buck2@v1\n",
        )
        .unwrap();

        let policy = json!({
            "scan": {
                "workflow_forbidden_uses": {
                    "enabled": true,
                    "roots": [".github/actions"],
                    "extensions": [".yml", ".yaml"],
                    "forbidden_uses_substrings": ["setup-buck2"]
                }
            }
        });
        let observed = collect_observed_forbidden_workflow_uses(&root, &policy).unwrap();
        let _ = std::fs::remove_dir_all(&root);

        let findings = evaluate_forbidden_workflow_uses(&observed);
        assert!(
            findings.iter().any(|f| {
                f.code == "rust_first_automation_forbidden_workflow_action"
                    && f.key
                        == ".github/actions/buck/action.yml::runs::step-0::example/setup-buck2@v1"
            }),
            "nested composite action setup-buck2 use must be found; observed={observed:#?}; \
             findings={findings:#?}"
        );
    }
}
