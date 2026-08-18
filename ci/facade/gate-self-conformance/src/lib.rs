//! Pipeline-as-product gate self-conformance.
//!
//! The engine is intentionally shape-neutral: repository-specific gate roots, non-gate producers,
//! no-autofix reasons, and transitional orchestrator exceptions live in policy JSON. The live
//! collector is read-only and the pure evaluator is fixture-friendly, so the same gate can be
//! adopted by another repo by changing data instead of forking logic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

pub const GATE_ID: &str = "cloud-ci-gate-self-conformance";

pub const VIOLATION_CODES: [&str; 9] = [
    "gate_self_conformance_policy_gate_id_mismatch",
    "gate_self_conformance_policy_exception_malformed",
    "gate_self_conformance_no_gates_collected",
    "gate_self_conformance_buck2_missing_gate_target",
    "gate_self_conformance_buck2_missing_unittest_target",
    "gate_self_conformance_workflow_unregistered",
    "gate_self_conformance_automated_missing_fix_contract",
    "gate_self_conformance_hermetic_violation",
    "gate_self_conformance_policy_as_data_violation",
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub findings: BTreeSet<Finding>,
}

impl Report {
    fn from_findings(findings: BTreeSet<Finding>) -> Self {
        let verdict = if findings.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self { verdict, findings }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanError {
    MissingPolicyField(String),
    InvalidPolicyField(String),
    Io(String),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::MissingPolicyField(field) => write!(f, "missing policy field: {field}"),
            ScanError::InvalidPolicyField(field) => write!(f, "invalid policy field: {field}"),
            ScanError::Io(message) => write!(f, "gate self-conformance scan I/O: {message}"),
        }
    }
}

impl std::error::Error for ScanError {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ExceptionKey {
    gate: String,
    code: String,
    token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExceptionRule {
    allowed_paths: BTreeSet<String>,
    max_occurrences: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PolicyLiteralRules {
    allowed_prefixes: Vec<String>,
    forbidden_prefixes: Vec<String>,
    forbidden_contains: Vec<String>,
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn bool_field(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool) == Some(true)
}

fn array_field<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn policy_string<'a>(policy: &'a Value, path: &[&str]) -> Result<&'a str, ScanError> {
    let mut cursor = policy;
    for segment in path {
        cursor = cursor
            .get(*segment)
            .ok_or_else(|| ScanError::MissingPolicyField(path.join(".")))?;
    }
    cursor
        .as_str()
        .ok_or_else(|| ScanError::InvalidPolicyField(path.join(".")))
}

fn policy_string_set(policy: &Value, path: &[&str]) -> Result<BTreeSet<String>, ScanError> {
    let mut cursor = policy;
    for segment in path {
        cursor = cursor
            .get(*segment)
            .ok_or_else(|| ScanError::MissingPolicyField(path.join(".")))?;
    }
    let Some(values) = cursor.as_array() else {
        return Err(ScanError::InvalidPolicyField(path.join(".")));
    };
    let mut out = BTreeSet::new();
    for value in values {
        let Some(text) = value.as_str() else {
            return Err(ScanError::InvalidPolicyField(path.join(".")));
        };
        out.insert(text.to_owned());
    }
    Ok(out)
}
fn optional_policy_string_set(
    policy: &Value,
    path: &[&str],
) -> Result<BTreeSet<String>, ScanError> {
    let mut cursor = policy;
    for segment in path {
        let Some(next) = cursor.get(*segment) else {
            return Ok(BTreeSet::new());
        };
        cursor = next;
    }
    let Some(values) = cursor.as_array() else {
        return Err(ScanError::InvalidPolicyField(path.join(".")));
    };
    let mut out = BTreeSet::new();
    for value in values {
        let Some(text) = value.as_str() else {
            return Err(ScanError::InvalidPolicyField(path.join(".")));
        };
        out.insert(text.to_owned());
    }
    Ok(out)
}

fn io_error(path: &Path, context: &str, error: std::io::Error) -> ScanError {
    ScanError::Io(format!("{context} {}: {error}", path.display()))
}

fn sorted_dir_entries(dir: &Path) -> Result<Vec<PathBuf>, ScanError> {
    let mut entries = fs::read_dir(dir)
        .map_err(|e| io_error(dir, "read_dir", e))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|e| io_error(dir, "read_dir entry", e))
        })
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    Ok(entries)
}

