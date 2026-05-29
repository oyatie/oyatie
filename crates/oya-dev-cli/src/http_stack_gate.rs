//! HTTP-framework selection-discipline gate (the strategic hyper/axum split).
//!
//! Codifies ADR-0090 Amendment 2026-05-29 + ADR-0509 §6: hyper is the PREFERRED
//! low-level default; axum is a SANCTIONED strategic exception that requires a
//! recorded per-crate justification; every other HTTP framework is FORBIDDEN.
//!
//! 1. It reads each workspace member's own `[dependencies]`,
//!    `[dev-dependencies]`, and `[build-dependencies]`.
//! 2. For every DIRECT HTTP-framework dependency it classifies against a
//!    data-driven policy (`specs/http-stack-policy.json`):
//!    - a FORBIDDEN framework (actix-web, poem, warp, rocket, …) -> hard finding;
//!    - a justification-required framework (axum) declared by a crate with NO
//!      recorded rationale in `justified_crates` -> a WARN finding (never fails);
//!    - a preferred (hyper, …) or justified sanctioned framework -> OK.
//! 3. Forbidden findings FAIL the gate when enforcing (the default for this gate
//!    — there are zero on-dev violations as of 2026-05-29, so it is fail-closed
//!    from day one). Unjustified-axum findings are advisory and never fail.
//!
//! Workspace-internal crates (name prefix `oya-`) and any dependency declared
//! with an explicit `path = ...` are EXEMPT — they are first-party code, not
//! external HTTP-framework surface. This mirrors the dependency-blessed-allowlist
//! gate's line-oriented manifest parsing convention.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value as JsonValue, json};

use crate::usage;
use crate::workspace_manifest::{read_package_name, read_workspace_member_paths};

/// Default repo-relative path to the HTTP-stack policy data file.
pub(crate) const DEFAULT_HTTP_STACK_POLICY_PATH: &str = "specs/http-stack-policy.json";

/// Dependency tables that contribute direct dependencies.
const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HttpStackSeverity {
    /// Report findings but never fail.
    ReportOnly,
    /// Fail when any FORBIDDEN-framework finding is present (the default).
    Enforce,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HttpStackArgs {
    repo_root: PathBuf,
    policy_path: PathBuf,
    severity: HttpStackSeverity,
    emit_report_path: Option<PathBuf>,
}

/// What kind of policy departure a finding represents.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum HttpStackFindingKind {
    /// A categorically forbidden HTTP framework (hard fail when enforcing).
    Forbidden,
    /// A justification-required sanctioned framework (axum) used by a crate that
    /// has no recorded rationale in the policy (advisory warning, never fails).
    UnjustifiedSanctioned,
}

impl HttpStackFindingKind {
    fn as_str(self) -> &'static str {
        match self {
            HttpStackFindingKind::Forbidden => "forbidden",
            HttpStackFindingKind::UnjustifiedSanctioned => "unjustified-sanctioned",
        }
    }
}

/// One HTTP-stack policy finding, scoped to a crate + framework.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct HttpStackFinding {
    pub kind: HttpStackFindingKind,
    /// Workspace member crate name (e.g. `oya-identity-workload-rest`).
    pub crate_name: String,
    /// Repo-relative crate manifest path (e.g. `crates/<name>/Cargo.toml`).
    pub crate_path: String,
    /// The HTTP-framework dependency name.
    pub framework: String,
    /// Which manifest table declared it.
    pub table: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HttpStackReport {
    pub crates_scanned: usize,
    pub hyper_crate_count: usize,
    pub axum_crate_count: usize,
    pub findings: Vec<HttpStackFinding>,
    pub enforced: bool,
}

