//! Every Helm chart in the repository must have a renderable `templates/` directory.
//!
//! Runs inside `cargo test --locked --workspace`, i.e. the required `test (workspace + gates)` job,
//! so a new non-manifest file under `templates/` is caught at merge rather than at deploy.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};

use ci_helm_chart_shape::{Finding, helmignore_entries, template_file_finding};

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        assert!(
            dir.pop(),
            "repository root marker not found above the crate"
        );
    }
}

/// Every `Chart.yaml` in the tree, excluding build outputs.
fn chart_dirs(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                if !matches!(
                    name.as_ref(),
                    "target" | "buck-out" | ".git" | "node_modules"
                ) {
                    stack.push(path);
                }
            } else if name == "Chart.yaml" {
                found.push(dir.clone());
            }
        }
    }
    found.sort();
    found
}

fn findings_for(chart: &Path, root: &Path) -> Vec<Finding> {
    let templates = chart.join("templates");
    if !templates.is_dir() {
        return Vec::new();
    }
    let ignore = std::fs::read_to_string(chart.join(".helmignore"))
        .map(|text| helmignore_entries(&text))
        .unwrap_or_default();
    let label = chart
        .strip_prefix(root)
        .unwrap_or(chart)
        .to_string_lossy()
        .to_string();
    let mut out = Vec::new();
    let mut stack = vec![templates.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(path);
                continue;
            }
            let relative = path
                .strip_prefix(&templates)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            if let Some(finding) = template_file_finding(&label, &relative, &ignore) {
                out.push(finding);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn no_chart_ships_an_unrenderable_file_under_templates() {
    let root = repo_root();
    let charts = chart_dirs(&root);
    assert!(
        charts.len() > 50,
        "expected the chart corpus, found {}",
        charts.len()
    );
    let findings: Vec<Finding> = charts
        .iter()
        .flat_map(|chart| findings_for(chart, &root))
        .collect();
    assert!(
        findings.is_empty(),
        "Helm parses every file under templates/ as a manifest. These would break their chart:\n{}",
        findings
            .iter()
            .map(|f| format!("  {}/templates/{} — {}", f.chart, f.path, f.reason))
            .collect::<Vec<_>>()
            .join("\n")
    );
    println!("checked {} charts", charts.len());
}

/// The shared-chart arrangement must not silently regress.
///
/// 71 services were collapsed onto `iac/charts/oyatie-microservice` after render comparison proved
/// their output identical. 11 keep their own chart because their output genuinely differs. This
/// freezes that split shrink-only: a NEW service must use the shared chart, and reconciling a
/// bespoke one means deleting its entry in the same change.
#[test]
fn no_service_reintroduces_its_own_copy_of_the_shared_chart() {
    let root = repo_root();
    let frozen = ci_helm_chart_shape::bespoke_charts(
        &std::fs::read_to_string(root.join("ci/facade/helm-chart-shape/bespoke-charts.json"))
            .expect("frozen bespoke list is readable"),
    );
    let live: Vec<String> = chart_dirs(&root)
        .iter()
        .filter_map(|c| {
            let rel = c.strip_prefix(&root).ok()?.to_string_lossy().to_string();
            rel.strip_suffix("/iac/k8s/helm").map(str::to_string)
        })
        .collect();
    let added: Vec<&String> = live.iter().filter(|c| !frozen.contains(c)).collect();
    assert!(
        added.is_empty(),
        "these services carry their own Helm chart but are not in the frozen list:\n{}\n\n\
         Render from iac/charts/oyatie-microservice with a values.yaml instead. If the output \
         genuinely differs, add the service to bespoke-charts.json with the render diff as \
         justification.",
        added
            .iter()
            .map(|c| format!("  {c}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let gone: Vec<&String> = frozen.iter().filter(|c| !live.contains(c)).collect();
    assert!(
        gone.is_empty(),
        "these are listed as bespoke but no longer carry a chart — delete them from \
         bespoke-charts.json in this change so the list never overstates the remaining split:\n{}",
        gone.iter()
            .map(|c| format!("  {c}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