fn visit_rs_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), ScanError> {
    if !dir.exists() {
        return Ok(());
    }
    for path in sorted_dir_entries(dir)? {
        let metadata = fs::symlink_metadata(&path).map_err(|e| io_error(&path, "metadata", e))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            visit_rs_files(&path, files)?;
            continue;
        }
        if metadata.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

/// Return the production portion of a Rust file for this lexical scanner.
///
/// Gate production code conventionally keeps inline tests behind a final `#[cfg(test)]` module;
/// tests and fixtures intentionally use local paths, clocks, tempdirs, and subprocess doubles. The
/// meta-gate targets the production verdict/fixer path, so it trims that suffix before scanning.
pub fn production_rust_slice(text: &str) -> String {
    let mut out = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            break;
        }
        out.push(line);
    }
    out.join("\n")
}

fn strip_line_comments(line: &str) -> &str {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        ""
    } else {
        line
    }
}

fn strip_string_literals(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_string = false;
    let mut escaped = false;
    for ch in line.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
                out.push('"');
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push('"');
        } else {
            out.push(ch);
        }
    }
    out
}

fn contains_code_token(line: &str, token: &str) -> bool {
    let code = strip_string_literals(strip_line_comments(line));
    if token == "rand" {
        return code.contains("rand::")
            || code.contains("thread_rng")
            || code.contains("StdRng")
            || code.contains("SmallRng")
            || code.contains("OsRng");
    }
    if token == "reqwest" {
        return code.contains("reqwest::") || code.contains("reqwest(");
    }
    if token == "git2" {
        return code.contains("git2::");
    }
    code.contains(token)
}

fn hermetic_token_observations(gate: &str, rel_path: &str, text: &str, rows: &mut Vec<Value>) {
    const TOKENS: [&str; 9] = [
        "std::process::Command",
        "Command::new",
        "tokio::process",
        "reqwest",
        "git2",
        "std::net",
        "SystemTime::now",
        "Instant::now",
        "rand",
    ];
    for (idx, line) in text.lines().enumerate() {
        for token in TOKENS {
            if contains_code_token(line, token) {
                rows.push(json!({
                    "gate": gate,
                    "path": rel_path,
                    "line": idx + 1,
                    "token": token,
                }));
            }
        }
    }
}

/// Extract Rust string literals with 1-based line numbers, skipping `//` line comments and
/// `/* … */` block comments.
///
/// Comment awareness is load-bearing in both directions. Without it, a doc comment that *quotes*
/// a repo path reads as production code hardcoding that path — a false positive that makes the
/// gate a source of friction while catching nothing. The sharper half is the false green: a
/// commented-out `"--fix"` satisfies `has_autofix_contract`, letting a gate claim a remediation
/// surface it does not implement.
///
/// The scan is left-to-right so a `//` *inside* a literal (a URL, say) is consumed as part of the
/// literal rather than opening a comment — truncating there would be a false negative.
fn string_literals_with_lines(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut in_block_comment = false;
    for (line_idx, line) in text.lines().enumerate() {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if in_block_comment {
                if bytes[i] == b'*' && bytes.get(i + 1) == Some(&b'/') {
                    in_block_comment = false;
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
                break;
            }
            if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
                in_block_comment = true;
                i += 2;
                continue;
            }
            if bytes[i] != b'"' {
                i += 1;
                continue;
            }
            let mut j = i + 1;
            let mut literal = String::new();
            let mut escaped = false;
            while j < bytes.len() {
                let ch = bytes[j] as char;
                if escaped {
                    literal.push(ch);
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    break;
                } else {
                    literal.push(ch);
                }
                j += 1;
            }
            if j < bytes.len() {
                out.push((line_idx + 1, literal));
                i = j + 1;
            } else {
                break;
            }
        }
    }
    out
}

fn policy_literal_rules(policy: &Value) -> Result<PolicyLiteralRules, ScanError> {
    Ok(PolicyLiteralRules {
        allowed_prefixes: optional_policy_string_set(
            policy,
            &["policy_literal_rules", "allowed_prefixes"],
        )?
        .into_iter()
        .collect(),
        forbidden_prefixes: optional_policy_string_set(
            policy,
            &["policy_literal_rules", "forbidden_prefixes"],
        )?
        .into_iter()
        .collect(),
        forbidden_contains: optional_policy_string_set(
            policy,
            &["policy_literal_rules", "forbidden_contains"],
        )?
        .into_iter()
        .collect(),
    })
}

fn is_policy_shape_literal(literal: &str, rules: &PolicyLiteralRules) -> bool {
    let normalized = literal.trim_start_matches("./");
    if rules
        .allowed_prefixes
        .iter()
        .any(|prefix| normalized.starts_with(prefix))
    {
        return false;
    }
    rules
        .forbidden_prefixes
        .iter()
        .any(|prefix| normalized.starts_with(prefix))
        || rules
            .forbidden_contains
            .iter()
            .any(|needle| normalized.contains(needle))
}

