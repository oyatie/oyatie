//! cloud-ci-gitops-chart-license (ADR-0706 D-5 enforcement; bead oyatie-f2fg).
//!
//! The Cargo license gates (`ci-license-policy`) resolve `Cargo.toml` `license` fields only, so
//! they never see a Helm chart pull -- the AGPL-3.0 Grafana Labs charts under
//! `observability/iac/helm/` and `observability/iac/k8s/helm/` (grafana/loki/mimir/pyroscope/oncall/tempo)
//! sit entirely outside their reach. This gate closes that gap for the structural corpora GitOps
//! actually reads:
//!
//! 1. `infra/gitops/values.yaml`'s `apps[]` array -- `type: chart` entries naming direct
//!    third-party Helm pulls via `repoURL` + `chart`, and `type: path` entries with `helmPath: true`
//!    pointing to local charts whose `Chart.yaml` declares `dependencies[]`.
//! 2. `observability/iac/k8s/helm/Chart.yaml` -- the first-party umbrella chart GitOps deploys.
//! 3. `observability/iac/helm/*/Chart.yaml`'s `dependencies[]` array -- each first-party wrapper
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
use std::path::{Path, PathBuf};

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

/// Walk the GitOps corpora and emit one row per distinct `(repository, chart)` pull.
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

fn resolve_chart_dir(repo_root: &Path, rel_path: &str) -> PathBuf {
    let direct = repo_root.join(rel_path);
    if direct.is_dir() {
        return direct;
    }
    if let Some(stripped) = rel_path.strip_prefix("microservices/") {
        let stripped_path = repo_root.join(stripped);
        if stripped_path.is_dir() {
            return stripped_path;
        }
    }
    direct
}

fn parse_chart_dependencies(chart_yaml: &Path) -> Result<Vec<ChartKey>, CollectError> {
    if !chart_yaml.is_file() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(chart_yaml)
        .map_err(|e| err(format!("read {}: {e}", chart_yaml.display())))?;
    let doc: serde_yaml::Value = serde_yaml::from_str(&text)
        .map_err(|e| err(format!("parse {}: {e}", chart_yaml.display())))?;

    let deps = match doc.get("dependencies") {
        None => return Ok(Vec::new()),
        Some(val) => val.as_sequence().ok_or_else(|| {
            err(format!(
                "{}: `dependencies` must be a sequence",
                chart_yaml.display()
            ))
        })?,
    };

    let mut keys = Vec::new();
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
    Ok(keys)
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
        if app_type == Some("chart") {
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
        } else if app_type == Some("path") {
            let is_helm_path = app
                .get("helmPath")
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(false);
            if is_helm_path {
                let name = app
                    .get("name")
                    .and_then(serde_yaml::Value::as_str)
                    .unwrap_or("<unnamed-app>");
                let rel_path = app
                    .get("path")
                    .and_then(serde_yaml::Value::as_str)
                    .ok_or_else(|| err(format!("apps[{name}]: type: path with no path")))?;
                let chart_dir = resolve_chart_dir(repo_root, rel_path);
                let chart_yaml = chart_dir.join("Chart.yaml");
                if chart_yaml.is_file() {
                    keys.extend(parse_chart_dependencies(&chart_yaml)?);
                }
            }
        }
    }
    Ok(keys)
}

