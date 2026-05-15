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
    if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
        for entry in entries.filter_map(Result::ok) {
            let p = entry.path();
            if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                if exts.contains(&ext) {
                    total_non_rust += 1;
                }
            }
        }
    }
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
    let mut violations: Vec<String> = Vec::new();
    for home in &homes {
        let p = ctx.workspace_root.join(home);
        if let Ok(entries) = std::fs::read_dir(&p) {
            for entry in entries.filter_map(Result::ok) {
                let name = entry.file_name();
                let s = name.to_string_lossy().to_string();
                if s.starts_with('.') {
                    continue;
                }
                scanned += 1;
                if !s
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
                {
                    if violations.len() < 10 {
                        violations.push(format!("{}/{}", home, s));
                    }
                }
            }
        }
    }
    let mut findings = vec![format!(
        "scanned {} top-level entries across {} canonical homes; kebab-case violations: {}",
        scanned,
        homes.len(),
        violations.len()
    )];
    for v in &violations {
        findings.push(format!("  - {}", v));
    }
    findings.push(
        "version-suffix -vMAJOR.MINOR.PATCH check NOT YET ARMED — F-NAMING-CONVENTION-ENFORCE"
            .into(),
    );
    SubCheckResult {
        id: "naming-convention",
        status: if violations.is_empty() {
            SubCheckStatus::Pass
        } else {
            SubCheckStatus::Fail
        },
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
    for dir in [&evidence_dir, &per_change_dir] {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                scanned += 1;
                if let Ok(raw) = std::fs::read_to_string(&p) {
                    if raw.contains("\"change_class_id\"") && raw.contains("\"facets\"") {
                        renderable += 1;
                    }
                }
            }
        }
    }
    let findings = vec![
        format!(
            "evidence files scanned: {}; minimum-renderable as scorecard: {}",
            scanned, renderable
        ),
        "render_evidence_scorecard() implementation NOT YET ARMED — F-LANE-SCORECARD-RENDER".into(),
    ];
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
    for dir in [&evidence_dir, &per_change_dir] {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                if let Ok(raw) = std::fs::read_to_string(&p) {
                    if raw.contains("\"meta_review_triggered\": true")
                        || raw.contains("\"meta_review_triggered\":true")
                    {
                        meta_triggered += 1;
                    }
                }
            }
        }
    }
    let mut synthesis_present = 0usize;
    if let Ok(entries) = std::fs::read_dir(&debate_dir) {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name();
            if name.to_string_lossy().ends_with("-synthesis.json") {
                synthesis_present += 1;
            }
        }
    }
    let findings = vec![
        format!(
            "evidence files with meta_review_triggered: {}; synthesis files present: {}",
            meta_triggered, synthesis_present
        ),
        "termination_reason allow-list match-by-change_id NOT YET ARMED — F-LANE-DEBATE-SUBCHECK"
            .into(),
    ];
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
    fn run_composite_returns_ten_sub_checks() {
        let ctx = WorkspaceContext::new(workspace_root_from_test());
        let report = run_composite(&ctx);
        assert_eq!(report.sub_checks.len(), 10);
        // IDs in canonical order per ADR-0092 D13 + TG2 extension.
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
