//! Live-corpus enforcement.
//!
//! Until this file existed the crate compiled, its fixture tests ran under
//! `cargo test --workspace`, and it never once evaluated the real tree — so its
//! verdict on this repository was computed by nobody. It was carried in
//! `ci/facade/scan-root-liveness`'s `baselined_dark_gate_crates` for exactly that
//! reason, and this change removes it from that list.
//!
//! The gate reads the policy JSON rather than hard-coding repo facts, so a
//! capability rehome is a data edit.

#![allow(clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use check_id_discipline::{FrozenEntry, Policy, SchemaDocument, audit_all};

/// Walk up to the repository root using the sentinel every other gate lane uses.
fn repo_root() -> PathBuf {
    let mut directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if directory.join("specs/root-hub-pointers.json").is_file() {
            return directory;
        }
        assert!(
            directory.pop(),
            "repository root not found above the manifest dir"
        );
    }
}

fn policy_value(root: &Path) -> serde_json::Value {
    let path = root.join("governance/check/id-discipline/id-discipline-policy.json");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn strings(value: &serde_json::Value, key: &str) -> BTreeSet<String> {
    value[key]
        .as_array()
        .unwrap_or_else(|| panic!("policy `{key}` must be an array"))
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("policy `{key}` entries must be strings"))
                .to_owned()
        })
        .collect()
}

fn parse_policy(value: &serde_json::Value) -> Policy {
    let frozen = value["frozen_underspecified_id_formats"]
        .as_array()
        .expect("policy `frozen_underspecified_id_formats` must be an array")
        .iter()
        .map(|entry| {
            let reason = entry["reason"].as_str().unwrap_or_default();
            // A blank reason is not an exemption. Without this the list becomes a
            // laundering path: an entry with no justification silently removes a
            // declaration from coverage.
            assert!(
                !reason.trim().is_empty(),
                "every frozen entry needs a non-empty reason; `{}` has none",
                entry["path"].as_str().unwrap_or("<no path>")
            );
            FrozenEntry {
                path: entry["path"]
                    .as_str()
                    .expect("frozen entry `path`")
                    .to_owned(),
                field: entry["field"]
                    .as_str()
                    .expect("frozen entry `field`")
                    .to_owned(),
                declared: entry["declared"]
                    .as_str()
                    .expect("frozen entry `declared`")
                    .to_owned(),
            }
        })
        .collect();

    Policy {
        canonical_id_fields: strings(value, "canonical_id_fields"),
        uuidv7_pattern: value["uuidv7_pattern"]
            .as_str()
            .expect("policy `uuidv7_pattern`")
            .to_owned(),
        frozen,
        min_expected_scanned_files: value["min_expected_scanned_files"]
            .as_u64()
            .expect("policy `min_expected_scanned_files`")
            as usize,
        min_expected_id_fields: value["min_expected_id_fields"]
            .as_u64()
            .expect("policy `min_expected_id_fields`") as usize,
    }
}

/// Collect every schema/contract document under the declared scan roots.
fn collect(root: &Path, policy_json: &serde_json::Value) -> Vec<SchemaDocument> {
    let mut documents = Vec::new();
    for scan_root in strings(policy_json, "scan_roots") {
        let base = root.join(&scan_root);
        // A declared root that resolves to nothing scans an empty set and reports
        // green over it — the same defect `scan-root-liveness` exists to catch.
        assert!(
            base.is_dir(),
            "declared scan root `{scan_root}` resolves to no directory; delete the root in the \
             change that empties it"
        );
        walk(&base, root, &mut documents);
    }
    documents.sort_by(|a, b| a.path.cmp(&b.path));
    documents
}

fn walk(directory: &Path, root: &Path, out: &mut Vec<SchemaDocument>) {
    let entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()));
    for entry in entries {
        let path = entry.expect("directory entry").path();
        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
        if matches!(name, ".git" | "target" | "buck-out" | "third-party") {
            continue;
        }
        if path.is_dir() {
            walk(&path, root, out);
            continue;
        }
        let extension = path.extension().and_then(|v| v.to_str()).unwrap_or("");
        if !matches!(extension, "json" | "yaml" | "yml") {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        let relative = path
            .strip_prefix(root)
            .expect("path below repository root")
            .to_string_lossy()
            .replace('\\', "/");
        let microservice = relative.split('/').nth(1).unwrap_or("<root>").to_owned();
        out.push(SchemaDocument {
            path: relative,
            microservice,
            contents,
        });
    }
}

#[test]
fn live_schema_corpus_is_green_against_the_frozen_policy() {
    let root = repo_root();
    let policy_json = policy_value(&root);
    let policy = parse_policy(&policy_json);
    let documents = collect(&root, &policy_json);

    let (report, findings) = audit_all(&documents, &policy);
    assert!(
        findings.is_empty(),
        "id-discipline must be green on the live corpus; got {} finding(s):\n{}",
        findings.len(),
        findings
            .iter()
            .map(|f| format!("  {} {}:{} {}", f.code, f.path, f.field, f.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
    eprintln!(
        "ID-DISCIPLINE live corpus: documents={} id_fields={} declared_formats={} findings=0",
        report.documents_checked, report.id_fields_inspected, report.id_fields_with_declared_format
    );
}

/// The verdict above is only meaningful if the matcher actually fires on this
/// tree. `specs/residency-attestation-schema.json` pins the UUIDv7 version bits
/// for `attestation_id`; if that stops being seen as a compliant declaration,
/// the green above is green over nothing.
#[test]
fn the_corpus_contains_a_compliant_declaration_the_matcher_recognises() {
    let root = repo_root();
    let policy_json = policy_value(&root);
    let mut policy = parse_policy(&policy_json);
    let documents = collect(&root, &policy_json);

    // This check runs over ONE document, so the whole-corpus guards do not
    // apply: the frozen entries name violations in a different file and would
    // read as stale, and the census floors are sized for 1251 documents. Both
    // are asserted by the corpus-wide tests above; the question here is only
    // whether the matcher recognises a compliant declaration at all.
    policy.frozen = BTreeSet::new();
    policy.min_expected_scanned_files = 0;
    policy.min_expected_id_fields = 0;

    let control = documents
        .iter()
        .find(|d| d.path == "specs/residency-attestation-schema.json")
        .expect("the positive control schema must exist");
    let (report, findings) = audit_all(std::slice::from_ref(control), &policy);

    assert!(
        report.id_fields_with_declared_format >= 1,
        "the positive control declares a UUIDv7 id format; the matcher saw {} — it is not firing",
        report.id_fields_with_declared_format
    );
    assert!(
        findings.is_empty(),
        "the positive control must be clean; got {findings:?}"
    );
}

/// The frozen list must describe the tree, not outlive it.
#[test]
fn every_frozen_entry_still_names_a_live_underspecified_declaration() {
    let root = repo_root();
    let policy_json = policy_value(&root);
    let mut policy = parse_policy(&policy_json);
    let documents = collect(&root, &policy_json);

    let frozen = std::mem::take(&mut policy.frozen);
    let (_report, findings) = audit_all(&documents, &policy);
    let live: BTreeSet<(String, String)> = findings
        .iter()
        .filter(|f| f.code == check_id_discipline::CODE_UNDERSPECIFIED_FORMAT)
        .map(|f| (f.path.clone(), f.field.clone()))
        .collect();
    let declared: BTreeSet<(String, String)> = frozen
        .iter()
        .map(|f| (f.path.clone(), f.field.clone()))
        .collect();

    assert_eq!(
        declared, live,
        "the frozen list must equal all and only the live underspecified declarations"
    );
}
