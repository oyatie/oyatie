// ADR-0538 cloud-ci-workspace-glob-coverage live-corpus gate. Runs the producer
// `--face workspace-glob-coverage`, then asserts the verdict is consistent with current
// findings. ADR-0083 Tier-3: integration tests assert with unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_workspace_member_coverage::{Verdict, evaluate, evaluate_keyed};
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

fn producer_binary(root: &Path, producer_bin: Option<&str>) -> Result<PathBuf, String> {
    let Some(bin) = producer_bin else {
        return Err(
            "FAIL-CLOSED: missing OYA_CI_PRODUCER_BIN; Cargo fallback is forbidden".to_owned(),
        );
    };
    ci_path_resolver_adapters::resolve_cargo_test_binary(root, std::ffi::OsStr::new(bin))
}

#[test]
fn producer_binary_env_is_required_for_hermetic_gate() {
    let err = producer_binary(Path::new("/repo"), None)
        .expect_err("missing OYA_CI_PRODUCER_BIN must fail closed");
    assert!(err.contains("OYA_CI_PRODUCER_BIN"));
}

fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json");
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let bin = producer_binary(root, producer_bin.as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(bin)
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(&scm_facts)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect("run producer binary");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}

/// Assert two enumerated sets are equal, naming exactly which keys are missing/extra on
/// mismatch. A SET, never a count: a count is unattributable, a set is a reviewable diff of
/// named keys, and only a set can tell "the corpus shrank" apart from "the producer stopped
/// seeing most of it".
fn assert_set_equals(label: &str, face: &BTreeSet<String>, census: &BTreeSet<String>) {
    let missing_from_face: Vec<&String> = census.difference(face).collect();
    let extra_in_face: Vec<&String> = face.difference(census).collect();
    assert!(
        missing_from_face.is_empty() && extra_in_face.is_empty(),
        "{label} SET MISMATCH — missing_from_face={missing_from_face:?} \
         extra_in_face={extra_in_face:?}"
    );
}

#[test]
fn workspace_glob_coverage_verdict_matches_the_live_corpus() {
    let root = repo_root();
    let face = run_producer_face(&root, "workspace-glob-coverage");
    let rows = face["rows"]
        .as_array()
        .expect("workspace-glob-coverage face rows");

    // INDEPENDENT DYNAMIC CENSUS, not a magnitude floor. The producer derives every one of its
    // faces from ONE `tracked_paths` vector, so a single narrowing there (an over-broad
    // exclusion rule, a lost scan root, a truncated SCM face) shrinks this face with no other
    // signal — measured: a 43% producer-side narrowing left this gate GREEN behind
    // `rows.len() > 500`. Re-derive both halves of the face from the canonical resolver and
    // assert EXACT set equality, so a producer that silently drops even ONE member is caught
    // as a named set difference. Both assertions strictly imply the magnitude floor they
    // replace. Self-adjusting: the census moves in lockstep with the corpus, never needs a bump.
    //
    // The `[workspace].members` entries the face echoes back must be exactly the ones the root
    // manifest declares — this is the half that proves the face read the real manifest.
    let face_entries: BTreeSet<String> = rows
        .iter()
        .filter_map(|row| row.get("member_entry").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    let declared_entries: BTreeSet<String> =
        oya_workspace_members_kernel::read_workspace_manifest_entries(&root)
            .expect("read the live root workspace manifest entries")
            .members
            .into_iter()
            .collect();
    assert_set_equals("member_entry", &face_entries, &declared_entries);

    // Every crate dir a workspace glob COVERS must be exactly a canonically resolved member.
    // The uncovered remainder is the debt this gate exists to report, so it is deliberately
    // outside this equality — but the covered majority (887 of 888 rows today) is now pinned
    // by name rather than by magnitude.
    let face_covered: BTreeSet<String> = rows
        .iter()
        .filter(|row| row.get("covered").and_then(Value::as_bool) == Some(true))
        .map(|row| {
            row["crate_dir"]
                .as_str()
                .expect("covered row crate_dir")
                .to_owned()
        })
        .collect();
    let resolved_members: BTreeSet<String> =
        oya_workspace_members_kernel::resolve_member_dirs(&root)
            .expect("resolve_member_dirs must resolve the live root workspace Cargo.toml")
            .into_iter()
            .collect();
    assert_set_equals("covered crate_dir", &face_covered, &resolved_members);

    let findings = evaluate_keyed(&face);
    let verdict = evaluate(&face).verdict;
    eprintln!(
        "BORN-BLOCKING workspace-glob-coverage: rows={} total_findings={} verdict={:?}",
        rows.len(),
        findings.len(),
        verdict
    );

    if findings.is_empty() {
        assert_eq!(verdict, Verdict::Green, "no findings must mean GREEN");
    } else {
        assert_eq!(verdict, Verdict::Red, "findings present must mean RED");
    }
}
