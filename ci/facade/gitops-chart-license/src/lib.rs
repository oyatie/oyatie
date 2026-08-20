//! cloud-ci-gitops-chart-license (ADR-0706 D-5 enforcement; bead oyatie-f2fg).
//!
//! The Cargo license gates (`ci-license-policy`) resolve `Cargo.toml` `license` fields only, so
//! they never see a Helm chart pull -- the AGPL-3.0 Grafana Labs charts under
//! `observability/iac/helm/` (grafana/loki/mimir/pyroscope/oncall/tempo) sit entirely outside
//! their reach. This gate closes that gap for the two structural corpora GitOps actually reads:
//!
//! 1. `infra/gitops/values.yaml`'s `apps[]` array -- every `type: chart` entry names a direct
//!    third-party Helm pull via `repoURL` + `chart`.
//! 2. `observability/iac/helm/*/Chart.yaml`'s `dependencies[]` array -- each first-party umbrella
//!    chart names the real upstream chart it wraps, the same signal Helm itself uses to resolve
//!    subcharts.
//!
//! `collect_chart_rows` is a pure filesystem read (no cargo, no network, no Helm CLI); rows are
//! deduplicated by `(repository, chart)` since the license question is per distinct pull, not per
//! call site. `evaluate_keyed` looks up each row in the curated, plane-keyed policy allow-list
//! (`gitops-chart-license-policy.json`) and fails closed on anything undeclared -- ADR-0706 D-5:
//! "an undeclared chart fails closed."
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

use serde_json::{Value, json};

pub const GATE_ID: &str = "cloud-ci-gitops-chart-license";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String, // data_class: INTERNAL_ONLY
    pub key: String,  // data_class: INTERNAL_ONLY
}

impl Finding {
    fn new(code: &str, key: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,             // data_class: INTERNAL_ONLY
    pub violations: BTreeSet<String>, // data_class: INTERNAL_ONLY
}

impl Report {
    fn from_codes(violations: BTreeSet<String>) -> Self {
        let verdict = if violations.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            violations,
        }
    }
}

#[derive(Debug)]
pub struct CollectError(String);

impl fmt::Display for CollectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CollectError {}

fn err(message: impl Into<String>) -> CollectError {
    CollectError(message.into())
}

/// A distinct third-party Helm pull, keyed by `(repository, chart)`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ChartKey {
    repository: String,
    chart: String,
}

/// Walk the two GitOps corpora and emit one row per distinct `(repository, chart)` pull.
/// Read-only: no cargo, no network, no Helm CLI -- structural YAML parsing only.
pub fn collect_chart_rows(repo_root: &Path) -> Result<Value, CollectError> {
    let mut sources: BTreeMap<ChartKey, &'static str> = BTreeMap::new();

    for key in collect_gitops_values_charts(repo_root)? {
        sources.entry(key).or_insert("infra/gitops");
    }
    for key in collect_observability_umbrella_charts(repo_root)? {
        sources.entry(key).or_insert("observability");
    }

    let rows: Vec<Value> = sources
        .into_iter()
        .map(|(key, source)| {
            json!({
                "repository": key.repository,
                "chart": key.chart,
                "source": source,
            })
        })
        .collect();
    Ok(json!({ "rows": rows }))
}

fn collect_gitops_values_charts(repo_root: &Path) -> Result<Vec<ChartKey>, CollectError> {
    let path = repo_root.join("infra/gitops/values.yaml");
    let text =
        std::fs::read_to_string(&path).map_err(|e| err(format!("read {}: {e}", path.display())))?;
    let doc: serde_yaml::Value =
        serde_yaml::from_str(&text).map_err(|e| err(format!("parse {}: {e}", path.display())))?;

    let apps = doc
        .get("apps")
        .and_then(serde_yaml::Value::as_sequence)
        .ok_or_else(|| {
            err(format!(
                "{}: missing top-level `apps` sequence",
                path.display()
            ))
        })?;

    let mut keys = Vec::new();
    for app in apps {
        let app_type = app.get("type").and_then(serde_yaml::Value::as_str);
        if app_type != Some("chart") {
            continue;
        }
        let name = app
            .get("name")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or("<unnamed-app>");
        let repository = app
            .get("repoURL")
            .and_then(serde_yaml::Value::as_str)
            .ok_or_else(|| err(format!("apps[{name}]: type: chart with no repoURL")))?
            .to_owned();
        let chart = app
            .get("chart")
            .and_then(serde_yaml::Value::as_str)
            .ok_or_else(|| err(format!("apps[{name}]: type: chart with no chart")))?
            .to_owned();
        keys.push(ChartKey { repository, chart });
    }
    Ok(keys)
}