fn policy_literal_observations(
    gate: &str,
    rel_path: &str,
    text: &str,
    rules: &PolicyLiteralRules,
    rows: &mut Vec<Value>,
) {
    for (line, literal) in string_literals_with_lines(text) {
        if is_policy_shape_literal(&literal, rules) {
            rows.push(json!({
                "gate": gate,
                "path": rel_path,
                "line": line,
                "literal": literal,
            }));
        }
    }
}

fn has_autofix_contract(texts: &[String]) -> bool {
    texts.iter().any(|text| {
        string_literals_with_lines(text)
            .into_iter()
            .any(|(_, literal)| {
                literal.contains("--fix")
                    || literal.contains("--write")
                    || literal.contains("--allow-new")
            })
    })
}

fn package_name(cargo_toml: &str) -> Option<String> {
    let mut in_package = false;
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package && trimmed.starts_with("name") {
            let (_, value) = trimmed.split_once('=')?;
            return Some(value.trim().trim_matches('"').to_owned());
        }
    }
    None
}

fn rel_path(repo_root: &Path, path: &Path) -> Result<String, ScanError> {
    path.strip_prefix(repo_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .map_err(|e| ScanError::Io(format!("strip repo root from {}: {e}", path.display())))
}

fn workflow_matrix_includes_crate(workflow: &str, name: &str) -> bool {
    workflow.lines().any(|line| {
        let trimmed = line.trim();
        (trimmed.starts_with("- { crate: ") && trimmed.contains(&format!("crate: {name},")))
            || trimmed == format!("crate: {name}")
    })
}

fn workflow_invokes_buck_target(workflow: &str, name: &str, gates_root_rel: &str) -> bool {
    let label_prefix = format!("//{}/{name}:", gates_root_rel.trim_matches('/'));
    workflow.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.starts_with('#') && trimmed.contains(&label_prefix)
    })
}

/// True iff an executable workflow line runs a RECURSIVE `buck2 test` pattern that covers the
/// gates root — `buck2 test //ci/...` executes every `//ci/facade/<gate>:` target, so after the
/// 48->2 matrix collapse (2026-08-01) a gate needs no per-crate workflow line at all.
///
/// `--keep-going` / `|| true` invocations are excluded: those are baseline measurements whose
/// exit code is discarded by construction (the affected-set lane's merge-base pass runs
/// `buck2 test //... --keep-going ... || true`), and `//...` would otherwise register every gate
/// in the repo while binding none of them.
fn workflow_executes_recursive_gates_pattern(workflow: &str, gates_root_rel: &str) -> bool {
    let root = gates_root_rel.trim_matches('/');
    workflow.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('#')
            || trimmed.contains("--keep-going")
            || trimmed.contains("|| true")
        {
            return false;
        }
        let Some((_, args)) = trimmed.split_once("buck2 test ") else {
            return false;
        };
        args.split_whitespace().any(|token| {
            token
                .strip_prefix("//")
                .and_then(|rest| rest.strip_suffix("..."))
                .is_some_and(|prefix| {
                    let prefix = prefix.trim_end_matches('/');
                    prefix.is_empty() || root == prefix || root.starts_with(&format!("{prefix}/"))
                })
        })
    })
}

/// NOTE ON SCOPE: this performs no fan-in-REACHABILITY check — a `buck2 test //ci/...` sitting in
/// a job the required context never joins would satisfy it. That is deliberate and safe only
/// because `gate_registration.rs` owns the binding invariant: its
/// `every_gate_crate_is_registered_in_oya_ci_required_workflow` resolves the same patterns but
/// restricted to `oya-ci-required`'s `needs:` list, and
/// `every_gate_lane_is_a_dependency_of_the_fan_in_job` additionally requires the executing job's
/// result to be compared in the fan-in success chain. This function feeds the descriptive
/// `workflow_registered` property; do not promote it to an admission check without adding that
/// reachability restriction.
/// True iff an executable workflow line runs `cargo test --workspace` (with or without
/// `--locked`), which executes every workspace-member gate crate under the gates root. The
/// Cargo merge path (ADR-0716) replaces the buck2 matrix as the registration surface.
fn workflow_executes_workspace_tests(workflow: &str) -> bool {
    workflow.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('#')
            || trimmed.contains("--keep-going")
            || trimmed.contains("|| true")
        {
            return false;
        }
        let Some((_, args)) = trimmed.split_once("cargo test ") else {
            return false;
        };
        args.split_whitespace().any(|token| token == "--workspace")
    })
}

fn workflow_registers_gate(workflow: &str, name: &str, gates_root_rel: &str) -> bool {
    workflow_matrix_includes_crate(workflow, name)
        || workflow_invokes_buck_target(workflow, name, gates_root_rel)
        || workflow_executes_recursive_gates_pattern(workflow, gates_root_rel)
        || workflow_executes_workspace_tests(workflow)
}

