//! Three shapes, mirroring `ci/facade/crate-catalog-coverage`:
//!   1. GREEN     — today's corpus matches the frozen baseline.
//!   2. RED FIXTURE — a synthetic new `cloud-` name MUST fail. A gate only ever observed passing
//!                    is not evidence of anything.
//!   3. FIDELITY  — every frozen entry is still genuinely present, so the baseline is neither
//!                  stale nor over-broad.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ci_cloud_name_ratchet::{compare, findings, parse_baseline};

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

fn census(root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                if !matches!(
                    name.as_str(),
                    "target" | "buck-out" | ".git" | "node_modules" | ".jj"
                ) {
                    stack.push(path);
                }
                continue;
            }
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let contents = if matches!(name.as_str(), "Cargo.toml" | "Chart.yaml") {
                std::fs::read_to_string(&path).unwrap_or_default()
            } else {
                String::new()
            };
            out.extend(findings(&relative.to_string_lossy(), &contents));
        }
    }
    out
}

fn baseline() -> BTreeSet<String> {
    let path = repo_root().join("ci/facade/cloud-name-ratchet/cloud-name-baseline.json");
    parse_baseline(&std::fs::read_to_string(&path).expect("frozen baseline is readable"))
}

#[test]
fn the_deprecated_cloud_name_set_never_grows() {
    let verdict = compare(&census(&repo_root()), &baseline());
    assert!(
        verdict.added.is_empty(),
        "NEW deprecated `cloud-` names beyond the frozen baseline:\n{}\n\n\
         `cloud-` is deprecated. Name the new thing without it. If a rename genuinely requires a \
         transitional name, that is a founder call, not a baseline edit.",
        verdict
            .added
            .iter()
            .map(|k| format!("  {k}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn burn_down_must_be_recorded_in_the_same_change() {
    // Shrink is the point, but the frozen file must not overstate the remaining debt: a rename
    // without a baseline regen leaves a phantom entry that hides the next real one.
    let verdict = compare(&census(&repo_root()), &baseline());
    assert!(
        verdict.removed.is_empty(),
        "these baselined names are gone — regenerate the baseline in this change:\n{}\n\n\
           cargo run -p ci-cloud-name-ratchet --bin oya-ci-cloud-name-baseline -- --repo-root . \\\n\
             > ci/facade/cloud-name-ratchet/cloud-name-baseline.json",
        verdict
            .removed
            .iter()
            .map(|k| format!("  {k}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn red_fixture_a_new_cloud_name_fails_closed() {
    let base = baseline();
    let mut synthetic = census(&repo_root());
    synthetic.insert("dir:secrets/cloud-brand-new-service".to_owned());
    let verdict = compare(&synthetic, &base);
    assert_eq!(
        verdict.added,
        BTreeSet::from(["dir:secrets/cloud-brand-new-service".to_owned()]),
        "a newly introduced `cloud-` name must be caught"
    );
}

#[test]
fn the_baseline_is_not_empty_and_is_shaped_as_expected() {
    let base = baseline();
    assert!(
        base.len() > 100,
        "the frozen baseline looks truncated: {} entries",
        base.len()
    );
    assert!(
        base.iter()
            .all(|k| k.starts_with("dir:") || k.starts_with("name:")),
        "every key must name its rename unit"
    );
}
