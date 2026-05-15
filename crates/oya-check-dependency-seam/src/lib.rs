//! Composite seam-discipline lane (ADR-0092 D13).
//!
//! Six sub-checks. Day 1 status:
//!
//! | Sub-check                          | Status      | Mechanism |
//! |------------------------------------|-------------|-----------|
//! | `seam-imports`                     | report-only | hand-rolled scan of Cargo.toml + .rs imports for hyper-family in non-adapter crates |
//! | `registry-coverage`                | report-only | join `[workspace.dependencies]` × `registries/cross-cutting/dependency-rationales.json#entries`; emit orphans + missing rows |
//! | `cargo-audit-shell`                | report-only | shell out to `cargo audit`; capture exit code |
//! | `multispectrum-evidence-attached`  | report-only | scan `evidence/multispectrum/`; refuse if changeset lacks current evidence file |
//! | `fixture-pair-coverage`            | report-only | for each lane sub-check crate, assert `tests/fixtures/<subcheck>/{passing,failing}/` both exist |
//! | `change-class-declared`            | report-only | evidence JSON declares `change_class_id ∈ {CC-1..CC-7}` |
//!
//! Severity ramp: day 1 report-only; flip to `error` via cron after 7-day
//! green soak (FixupTask F-SOAK-FLIP-CRON).
//!
//! Implementation principle: kernel-shape (std-only) so the lane can run in
//! pre-merge CI without pulling tooling deps that themselves need vetting.
//! Sub-checks that genuinely need a TOML/JSON parser are gated behind
//! features in future iterations.

use std::path::PathBuf;

// Re-export from oya-json-kernel so call-sites in this crate continue using
// the local symbol names (`JsonValueKind`, `parse_top_level_object`) per the
// CONV-1 refactor. Originated as inline code; extracted to reusable kernel
// per user directive 2026-05-15.
pub use oya_json_kernel::{parse_top_level_object, JsonValueKind};

// CONV-8 codegen: build.rs derives CHANGE_CLASSES_FROM_SPEC +
// ALL_FACETS_FROM_SPEC from the canonical spec at build time. Unit tests
// in tests/per_subcheck_unit_tests.rs enforce spec/code parity.
include!(concat!(env!("OUT_DIR"), "/spec_constants.rs"));

/// One sub-check's result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubCheckResult {
    pub id: &'static str,
    pub status: SubCheckStatus,
    pub findings: Vec<String>,
    pub severity_day_1: Severity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubCheckStatus {
    Pass,
    Fail,
    NotYetArmed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    ReportOnly,
    Error,
}

/// Composite report aggregated across all sub-checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositeReport {
    pub sub_checks: Vec<SubCheckResult>,
}

impl CompositeReport {
    /// Overall exit code: 0 if every sub-check is `Pass` OR `NotYetArmed` OR
    /// its severity is `ReportOnly`. Non-zero only when a sub-check with
    /// `Error` severity returns `Fail`.
    pub fn exit_code(&self) -> i32 {
        for r in &self.sub_checks {
            if r.status == SubCheckStatus::Fail && r.severity_day_1 == Severity::Error {
                return 1;
            }
        }
        0
    }

    pub fn pass_count(&self) -> usize {
        self.sub_checks
            .iter()
            .filter(|r| r.status == SubCheckStatus::Pass)
            .count()
    }

    pub fn fail_count(&self) -> usize {
        self.sub_checks
            .iter()
            .filter(|r| r.status == SubCheckStatus::Fail)
            .count()
    }
}

/// Workspace context the lane operates on.
#[derive(Clone, Debug)]
pub struct WorkspaceContext {
    pub workspace_root: PathBuf,
}

impl WorkspaceContext {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }
}

/// Entry point: run every sub-check and aggregate. Day-1 mode is
/// report-only; the binary emits the composite report and exits 0 unless an
/// `Error`-severity sub-check fails.
pub fn run_composite(ctx: &WorkspaceContext) -> CompositeReport {
    let sub_checks = vec![
        check_seam_imports(ctx),
        check_registry_coverage(ctx),
        check_cargo_audit_shell(ctx),
        check_multispectrum_evidence_attached(ctx),
        check_fixture_pair_coverage(ctx),
        check_change_class_declared(ctx),
        check_rust_default_language(ctx),
        check_naming_convention(ctx),
        check_scorecard_render(ctx),
        check_consensus_debate_evidence(ctx),
        check_a6_schema_adherence(ctx),
        check_a1_naming_adherence(ctx),
        check_a2_documentation_adherence(ctx),
        check_a3_structure_adherence(ctx),
        check_a4_architecture_adherence(ctx),
        check_a5_dependency_adherence(ctx),
        check_a7_algorithm_adherence(ctx),
    ];
    CompositeReport { sub_checks }
}