fn is_rust_test_source(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("tests.rs")
        || path
            .components()
            .any(|component| component.as_os_str() == "tests")
}

/// Collect the current gate self-conformance observations from a repository checkout.
pub fn collect_observed_gates(repo_root: &Path, policy: &Value) -> Result<Value, ScanError> {
    let gates_root_rel = policy_string(policy, &["scan", "gates_root"])?;
    let workflow_rel = policy_string(policy, &["scan", "workflow_path"])?;
    let gate_prefix = policy_string(policy, &["scan", "gate_crate_prefix"])?;
    let non_gate_crates = policy_string_set(policy, &["scan", "non_gate_crates"])?;
    let bespoke_buck2_gates =
        optional_policy_string_set(policy, &["scan", "bespoke_buck2_gate_crates"])?;
    let literal_rules = policy_literal_rules(policy)?;
    let gates_root = repo_root.join(gates_root_rel);
    let workflow_path = repo_root.join(workflow_rel);
    let workflow = fs::read_to_string(&workflow_path)
        .map_err(|e| io_error(&workflow_path, "read workflow", e))?;

    let mut gates = Vec::new();
    for dir in sorted_dir_entries(&gates_root)? {
        let metadata = fs::symlink_metadata(&dir).map_err(|e| io_error(&dir, "metadata", e))?;
        if !metadata.is_dir() {
            continue;
        }
        let Some(name) = dir.file_name().and_then(|v| v.to_str()).map(str::to_owned) else {
            continue;
        };
        if !name.starts_with(gate_prefix) || non_gate_crates.contains(&name) {
            continue;
        }
        let cargo_path = dir.join("Cargo.toml");
        if !cargo_path.exists() {
            continue;
        }
        let cargo_toml = fs::read_to_string(&cargo_path)
            .map_err(|e| io_error(&cargo_path, "read manifest", e))?;
        let buck_path = dir.join("BUCK");
        let buck = fs::read_to_string(&buck_path).unwrap_or_default();
        let mut rs_files = Vec::new();
        visit_rs_files(&dir.join("src"), &mut rs_files)?;
        rs_files.sort();

        let mut hermetic = Vec::new();
        let mut policy_literals = Vec::new();
        let mut production_texts = Vec::new();
        for source in rs_files {
            if is_rust_test_source(&source) {
                continue;
            }
            let raw =
                fs::read_to_string(&source).map_err(|e| io_error(&source, "read source", e))?;
            let production = production_rust_slice(&raw);
            let rel = rel_path(repo_root, &source)?;
            hermetic_token_observations(&name, &rel, &production, &mut hermetic);
            policy_literal_observations(
                &name,
                &rel,
                &production,
                &literal_rules,
                &mut policy_literals,
            );
            production_texts.push(production);
        }

        gates.push(json!({
            "name": name,
            "package_name": package_name(&cargo_toml).unwrap_or_default(),
            "has_buck_gate": buck.contains(&format!("name = \"ci-{name}-gate\""))
                || bespoke_buck2_gates.contains(&name),
            "has_buck_unittest": buck.contains(&format!("name = \"ci-{name}-unittest\"")),
            "workflow_registered": workflow_registers_gate(&workflow, &name, gates_root_rel),
            "has_autofix": has_autofix_contract(&production_texts),
            "hermetic_observations": hermetic,
            "policy_literal_observations": policy_literals,
        }));
    }
    gates.sort_by(|left, right| string_field(left, "name").cmp(&string_field(right, "name")));
    Ok(json!({ "gates": gates }))
}

fn exception_metadata_ok(row: &Value) -> bool {
    string_field(row, "gate")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        && ["class", "reason", "cutover_note"].iter().all(|key| {
            string_field(row, key)
                .map(|value| value.trim().len() >= 20)
                .unwrap_or(false)
        })
        && !array_field(row, "tokens").is_empty()
}