fn collect_observability_umbrella_charts(repo_root: &Path) -> Result<Vec<ChartKey>, CollectError> {
    let helm_dir = repo_root.join("observability/iac/helm");
    if !helm_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut chart_dirs: Vec<_> = std::fs::read_dir(&helm_dir)
        .map_err(|e| err(format!("read_dir {}: {e}", helm_dir.display())))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path())
        .collect();
    chart_dirs.sort();

    let mut keys = Vec::new();
    for dir in chart_dirs {
        let chart_yaml = dir.join("Chart.yaml");
        if !chart_yaml.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&chart_yaml)
            .map_err(|e| err(format!("read {}: {e}", chart_yaml.display())))?;
        let doc: serde_yaml::Value = serde_yaml::from_str(&text)
            .map_err(|e| err(format!("parse {}: {e}", chart_yaml.display())))?;

        let Some(deps) = doc
            .get("dependencies")
            .and_then(serde_yaml::Value::as_sequence)
        else {
            continue;
        };
        for dep in deps {
            let dep_name = dep
                .get("name")
                .and_then(serde_yaml::Value::as_str)
                .ok_or_else(|| err(format!("{}: dependency with no name", chart_yaml.display())))?
                .to_owned();
            let repository = dep
                .get("repository")
                .and_then(serde_yaml::Value::as_str)
                .ok_or_else(|| {
                    err(format!(
                        "{}: dependency {dep_name} with no repository",
                        chart_yaml.display()
                    ))
                })?
                .to_owned();
            keys.push(ChartKey {
                repository,
                chart: dep_name,
            });
        }
    }
    Ok(keys)
}

fn policy_entries(policy: &Value) -> BTreeMap<ChartKey, (String, String)> {
    let mut map = BTreeMap::new();
    let Some(entries) = policy.get("entries").and_then(Value::as_array) else {
        return map;
    };
    for entry in entries {
        let (Some(repository), Some(chart), Some(license), Some(plane)) = (
            entry.get("repository").and_then(Value::as_str),
            entry.get("chart").and_then(Value::as_str),
            entry.get("license").and_then(Value::as_str),
            entry.get("plane").and_then(Value::as_str),
        ) else {
            continue;
        };
        map.insert(
            ChartKey {
                repository: repository.to_owned(),
                chart: chart.to_owned(),
            },
            (license.to_owned(), plane.to_owned()),
        );
    }
    map
}

fn plane_accepts(policy: &Value, plane: &str, license: &str) -> bool {
    policy["planes"][plane]["accepted_licenses"]
        .as_array()
        .is_some_and(|accepted| accepted.iter().any(|value| value.as_str() == Some(license)))
}

/// Row key: `<repository>#<chart>`, stable and grep-able in a failure message.
fn row_key(repository: &str, chart: &str) -> String {
    format!("{repository}#{chart}")
}