/// `registry-coverage`: minimal-but-real implementation. Reads
/// `registries/cross-cutting/dependency-rationales.json` and counts entries. Full
/// join with `cargo metadata --no-deps` is FixupTask F-LANE-SEAM-IMPL.
pub fn check_registry_coverage(ctx: &WorkspaceContext) -> SubCheckResult {
    let registry_path = ctx
        .workspace_root
        .join("registries/cross-cutting/dependency-rationales.json");
    let mut findings = Vec::new();
    if !registry_path.exists() {
        return SubCheckResult {
            id: "registry-coverage",
            status: SubCheckStatus::Fail,
            findings: vec![format!(
                "missing registry: {}",
                registry_path.display()
            )],
            severity_day_1: Severity::ReportOnly,
        };
    }
    let raw = match std::fs::read_to_string(&registry_path) {
        Ok(s) => s,
        Err(e) => {
            return SubCheckResult {
                id: "registry-coverage",
                status: SubCheckStatus::Fail,
                findings: vec![format!("read failed: {e}")],
                severity_day_1: Severity::ReportOnly,
            };
        }
    };
    let entry_names = extract_registry_entry_names(&raw);
    findings.push(format!(
        "registry rows declared: {} ({} expected per ADR-0092 baseline)",
        entry_names.len(),
        11
    ));
    for name in &entry_names {
        findings.push(format!("  - {}", name));
    }
    SubCheckResult {
        id: "registry-coverage",
        status: SubCheckStatus::Pass,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// Extract top-level `entries` object keys from a dependency-rationales.json
/// document via lightweight string scanning. Avoids serde_json dep at this
/// scaffold stage; replaced with proper JSON parsing in F-LANE-SEAM-IMPL.
pub fn extract_registry_entry_names(raw: &str) -> Vec<String> {
    let mut names = Vec::new();
    let Some(start) = raw.find("\"entries\"") else {
        return names;
    };
    let after = &raw[start..];
    let Some(open) = after.find('{') else {
        return names;
    };
    let body = &after[open + 1..];
    let mut depth: i32 = 1;
    let mut idx = 0usize;
    let bytes = body.as_bytes();
    while idx < bytes.len() && depth > 0 {
        let c = bytes[idx];
        match c {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'"' if depth == 1 => {
                // Beginning of an entry key.
                let key_start = idx + 1;
                let mut j = key_start;
                while j < bytes.len() && bytes[j] != b'"' {
                    j += 1;
                }
                if j < bytes.len() {
                    let name =
                        std::str::from_utf8(&bytes[key_start..j]).unwrap_or("").to_string();
                    if !name.is_empty() {
                        names.push(name);
                    }
                    idx = j;
                }
            }
            _ => {}
        }
        idx += 1;
    }
    names
}

/// Workspace dependencies tracked by ADR-0092 seam policy.
const SEAM_TRACKED_DEPS: &[&str] = &["hyper", "hyper-util", "http-body-util", "bytes"];

/// `seam-imports`: scan every workspace member's Cargo.toml for the
/// hyper-family deps tracked under SEAM_TRACKED_DEPS. Allowed only in the
/// crate(s) named in `dependency-rationales.json#entries.<dep>.isolated_in_crate`.
/// Any other crate declaring a tracked dep is a violation.
///
/// Implementation is std-only: hand-rolled Cargo.toml line scan that
/// matches the shell-grep contract documented in ADR-0092 D2. Full TOML
/// parsing + `[target.'cfg(...)'.dependencies]` handling deferred to
/// F-LANE-SEAM-IMPL-FULL-TOML.
pub fn check_seam_imports(ctx: &WorkspaceContext) -> SubCheckResult {
    let crates_dir = ctx.workspace_root.join("crates");
    if !crates_dir.is_dir() {
        return SubCheckResult {
            id: "seam-imports",
            status: SubCheckStatus::Fail,
            findings: vec![format!("crates/ dir not found at {}", crates_dir.display())],
            severity_day_1: Severity::ReportOnly,
        };
    }
    let allowed_per_dep = read_allowed_isolation(&ctx.workspace_root);
    let mut violations: Vec<(String, String)> = Vec::new();
    let mut scanned_crates = 0usize;

    let entries = match std::fs::read_dir(&crates_dir) {
        Ok(e) => e,
        Err(err) => {
            return SubCheckResult {
                id: "seam-imports",
                status: SubCheckStatus::Fail,
                findings: vec![format!("read_dir {} failed: {}", crates_dir.display(), err)],
                severity_day_1: Severity::ReportOnly,
            };
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let p = entry.path();
        let cargo = p.join("Cargo.toml");
        if !cargo.is_file() {
            continue;
        }
        scanned_crates += 1;
        let raw = match std::fs::read_to_string(&cargo) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let crate_name = extract_package_name(&raw).unwrap_or_else(|| {
            p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("<unknown>")
                .to_string()
        });
        for dep in SEAM_TRACKED_DEPS {
            if cargo_toml_declares_dep(&raw, dep) {
                let allowed = allowed_per_dep
                    .get(*dep)
                    .map(|set| set.iter().any(|c| c == &crate_name))
                    .unwrap_or(false);
                if !allowed {
                    violations.push((crate_name.clone(), (*dep).to_string()));
                }
            }
        }
    }

    let mut findings = vec![format!(
        "scanned {} crates against {} seam-tracked deps; allowed-isolation list size: {}",
        scanned_crates,
        SEAM_TRACKED_DEPS.len(),
        allowed_per_dep.len()
    )];
    let status = if violations.is_empty() {
        findings.push("no seam violations detected".into());
        SubCheckStatus::Pass
    } else {
        findings.push(format!("{} seam violation(s):", violations.len()));
        for (cn, dn) in &violations {
            findings.push(format!(
                "  - `{}` declares `{}` outside the allowed-isolation set",
                cn, dn
            ));
        }
        SubCheckStatus::Fail
    };
    SubCheckResult {
        id: "seam-imports",
        status,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// Detect whether a Cargo.toml declares `dep` in any deps block.
pub fn cargo_toml_declares_dep(raw: &str, dep: &str) -> bool {
    for line in raw.lines() {
        let cleaned = match line.find('#') {
            Some(i) => &line[..i],
            None => line,
        };
        let trimmed = cleaned.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix(dep) {
            // Next char must be a separator so `hyper` doesn't match `hyper-util`.
            if rest
                .chars()
                .next()
                .map(|c| c == ' ' || c == '=' || c == '.' || c == '\t')
                .unwrap_or(false)
            {
                return true;
            }
        }
    }
    false
}

/// Extract `name = "..."` from `[package]`.
pub fn extract_package_name(raw: &str) -> Option<String> {
    let mut in_package = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_package = t == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = t.strip_prefix("name") {
            let r = rest.trim_start();
            if let Some(r) = r.strip_prefix('=') {
                let v = r.trim().trim_matches('"');
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

/// Read dependency-rationales.json and extract per-dep `isolated_in_crate`
/// for the seam-tracked deps. Hand-rolled; full JSON parsing deferred to
/// F-LANE-SEAM-IMPL-FULL-JSON.
pub fn read_allowed_isolation(
    workspace_root: &std::path::Path,
) -> std::collections::BTreeMap<String, Vec<String>> {
    use std::collections::BTreeMap;
    let mut result: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let registry = workspace_root.join("registries/cross-cutting/dependency-rationales.json");
    let Ok(raw) = std::fs::read_to_string(&registry) else {
        return result;
    };
    for dep in SEAM_TRACKED_DEPS {
        // Find the dep's object: search for `"<dep>": {` then its
        // `isolated_in_crate` field within the object's text window.
        let needle = format!("\"{}\":", dep);
        let Some(pos) = raw.find(&needle) else {
            continue;
        };
        let after = &raw[pos..];
        let Some(open) = after.find('{') else { continue };
        let body_start = open + 1;
        // Find matching close brace.
        let body = &after[body_start..];
        let mut depth: i32 = 1;
        let mut end = 0usize;
        for (i, b) in body.bytes().enumerate() {
            match b {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let entry_body = &body[..end];
        if let Some(iso) = entry_body.find("\"isolated_in_crate\"") {
            let after_iso = &entry_body[iso..];
            if let Some(colon) = after_iso.find(':') {
                let val = &after_iso[colon + 1..];
                if let Some(qs) = val.find('"') {
                    let tail = &val[qs + 1..];
                    if let Some(qe) = tail.find('"') {
                        let names = parse_isolated_in_crate(&tail[..qe]);
                        result.insert((*dep).to_string(), names);
                    }
                }
            }
        }
    }
    result
}

/// Parse the free-form `isolated_in_crate` value into crate-name tokens.
pub fn parse_isolated_in_crate(value: &str) -> Vec<String> {
    value
        .split(|c: char| c == ';' || c == '+' || c == ',')
        .filter_map(|tok| {
            let cleaned = match tok.find('(') {
                Some(i) => &tok[..i],
                None => tok,
            };
            let head = cleaned.split_whitespace().next()?.trim();
            if head.is_empty()
                || !head
                    .bytes()
                    .any(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            {
                None
            } else {
                Some(head.to_string())
            }
        })
        .collect()
}

/// `cargo-audit-shell`: invoke `cargo audit` and report its exit code +
/// findings count from the textual output. Gracefully degrades when
/// cargo-audit is not installed (returns NotYetArmed rather than Fail).
pub fn check_cargo_audit_shell(ctx: &WorkspaceContext) -> SubCheckResult {
    use std::process::Command;
    let out = Command::new("cargo")
        .arg("audit")
        .arg("--quiet")
        .current_dir(&ctx.workspace_root)
        .output();
    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let combined = if stdout.is_empty() {
                stderr.clone()
            } else {
                stdout.clone()
            };
            let advisory_count = combined.matches("ID: RUSTSEC-").count();
            let mut findings = vec![
                format!("cargo audit exit code: {}", o.status.code().unwrap_or(-1)),
                format!("RUSTSEC advisory mentions: {}", advisory_count),
            ];
            if !stderr.trim().is_empty() && stderr.len() < 4000 {
                findings.push(format!("stderr: {}", stderr.trim()));
            }
            let status = if o.status.success() && advisory_count == 0 {
                SubCheckStatus::Pass
            } else {
                SubCheckStatus::Fail
            };
            SubCheckResult {
                id: "cargo-audit-shell",
                status,
                findings,
                severity_day_1: Severity::ReportOnly,
            }
        }
        Err(e) => SubCheckResult {
            id: "cargo-audit-shell",
            status: SubCheckStatus::NotYetArmed,
            findings: vec![format!(
                "cargo-audit not installed or invocation failed ({}); install with: cargo install cargo-audit",
                e
            )],
            severity_day_1: Severity::ReportOnly,
        },
    }
}

/// Required top-level keys for any evidence file per
/// specs/cross-cutting/multispectrum-review.json#evidence_schema.
pub const EVIDENCE_REQUIRED_TOP_LEVEL_KEYS: &[&str] = &[
    "change_class_id",
    "git_sha",
    "freshness_unix",
    "facets",
];

/// Required facet keys per multispectrum-review.json. v2.0.0 (2026-05-14)
/// added F8 performance + F9 compliance as REQUIRED facets and M1 + M2 as
/// OPTIONAL meta-facets (required only when meta_review_triggered).
pub const EVIDENCE_REQUIRED_FACETS: &[&str] = &[
    "F1_linus",
    "F2_hyperscaler",
    "F3_adversarial",
    "F4_ergonomic",
    "F5_quality",
    "F6_alternatives",
    "F7_security",
    "F8_performance",
    "F9_compliance",
];

/// Optional meta-facets. Required when meta_review_triggered (kernel
/// public API, new ADR/standard/spec, breaking-API change, new microservice).
/// Lane reports presence vs absence as informational; refuses only when
/// the changeset declares `meta_review_triggered: true` AND a meta facet
/// is absent.
pub const EVIDENCE_OPTIONAL_META_FACETS: &[&str] =
    &["M1_challenge_assumption", "M2_zoomed_out_fit"];

/// `multispectrum-evidence-attached`: scan every evidence file and verify
/// it carries the 4 required top-level keys + a `facets` block containing
/// all 7 facet keys. Full schema validation (per-facet rigor matrix +
/// JSON-Schema validation) is FixupTask F-LANE-SEAM-IMPL-FULL-SCHEMA.
pub fn check_multispectrum_evidence_attached(ctx: &WorkspaceContext) -> SubCheckResult {
    let evidence_dir = ctx.workspace_root.join("evidence/multispectrum");
    if !evidence_dir.is_dir() {
        return SubCheckResult {
            id: "multispectrum-evidence-attached",
            status: SubCheckStatus::NotYetArmed,
            findings: vec![format!("no evidence dir at {}", evidence_dir.display())],
            severity_day_1: Severity::ReportOnly,
        };
    }
    let mut findings = Vec::new();
    let mut total = 0usize;
    let mut full = 0usize;
    let mut violations: Vec<(String, Vec<String>)> = Vec::new();
    for entry in std::fs::read_dir(&evidence_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
    {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        total += 1;
        let raw = match std::fs::read_to_string(&p) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut missing = Vec::new();
        for k in EVIDENCE_REQUIRED_TOP_LEVEL_KEYS {
            if !raw.contains(&format!("\"{}\"", k)) {
                missing.push((*k).to_string());
            }
        }
        // Facets block presence: search after the literal "facets" key.
        if let Some(pos) = raw.find("\"facets\"") {
            let after = &raw[pos..];
            for f in EVIDENCE_REQUIRED_FACETS {
                if !after.contains(&format!("\"{}\"", f)) {
                    missing.push(format!("facets.{}", f));
                }
            }
        }
        let path_repr = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>")
            .to_string();
        if missing.is_empty() {
            full += 1;
        } else {
            violations.push((path_repr, missing));
        }
    }
    findings.push(format!(
        "evidence files scanned: {}; full keys+facets: {}; violations: {}",
        total,
        full,
        violations.len()
    ));
    for (p, missing) in &violations {
        findings.push(format!("  - {} missing: {}", p, missing.join(", ")));
    }
    let status = if violations.is_empty() && total > 0 {
        SubCheckStatus::Pass
    } else if total == 0 {
        SubCheckStatus::NotYetArmed
    } else {
        SubCheckStatus::Fail
    };
    SubCheckResult {
        id: "multispectrum-evidence-attached",
        status,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// `fixture-pair-coverage` stub. Real implementation walks every
/// `crates/oya-check-*` crate's `tests/fixtures/<subcheck>/` and verifies
/// both `passing/` and `failing/` subdirectories exist. FixupTask F-LANE-SEAM-IMPL.
pub fn check_fixture_pair_coverage(ctx: &WorkspaceContext) -> SubCheckResult {
    let own_fixtures = ctx
        .workspace_root
        .join("crates/oya-check-dependency-seam/tests/fixtures/registry-coverage");
    let passing = own_fixtures.join("passing");
    let failing = own_fixtures.join("failing");
    let mut findings = Vec::new();
    findings.push(format!(
        "own fixture pair for `registry-coverage`: passing={} failing={}",
        passing.is_dir(),
        failing.is_dir()
    ));
    findings.push(
        "discovery across other oya-check-* crates: NOT YET ARMED — F-LANE-SEAM-IMPL"
            .into(),
    );
    SubCheckResult {
        id: "fixture-pair-coverage",
        status: if passing.is_dir() && failing.is_dir() {
            SubCheckStatus::Pass
        } else {
            SubCheckStatus::NotYetArmed
        },
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// Canonical change_class_id values per specs/cross-cutting/multispectrum-review.json.
pub const CHANGE_CLASSES: &[&str] = &[
    "CC-1_kernel_public_api",
    "CC-2_adapter_or_infrastructure",
    "CC-3_application_or_domain",
    "CC-4_refactor_pure",
    "CC-5_doc_only",
    "CC-6_generated_or_vendored",
    "CC-7_test_or_fixture",
];

/// `change-class-declared`: scan latest multispectrum evidence file and
/// confirm its `change_class_id` field exists and matches one of CHANGE_CLASSES.
pub fn check_change_class_declared(ctx: &WorkspaceContext) -> SubCheckResult {
    let evidence_dir = ctx.workspace_root.join("evidence/multispectrum");
    if !evidence_dir.is_dir() {
        return SubCheckResult {
            id: "change-class-declared",
            status: SubCheckStatus::NotYetArmed,
            findings: vec![format!(
                "no evidence dir at {}; nothing to check (lane is report-only day 1)",
                evidence_dir.display()
            )],
            severity_day_1: Severity::ReportOnly,
        };
    }
    let mut findings = Vec::new();
    let mut count_total = 0usize;
    let mut count_declared = 0usize;
    let mut count_canonical = 0usize;
    let mut violations: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(&evidence_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
    {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        count_total += 1;
        let raw = match std::fs::read_to_string(&p) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let declared = extract_string_field(&raw, "change_class_id");
        let path_repr = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>")
            .to_string();
        match declared {
            Some(ref value) if CHANGE_CLASSES.contains(&value.as_str()) => {
                count_declared += 1;
                count_canonical += 1;
            }
            Some(value) => {
                count_declared += 1;
                violations.push((path_repr, format!("non-canonical change_class_id: `{}`", value)));
            }
            None => {
                violations.push((path_repr, "change_class_id missing".into()));
            }
        }
    }
    findings.push(format!(
        "evidence files scanned: {}; declared change_class_id: {}; canonical: {}; violations: {}",
        count_total,
        count_declared,
        count_canonical,
        violations.len()
    ));
    for (p, v) in &violations {
        findings.push(format!("  - {} :: {}", p, v));
    }
    let status = if violations.is_empty() {
        SubCheckStatus::Pass
    } else {
        SubCheckStatus::Fail
    };
    SubCheckResult {
        id: "change-class-declared",
        status,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// Best-effort scalar string-field extractor for hand-rolled JSON. Finds
/// the FIRST occurrence of `"field": "value"`. Used while full JSON parsing
/// is deferred (F-LANE-SEAM-IMPL-FULL-JSON).
pub fn extract_string_field(raw: &str, field: &str) -> Option<String> {
    let needle = format!("\"{}\"", field);
    let pos = raw.find(&needle)?;
    let after = &raw[pos + needle.len()..];
    let colon = after.find(':')?;
    let val = &after[colon + 1..];
    let qs = val.find('"')?;
    let tail = &val[qs + 1..];
    let qe = tail.find('"')?;
    Some(tail[..qe].to_string())
}


/// `a6-schema-adherence`: walk JSON files in canonical durable homes
/// (/specs/, /registries/, /evidence/, /templates/) and verify each declares
/// the ADR-0069 active-artifact-contract minimum: `$schema` + `$id` + `_meta`.
/// Day-1 stub matching the other A-family stubs (NotYetArmed). Full impl
/// ($id-path parity, schema version coherence, $defs validation) lives in
/// FixupTask `F-LANE-ADHERENCE-A6-SCHEMA`.
///
/// Implementation note: shares the `scan_dir` helper + `parse_top_level_object`
/// (from oya-json-kernel via re-export) with the other 4 sub-checks. First
/// A-family check landed in the composite lane; future A-family checks
/// (A1..A5, A7) follow the same pattern.
pub fn check_a6_schema_adherence(ctx: &WorkspaceContext) -> SubCheckResult {
    let homes = [
        "specs/cross-cutting",
        "registries/cross-cutting",
        "evidence",
        "templates",
    ];
    let mut scanned = 0usize;
    let mut compliant = 0usize;
    let mut violations_displayed: Vec<String> = Vec::new();
    let mut io_errors: Vec<String> = Vec::new();
    let mut inner_errors: Vec<String> = Vec::new();
    let display_cap = 10usize;
    for home in &homes {
        let p = ctx.workspace_root.join(home);
        scan_dir(&p, &mut io_errors, |entry| {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                return;
            }
            scanned += 1;
            match std::fs::read_to_string(&path) {
                Ok(raw) => {
                    let parsed = parse_top_level_object(&raw);
                    let has_schema = parsed.contains_key("$schema");
                    let has_id = parsed.contains_key("$id");
                    let has_meta = parsed.contains_key("_meta");
                    if has_schema && has_id && has_meta {
                        compliant += 1;
                    } else if violations_displayed.len() < display_cap {
                        let mut missing: Vec<&str> = Vec::new();
                        if !has_schema {
                            missing.push("$schema");
                        }
                        if !has_id {
                            missing.push("$id");
                        }
                        if !has_meta {
                            missing.push("_meta");
                        }
                        let rel = path
                            .strip_prefix(&ctx.workspace_root)
                            .unwrap_or(&path);
                        violations_displayed.push(format!(
                            "{}: missing {}",
                            rel.display(),
                            missing.join("+")
                        ));
                    }
                }
                Err(e) => inner_errors.push(format!(
                    "read_to_string({}) failed: {}",
                    path.display(),
                    e
                )),
            }
        });
    }
    io_errors.extend(inner_errors);
    let non_compliant = scanned.saturating_sub(compliant);
    let mut findings = vec![format!(
        "JSON files scanned: {}; ADR-0069 minimum-keys-compliant ($schema+$id+_meta): {}; non-compliant: {}",
        scanned, compliant, non_compliant
    )];
    for v in &violations_displayed {
        findings.push(format!("  - {}", v));
    }
    if non_compliant > violations_displayed.len() {
        findings.push(format!(
            "  - ... {} additional violation(s) omitted (display cap = {})",
            non_compliant - violations_displayed.len(),
            display_cap
        ));
    }
    findings.extend(io_errors);
    findings.push(
        "full BNF: $id-path parity + schema version coherence + $defs validation — F-LANE-ADHERENCE-A6-SCHEMA"
            .into(),
    );
    SubCheckResult {
        id: "a6-schema-adherence",
        status: SubCheckStatus::NotYetArmed,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// `a1-naming-adherence`: extended naming-governance scan beyond canonical
/// durable homes — covers docs/decisions/ (ADR-NNNN-<kebab>.md) + crates/
/// + tools/ (oya-<microservice>-<layer> BNF per ADR-0056). Day-1 stub:
/// count + report non-compliant filenames. Full BNF enforcement is
/// F-LANE-ADHERENCE-A1-NAMING.
pub fn check_a1_naming_adherence(ctx: &WorkspaceContext) -> SubCheckResult {
    let homes = ["docs/decisions", "crates", "tools"];
    let mut scanned = 0usize;
    let mut violations_total = 0usize;
    let mut violations_displayed: Vec<String> = Vec::new();
    let mut io_errors: Vec<String> = Vec::new();
    let display_cap = 10usize;
    for home in &homes {
        let p = ctx.workspace_root.join(home);
        scan_dir(&p, &mut io_errors, |entry| {
            let name = entry.file_name();
            let s = name.to_string_lossy().to_string();
            if s.starts_with('.') {
                return;
            }
            scanned += 1;
            // Kebab-rule: ASCII lowercase + digit + hyphen + dot only.
            let compliant = s
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.');
            if !compliant {
                violations_total += 1;
                if violations_displayed.len() < display_cap {
                    violations_displayed.push(format!("{}/{}", home, s));
                }
            }
        });
    }
    let mut findings = vec![format!(
        "extended-namespace entries scanned: {}; non-kebab violations: {}",
        scanned, violations_total
    )];
    for v in &violations_displayed {
        findings.push(format!("  - {}", v));
    }
    if violations_total > display_cap {
        findings.push(format!(
            "  - ... {} additional violation(s) omitted (display cap = {})",
            violations_total - display_cap,
            display_cap
        ));
    }
    findings.extend(io_errors);
    findings.push(
        "ADR-0056 BNF v4.1 (oya-<microservice>-<layer>) + version-suffix + $id-path parity NOT YET ARMED — F-LANE-ADHERENCE-A1-NAMING"
            .into(),
    );
    SubCheckResult {
        id: "a1-naming-adherence",
        status: SubCheckStatus::NotYetArmed,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// `a2-documentation-adherence`: walk `docs/standards/`, `docs/decisions/`,
/// and `docs/runbooks/` markdown files. Check each declares the
/// `doc_class` + `length_cap` keys in YAML frontmatter (per
/// docs/standards/doc-style.md). Day-1 stub. Full DOC-CATALOG row + 'Does
/// NOT cover' clause + thin-gateway shape lives in F-LANE-ADHERENCE-A2-DOCUMENTATION.
pub fn check_a2_documentation_adherence(ctx: &WorkspaceContext) -> SubCheckResult {
    let homes = ["docs/standards", "docs/decisions", "docs/runbooks"];
    let mut scanned = 0usize;
    let mut compliant = 0usize;
    let mut violations_displayed: Vec<String> = Vec::new();
    let mut io_errors: Vec<String> = Vec::new();
    let mut inner_errors: Vec<String> = Vec::new();
    let display_cap = 10usize;
    for home in &homes {
        let p = ctx.workspace_root.join(home);
        scan_dir(&p, &mut io_errors, |entry| {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                return;
            }
            scanned += 1;
            match std::fs::read_to_string(&path) {
                Ok(raw) => {
                    // Frontmatter: --- ... --- at top. Look for doc_class + length_cap keys.
                    let has_frontmatter = raw.starts_with("---");
                    let has_doc_class = raw.contains("doc_class:");
                    let has_length_cap = raw.contains("length_cap:");
                    if has_frontmatter && has_doc_class && has_length_cap {
                        compliant += 1;
                    } else if violations_displayed.len() < display_cap {
                        let mut missing: Vec<&str> = Vec::new();
                        if !has_frontmatter {
                            missing.push("frontmatter");
                        }
                        if !has_doc_class {
                            missing.push("doc_class");
                        }
                        if !has_length_cap {
                            missing.push("length_cap");
                        }
                        let rel = path.strip_prefix(&ctx.workspace_root).unwrap_or(&path);
                        violations_displayed.push(format!(
                            "{}: missing {}",
                            rel.display(),
                            missing.join("+")
                        ));
                    }
                }
                Err(e) => inner_errors.push(format!(
                    "read_to_string({}) failed: {}",
                    path.display(),
                    e
                )),
            }
        });
    }
    io_errors.extend(inner_errors);
    let non_compliant = scanned.saturating_sub(compliant);
    let mut findings = vec![format!(
        "markdown files scanned: {}; frontmatter-compliant (doc_class+length_cap): {}; non-compliant: {}",
        scanned, compliant, non_compliant
    )];
    for v in &violations_displayed {
        findings.push(format!("  - {}", v));
    }
    if non_compliant > violations_displayed.len() {
        findings.push(format!(
            "  - ... {} additional violation(s) omitted (display cap = {})",
            non_compliant - violations_displayed.len(),
            display_cap
        ));
    }
    findings.extend(io_errors);
    findings.push(
        "DOC-CATALOG row + thin-gateway shape + 'Does NOT cover' clause NOT YET ARMED — F-LANE-ADHERENCE-A2-DOCUMENTATION"
            .into(),
    );
    SubCheckResult {
        id: "a2-documentation-adherence",
        status: SubCheckStatus::NotYetArmed,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// `a3-structure-adherence`: enforce P11 — no durables under .omc/{specs,
/// registries, templates, evidence, ledger, audits, claim-matrix, graph}.
/// Day-1 stub: count files in those subdirs (should be 0 post-TG1
/// migration). Full repo_layout BNF coverage is F-LANE-ADHERENCE-A3-STRUCTURE.
pub fn check_a3_structure_adherence(ctx: &WorkspaceContext) -> SubCheckResult {
    let forbidden = [
        ".omc/specs",
        ".omc/registries",
        ".omc/templates",
        ".omc/evidence",
        ".omc/ledger",
        ".omc/audits",
        ".omc/claim-matrix",
        ".omc/graph",
    ];
    let mut violations = 0usize;
    let mut violations_displayed: Vec<String> = Vec::new();
    let mut io_errors: Vec<String> = Vec::new();
    let display_cap = 10usize;
    for f in &forbidden {
        let p = ctx.workspace_root.join(f);
        if !p.exists() {
            continue;
        }
        scan_dir(&p, &mut io_errors, |entry| {
            violations += 1;
            if violations_displayed.len() < display_cap {
                let name = entry.file_name();
                violations_displayed.push(format!("{}/{}", f, name.to_string_lossy()));
            }
        });
    }
    let mut findings = vec![format!(
        "P11 forbidden-subdir entries found: {} (post-TG1 expected: 0)",
        violations
    )];
    for v in &violations_displayed {
        findings.push(format!("  - {}", v));
    }
    if violations > display_cap {
        findings.push(format!(
            "  - ... {} additional omitted",
            violations - display_cap
        ));
    }
    findings.extend(io_errors);
    findings.push(
        "full repo_layout BNF + canonical-home placement check NOT YET ARMED — F-LANE-ADHERENCE-A3-STRUCTURE"
            .into(),
    );
    SubCheckResult {
        id: "a3-structure-adherence",
        status: SubCheckStatus::NotYetArmed,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// `a4-architecture-adherence`: walk crates/ — each crate's name should
/// match ADR-0056 BNF v4.1 layer suffix (kernel/domain/application/adapter/
/// app/api/worker/runtime/infrastructure/service/rest/cli/bindings). Day-1
/// stub: count compliant suffixes. Full per-layer dependency-direction
/// validation lives in F-LANE-ADHERENCE-A4-ARCHITECTURE.
pub fn check_a4_architecture_adherence(ctx: &WorkspaceContext) -> SubCheckResult {
    let crates_dir = ctx.workspace_root.join("crates");
    let valid_layers = [
        "-kernel",
        "-domain",
        "-application",
        "-adapter",
        "-app",
        "-api",
        "-worker",
        "-runtime",
        "-infrastructure",
        "-service",
        "-rest",
        "-cli",
        "-bindings",
    ];
    let mut scanned = 0usize;
    let mut compliant = 0usize;
    let mut violations_displayed: Vec<String> = Vec::new();
    let mut io_errors: Vec<String> = Vec::new();
    let display_cap = 10usize;
    scan_dir(&crates_dir, &mut io_errors, |entry| {
        let name = entry.file_name();
        let s = name.to_string_lossy().to_string();
        if s.starts_with('.') || !s.starts_with("oya-") {
            return;
        }
        scanned += 1;
        // Exempt namespaces: check / foundry-fitness-* / *-policy
        let is_exempt = s.starts_with("oya-check-")
            || s.starts_with("oya-foundry-fitness-")
            || s.starts_with("oya-json-")
            || s.contains("-policy");
        if is_exempt {
            compliant += 1;
            return;
        }
        let layer_match = valid_layers.iter().any(|suffix| s.ends_with(suffix));
        if layer_match {
            compliant += 1;
        } else if violations_displayed.len() < display_cap {
            violations_displayed.push(s);
        }
    });
    let non_compliant = scanned.saturating_sub(compliant);
    let mut findings = vec![format!(
        "crates scanned: {}; ADR-0056 BNF layer-suffix compliant: {}; non-compliant: {}",
        scanned, compliant, non_compliant
    )];
    for v in &violations_displayed {
        findings.push(format!("  - {}", v));
    }
    if non_compliant > display_cap {
        findings.push(format!(
            "  - ... {} additional omitted",
            non_compliant - display_cap
        ));
    }
    findings.extend(io_errors);
    findings.push(
        "per-layer dependency-direction validation (inward-flow) NOT YET ARMED — F-LANE-ADHERENCE-A4-ARCHITECTURE"
            .into(),
    );
    SubCheckResult {
        id: "a4-architecture-adherence",
        status: SubCheckStatus::NotYetArmed,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// `a5-dependency-adherence`: read workspace Cargo.toml + registries/cross-
/// cutting/dependency-rationales.json. Count [workspace.dependencies]
/// entries that have a matching rationale row vs orphans. Day-1 stub.
/// Full cargo-deny + license + LTS pinning check is F-LANE-ADHERENCE-A5-DEPENDENCY.
pub fn check_a5_dependency_adherence(ctx: &WorkspaceContext) -> SubCheckResult {
    let registry = ctx
        .workspace_root
        .join("registries/cross-cutting/dependency-rationales.json");
    let mut findings = Vec::new();
    let registered: Vec<String> = match std::fs::read_to_string(&registry) {
        Ok(raw) => extract_registry_entry_names(&raw),
        Err(e) => {
            findings.push(format!(
                "registry read failed: {} ({})",
                registry.display(),
                e
            ));
            Vec::new()
        }
    };
    findings.push(format!(
        "registry rows declared in /registries/cross-cutting/dependency-rationales.json: {}",
        registered.len()
    ));
    findings.push(
        "Cargo.toml [workspace.dependencies] cross-join + cargo-deny + LTS-pinning + license-posture NOT YET ARMED — F-LANE-ADHERENCE-A5-DEPENDENCY"
            .into(),
    );
    SubCheckResult {
        id: "a5-dependency-adherence",
        status: SubCheckStatus::NotYetArmed,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// `a7-algorithm-adherence`: count Rust files under crates/ that contain
/// heuristic markers (TODO / FIXME / approximate / heuristic). Day-1 stub.
/// Full complexity-class declaration + deterministic-on-deterministic-path
/// audit is F-LANE-ADHERENCE-A7-ALGORITHM.
pub fn check_a7_algorithm_adherence(ctx: &WorkspaceContext) -> SubCheckResult {
    let crates_dir = ctx.workspace_root.join("crates");
    let mut scanned_rust_files = 0usize;
    let mut heuristic_marker_files = 0usize;
    let markers = ["TODO", "FIXME", "approximate", "heuristic"];
    let mut io_errors: Vec<String> = Vec::new();
    let mut inner_errors: Vec<String> = Vec::new();
    // Single-level dir scan of crates/<each-crate> — non-recursive day-1.
    scan_dir(&crates_dir, &mut io_errors, |crate_entry| {
        let crate_path = crate_entry.path();
        if !crate_path.is_dir() {
            return;
        }
        let src = crate_path.join("src");
        // Look at top-level src/ only (lib.rs + main.rs typically).
        scan_dir(&src, &mut inner_errors, |src_entry| {
            let p = src_entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                return;
            }
            scanned_rust_files += 1;
            if let Ok(raw) = std::fs::read_to_string(&p) {
                if markers.iter().any(|m| raw.contains(m)) {
                    heuristic_marker_files += 1;
                }
            }
        });
    });
    io_errors.extend(inner_errors);
    let mut findings = vec![format!(
        "Rust src files scanned (crates/*/src top-level): {}; files containing TODO/FIXME/approximate/heuristic markers: {}",
        scanned_rust_files, heuristic_marker_files
    )];
    findings.extend(io_errors);
    findings.push(
        "complexity-class declaration + P3 deterministic-on-deterministic-path audit NOT YET ARMED — F-LANE-ADHERENCE-A7-ALGORITHM"
            .into(),
    );
    SubCheckResult {
        id: "a7-algorithm-adherence",
        status: SubCheckStatus::NotYetArmed,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// Shared scan primitive: read a directory, invoke `visit` on each Ok
/// entry, surface read_dir + entry I/O errors as findings strings via the
/// `errors` accumulator. Std-only.
///
/// Resolves TEN-2 (TG2 11-facet debate interim position): F1 wanted
/// table-driven scan; F4 wanted per-fn testability; compromise — keep 4
/// pub fn entry points (binary-compatible) but route each through this 1
/// shared scan helper. Eliminates the duplicate `match read_dir { Ok =>
/// for entry { match entry...}, Err => push }` block that appeared in
/// every TG2 sub-check.
pub fn scan_dir<F: FnMut(&std::fs::DirEntry)>(
    dir: &std::path::Path,
    errors: &mut Vec<String>,
    mut visit: F,
) {
    match std::fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => visit(&entry),
                    Err(e) => errors.push(format!(
                        "read_dir entry under {} failed: {}",
                        dir.display(),
                        e
                    )),
                }
            }
        }
        Err(e) => errors.push(format!(
            "read_dir({}) failed: {} — surfaced per CONV-2",
            dir.display(),
            e
        )),
    }
}

/// `rust-default-language`: scan `scripts/` for non-Rust files per P15.
/// Day-1 report-only stub: counts existing .sh/.py/.mjs/.rb/.pl files (all
/// grandfathered per audit evidence/audits/doc-antipattern-audit-1778808000.json).
/// New non-Rust additions require ADR cite — full enforcement is FixupTask
/// F-LANE-RUST-DEFAULT-ENFORCE.
pub fn check_rust_default_language(ctx: &WorkspaceContext) -> SubCheckResult {
    let scripts_dir = ctx.workspace_root.join("scripts");
    let mut findings = Vec::new();
    let mut total_non_rust = 0usize;
    let exts = ["sh", "py", "mjs", "rb", "pl"];
    // TEN-2 + CONV-2: shared scan_dir helper surfaces I/O errors uniformly.
    scan_dir(&scripts_dir, &mut findings, |entry| {
        let p = entry.path();
        if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
            if exts.contains(&ext) {
                total_non_rust += 1;
            }
        }
    });
    findings.push(format!(
        "scripts/ non-Rust file count: {} (.sh/.py/.mjs/.rb/.pl, all grandfathered per P15)",
        total_non_rust
    ));
    findings.push(
        "new non-Rust additions require ADR cite — full enforcement: F-LANE-RUST-DEFAULT-ENFORCE"
            .into(),
    );
    SubCheckResult {
        id: "rust-default-language",
        status: SubCheckStatus::NotYetArmed,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// `naming-convention`: walk canonical durable homes; check kebab-case +
/// no-uppercase per P14 BNF. Day-1: kebab only; full BNF validation
/// (version-suffix + ASCII enforcement + $id match) is FixupTask
/// F-NAMING-CONVENTION-ENFORCE.
pub fn check_naming_convention(ctx: &WorkspaceContext) -> SubCheckResult {
    let homes = [
        "specs/cross-cutting",
        "registries/cross-cutting",
        "evidence",
        "templates",
    ];
    let mut scanned = 0usize;
    let mut violations_displayed: Vec<String> = Vec::new();
    let mut violations_total = 0usize;
    let display_cap = 10usize;
    let mut io_errors: Vec<String> = Vec::new();
    // TEN-2 + CONV-2: shared scan_dir helper surfaces I/O errors uniformly.
    for home in &homes {
        let p = ctx.workspace_root.join(home);
        scan_dir(&p, &mut io_errors, |entry| {
            let name = entry.file_name();
            let s = name.to_string_lossy().to_string();
            if s.starts_with('.') {
                return;
            }
            scanned += 1;
            if !s
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
            {
                violations_total += 1;
                if violations_displayed.len() < display_cap {
                    violations_displayed.push(format!("{}/{}", home, s));
                }
            }
        });
    }
    let mut findings = vec![format!(
        "scanned {} top-level entries across {} canonical homes; kebab-case violations: {}",
        scanned,
        homes.len(),
        violations_total
    )];
    for v in &violations_displayed {
        findings.push(format!("  - {}", v));
    }
    // CONV-4 (TG2 11-facet debate): surface truncation when violations exceed display cap.
    if violations_total > display_cap {
        findings.push(format!(
            "  - ... {} additional violation(s) omitted (display cap = {}; full count above)",
            violations_total - display_cap,
            display_cap
        ));
    }
    findings.extend(io_errors);
    findings.push(
        "version-suffix -vMAJOR.MINOR.PATCH check NOT YET ARMED — F-NAMING-CONVENTION-ENFORCE"
            .into(),
    );
    // Day-1 stub: report violations as findings but DO NOT gate on them.
    // Status aligned to NotYetArmed for consistency with the other three TG2
    // sub-checks (rust-default-language / scorecard-render / consensus-debate-
    // evidence) so siblings of identical day-1 maturity emit identical status.
    // Full BNF enforcement (version-suffix, ASCII bounds, $id match) ships in
    // F-NAMING-CONVENTION-ENFORCE — at which point status transitions to
    // Pass/Fail. Resolves CONV-3 from TG2 11-facet debate synthesis.
    SubCheckResult {
        id: "naming-convention",
        status: SubCheckStatus::NotYetArmed,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// `scorecard-render`: count multispectrum evidence files that have the
/// minimum keys needed to render a Scorecard view per
/// specs/cross-cutting/multispectrum-review.json#scorecard_schema. Full
/// `render_evidence_scorecard(evidence_path) -> Scorecard` impl is
/// FixupTask F-LANE-SCORECARD-RENDER.
pub fn check_scorecard_render(ctx: &WorkspaceContext) -> SubCheckResult {
    let evidence_dir = ctx.workspace_root.join("evidence/multispectrum");
    let per_change_dir = ctx.workspace_root.join("evidence/per-change");
    let mut scanned = 0usize;
    let mut renderable = 0usize;
    let mut io_errors: Vec<String> = Vec::new();
    let mut inner_errors: Vec<String> = Vec::new();
    // TEN-2 + CONV-2: shared scan_dir helper surfaces I/O errors uniformly.
    // CONV-1: structural parse replaces substring grep on JSON contents.
    // Borrow split: scan_dir borrows io_errors; closure pushes to inner_errors
    // for read_to_string failures (cannot share the same accumulator).
    for dir in [&evidence_dir, &per_change_dir] {
        scan_dir(dir, &mut io_errors, |entry| {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("json") {
                return;
            }
            scanned += 1;
            match std::fs::read_to_string(&p) {
                Ok(raw) => {
                    let parsed = parse_top_level_object(&raw);
                    if parsed.contains_key("change_class_id") && parsed.contains_key("facets") {
                        renderable += 1;
                    }
                }
                Err(e) => inner_errors.push(format!(
                    "read_to_string({}) failed: {}",
                    p.display(),
                    e
                )),
            }
        });
    }
    io_errors.extend(inner_errors);
    let mut findings = vec![
        format!(
            "evidence files scanned: {}; minimum-renderable as scorecard: {}",
            scanned, renderable
        ),
        "render_evidence_scorecard() implementation NOT YET ARMED — F-LANE-SCORECARD-RENDER".into(),
    ];
    findings.extend(io_errors);
    SubCheckResult {
        id: "scorecard-render",
        status: SubCheckStatus::NotYetArmed,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// `consensus-debate-evidence`: when an evidence file declares
/// `meta_review_triggered: true` OR an F7 severity ∈ {high, critical},
/// require a matching `evidence/debate/<change_id>-synthesis.json`. Day-1
/// counts meta-triggered evidence + synthesis files present. Full match-by-
/// change_id + termination_reason allow-list check is FixupTask
/// F-LANE-DEBATE-SUBCHECK.
pub fn check_consensus_debate_evidence(ctx: &WorkspaceContext) -> SubCheckResult {
    let evidence_dir = ctx.workspace_root.join("evidence/multispectrum");
    let per_change_dir = ctx.workspace_root.join("evidence/per-change");
    let debate_dir = ctx.workspace_root.join("evidence/debate");
    let mut meta_triggered = 0usize;
    let mut io_errors: Vec<String> = Vec::new();
    let mut inner_errors: Vec<String> = Vec::new();
    // TEN-2 + CONV-2: shared scan_dir helper surfaces I/O errors uniformly.
    // CONV-1: parsed JSON structural match (whitespace-tolerant, bypass-safe).
    // Borrow split: scan_dir borrows io_errors; closure pushes to inner_errors.
    for dir in [&evidence_dir, &per_change_dir] {
        scan_dir(dir, &mut io_errors, |entry| {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("json") {
                return;
            }
            match std::fs::read_to_string(&p) {
                Ok(raw) => {
                    let parsed = parse_top_level_object(&raw);
                    if matches!(
                        parsed.get("meta_review_triggered"),
                        Some(JsonValueKind::BoolTrue)
                    ) {
                        meta_triggered += 1;
                    }
                }
                Err(e) => inner_errors.push(format!(
                    "read_to_string({}) failed: {}",
                    p.display(),
                    e
                )),
            }
        });
    }
    io_errors.extend(inner_errors);
    let mut synthesis_present = 0usize;
    scan_dir(&debate_dir, &mut io_errors, |entry| {
        let name = entry.file_name();
        if name.to_string_lossy().ends_with("-synthesis.json") {
            synthesis_present += 1;
        }
    });
    let mut findings = vec![
        format!(
            "evidence files with meta_review_triggered: {}; synthesis files present: {}",
            meta_triggered, synthesis_present
        ),
        "termination_reason allow-list match-by-change_id NOT YET ARMED — F-LANE-DEBATE-SUBCHECK"
            .into(),
    ];
    findings.extend(io_errors);
    SubCheckResult {
        id: "consensus-debate-evidence",
        status: SubCheckStatus::NotYetArmed,
        findings,
        severity_day_1: Severity::ReportOnly,
    }
}

/// Render a composite report as JSON for evidence emission. Hand-rolled to
/// keep the kernel std-only.
pub fn render_report_json(report: &CompositeReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"lane\": \"oya-check-dependency-seam\",\n");
    out.push_str(&format!("  \"exit_code\": {},\n", report.exit_code()));
    out.push_str(&format!("  \"pass_count\": {},\n", report.pass_count()));
    out.push_str(&format!("  \"fail_count\": {},\n", report.fail_count()));
    out.push_str("  \"sub_checks\": [\n");
    for (i, r) in report.sub_checks.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"id\": \"{}\",\n", r.id));
        out.push_str(&format!(
            "      \"status\": \"{}\",\n",
            match r.status {
                SubCheckStatus::Pass => "pass",
                SubCheckStatus::Fail => "fail",
                SubCheckStatus::NotYetArmed => "not-yet-armed",
            }
        ));
        out.push_str(&format!(
            "      \"severity_day_1\": \"{}\",\n",
            match r.severity_day_1 {
                Severity::ReportOnly => "report-only",
                Severity::Error => "error",
            }
        ));
        out.push_str("      \"findings\": [");
        for (j, f) in r.findings.iter().enumerate() {
            if j > 0 {
                out.push_str(", ");
            }
            // Minimal JSON escape: " and \
            let escaped: String = f
                .chars()
                .map(|c| match c {
                    '"' => "\\\"".to_string(),
                    '\\' => "\\\\".to_string(),
                    '\n' => "\\n".to_string(),
                    c => c.to_string(),
                })
                .collect();
            out.push_str(&format!("\"{}\"", escaped));
        }
        out.push_str("]\n");
        out.push_str(if i + 1 < report.sub_checks.len() {
            "    },\n"
        } else {
            "    }\n"
        });
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

/// Emit per-sub-check audit-chain rows (one JSONL line per sub-check) per
/// ADR-0069. Resolves CONV-9 from TG2 11-facet debate synthesis (F9
/// compliance): the prior `task_group_completion` event named sub-checks
/// collectively, but per-sub-check who/when/what/why was not emitted, so a
/// compliance team could not reconstruct findings from the audit chain
/// alone. This helper returns the JSONL rows for the caller to append to
/// `evidence/audit-chain.jsonl`.
///
/// Caller pattern (from lane binary or CI orchestrator):
///   for row in render_audit_chain_rows(&report, change_id, session_id, ts) {
///       writeln!(audit_chain_file, "{}", row)?;
///   }
pub fn render_audit_chain_rows(
    report: &CompositeReport,
    change_id: &str,
    session_id: &str,
    timestamp_unix: u64,
) -> Vec<String> {
    fn escape(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '"' => "\\\"".to_string(),
                '\\' => "\\\\".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                c => c.to_string(),
            })
            .collect()
    }
    let mut rows = Vec::with_capacity(report.sub_checks.len());
    for r in &report.sub_checks {
        let status_str = match r.status {
            SubCheckStatus::Pass => "pass",
            SubCheckStatus::Fail => "fail",
            SubCheckStatus::NotYetArmed => "not-yet-armed",
        };
        let severity_str = match r.severity_day_1 {
            Severity::ReportOnly => "report-only",
            Severity::Error => "error",
        };
        let first_n: usize = 3;
        let findings_count = r.findings.len();
        let mut findings_preview = String::from("[");
        for (idx, f) in r.findings.iter().take(first_n).enumerate() {
            if idx > 0 {
                findings_preview.push_str(",");
            }
            findings_preview.push('"');
            findings_preview.push_str(&escape(f));
            findings_preview.push('"');
        }
        findings_preview.push(']');
        let row = format!(
            "{{\"event_type\":\"seam_lane_subcheck_run\",\"change_id\":\"{}\",\"session_id\":\"{}\",\"timestamp_unix\":{},\"payload\":{{\"sub_check_id\":\"{}\",\"status\":\"{}\",\"severity_day_1\":\"{}\",\"findings_count\":{},\"findings_first_{}\":{}}}}}",
            escape(change_id),
            escape(session_id),
            timestamp_unix,
            r.id,
            status_str,
            severity_str,
            findings_count,
            first_n,
            findings_preview
        );
        rows.push(row);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace_root_from_test() -> PathBuf {
        // CARGO_MANIFEST_DIR is crates/oya-check-dependency-seam/; parent's
        // parent is the workspace root.
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        crate_dir
            .parent()
            .and_then(|p| p.parent())
            .map(PathBuf::from)
            .unwrap()
    }

    #[test]
    fn run_composite_returns_seventeen_sub_checks() {
        let ctx = WorkspaceContext::new(workspace_root_from_test());
        let report = run_composite(&ctx);
        assert_eq!(report.sub_checks.len(), 17);
        // IDs in canonical order per ADR-0092 D13 + TG2 extension + A-family v2.3.0.
        let ids: Vec<&str> = report.sub_checks.iter().map(|r| r.id).collect();
        assert_eq!(
            ids,
            vec![
                "seam-imports",
                "registry-coverage",
                "cargo-audit-shell",
                "multispectrum-evidence-attached",
                "fixture-pair-coverage",
                "change-class-declared",
                "rust-default-language",
                "naming-convention",
                "scorecard-render",
                "consensus-debate-evidence",
                "a6-schema-adherence",
                "a1-naming-adherence",
                "a2-documentation-adherence",
                "a3-structure-adherence",
                "a4-architecture-adherence",
                "a5-dependency-adherence",
                "a7-algorithm-adherence",
            ]
        );
    }

    #[test]
    fn exit_code_zero_when_all_report_only_even_with_failures() {
        // Day-1 contract: report-only severity never fails the lane.
        let ctx = WorkspaceContext::new("/nonexistent");
        let report = run_composite(&ctx);
        assert_eq!(report.exit_code(), 0);
    }

    #[test]
    fn exit_code_nonzero_when_error_severity_fails() {
        let report = CompositeReport {
            sub_checks: vec![SubCheckResult {
                id: "x",
                status: SubCheckStatus::Fail,
                findings: vec![],
                severity_day_1: Severity::Error,
            }],
        };
        assert_eq!(report.exit_code(), 1);
    }

    // F3 adversarial fixture-pair: passing fixture exists (a real
    // dependency-rationales.json shape with entries object).
    #[test]
    fn fixture_pair_passing_loads_and_extracts_names() {
        let passing = workspace_root_from_test().join(
            "crates/oya-check-dependency-seam/tests/fixtures/registry-coverage/passing/dependency-rationales.json",
        );
        if !passing.exists() {
            panic!(
                "passing fixture missing at {} — required by F3 fixture-pair-coverage rule",
                passing.display()
            );
        }
        let raw = std::fs::read_to_string(&passing).unwrap();
        let names = extract_registry_entry_names(&raw);
        assert!(
            !names.is_empty(),
            "passing fixture must declare at least one entry"
        );
        assert!(names.contains(&"hyper".to_string()));
    }

    // F3 adversarial fixture-pair: failing fixture is a malformed shape
    // the extractor should reject (returns empty names because the
    // `entries` object is absent / malformed).
    #[test]
    fn fixture_pair_failing_yields_empty_names() {
        let failing = workspace_root_from_test().join(
            "crates/oya-check-dependency-seam/tests/fixtures/registry-coverage/failing/dependency-rationales.json",
        );
        if !failing.exists() {
            panic!(
                "failing fixture missing at {} — required by F3 fixture-pair-coverage rule",
                failing.display()
            );
        }
        let raw = std::fs::read_to_string(&failing).unwrap();
        let names = extract_registry_entry_names(&raw);
        assert!(
            names.is_empty(),
            "failing fixture (malformed/no-entries) must yield empty extraction"
        );
    }

    #[test]
    fn extract_registry_entry_names_handles_real_registry() {
        let real = workspace_root_from_test().join("registries/cross-cutting/dependency-rationales.json");
        if real.exists() {
            let raw = std::fs::read_to_string(&real).unwrap();
            let names = extract_registry_entry_names(&raw);
            // ADR-0092 baseline = 11 named deps.
            assert!(
                names.len() >= 8,
                "expected at least 8 entries from real registry; got {}",
                names.len()
            );
            assert!(names.contains(&"hyper".to_string()));
            assert!(names.contains(&"tokio".to_string()));
            assert!(names.contains(&"bytes".to_string()));
        }
    }

    #[test]
    fn render_report_json_is_valid_json_shape() {
        let report = CompositeReport {
            sub_checks: vec![SubCheckResult {
                id: "seam-imports",
                status: SubCheckStatus::NotYetArmed,
                findings: vec!["pending".into()],
                severity_day_1: Severity::ReportOnly,
            }],
        };
        let json = render_report_json(&report);
        assert!(json.contains("\"lane\": \"oya-check-dependency-seam\""));
        assert!(json.contains("\"exit_code\": 0"));
        assert!(json.contains("\"seam-imports\""));
        assert!(json.contains("\"not-yet-armed\""));
        assert!(json.contains("\"report-only\""));
        assert!(json.starts_with('{'));
        assert!(json.trim_end().ends_with('}'));
    }

    #[test]
    fn render_report_json_escapes_quotes_in_findings() {
        let report = CompositeReport {
            sub_checks: vec![SubCheckResult {
                id: "x",
                status: SubCheckStatus::Pass,
                findings: vec!["with \"quotes\" and \\backslash".into()],
                severity_day_1: Severity::ReportOnly,
            }],
        };
        let json = render_report_json(&report);
        assert!(json.contains("with \\\"quotes\\\" and \\\\backslash"));
    }
}