impl HttpStackReport {
    pub fn forbidden_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.kind == HttpStackFindingKind::Forbidden)
            .count()
    }

    pub fn unjustified_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.kind == HttpStackFindingKind::UnjustifiedSanctioned)
            .count()
    }

    /// Render the report as a `serde_json::Value` for `--emit-report`.
    ///
    /// `oya-dev-cli` does not take a direct `serde` derive dependency, so the
    /// report is constructed explicitly rather than via `#[derive(Serialize)]`.
    fn to_json_value(&self) -> JsonValue {
        let findings = self
            .findings
            .iter()
            .map(|finding| {
                json!({
                    "kind": finding.kind.as_str(),
                    "crate_name": finding.crate_name,
                    "crate_path": finding.crate_path,
                    "framework": finding.framework,
                    "table": finding.table,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "crates_scanned": self.crates_scanned,
            "hyper_crate_count": self.hyper_crate_count,
            "axum_crate_count": self.axum_crate_count,
            "forbidden_count": self.forbidden_count(),
            "unjustified_count": self.unjustified_count(),
            "enforced": self.enforced,
            "findings": findings,
        })
    }
}

pub(crate) fn parse_http_stack_validate_args(args: Vec<String>) -> Result<HttpStackArgs, String> {
    let mut repo_root = PathBuf::from(".");
    let mut policy_path = PathBuf::from(DEFAULT_HTTP_STACK_POLICY_PATH);
    let mut severity = HttpStackSeverity::Enforce;
    let mut emit_report_path = None;

    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => repo_root = next_path(&mut iter)?,
            "--policy" => policy_path = next_path(&mut iter)?,
            "--emit-report" => emit_report_path = Some(next_path(&mut iter)?),
            "--enforce" => severity = HttpStackSeverity::Enforce,
            "--report-only" => severity = HttpStackSeverity::ReportOnly,
            _ => return Err(usage()),
        }
    }

    Ok(HttpStackArgs {
        policy_path: resolve_repo_path(&repo_root, policy_path),
        emit_report_path: emit_report_path.map(|path| resolve_repo_path(&repo_root, path)),
        repo_root,
        severity,
    })
}

pub(crate) fn validate_http_stack_gate(args: HttpStackArgs) -> Result<HttpStackReport, String> {
    let policy = read_http_stack_policy(&args.policy_path)?;
    let members = read_workspace_member_manifests(&args.repo_root)?;

    let mut findings = Vec::new();
    let mut hyper_crate_count = 0;
    let mut axum_crate_count = 0;

    for member in &members {
        let mut uses_hyper = false;
        let mut uses_axum = false;
        for (table, dependency) in &member.http_framework_dependencies {
            if dependency == "hyper" {
                uses_hyper = true;
            }
            if dependency == "axum" {
                uses_axum = true;
            }
            if policy.forbidden.contains(dependency) {
                findings.push(HttpStackFinding {
                    kind: HttpStackFindingKind::Forbidden,
                    crate_name: member.name.clone(),
                    crate_path: member.relative_manifest.clone(),
                    framework: dependency.clone(),
                    table: table.clone(),
                });
            } else if policy.requires_justification.contains(dependency)
                && !policy.is_justified(dependency, &member.name)
            {
                findings.push(HttpStackFinding {
                    kind: HttpStackFindingKind::UnjustifiedSanctioned,
                    crate_name: member.name.clone(),
                    crate_path: member.relative_manifest.clone(),
                    framework: dependency.clone(),
                    table: table.clone(),
                });
            }
        }
        if uses_hyper {
            hyper_crate_count += 1;
        }
        if uses_axum {
            axum_crate_count += 1;
        }
    }
    findings.sort();

    let report = HttpStackReport {
        crates_scanned: members.len(),
        hyper_crate_count,
        axum_crate_count,
        findings,
        enforced: matches!(args.severity, HttpStackSeverity::Enforce),
    };

    if let Some(path) = &args.emit_report_path {
        write_report(path, &report)?;
    }

    Ok(report)
}

/// The parsed, data-driven HTTP-stack policy.
#[derive(Clone, Debug, Eq, PartialEq)]
struct HttpStackPolicy {
    forbidden: BTreeSet<String>,
    requires_justification: BTreeSet<String>,
    /// framework -> set of crate names with a recorded justification.
    justified: BTreeMap<String, BTreeSet<String>>,
    /// The full universe of HTTP frameworks this policy is aware of
    /// (preferred ∪ sanctioned ∪ forbidden) — only these names are classified.
    known: BTreeSet<String>,
}

impl HttpStackPolicy {
    fn is_justified(&self, framework: &str, crate_name: &str) -> bool {
        self.justified
            .get(framework)
            .is_some_and(|crates| crates.contains(crate_name))
    }
}

