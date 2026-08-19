//! Live-corpus enforcement. Until this file existed the crate compiled, its fixture tests ran
//! under `cargo nextest run --workspace`, and it never once evaluated the real tree -- so its
//! verdict on the repository was computed by nobody. It reported `Failed` when run by hand and
//! that verdict reached no gate.
//!
//! The check is frozen shrink-only rather than terminal-zero on purpose: 66 stamped groups
//! already exist, and demanding they be rewritten before the detector may run is what kept the
//! detector switched off. Grandfathering makes it born-blocking on NEW stamping today.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join("governance/capability-registry.json").is_file() {
        assert!(root.pop(), "repository root not found above the manifest dir");
    }
    root
}

fn frozen(root: &Path) -> serde_json::Value {
    let path = root.join("libs/oya-governance-no-template-stamping/template-stamping-baseline.json");
    serde_json::from_str(&std::fs::read_to_string(&path).expect("read the frozen baseline"))
        .expect("parse the frozen baseline")
}

/// Observed stamped groups, keyed the same way the baseline is: directory -> sorted group sizes.
fn observed(root: &Path) -> BTreeMap<String, Vec<usize>> {
    let outcome = oya_governance_no_template_stamping::enforce_no_template_stamping(root)
        .expect("run the detector over the live tree");
    let mut by_dir: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for violation in &outcome.violations {
        by_dir
            .entry(violation.directory.to_string_lossy().into_owned())
            .or_default()
            .push(violation.files.len());
    }
    for sizes in by_dir.values_mut() {
        sizes.sort_unstable_by(|left, right| right.cmp(left));
    }
    by_dir
}

fn baseline_groups(frozen: &serde_json::Value) -> BTreeMap<String, Vec<usize>> {
    frozen["groups"]
        .as_object()
        .expect("baseline groups object")
        .iter()
        .map(|(dir, sizes)| {
            (
                dir.clone(),
                sizes
                    .as_array()
                    .expect("group sizes array")
                    .iter()
                    .map(|size| size.as_u64().expect("group size integer") as usize)
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn live_template_stamping_matches_the_frozen_baseline() {
    let root = repo_root();
    let observed = observed(&root);
    let frozen = frozen(&root);
    let baseline = baseline_groups(&frozen);

    let mut grown = Vec::new();
    for (dir, sizes) in &observed {
        match baseline.get(dir) {
            None => grown.push(format!("{dir}: newly stamped, {sizes:?}")),
            Some(frozen_sizes) if sizes > frozen_sizes => {
                grown.push(format!("{dir}: {frozen_sizes:?} -> {sizes:?}"));
            }
            Some(_) => {}
        }
    }
    assert!(
        grown.is_empty(),
        "template stamping grew beyond the frozen baseline. Collapse the duplicated template \
         prose into a shared standard, or rewrite each document with artifact-specific \
         structure. Do NOT raise the baseline to admit new stamping:\n  {}",
        grown.join("\n  ")
    );

    let stale: Vec<_> = baseline
        .iter()
        .filter(|(dir, frozen_sizes)| {
            observed
                .get(*dir)
                .is_none_or(|sizes| sizes < *frozen_sizes)
        })
        .map(|(dir, frozen_sizes)| format!("{dir}: frozen {frozen_sizes:?}, now {:?}", observed.get(dir)))
        .collect();
    assert!(
        stale.is_empty(),
        "template stamping SHRANK -- record the win by lowering these baseline entries in this \
         same change, so the reduction cannot silently reappear:\n  {}",
        stale.join("\n  ")
    );
}

#[test]
fn the_detector_still_sees_the_whole_repository() {
    // Anti-vacuity. The defect this file exists to close was a scan scope of
    // ["docs", "microservices"] where microservices/ held zero tracked files, so the detector
    // silently examined 1435 of 3990 markdown files. A scope regression would make every
    // assertion above pass by seeing nothing.
    let root = repo_root();
    let outcome = oya_governance_no_template_stamping::enforce_no_template_stamping(&root)
        .expect("run the detector over the live tree");
    let frozen = frozen(&root);
    let floor = frozen["scanned_markdown_files"]
        .as_u64()
        .expect("frozen scanned_markdown_files") as usize;
    assert!(
        outcome.scanned_markdown_files >= floor,
        "the detector scanned {} markdown files but the frozen scope saw {floor}; the scan scope \
         has narrowed, which makes every stamping assertion vacuous",
        outcome.scanned_markdown_files
    );
}