fn collect_observability_umbrella_charts(repo_root: &Path) -> Result<Vec<ChartKey>, CollectError> {
    let mut keys = Vec::new();

    // 1. Umbrella chart deployed to Kubernetes (observability/iac/k8s/helm/Chart.yaml)
    let k8s_umbrella_chart = repo_root.join("observability/iac/k8s/helm/Chart.yaml");
    if k8s_umbrella_chart.is_file() {
        keys.extend(parse_chart_dependencies(&k8s_umbrella_chart)?);
    }

    // 2. Component wrapper charts (observability/iac/helm/*/Chart.yaml)
    let helm_dir = repo_root.join("observability/iac/helm");
    if helm_dir.is_dir() {
        let mut chart_dirs: Vec<_> = std::fs::read_dir(&helm_dir)
            .map_err(|e| err(format!("read_dir {}: {e}", helm_dir.display())))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.path())
            .collect();
        chart_dirs.sort();

        for dir in chart_dirs {
            let chart_yaml = dir.join("Chart.yaml");
            if chart_yaml.is_file() {
                keys.extend(parse_chart_dependencies(&chart_yaml)?);
            }
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

    let rows = observed.get("rows").and_then(Value::as_array);
    let rows: &[Value] = rows.map_or(&[], Vec::as_slice);

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
    for row in rows {
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

    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let id = format!(
                "oya-chart-test-{}-{}-{}",
                name,
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            );
            let path = std::env::temp_dir().join(id);
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn chart_yaml_with_absent_dependencies_is_skipped() {
        let temp = TempDir::new("absent-deps");
        let chart_dir = temp.path.join("observability/iac/helm/chart-without-deps");
        std::fs::create_dir_all(&chart_dir).expect("create chart dir");
        std::fs::write(
            chart_dir.join("Chart.yaml"),
            "name: chart-without-deps\nversion: 0.1.0\n",
        )
        .expect("write Chart.yaml");

        let keys = collect_observability_umbrella_charts(&temp.path)
            .expect("absent dependencies should be skipped cleanly");
        assert!(keys.is_empty());
    }

    #[test]
    fn chart_yaml_with_valid_sequence_dependencies_is_collected() {
        let temp = TempDir::new("valid-deps");
        let chart_dir = temp.path.join("observability/iac/helm/chart-with-deps");
        std::fs::create_dir_all(&chart_dir).expect("create chart dir");
        std::fs::write(
            chart_dir.join("Chart.yaml"),
            "name: chart-with-deps\nversion: 0.1.0\ndependencies:\n  - name: upstream\n    repository: https://example.com/charts\n",
        )
        .expect("write Chart.yaml");

        let keys = collect_observability_umbrella_charts(&temp.path)
            .expect("valid sequence dependencies should collect");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].chart, "upstream");
        assert_eq!(keys[0].repository, "https://example.com/charts");
    }

    #[test]
    fn chart_yaml_with_mapping_dependencies_fails_closed() {
        let temp = TempDir::new("mapping-deps");
        let chart_dir = temp.path.join("observability/iac/helm/bad-chart");
        std::fs::create_dir_all(&chart_dir).expect("create chart dir");
        let chart_yaml = chart_dir.join("Chart.yaml");
        std::fs::write(
            &chart_yaml,
            "name: bad-chart\nversion: 0.1.0\ndependencies:\n  upstream:\n    repo: https://example.com\n",
        )
        .expect("write Chart.yaml");

        let err = collect_observability_umbrella_charts(&temp.path)
            .expect_err("mapping dependencies must return CollectError");
        let expected_msg = format!(
            "{}: `dependencies` must be a sequence",
            chart_yaml.display()
        );
        assert_eq!(err.to_string(), expected_msg);
    }

    #[test]
    fn chart_yaml_with_scalar_dependencies_fails_closed() {
        let temp = TempDir::new("scalar-deps");
        let chart_dir = temp.path.join("observability/iac/helm/bad-scalar-chart");
        std::fs::create_dir_all(&chart_dir).expect("create chart dir");
        let chart_yaml = chart_dir.join("Chart.yaml");
        std::fs::write(
            &chart_yaml,
            "name: bad-scalar-chart\nversion: 0.1.0\ndependencies: invalid-scalar\n",
        )
        .expect("write Chart.yaml");

        let err = collect_observability_umbrella_charts(&temp.path)
            .expect_err("scalar dependencies must return CollectError");
        let expected_msg = format!(
            "{}: `dependencies` must be a sequence",
            chart_yaml.display()
        );
        assert_eq!(err.to_string(), expected_msg);
    }

    #[test]
    fn observability_k8s_umbrella_chart_is_collected() {
        let temp = TempDir::new("k8s-umbrella");
        let chart_dir = temp.path.join("observability/iac/k8s/helm");
        std::fs::create_dir_all(&chart_dir).expect("create umbrella dir");
        std::fs::write(
            chart_dir.join("Chart.yaml"),
            "name: oya-observability\nversion: 0.1.0\ndependencies:\n  - name: loki\n    repository: https://grafana.github.io/helm-charts\n",
        )
        .expect("write Chart.yaml");

        let keys = collect_observability_umbrella_charts(&temp.path)
            .expect("umbrella chart dependencies should collect");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].chart, "loki");
        assert_eq!(keys[0].repository, "https://grafana.github.io/helm-charts");
    }

    #[test]
    fn gitops_values_helm_path_app_dependencies_are_collected() {
        let temp = TempDir::new("helm-path-app");
        let gitops_dir = temp.path.join("infra/gitops");
        let app_chart_dir = temp.path.join("observability/iac/k8s/helm");
        std::fs::create_dir_all(&gitops_dir).expect("create gitops dir");
        std::fs::create_dir_all(&app_chart_dir).expect("create app chart dir");

        std::fs::write(
            gitops_dir.join("values.yaml"),
            "apps:\n  - name: direct-chart\n    type: chart\n    repoURL: https://helm.cilium.io\n    chart: cilium\n  - name: local-helm-app\n    type: path\n    path: microservices/observability/iac/k8s/helm\n    helmPath: true\n",
        )
        .expect("write values.yaml");

        std::fs::write(
            app_chart_dir.join("Chart.yaml"),
            "name: oya-observability\nversion: 0.1.0\ndependencies:\n  - name: tempo-distributed\n    repository: https://grafana.github.io/helm-charts\n",
        )
        .expect("write Chart.yaml");

        let keys = collect_gitops_values_charts(&temp.path)
            .expect("both type: chart and type: path helmPath: true should collect");
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].chart, "cilium");
        assert_eq!(keys[0].repository, "https://helm.cilium.io");
        assert_eq!(keys[1].chart, "tempo-distributed");
        assert_eq!(keys[1].repository, "https://grafana.github.io/helm-charts");
    }
}