fn read_http_stack_policy(path: &Path) -> Result<HttpStackPolicy, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "http-stack policy file unreadable {}: {error}",
            path.display()
        )
    })?;
    let value: JsonValue = serde_json::from_str(&contents)
        .map_err(|error| format!("http-stack policy file invalid {}: {error}", path.display()))?;

    let preferred = string_array(&value, "preferred_frameworks", "frameworks");
    let sanctioned = string_array(&value, "sanctioned_frameworks", "frameworks");
    let forbidden = string_array(&value, "forbidden_frameworks", "frameworks");
    let requires_justification =
        string_array(&value, "sanctioned_frameworks", "requires_justification");

    if forbidden.is_empty() {
        return Err(format!(
            "http-stack policy {} lacks a non-empty forbidden_frameworks.frameworks array",
            path.display()
        ));
    }
    if preferred.is_empty() && sanctioned.is_empty() {
        return Err(format!(
            "http-stack policy {} declares no preferred or sanctioned frameworks",
            path.display()
        ));
    }

    let mut justified: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    if let Some(map) = value.get("justified_crates").and_then(JsonValue::as_object) {
        for (framework, crates) in map {
            let names = crates
                .as_object()
                .map(|obj| obj.keys().cloned().collect::<BTreeSet<String>>())
                .unwrap_or_default();
            justified.insert(framework.clone(), names);
        }
    }

    let mut known = BTreeSet::new();
    known.extend(preferred.iter().cloned());
    known.extend(sanctioned.iter().cloned());
    known.extend(forbidden.iter().cloned());

    Ok(HttpStackPolicy {
        forbidden,
        requires_justification,
        justified,
        known,
    })
}

/// Read a `["a","b"]` array nested under `value[section][field]` into a set.
fn string_array(value: &JsonValue, section: &str, field: &str) -> BTreeSet<String> {
    value
        .get(section)
        .and_then(|s| s.get(field))
        .and_then(JsonValue::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MemberManifest {
    name: String,
    relative_manifest: String,
    /// (table, framework-name) pairs for every declared HTTP framework.
    http_framework_dependencies: Vec<(String, String)>,
}

fn read_workspace_member_manifests(repo_root: &Path) -> Result<Vec<MemberManifest>, String> {
    // The policy universe is read once so we only attribute HTTP frameworks.
    let policy_path = repo_root.join(DEFAULT_HTTP_STACK_POLICY_PATH);
    let known = read_http_stack_policy(&policy_path)
        .map(|policy| policy.known)
        .unwrap_or_default();

    let workspace_manifest = repo_root.join("Cargo.toml");
    let relative_paths = read_workspace_member_paths(&workspace_manifest)?;

    let mut out = Vec::new();
    for relative_path in relative_paths {
        let manifest_path = repo_root.join(&relative_path).join("Cargo.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let Ok(name) = read_package_name(&manifest_path) else {
            continue;
        };
        out.push(MemberManifest {
            name,
            relative_manifest: format!("{relative_path}/Cargo.toml"),
            http_framework_dependencies: http_framework_dependencies(&manifest_path, &known)?,
        });
    }
    Ok(out)
}

/// Collect `(table, framework)` pairs for every DIRECT HTTP-framework dependency
/// declared in a crate manifest, restricted to the policy's known framework set.
///
/// Follows the same line-oriented convention as the dependency-blessed-allowlist
/// gate: `oya-*` first-party crates and explicit `path = ...` deps are exempt.
fn http_framework_dependencies(
    manifest_path: &Path,
    known: &BTreeSet<String>,
) -> Result<Vec<(String, String)>, String> {
    let contents = fs::read_to_string(manifest_path).map_err(|error| {
        format!(
            "package manifest unreadable {}: {error}",
            manifest_path.display()
        )
    })?;
    let mut findings: BTreeSet<(String, String)> = BTreeSet::new();
    let mut current_table: Option<&str> = None;
    for raw_line in contents.lines() {
        let line = strip_inline_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(section) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            current_table = DEPENDENCY_TABLES
                .into_iter()
                .find(|table| *table == section.trim());
            continue;
        }
        let Some(table_name) = current_table else {
            continue;
        };
        let Some(alias) = dependency_key(line) else {
            continue;
        };
        if alias.starts_with("oya-") {
            continue;
        }
        if is_path_dependency(line) {
            continue;
        }
        let resolved = renamed_package(line).unwrap_or_else(|| alias.to_string());
        if known.contains(&resolved) {
            findings.insert((table_name.to_string(), resolved));
        }
    }
    Ok(findings.into_iter().collect())
}

fn dependency_key(line: &str) -> Option<&str> {
    let key_region = line.split('=').next().unwrap_or(line);
    let key = key_region.split('.').next().unwrap_or(key_region).trim();
    if key.is_empty() { None } else { Some(key) }
}

fn is_path_dependency(line: &str) -> bool {
    inline_table_field(line, "path").is_some()
}

fn renamed_package(line: &str) -> Option<String> {
    inline_table_field(line, "package")
}

fn inline_table_field(line: &str, field: &str) -> Option<String> {
    let needle = format!("{field} =");
    let start = line.find(&needle)? + needle.len();
    let rest = line[start..].trim_start();
    if let Some(after_quote) = rest.strip_prefix('"') {
        let end = after_quote.find('"')?;
        return Some(after_quote[..end].to_string());
    }
    if field == "path" {
        return Some(String::new());
    }
    None
}

fn strip_inline_comment(line: &str) -> &str {
    match line.find('#') {
        Some(index) => &line[..index],
        None => line,
    }
}

fn write_report(path: &Path, report: &HttpStackReport) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "http-stack report parent unwritable {}: {error}",
                parent.display()
            )
        })?;
    }
    let encoded = serde_json::to_string_pretty(&report.to_json_value())
        .map_err(|error| format!("http-stack report serialization failed: {error}"))?;
    fs::write(path, encoded)
        .map_err(|error| format!("http-stack report write failed {}: {error}", path.display()))
}