/// Pure evaluator: takes `collect_chart_rows`' output plus the policy, emits one finding per
/// undeclared chart pull or plane-forbidden licence, plus an anti-vacuity finding if the scan
/// saw fewer chart pulls than the policy's floor (a narrowed scan would otherwise pass vacuously).
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let rows = observed
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let floor = policy["min_expected_rows"].as_u64().unwrap_or(0) as usize;
    if rows.len() < floor {
        findings.insert(Finding::new(
            "gitops_chart_license_scan_narrowed",
            format!(
                "scanned {} chart pulls, policy floor is {floor}",
                rows.len()
            ),
        ));
    }

    let entries = policy_entries(policy);
    for row in &rows {
        let (Some(repository), Some(chart)) = (
            row.get("repository").and_then(Value::as_str),
            row.get("chart").and_then(Value::as_str),
        ) else {
            continue;
        };
        let key = ChartKey {
            repository: repository.to_owned(),
            chart: chart.to_owned(),
        };
        let Some((license, plane)) = entries.get(&key) else {
            findings.insert(Finding::new(
                "gitops_chart_license_undeclared_chart",
                row_key(repository, chart),
            ));
            continue;
        };
        if !plane_accepts(policy, plane, license) {
            findings.insert(Finding::new(
                "gitops_chart_license_forbidden_for_plane",
                format!(
                    "{} license={license} plane={plane}",
                    row_key(repository, chart)
                ),
            ));
        }
    }

    findings
}

/// Bare-code projection of [`evaluate_keyed`] -- the single source of truth for the verdict.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    let codes: BTreeSet<String> = evaluate_keyed(policy, observed)
        .into_iter()
        .map(|f| f.code)
        .collect();
    Report::from_codes(codes)
}

pub fn render_findings(findings: &BTreeSet<Finding>) -> String {
    if findings.is_empty() {
        return format!("{GATE_ID}: GREEN (no undeclared or plane-forbidden chart pulls)\n");
    }
    let mut out = format!("{GATE_ID}: RED ({} findings)\n", findings.len());
    for finding in findings {
        out.push_str(&format!("  {}: {}\n", finding.code, finding.key));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy_fixture() -> Value {
        json!({
            "planes": {
                "ops-internal": {"accepted_licenses": ["Apache-2.0", "AGPL-3.0"]},
                "tenant-facing": {"accepted_licenses": ["Apache-2.0"]}
            },
            "entries": [
                {"repository": "https://example.com/charts", "chart": "widget", "license": "Apache-2.0", "plane": "ops-internal"},
                {"repository": "https://example.com/charts", "chart": "gadget", "license": "AGPL-3.0", "plane": "tenant-facing"}
            ],
            "min_expected_rows": 1
        })
    }

    #[test]
    fn a_declared_chart_within_its_plane_allow_list_is_green() {
        let policy = policy_fixture();
        let observed =
            json!({"rows": [{"repository": "https://example.com/charts", "chart": "widget"}]});
        assert!(evaluate_keyed(&policy, &observed).is_empty());
        assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);
    }

    #[test]
    fn a_declared_chart_outside_its_plane_allow_list_is_forbidden() {
        let policy = policy_fixture();
        let observed =
            json!({"rows": [{"repository": "https://example.com/charts", "chart": "gadget"}]});
        let findings = evaluate_keyed(&policy, &observed);
        assert_eq!(findings.len(), 1);
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "gitops_chart_license_forbidden_for_plane");
        assert!(finding.key.contains("gadget"));
        assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
    }

    #[test]
    fn an_undeclared_chart_fails_closed() {
        let policy = policy_fixture();
        let observed = json!({"rows": [{"repository": "https://example.com/charts", "chart": "unknown-thing"}]});
        let findings = evaluate_keyed(&policy, &observed);
        assert_eq!(findings.len(), 1);
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "gitops_chart_license_undeclared_chart");
        assert!(finding.key.contains("unknown-thing"));
    }

    #[test]
    fn a_scan_narrower_than_the_floor_is_a_finding_not_a_silent_pass() {
        let policy = policy_fixture();
        let observed = json!({"rows": []});
        let findings = evaluate_keyed(&policy, &observed);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "gitops_chart_license_scan_narrowed")
        );
        assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let policy = policy_fixture();
        let observed = json!({"rows": [
            {"repository": "https://example.com/charts", "chart": "gadget"},
            {"repository": "https://example.com/charts", "chart": "unknown-thing"}
        ]});
        let projected: BTreeSet<String> = evaluate_keyed(&policy, &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert_eq!(evaluate(&policy, &observed).violations, projected);
    }
}
