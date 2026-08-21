// cloud-ci-gitops-chart-license live-corpus self-test (ADR-0706 D-5 / bead oyatie-f2fg). Runs
// the real collector over today's repo tree and asserts the live corpus is born-blocking GREEN:
// every chart pull GitOps declares today is a policy-declared entry within its plane's allow-list.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use ci_gitops_chart_license::{Verdict, collect_chart_rows, evaluate, evaluate_keyed};
use serde_json::Value;

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn policy(root: &std::path::Path) -> Value {
    let path = root.join("ci/facade/gitops-chart-license/gitops-chart-license-policy.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

#[test]
fn live_gitops_corpus_is_born_blocking_green() {
    let root = repo_root();
    let policy = policy(&root);
    let observed = collect_chart_rows(&root).expect("collect chart pulls from the live tree");
    let rows = observed["rows"].as_array().expect("rows array");

    eprintln!(
        "GITOPS-CHART-LICENSE live corpus: {} distinct chart pulls",
        rows.len()
    );

    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "every chart pull GitOps declares today must be a policy-declared entry within its \
         plane's allow-list; found: {findings:?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);
}

#[test]
fn the_scan_actually_found_both_corpora() {
    // Anti-vacuity: a broken parse of either source (e.g. a renamed `apps:` key, a moved
    // observability/iac/helm/ directory) would make `live_gitops_corpus_is_born_blocking_green`
    // pass vacuously by seeing nothing. Assert both sources contributed at least one row.
    let root = repo_root();
    let observed = collect_chart_rows(&root).expect("collect chart pulls from the live tree");
    let rows = observed["rows"].as_array().expect("rows array");

    let infra_rows = rows
        .iter()
        .filter(|row| row["source"] == "infra/gitops")
        .count();
    let observability_rows = rows
        .iter()
        .filter(|row| row["source"] == "observability")
        .count();

    assert!(
        infra_rows > 0,
        "infra/gitops/values.yaml contributed zero chart rows"
    );
    assert!(
        observability_rows > 0,
        "observability/iac/helm/*/Chart.yaml contributed zero chart rows"
    );
}

#[test]
fn a_chart_missing_from_the_policy_fails_closed_not_silently() {
    let root = repo_root();
    let policy = policy(&root);
    let observed = serde_json::json!({
        "rows": [{"repository": "https://example.com/charts", "chart": "not-in-the-policy"}]
    });
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "gitops_chart_license_undeclared_chart"
                && f.key.contains("not-in-the-policy")),
        "got {findings:?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

#[test]
fn umbrella_chart_dependencies_are_present_in_collected_rows() {
    let root = repo_root();
    let observed = collect_chart_rows(&root).expect("collect chart pulls from the live tree");
    let rows = observed["rows"].as_array().expect("rows array");

    // observability/iac/k8s/helm/Chart.yaml declares loki, tempo-distributed, mimir-distributed, grafana
    let expected_umbrella_charts = ["loki", "tempo-distributed", "mimir-distributed", "grafana"];
    for chart_name in expected_umbrella_charts {
        assert!(
            rows.iter().any(|row| {
                row["chart"] == chart_name
                    && row["repository"] == "https://grafana.github.io/helm-charts"
            }),
            "expected umbrella chart dependency {chart_name} from https://grafana.github.io/helm-charts in collected rows"
        );
    }
}
