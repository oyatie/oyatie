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