fn next_path(iter: &mut impl Iterator<Item = String>) -> Result<PathBuf, String> {
    iter.next().map(PathBuf::from).ok_or_else(usage)
}

fn resolve_repo_path(repo_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch_root(slug: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "oya-http-stack-{slug}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("scratch root created");
        root
    }

    fn cleanup(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }

    /// Writes the policy at the repo-root-relative default path so both the
    /// gate and the manifest scanner resolve it.
    fn write_policy(root: &Path, justified_axum: &[&str]) {
        let specs = root.join("specs");
        fs::create_dir_all(&specs).expect("specs dir");
        let justified = justified_axum
            .iter()
            .map(|c| format!("\"{c}\": \"test rationale\""))
            .collect::<Vec<_>>()
            .join(",");
        let body = format!(
            "{{\
             \"preferred_frameworks\": {{\"frameworks\": [\"hyper\", \"hyper-util\"]}},\
             \"sanctioned_frameworks\": {{\"frameworks\": [\"axum\", \"tower\"], \"requires_justification\": [\"axum\"]}},\
             \"forbidden_frameworks\": {{\"frameworks\": [\"actix-web\", \"poem\", \"warp\", \"rocket\"]}},\
             \"justified_crates\": {{\"axum\": {{{justified}}}}}\
             }}"
        );
        fs::write(specs.join("http-stack-policy.json"), body).expect("policy written");
    }

    fn write_workspace(root: &Path, members: &[&str]) {
        let rendered = members
            .iter()
            .map(|member| format!("\"{member}\""))
            .collect::<Vec<_>>()
            .join(", ");
        fs::write(
            root.join("Cargo.toml"),
            format!("[workspace]\nmembers = [{rendered}]\n"),
        )
        .expect("workspace manifest written");
    }

    fn write_member(root: &Path, path: &str, name: &str, deps_block: &str) {
        let crate_dir = root.join(path);
        fs::create_dir_all(crate_dir.join("src")).expect("crate dir created");
        fs::write(
            crate_dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{name}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"Apache-2.0\"\n{deps_block}"
            ),
        )
        .expect("member manifest written");
        fs::write(crate_dir.join("src/lib.rs"), "pub fn it() {}\n").expect("member lib written");
    }

    fn run(root: &Path, enforce: bool) -> HttpStackReport {
        let mut args = vec![
            "--repo-root".to_string(),
            root.to_str().expect("utf8").to_string(),
        ];
        args.push(
            if enforce {
                "--enforce"
            } else {
                "--report-only"
            }
            .to_string(),
        );
        let parsed = parse_http_stack_validate_args(args).expect("args parse");
        validate_http_stack_gate(parsed).expect("gate runs")
    }

    #[test]
    fn forbidden_framework_is_flagged_and_fails_when_enforcing() {
        let root = scratch_root("forbidden");
        write_policy(&root, &[]);
        write_workspace(&root, &["crates/bad"]);
        write_member(
            &root,
            "crates/bad",
            "oya-bad-rest",
            "[dependencies]\nactix-web = \"4\"\nhyper.workspace = true\n",
        );
        let report = run(&root, true);
        assert_eq!(report.forbidden_count(), 1, "report={report:?}");
        let finding = report
            .findings
            .iter()
            .find(|f| f.kind == HttpStackFindingKind::Forbidden)
            .expect("forbidden finding");
        assert_eq!(finding.framework, "actix-web");
        assert_eq!(finding.crate_name, "oya-bad-rest");
        assert_eq!(report.hyper_crate_count, 1);
        cleanup(&root);
    }

    #[test]
    fn unjustified_axum_warns_but_does_not_fail() {
        let root = scratch_root("unjustified-axum");
        write_policy(&root, &[]); // no crate justified
        write_workspace(&root, &["crates/svc"]);
        write_member(
            &root,
            "crates/svc",
            "oya-some-rest",
            "[dependencies]\naxum = \"0.8\"\n",
        );
        let report = run(&root, true);
        assert_eq!(report.forbidden_count(), 0);
        assert_eq!(report.unjustified_count(), 1, "report={report:?}");
        assert_eq!(report.axum_crate_count, 1);
        cleanup(&root);
    }

    #[test]
    fn justified_axum_crate_is_clean() {
        let root = scratch_root("justified-axum");
        write_policy(&root, &["oya-some-rest"]);
        write_workspace(&root, &["crates/svc"]);
        write_member(
            &root,
            "crates/svc",
            "oya-some-rest",
            "[dependencies]\naxum = \"0.8\"\ntower = \"0.5\"\n",
        );
        let report = run(&root, true);
        assert_eq!(report.forbidden_count(), 0);
        assert_eq!(report.unjustified_count(), 0, "report={report:?}");
        assert_eq!(report.axum_crate_count, 1);
        cleanup(&root);
    }

    #[test]
    fn preferred_hyper_crate_is_clean() {
        let root = scratch_root("preferred-hyper");
        write_policy(&root, &[]);
        write_workspace(&root, &["crates/svc"]);
        write_member(
            &root,
            "crates/svc",
            "oya-http-runtime-hyper-adapter",
            "[dependencies]\nhyper = \"1\"\nhyper-util = \"0.1\"\noya-kernel = { path = \"../oya-kernel\" }\n",
        );
        let report = run(&root, true);
        assert_eq!(report.findings.len(), 0, "report={report:?}");
        assert_eq!(report.hyper_crate_count, 1);
        assert_eq!(report.axum_crate_count, 0);
        cleanup(&root);
    }

    #[test]
    fn report_only_never_fails_even_with_forbidden() {
        // ReportOnly still records findings; the dispatcher decides pass/fail.
        let root = scratch_root("report-only");
        write_policy(&root, &[]);
        write_workspace(&root, &["crates/bad"]);
        write_member(
            &root,
            "crates/bad",
            "oya-bad",
            "[dependencies]\npoem = \"3\"\n",
        );
        let report = run(&root, false);
        assert_eq!(report.forbidden_count(), 1);
        assert!(!report.enforced);
        cleanup(&root);
    }

    #[test]
    fn missing_forbidden_array_is_an_error() {
        let root = scratch_root("bad-policy");
        let specs = root.join("specs");
        fs::create_dir_all(&specs).expect("specs");
        fs::write(
            specs.join("http-stack-policy.json"),
            "{\"preferred_frameworks\":{\"frameworks\":[\"hyper\"]}}",
        )
        .expect("bad policy written");
        write_workspace(&root, &["crates/svc"]);
        write_member(
            &root,
            "crates/svc",
            "oya-svc",
            "[dependencies]\nhyper = \"1\"\n",
        );
        let parsed = parse_http_stack_validate_args(vec![
            "--repo-root".to_string(),
            root.to_str().expect("utf8").to_string(),
        ])
        .expect("args parse");
        assert!(validate_http_stack_gate(parsed).is_err());
        cleanup(&root);
    }
}