fn hermetic_exception_rules(
    policy: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<ExceptionKey, ExceptionRule> {
    let mut rules = BTreeMap::new();
    for row in array_field(policy, "hermetic_exceptions") {
        if !exception_metadata_ok(row) {
            findings.insert(Finding::new(
                "gate_self_conformance_policy_exception_malformed",
                string_field(row, "gate").unwrap_or("<missing-gate>"),
                "hermetic exception rows require gate, class, reason, cutover_note, tokens, allowed_paths, and max_occurrences",
            ));
            continue;
        }
        let gate = string_field(row, "gate").unwrap_or_default();
        let allowed_paths = array_field(row, "allowed_paths")
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        if allowed_paths.is_empty() {
            findings.insert(Finding::new(
                "gate_self_conformance_policy_exception_malformed",
                gate,
                "hermetic exception rows must scope allowed_paths so gate-wide token allowlists cannot hide new uses",
            ));
            continue;
        }
        for token in array_field(row, "tokens") {
            let Some(token) = token.as_str() else {
                findings.insert(Finding::new(
                    "gate_self_conformance_policy_exception_malformed",
                    gate,
                    "hermetic exception tokens must be strings",
                ));
                continue;
            };
            let max_occurrences = row
                .get("max_occurrences_by_token")
                .and_then(Value::as_object)
                .and_then(|map| map.get(token))
                .and_then(Value::as_u64)
                .or_else(|| row.get("max_occurrences").and_then(Value::as_u64))
                .and_then(|value| usize::try_from(value).ok())
                .unwrap_or(0);
            if max_occurrences == 0 {
                findings.insert(Finding::new(
                    "gate_self_conformance_policy_exception_malformed",
                    gate,
                    format!("hermetic exception token {token} needs positive max_occurrences"),
                ));
                continue;
            }
            rules.insert(
                ExceptionKey {
                    gate: gate.to_owned(),
                    code: "gate_self_conformance_hermetic_violation".to_owned(),
                    token: token.to_owned(),
                },
                ExceptionRule {
                    allowed_paths: allowed_paths.clone(),
                    max_occurrences,
                },
            );
        }
    }
    rules
}

fn policy_literal_exception_keys(
    policy: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<ExceptionKey> {
    let mut keys = BTreeSet::new();
    for row in array_field(policy, "policy_literal_exceptions") {
        if !exception_metadata_ok(row) {
            findings.insert(Finding::new(
                "gate_self_conformance_policy_exception_malformed",
                string_field(row, "gate").unwrap_or("<missing-gate>"),
                "policy-literal exception rows require gate, class, reason, cutover_note, and tokens",
            ));
            continue;
        }
        let gate = string_field(row, "gate").unwrap_or_default();
        for token in array_field(row, "tokens") {
            let Some(token) = token.as_str() else {
                findings.insert(Finding::new(
                    "gate_self_conformance_policy_exception_malformed",
                    gate,
                    "policy-literal exception tokens must be strings",
                ));
                continue;
            };
            keys.insert(ExceptionKey {
                gate: gate.to_owned(),
                code: "gate_self_conformance_policy_as_data_violation".to_owned(),
                token: token.to_owned(),
            });
        }
    }
    keys
}

fn no_autofix_reasons(
    policy: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Some(map) = policy.get("no_autofix_reason").and_then(Value::as_object) else {
        return out;
    };
    for (gate, reason) in map {
        let Some(reason) = reason.as_str() else {
            findings.insert(Finding::new(
                "gate_self_conformance_policy_exception_malformed",
                gate,
                "no_autofix_reason values must be explanatory strings",
            ));
            continue;
        };
        if reason.trim().len() < 20 {
            findings.insert(Finding::new(
                "gate_self_conformance_policy_exception_malformed",
                gate,
                "no_autofix_reason must explain why an automatic rewrite is unsafe or meaningless",
            ));
            continue;
        }
        out.insert(gate.to_owned(), reason.to_owned());
    }
    out
}

fn observation_key(gate: &str, row: &Value, field: &str) -> String {
    let path = string_field(row, "path").unwrap_or("<unknown-path>");
    let line = row.get("line").and_then(Value::as_u64).unwrap_or(0);
    let token = string_field(row, field).unwrap_or("<unknown>");
    format!("{gate}:{path}:{line}:{token}")
}

/// Pure evaluator over injected policy + observations. This is the stable product surface: tests,
/// CI, and future cloud-native controllers can feed the same observed JSON and get the same verdict.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    if string_field(policy, "gate_id") != Some(GATE_ID) {
        findings.insert(Finding::new(
            "gate_self_conformance_policy_gate_id_mismatch",
            "gate_id",
            format!("policy.gate_id must equal {GATE_ID}"),
        ));
    }
    let exceptions = hermetic_exception_rules(policy, &mut findings);
    let policy_literal_exceptions = policy_literal_exception_keys(policy, &mut findings);
    let no_autofix = no_autofix_reasons(policy, &mut findings);
    let autofix_contracts = policy
        .get("autofix_contract")
        .and_then(Value::as_object)
        .map(|map| map.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    let gates = array_field(observed, "gates");
    if gates.is_empty() {
        findings.insert(Finding::new(
            "gate_self_conformance_no_gates_collected",
            "gates",
            "collector produced no gate rows; fail closed rather than certifying an empty universe",
        ));
        return findings;
    }

    for gate in gates {
        let gate_name = string_field(gate, "name").unwrap_or("<missing-gate-name>");
        if !bool_field(gate, "has_buck_gate") {
            findings.insert(Finding::new(
                "gate_self_conformance_buck2_missing_gate_target",
                gate_name,
                "gate crate must expose a Buck2 <crate>-gate rust_test target",
            ));
        }
        if !bool_field(gate, "has_buck_unittest") {
            findings.insert(Finding::new(
                "gate_self_conformance_buck2_missing_unittest_target",
                gate_name,
                "gate crate must expose a Buck2 <crate>-unittest rust_test target",
            ));
        }
        if !bool_field(gate, "workflow_registered") {
            findings.insert(Finding::new(
                "gate_self_conformance_workflow_unregistered",
                gate_name,
                "gate crate must be wired into the required oya-ci fan-in through Buck2",
            ));
        }
        if !no_autofix.contains_key(gate_name) && !autofix_contracts.contains(gate_name) {
            findings.insert(Finding::new(
                "gate_self_conformance_automated_missing_fix_contract",
                gate_name,
                "gate must declare a policy autofix_contract or no_autofix_reason",
            ));
        }

        let mut exception_counts: BTreeMap<ExceptionKey, usize> = BTreeMap::new();
        for row in array_field(gate, "hermetic_observations") {
            let token = string_field(row, "token").unwrap_or("<missing-token>");
            let key = ExceptionKey {
                gate: gate_name.to_owned(),
                code: "gate_self_conformance_hermetic_violation".to_owned(),
                token: token.to_owned(),
            };
            let allowed = exceptions.get(&key).is_some_and(|rule| {
                let path = string_field(row, "path").unwrap_or_default();
                if !rule.allowed_paths.contains(path) {
                    return false;
                }
                let count = exception_counts.entry(key.clone()).or_default();
                *count += 1;
                *count <= rule.max_occurrences
            });
            if !allowed {
                findings.insert(Finding::new(
                    "gate_self_conformance_hermetic_violation",
                    observation_key(gate_name, row, "token"),
                    "gate production code uses a subprocess/network/clock/randomness primitive outside a path/count-scoped orchestrator exception",
                ));
            }
        }

        for row in array_field(gate, "policy_literal_observations") {
            let literal = string_field(row, "literal").unwrap_or("<missing-literal>");
            let key = ExceptionKey {
                gate: gate_name.to_owned(),
                code: "gate_self_conformance_policy_as_data_violation".to_owned(),
                token: literal.to_owned(),
            };
            if !policy_literal_exceptions.contains(&key) {
                findings.insert(Finding::new(
                    "gate_self_conformance_policy_as_data_violation",
                    observation_key(gate_name, row, "literal"),
                    "gate production code hardcodes an Oyatie product-shape path; move the repo shape into policy data",
                ));
            }
        }
    }
    findings
}

pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(evaluate_keyed(policy, observed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "hermetic_exceptions": [],
            "no_autofix_reason": {
                "green-gate": "Human review must choose the source-level remediation; no safe mechanical rewrite exists."
            }
        })
    }

    fn gate(name: &str) -> Value {
        json!({
            "name": name,
            "has_buck_gate": true,
            "has_buck_unittest": true,
            "workflow_registered": true,
            "has_autofix": false,
            "hermetic_observations": [],
            "policy_literal_observations": []
        })
    }

    fn codes(findings: &BTreeSet<Finding>) -> BTreeSet<String> {
        findings.iter().map(|f| f.code.clone()).collect()
    }

    #[test]
    fn green_gate_with_no_autofix_reason_passes() {
        let observed = json!({ "gates": [gate("green-gate")] });
        let report = evaluate(&policy(), &observed);
        assert_eq!(report.verdict, Verdict::Green, "{:#?}", report.findings);
    }

    #[test]
    fn raw_command_in_non_exception_gate_is_red() {
        let mut row = gate("bad-gate");
        row["has_autofix"] = json!(true);
        row["hermetic_observations"] = json!([{
            "gate": "bad-gate",
            "path": "ci/facade/bad/src/lib.rs",
            "line": 7,
            "token": "Command::new"
        }]);
        let observed = json!({ "gates": [row] });
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(codes(&findings).contains("gate_self_conformance_hermetic_violation"));
    }

    #[test]
    fn hermetic_exception_requires_cutover_metadata_and_allows_token() {
        let mut p = policy();
        p["hermetic_exceptions"] = json!([{
            "gate": "orchestrator-gate",
            "class": "orchestrator_regeneration_boundary",
            "reason": "This gate invokes owned generated-face tools during transition while the typed reconciler API lands.",
            "cutover_note": "Replace the subprocess edge with the typed reconciler API tracked by GH-777 before removing this exception.",
            "allowed_paths": ["ci/facade/orchestrator/src/lib.rs"],
            "max_occurrences": 1,
            "tokens": ["Command::new"]
        }]);
        p["no_autofix_reason"]["orchestrator-gate"] = json!(
            "The orchestrator compares generated outputs; remediation is source-data repair, not a safe rewrite."
        );
        let mut row = gate("orchestrator-gate");
        row["hermetic_observations"] = json!([{
            "gate": "orchestrator-gate",
            "path": "ci/facade/orchestrator/src/lib.rs",
            "line": 12,
            "token": "Command::new"
        }]);
        let report = evaluate(&p, &json!({ "gates": [row] }));
        assert_eq!(report.verdict, Verdict::Green, "{:#?}", report.findings);
    }

    #[test]
    fn scoped_hermetic_exception_blocks_extra_occurrence() {
        let mut p = policy();
        p["hermetic_exceptions"] = json!([{
            "gate": "orchestrator-gate",
            "class": "orchestrator_regeneration_boundary",
            "reason": "This gate invokes owned generated-face tools during transition while the typed reconciler API lands.",
            "cutover_note": "Replace the subprocess edge with the typed reconciler API tracked by GH-777 before removing this exception.",
            "allowed_paths": ["ci/facade/orchestrator/src/lib.rs"],
            "max_occurrences": 1,
            "tokens": ["Command::new"]
        }]);
        p["no_autofix_reason"]["orchestrator-gate"] = json!(
            "The orchestrator compares generated outputs; remediation is source-data repair, not a safe rewrite."
        );
        let mut row = gate("orchestrator-gate");
        row["hermetic_observations"] = json!([
            {
                "gate": "orchestrator-gate",
                "path": "ci/facade/orchestrator/src/lib.rs",
                "line": 12,
                "token": "Command::new"
            },
            {
                "gate": "orchestrator-gate",
                "path": "ci/facade/orchestrator/src/lib.rs",
                "line": 20,
                "token": "Command::new"
            }
        ]);
        let findings = evaluate_keyed(&p, &json!({ "gates": [row] }));
        assert!(codes(&findings).contains("gate_self_conformance_hermetic_violation"));
    }

    #[test]
    fn malformed_hermetic_exception_is_red() {
        let mut p = policy();
        p["hermetic_exceptions"] = json!([{
            "gate": "orchestrator-gate",
            "tokens": ["Command::new"]
        }]);
        let report = evaluate(&p, &json!({ "gates": [gate("green-gate")] }));
        assert!(
            codes(&report.findings).contains("gate_self_conformance_policy_exception_malformed")
        );
    }

    #[test]
    fn flag_only_gate_without_fix_or_reason_is_red() {
        let observed = json!({ "gates": [gate("flag-only-no-reason")] });
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(codes(&findings).contains("gate_self_conformance_automated_missing_fix_contract"));
    }

    #[test]
    fn fake_autofix_mention_without_policy_contract_is_red() {
        let mut row = gate("fake-autofix");
        row["has_autofix"] = json!(true);
        let findings = evaluate_keyed(&policy(), &json!({ "gates": [row] }));
        assert!(codes(&findings).contains("gate_self_conformance_automated_missing_fix_contract"));
    }

    #[test]
    fn hardcoded_product_shape_path_is_red() {
        let mut row = gate("bad-gate");
        row["has_autofix"] = json!(true);
        row["policy_literal_observations"] = json!([{
            "gate": "bad-gate",
            "path": "ci/facade/bad/src/lib.rs",
            "line": 3,
            "literal": "oya/payroll/crates/oya-payroll-api"
        }]);
        let findings = evaluate_keyed(&policy(), &json!({ "gates": [row] }));
        assert!(codes(&findings).contains("gate_self_conformance_policy_as_data_violation"));
    }

    #[test]
    fn source_scanner_detects_raw_command_new() {
        let mut rows = Vec::new();
        hermetic_token_observations(
            "bad-gate",
            "platform/gates/bad/src/lib.rs",
            "fn run() { Command::new(\"git\"); }",
            &mut rows,
        );
        let mut row = gate("bad-gate");
        row["has_autofix"] = json!(true);
        row["hermetic_observations"] = json!(rows);
        let findings = evaluate_keyed(&policy(), &json!({ "gates": [row] }));
        assert!(codes(&findings).contains("gate_self_conformance_hermetic_violation"));
    }

    #[test]
    fn policy_literal_scanner_is_policy_driven_not_oyatie_hardcoded() {
        let rules = PolicyLiteralRules {
            allowed_prefixes: vec!["platform/gates/".to_owned()],
            forbidden_prefixes: vec!["acme/".to_owned()],
            forbidden_contains: vec!["/acme/".to_owned()],
        };
        let mut rows = Vec::new();
        policy_literal_observations(
            "bad-gate",
            "platform/gates/bad/src/lib.rs",
            "const PRODUCT_PATH: &str = \"acme/payroll/crates/payroll-api\";",
            &rules,
            &mut rows,
        );
        assert_eq!(rows.len(), 1, "policy-driven acme/ prefix must be detected");
        let mut row = gate("bad-gate");
        row["has_autofix"] = json!(true);
        row["policy_literal_observations"] = json!(rows);
        let findings = evaluate_keyed(&policy(), &json!({ "gates": [row] }));
        assert!(codes(&findings).contains("gate_self_conformance_policy_as_data_violation"));
    }

    #[test]
    fn comments_are_not_production_code_in_either_direction() {
        let rules = PolicyLiteralRules {
            allowed_prefixes: vec![],
            forbidden_prefixes: vec!["acme/".to_owned()],
            forbidden_contains: vec![],
        };

        // A doc comment that *quotes* a path documents the shape; it hardcodes nothing.
        let mut rows = Vec::new();
        policy_literal_observations(
            "documented-gate",
            "platform/gates/documented/src/lib.rs",
            "/// Degenerate shapes such as \"acme/\" now yield `None`.\n\
             /* Block form: \"acme/payroll\" was the old bogus prefix. */\n\
             fn f() {}",
            &rules,
            &mut rows,
        );
        assert_eq!(
            rows,
            Vec::<Value>::new(),
            "commented-out paths are documentation, not a hardcoded product shape"
        );

        // The same blindness the other way: a real literal on a line that also carries a
        // comment is still production code and must still be caught.
        let mut rows = Vec::new();
        policy_literal_observations(
            "mixed-gate",
            "platform/gates/mixed/src/lib.rs",
            "const P: &str = \"acme/payroll\"; // \"acme/ignored\"",
            &rules,
            &mut rows,
        );
        assert_eq!(rows.len(), 1, "code before a trailing comment still counts");
        assert_eq!(rows[0]["literal"], json!("acme/payroll"));

        // A `//` inside a literal must not open a comment — truncating there loses the tail.
        let mut rows = Vec::new();
        policy_literal_observations(
            "url-gate",
            "platform/gates/url/src/lib.rs",
            "const U: &str = \"https://example.test/acme/payroll\";",
            &rules,
            &mut rows,
        );
        assert_eq!(
            rows.len(),
            0,
            "the scheme's // must not truncate the literal (this one is not acme/-prefixed)"
        );
        let literals =
            string_literals_with_lines("const U: &str = \"https://example.test/acme/payroll\";");
        assert_eq!(
            literals,
            vec![(1, "https://example.test/acme/payroll".to_owned())],
            "the literal must survive its own // intact"
        );
    }

    #[test]
    fn a_commented_out_autofix_flag_does_not_satisfy_the_contract() {
        assert!(
            !has_autofix_contract(&["// TODO: support \"--fix\" one day".to_owned()]),
            "a commented-out flag is an intention, not a remediation surface"
        );
        assert!(
            has_autofix_contract(&["const ARG: &str = \"--fix\";".to_owned()]),
            "a real flag literal still declares the contract"
        );
    }

    #[test]
    fn buck2_and_workflow_wiring_are_required() {
        let mut row = gate("missing-wiring");
        row["has_buck_gate"] = json!(false);
        row["has_buck_unittest"] = json!(false);
        row["workflow_registered"] = json!(false);
        row["has_autofix"] = json!(true);
        let findings = evaluate_keyed(&policy(), &json!({ "gates": [row] }));
        let codes = codes(&findings);
        assert!(codes.contains("gate_self_conformance_buck2_missing_gate_target"));
        assert!(codes.contains("gate_self_conformance_buck2_missing_unittest_target"));
        assert!(codes.contains("gate_self_conformance_workflow_unregistered"));
    }

    #[test]
    fn workflow_registration_ignores_comments() {
        let workflow = r#"
jobs:
  gate:
    strategy:
      matrix:
        include:
          # - { crate: fake-gate, label: "comment only" }
          - { crate: real-gate, label: "real" }
"#;
        assert!(workflow_registers_gate(
            workflow,
            "real-gate",
            "platform/gates"
        ));
        assert!(!workflow_registers_gate(
            workflow,
            "fake-gate",
            "platform/gates"
        ));
    }

    #[test]
    fn production_slice_trims_inline_tests_before_scanning() {
        let text = "fn production() { Command::new(\"x\"); }\n#[cfg(test)]\nmod tests { fn t() { Command::new(\"fixture\"); } }";
        let sliced = production_rust_slice(text);
        assert!(sliced.contains("production"));
        assert!(!sliced.contains("fixture"));
    }
}
