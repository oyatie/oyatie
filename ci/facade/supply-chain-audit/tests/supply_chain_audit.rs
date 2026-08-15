//! cloud-ci-supply-chain-audit live-corpus self-test (owned RustSec advisory scan).
//!
//! Legs:
//!   1. LIVE: parse the real Cargo.lock + the committed advisory mirror under the committed policy
//!      and assert the gate is born-blocking GREEN (quinn-proto on the patched 0.11.15; the only
//!      other live affected advisories are the three unmaintained ids in policy.ignore). This is the
//!      load-bearing "lands green" acceptance.
//!   2. RED self-test: a synthetic locked quinn-proto 0.11.14 + a fixture advisory => SCA-VULN keyed
//!      to RUSTSEC-2026-0185; 0.11.15 => clean.
//!   3. UNMAINTAINED: an unmaintained fixture absent from ignore => SCA-UNMAINTAINED; present => clean.
//!   4. MIRROR INTEGRITY: a tampered manifest content_hash => SCA-MIRROR-MALFORMED.
//!
//! Pure filesystem; no network, no clock. ADR-0083 Tier-3: integration tests use unwrap/expect/panic.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use ci_supply_chain_audit::{
    GATE_ID, collect, configured_lockfiles, evaluate_keyed, render_findings,
};
use oya_advisory_mirror_kernel::{Advisory, canonical_hash};
use serde_json::{Value, json};

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Mirrors the helper the firewall meta-gates use.
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

fn policy_path(root: &Path) -> PathBuf {
    root.join("ci/facade/supply-chain-audit/supply-chain-audit-policy.json")
}

fn load_policy(root: &Path) -> Value {
    let text = std::fs::read_to_string(policy_path(root)).expect("read committed policy");
    serde_json::from_str(&text).expect("parse committed policy")
}

const MINIMAL_LOCK: &str = "version = 4\n\n[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\n";

struct TempRepo {
    root: PathBuf,
}

impl TempRepo {
    fn new(name: &str) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};

        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "oya-supply-chain-{name}-{}-{id}",
            std::process::id()
        ));
        std::fs::create_dir(&root).expect("create synthetic repo");
        Self { root }
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create fixture parent");
        }
        std::fs::write(path, contents).expect("write fixture");
    }

    fn write_mirror(&self, advisories: &[Advisory]) {
        self.write(
            "mirror/advisories.json",
            &serde_json::to_string(advisories).expect("serialize fixture advisories"),
        );
        self.write(
            "mirror/mirror-manifest.json",
            &serde_json::to_string(&json!({
                "content_hash": canonical_hash(advisories),
                "advisory_count": advisories.len(),
            }))
            .expect("serialize fixture manifest"),
        );
    }

    fn write_scm_facts(&self, tracked_paths: &[&str]) {
        let mut tracked_paths = tracked_paths.to_vec();
        tracked_paths.sort_unstable();
        tracked_paths.dedup();
        self.write(
            "scm-facts.json",
            &serde_json::to_string(&json!({
                "schema": "oya-ci/scm-facts/v2",
                "tracked_paths": tracked_paths,
            }))
            .expect("serialize fixture scm-facts"),
        );
    }

    fn policy(&self, corpus: Value) -> Value {
        let tracked_paths = corpus
            .as_array()
            .expect("fixture corpus array")
            .iter()
            .flat_map(|entry| {
                ["manifest_path", "lockfile_path"]
                    .into_iter()
                    .map(|key| entry[key].as_str().expect("fixture corpus path").to_owned())
            })
            .collect::<Vec<_>>();
        let tracked_path_refs = tracked_paths.iter().map(String::as_str).collect::<Vec<_>>();
        self.policy_with_tracked_paths(corpus, &tracked_path_refs)
    }

    fn policy_with_tracked_paths(&self, corpus: Value, tracked_paths: &[&str]) -> Value {
        let count = corpus.as_array().expect("fixture corpus array").len();
        self.write_scm_facts(tracked_paths);
        json!({
            "gate_id": GATE_ID,
            "lockfile_corpus": corpus,
            "min_lockfiles": count,
            "scm_facts_path": "scm-facts.json",
            "mirror_dir": "mirror",
            "unmaintained_policy": "all",
            "min_advisories": 0,
            "ignore": [],
        })
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn live_corpus_is_born_blocking_green() {
    let root = repo_root();
    let policy = load_policy(&root);
    assert_eq!(
        policy.get("gate_id").and_then(Value::as_str),
        Some(GATE_ID),
        "committed policy gate_id must be {GATE_ID}"
    );

    let observed = collect(&root, &policy).expect("collect live lock + mirror");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "the supply-chain-audit gate must be born-blocking GREEN on the live corpus (quinn fixed; \
         the 3 unmaintained ids in policy.ignore). Live findings:\n{}",
        render_findings(&findings)
    );
}

#[test]
fn committed_policy_names_the_authoritative_workspace_lockfile_corpus() {
    let root = repo_root();
    let policy = load_policy(&root);
    let configured = configured_lockfiles(&policy).expect("parse committed lockfile corpus");
    let paths = configured
        .iter()
        .map(|source| (source.manifest_path.as_str(), source.lockfile_path.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(
        paths,
        vec![
            ("Cargo.toml", "Cargo.lock"),
            ("kernel/Cargo.toml", "kernel/Cargo.lock"),
        ],
        "the policy corpus is the reviewed authority; collection must not infer it from mutable filesystem state"
    );

    let observed = collect(&root, &policy).expect("collect committed corpus");
    let keys = observed
        .as_object()
        .expect("observed graph object")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert!(
        keys.iter().any(|value| *value == "locked")
            && keys.iter().any(|value| *value == "locked_by_source")
            && keys.iter().any(|value| *value == "advisories")
            && keys.iter().any(|value| *value == "manifest"),
        "multi-lockfile collection must preserve the public observed JSON shape"
    );
    assert!(
        observed["locked"].as_array().is_some_and(|packages| {
            packages.iter().all(|package| {
                package.as_object().is_some_and(|object| {
                    object.len() == 2
                        && object.contains_key("name")
                        && object.contains_key("version")
                })
            })
        }),
        "locked records must remain exactly {{name, version}}; provenance must not silently break consumers"
    );
}

#[test]
fn configured_nested_lockfile_is_scanned_but_unconfigured_filesystem_noise_is_not() {
    let repo = TempRepo::new("nested-lockfile");
    repo.write("Cargo.toml", "[workspace]\n");
    repo.write("Cargo.lock", MINIMAL_LOCK);
    repo.write(
        "nested/Cargo.toml",
        "[package]\nname = \"nested\"\nversion = \"0.1.0\"\n\n[workspace]\n",
    );
    repo.write(
        "nested/Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"quinn-proto\"\nversion = \"0.11.14\"\n",
    );
    repo.write("scratch/Cargo.toml", "[workspace]\n");
    repo.write(
        "scratch/Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"unconfigured-noise\"\nversion = \"9.9.9\"\n",
    );
    let advisories = vec![quinn_fixture()];
    repo.write_mirror(&advisories);
    let policy = repo.policy(json!([
        { "manifest_path": "nested/Cargo.toml", "lockfile_path": "nested/Cargo.lock" },
        { "manifest_path": "Cargo.toml", "lockfile_path": "Cargo.lock" }
    ]));

    let observed = collect(&repo.root, &policy).expect("collect declared lockfiles");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.iter().any(|finding| finding.code == "SCA-VULN"),
        "a vulnerable dependency present only in a configured nested lockfile must block"
    );
    assert!(
        findings.iter().any(|finding| finding.code == "SCA-VULN"
            && finding.key == "RUSTSEC-2026-0185::nested/Cargo.lock/0.11.14"
            && finding.detail.contains("lockfile `nested/Cargo.lock`")
            && finding.detail.contains("version `0.11.14`")),
        "nested provenance must be explicit in finding key and detail"
    );
    assert!(
        observed["locked"]
            .as_array()
            .is_some_and(|packages| packages
                .iter()
                .all(|package| package["name"] != "unconfigured-noise")),
        "untracked or tool-created filesystem noise must not expand the authoritative corpus"
    );

    assert_eq!(
        observed["locked_by_source"]
            .as_array()
            .map(|rows| rows.len()),
        Some(2),
        "multi-lockfile provenance must retain one row per declared lockfile package"
    );
}

#[test]
fn legacy_single_lockfile_policy_and_observed_shape_remain_supported() {
    let repo = TempRepo::new("legacy-lockfile");
    repo.write("Cargo.toml", "[workspace]\n");
    repo.write("Cargo.lock", MINIMAL_LOCK);
    repo.write_mirror(&[]);
    let policy = json!({
        "gate_id": GATE_ID,
        "lockfile_path": "Cargo.lock",
        "mirror_dir": "mirror",
        "unmaintained_policy": "all",
        "min_advisories": 0,
        "ignore": [],
    });

    let observed = collect(&repo.root, &policy).expect("legacy policy remains accepted");
    assert!(
        observed
            .as_object()
            .is_some_and(|object| object.len() == 4 && object.contains_key("locked_by_source")),
        "legacy policy should still emit the new locked_by_source projection"
    );
    assert_eq!(
        observed["locked_by_source"],
        json!([{"name": "serde", "version": "1.0.0", "lockfile": "Cargo.lock"}]),
    );
    assert_eq!(
        observed["locked"],
        json!([{"name": "serde", "version": "1.0.0"}]),
    );
}

#[test]
fn corpus_order_does_not_change_sorted_deduplicated_observed_packages() {
    let repo = TempRepo::new("deterministic-corpus");
    repo.write("Cargo.toml", "[workspace]\n");
    repo.write("Cargo.lock", MINIMAL_LOCK);
    repo.write(
        "nested/Cargo.toml",
        "[package]\nname = \"nested\"\nversion = \"0.1.0\"\n\n[workspace]\n",
    );
    repo.write("nested/Cargo.lock", MINIMAL_LOCK);
    repo.write_mirror(&[]);

    let reverse = repo.policy(json!([
        { "manifest_path": "nested/Cargo.toml", "lockfile_path": "nested/Cargo.lock" },
        { "manifest_path": "Cargo.toml", "lockfile_path": "Cargo.lock" }
    ]));
    let forward = repo.policy(json!([
        { "manifest_path": "Cargo.toml", "lockfile_path": "Cargo.lock" },
        { "manifest_path": "nested/Cargo.toml", "lockfile_path": "nested/Cargo.lock" }
    ]));

    let reverse_observed = collect(&repo.root, &reverse).expect("collect reverse corpus");
    let forward_observed = collect(&repo.root, &forward).expect("collect forward corpus");
    assert_eq!(reverse_observed, forward_observed);
    assert_eq!(
        reverse_observed["locked"],
        json!([{"name": "serde", "version": "1.0.0"}]),
        "duplicate name/version pairs across lockfiles preserve legacy deduplication"
    );
}

#[test]
fn newly_tracked_workspace_lockfile_must_be_declared_in_policy() {
    let repo = TempRepo::new("undeclared-workspace-lockfile");
    repo.write("Cargo.toml", "[workspace]\n");
    repo.write("Cargo.lock", MINIMAL_LOCK);
    repo.write("fifth/Cargo.toml", "[workspace]\n");
    repo.write("fifth/Cargo.lock", MINIMAL_LOCK);
    repo.write_mirror(&[]);
    let policy = repo.policy_with_tracked_paths(
        json!([
            { "manifest_path": "Cargo.toml", "lockfile_path": "Cargo.lock" }
        ]),
        &[
            "Cargo.lock",
            "Cargo.toml",
            "fifth/Cargo.lock",
            "fifth/Cargo.toml",
        ],
    );

    let error = collect(&repo.root, &policy)
        .expect_err("a tracked fifth workspace lock must fail until policy declares it");
    assert!(
        error.to_string().contains("fifth/Cargo.lock")
            && error.to_string().contains("undeclared workspace-owned"),
        "totality error must name the undeclared workspace lock: {error}"
    );
}

#[test]
fn tracked_member_and_orphan_lockfiles_do_not_expand_workspace_corpus() {
    let repo = TempRepo::new("member-orphan-lockfiles");
    repo.write("Cargo.toml", "[workspace]\nmembers = [\"member\"]\n");
    repo.write("Cargo.lock", MINIMAL_LOCK);
    repo.write(
        "member/Cargo.toml",
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
    );
    repo.write(
        "member/Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"quinn-proto\"\nversion = \"0.11.14\"\n",
    );
    repo.write(
        "orphan/Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"quinn-proto\"\nversion = \"0.11.14\"\n",
    );
    repo.write_mirror(&[quinn_fixture()]);
    let policy = repo.policy_with_tracked_paths(
        json!([
            { "manifest_path": "Cargo.toml", "lockfile_path": "Cargo.lock" }
        ]),
        &[
            "Cargo.lock",
            "Cargo.toml",
            "member/Cargo.lock",
            "member/Cargo.toml",
            "orphan/Cargo.lock",
        ],
    );

    let observed =
        collect(&repo.root, &policy).expect("member/orphan locks are not workspace roots");
    assert!(
        evaluate_keyed(&policy, &observed).is_empty(),
        "member-local and orphan lockfiles must not silently expand the configured workspace corpus"
    );
    assert_eq!(
        observed["locked"],
        json!([{"name": "serde", "version": "1.0.0"}])
    );

    let expanded_policy = repo.policy_with_tracked_paths(
        json!([
            { "manifest_path": "Cargo.toml", "lockfile_path": "Cargo.lock" },
            { "manifest_path": "member/Cargo.toml", "lockfile_path": "member/Cargo.lock" }
        ]),
        &[
            "Cargo.lock",
            "Cargo.toml",
            "member/Cargo.lock",
            "member/Cargo.toml",
            "orphan/Cargo.lock",
        ],
    );
    let error = collect(&repo.root, &expanded_policy)
        .expect_err("declaring a member-local lock must not expand the workspace corpus");
    assert!(
        error.to_string().contains("member/Cargo.lock")
            && error
                .to_string()
                .contains("declared paths absent from the workspace-owned projection"),
        "exact projection must reject member-local policy expansion: {error}"
    );
}

#[test]
fn malformed_or_empty_package_rows_fail_collection_without_silent_drop() {
    let cases = [
        (
            "missing-name",
            "version = 4\n\n[[package]]\nversion = \"1.0.0\"\n",
        ),
        (
            "non-string-name",
            "version = 4\n\n[[package]]\nname = 7\nversion = \"1.0.0\"\n",
        ),
        (
            "missing-version",
            "version = 4\n\n[[package]]\nname = \"serde\"\n",
        ),
        (
            "non-string-version",
            "version = 4\n\n[[package]]\nname = \"serde\"\nversion = 7\n",
        ),
        (
            "empty-name",
            "version = 4\n\n[[package]]\nname = \"\"\nversion = \"1.0.0\"\n",
        ),
        (
            "empty-version",
            "version = 4\n\n[[package]]\nname = \"serde\"\nversion = \"\"\n",
        ),
        ("non-table-package", "version = 4\npackage = [7]\n"),
        ("zero-packages", "version = 4\npackage = []\n"),
    ];

    for (case, lockfile) in cases {
        let repo = TempRepo::new(case);
        repo.write("Cargo.toml", "[workspace]\n");
        repo.write("Cargo.lock", lockfile);
        repo.write_mirror(&[]);
        let policy = repo.policy(json!([
            { "manifest_path": "Cargo.toml", "lockfile_path": "Cargo.lock" }
        ]));

        let error = collect(&repo.root, &policy)
            .expect_err("every malformed or empty configured lock must fail closed");
        assert!(
            error.to_string().contains("Cargo.lock"),
            "{case} must identify its lockfile: {error}"
        );
    }
}

#[test]
fn malformed_vulnerable_nested_package_cannot_disappear_from_scan() {
    let repo = TempRepo::new("malformed-vulnerable-nested");
    repo.write("Cargo.toml", "[workspace]\n");
    repo.write("Cargo.lock", MINIMAL_LOCK);
    repo.write("nested/Cargo.toml", "[workspace]\n");
    repo.write(
        "nested/Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"safe\"\nversion = \"1.0.0\"\n\n[[package]]\nname = \"quinn-proto\"\n",
    );
    repo.write_mirror(&[quinn_fixture()]);
    let policy = repo.policy(json!([
        { "manifest_path": "Cargo.toml", "lockfile_path": "Cargo.lock" },
        { "manifest_path": "nested/Cargo.toml", "lockfile_path": "nested/Cargo.lock" }
    ]));

    let error = collect(&repo.root, &policy)
        .expect_err("a malformed vulnerable row must block rather than disappear");
    assert!(
        error.to_string().contains("nested/Cargo.lock")
            && error.to_string().contains("package[1]")
            && error.to_string().contains("version"),
        "strict nested-row error must preserve source and row provenance: {error}"
    );
}

#[test]
fn malformed_or_underflowing_lockfile_corpus_fails_closed() {
    let valid = |corpus: Value, floor: Value| {
        json!({
            "gate_id": GATE_ID,
            "lockfile_corpus": corpus,
            "min_lockfiles": floor,
            "scm_facts_path": "scm-facts.json",
            "mirror_dir": "mirror",
            "unmaintained_policy": "all",
            "min_advisories": 0,
            "ignore": [],
        })
    };
    let entry = json!({"manifest_path": "Cargo.toml", "lockfile_path": "Cargo.lock"});

    let mut missing_scm_facts = valid(json!([entry.clone()]), json!(1));
    missing_scm_facts
        .as_object_mut()
        .expect("fixture policy object")
        .remove("scm_facts_path");
    assert!(
        configured_lockfiles(&missing_scm_facts)
            .expect_err("structured corpus requires independent scm-facts totality")
            .to_string()
            .contains("scm_facts_path")
    );

    for policy in [
        valid(json!([entry.clone(), entry.clone()]), json!(2)),
        valid(
            json!([{"manifest_path": "Cargo.toml", "lockfile_path": "../Cargo.lock"}]),
            json!(1),
        ),
        valid(
            json!([{"manifest_path": "Cargo.toml", "lockfile_path": "/Cargo.lock"}]),
            json!(1),
        ),
        valid(
            json!([{"manifest_path": "nested//Cargo.toml", "lockfile_path": "nested/Cargo.lock"}]),
            json!(1),
        ),
        valid(
            json!([{"manifest_path": "nested/Cargo.toml", "lockfile_path": "Cargo.lock"}]),
            json!(1),
        ),
        valid(json!([entry.clone()]), Value::Null),
        valid(json!([entry.clone()]), json!(0)),
    ] {
        assert!(
            configured_lockfiles(&policy).is_err(),
            "invalid corpus must be rejected: {policy}"
        );
    }

    let underflow = valid(json!([entry]), json!(2));
    let findings = evaluate_keyed(&underflow, &observed(&[], vec![]));
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "SCA-LOCKFILE-UNDERFLOW"),
        "a corpus below min_lockfiles must emit SCA-LOCKFILE-UNDERFLOW"
    );
}

#[test]
fn missing_workspace_lockfile_fails_collection() {
    let repo = TempRepo::new("missing-lockfile");
    repo.write("Cargo.toml", "[workspace]\n");
    repo.write_mirror(&[]);
    let policy = repo.policy(json!([
        { "manifest_path": "Cargo.toml", "lockfile_path": "Cargo.lock" }
    ]));

    let error = collect(&repo.root, &policy).expect_err("missing authoritative lockfile must fail");
    assert!(error.to_string().contains("Cargo.lock"));

    std::fs::create_dir(repo.root.join("Cargo.lock")).expect("create non-file lockfile");
    let error = collect(&repo.root, &policy).expect_err("non-file lockfile must fail");
    assert!(error.to_string().contains("not a regular file"));
}

#[test]
fn malformed_workspace_manifest_and_mirror_escape_fail_collection() {
    let repo = TempRepo::new("malformed-manifest");
    repo.write("Cargo.toml", "[dependencies]\nserde = \"1\"\n");
    repo.write("Cargo.lock", MINIMAL_LOCK);
    repo.write_mirror(&[]);
    let mut policy = repo.policy(json!([
        { "manifest_path": "Cargo.toml", "lockfile_path": "Cargo.lock" }
    ]));

    let error = collect(&repo.root, &policy).expect_err("non-root manifest must fail");
    assert!(
        error
            .to_string()
            .contains("must declare [workspace] or [package]")
    );

    repo.write("Cargo.toml", "[workspace]\n");
    policy["mirror_dir"] = json!("../mirror");
    let error = collect(&repo.root, &policy).expect_err("mirror path escape must fail");
    assert!(error.to_string().contains("normalized repo-relative"));
}

#[cfg(unix)]
#[test]
fn symlinked_lockfile_manifest_and_path_component_are_rejected() {
    use std::os::unix::fs::symlink;

    for case in ["lockfile", "manifest", "directory"] {
        let repo = TempRepo::new(case);
        repo.write_mirror(&[]);
        let outside = TempRepo::new(&format!("outside-{case}"));
        outside.write("Cargo.toml", "[workspace]\n");
        outside.write("Cargo.lock", MINIMAL_LOCK);

        match case {
            "lockfile" => {
                repo.write("Cargo.toml", "[workspace]\n");
                symlink(
                    outside.root.join("Cargo.lock"),
                    repo.root.join("Cargo.lock"),
                )
                .expect("create lockfile symlink");
            }
            "manifest" => {
                repo.write("Cargo.lock", MINIMAL_LOCK);
                symlink(
                    outside.root.join("Cargo.toml"),
                    repo.root.join("Cargo.toml"),
                )
                .expect("create manifest symlink");
            }
            "directory" => {
                symlink(&outside.root, repo.root.join("nested")).expect("create directory symlink");
            }
            _ => unreachable!(),
        }
        let (manifest_path, lockfile_path) = if case == "directory" {
            ("nested/Cargo.toml", "nested/Cargo.lock")
        } else {
            ("Cargo.toml", "Cargo.lock")
        };
        let policy = repo.policy(json!([{
            "manifest_path": manifest_path,
            "lockfile_path": lockfile_path,
        }]));

        let error = collect(&repo.root, &policy).expect_err("symlinks must fail closed");
        assert!(
            error.to_string().contains("symlink"),
            "{case} error must diagnose the symlink: {error}"
        );
    }
}

#[test]
fn active_admission_wires_signature_provenance_sbom_and_vet_posture() {
    let root = repo_root();

    // The 48->2 matrix collapse (2026-08-01) removed this gate's dedicated matrix leg: the whole
    // `ci/facade` fleet is now executed once by `buck2 test //ci/...`, a strict superset of what
    // the matrix ran. The matrix-text assertion that used to live here is deliberately NOT
    // replaced by `workflow.contains("buck2 test //ci/...")` — that would re-commit the same
    // substring defect (a comment satisfies it) one rename later. Registration is owned by
    // `gate_registration.rs`, which resolves the patterns fan-in-reachable jobs actually execute
    // and requires this crate's BUCK to declare a test rule. The residual obligation THIS test
    // keeps is the one it can check locally: the gate's own Buck target must exist.
    assert!(
        root.join("ci/facade/supply-chain-audit/BUCK").is_file(),
        "the supply-chain audit gate must declare a Buck target to be executed by //ci/..."
    );

    let kyverno =
        std::fs::read_to_string(root.join("infra/kyverno/policies/verify-image-signed.yaml"))
            .expect("read keyless image policy");
    let broad_github_workflow = ["https://github.com/", ".+/.+", "/.github/workflows"].concat();
    for required in [
        "keyless:",
        "rekor:",
        "https://slsa.dev/provenance/v1",
        "https://cyclonedx.org/bom",
        "https://token.actions.githubusercontent.com",
        "https://github.com/jason931225/oyatie/.github/workflows/.+@refs/(heads/dev|tags/v.+)",
    ] {
        assert!(
            kyverno.contains(required),
            "keyless supply-chain admission policy must contain {required:?}"
        );
    }
    for forbidden in [
        "ExternalSecret",
        "cosign-key",
        "cosign-pub",
        "publicKeys:",
        broad_github_workflow.as_str(),
    ] {
        assert!(
            !kyverno.contains(forbidden),
            "static-key-only Cosign admission is not readiness authority; found {forbidden:?}"
        );
    }

    let legacy_keyless_policy =
        std::fs::read_to_string(root.join("infra/kyverno/policies/require-signed-images.yaml"))
            .expect("read legacy keyless image policy");
    assert!(
        legacy_keyless_policy.contains(
            "https://github.com/jason931225/oyatie/.github/workflows/.+@refs/(heads/dev|tags/v.+)"
        ) && !legacy_keyless_policy.contains(&broad_github_workflow),
        "secondary keyless policy must not keep the any-GitHub-repository wildcard"
    );

    for rel in [
        "marketplace/facade/dev-cli/src/commands/supply_chain.rs",
        "marketplace/facade/dev-cli/src/cloud_iac_kubewarden_admission_gate.rs",
    ] {
        let source = std::fs::read_to_string(root.join(rel)).expect("read supply-chain source");
        assert!(
            source.contains("https://github.com/jason931225/oyatie/.github/workflows/.+@refs/(heads/dev|tags/v.+)")
                && !source.contains(&broad_github_workflow),
            "{rel} must not preserve the any-GitHub-repository Cosign identity wildcard"
        );
    }

    for rel in [
        "iac/iac/k8s/kubewarden/verify-image-signatures-policy.yaml",
        "iac/iac/k8s/kubewarden/verification-config.yaml",
    ] {
        let policy = std::fs::read_to_string(root.join(rel)).expect("read kubewarden policy");
        assert!(
            policy.contains("owner: jason931225")
                && policy.contains("repo: oyatie")
                && !policy.contains("owner: oyatie"),
            "{rel} must bind GitHub Actions identity to the owned repository, not the owner wildcard"
        );
    }

    let cli_usage = std::fs::read_to_string(root.join("marketplace/facade/dev-cli/src/lib.rs"))
        .expect("read dev-cli usage");
    assert!(
        cli_usage.contains("--manifest <iac/manifest.json>")
            && cli_usage.contains("--chart-root <iac/iac/k8s/helm>")
            && cli_usage.contains("--kubewarden-root <iac/iac/k8s/kubewarden>"),
        "dev-cli help must advertise live iac/ supply-chain admission paths"
    );

    let checklist =
        std::fs::read_to_string(root.join("templates/checklists/release-readiness-checklist.md"))
            .expect("read release readiness checklist");
    assert!(
        checklist.contains("cloud-ci-supply-chain-audit")
            && !checklist.contains("Command:* `cargo vet`")
            && !checklist.contains("Command: `cargo vet`"),
        "release readiness must point at active cloud-ci supply-chain admission, not a missing cargo-vet command"
    );

    let cargo_vet_doc = std::fs::read_to_string(root.join("docs/governance-lanes/cargo-vet.md"))
        .expect("read cargo-vet governance lane");
    assert!(
        cargo_vet_doc.contains("Retired from live admission")
            && cargo_vet_doc.contains("ci_invocation: none while retired"),
        "cargo-vet must be explicitly retired from live readiness authority until maintained inputs exist"
    );

    let lane_index = std::fs::read_to_string(root.join("docs/governance-lanes/INDEX.md"))
        .expect("read governance lane index");
    assert!(
        lane_index.contains("cargo-vet | retired-until-inputs")
            && lane_index
                .contains("current dependency/advisory authority is `cloud-ci-supply-chain-audit`")
            && !lane_index.contains("cargo run -p oya-governance-cargo-vet"),
        "the canonical lane index must not present cargo-vet as live CI authority"
    );
}

/// The committed policy with the live-corpus DATA neutralized: empty ignore (so a synthetic 1-record
/// observation does not go stale), and a zero floor (so synthetic observations do not trip underflow).
/// The gate_id + unmaintained_policy are the REAL committed ones.
fn synthetic_policy(root: &Path) -> Value {
    let mut p = load_policy(root);
    p["ignore"] = json!([]);
    p["min_advisories"] = json!(0);
    p
}

fn observed(locked: &[(&str, &str)], advisories: Vec<Advisory>) -> Value {
    let hash = canonical_hash(&advisories);
    let count = advisories.len();
    json!({
        "locked": locked.iter().map(|(n, v)| json!({"name": n, "version": v})).collect::<Vec<_>>(),
        "advisories": serde_json::to_value(&advisories).unwrap(),
        "manifest": { "content_hash": hash, "advisory_count": count },
    })
}

fn quinn_fixture() -> Advisory {
    Advisory {
        id: "RUSTSEC-2026-0185".to_owned(),
        package: "quinn-proto".to_owned(),
        patched: vec![">= 0.11.15".to_owned()],
        unaffected: vec![],
        informational: None,
    }
}

#[test]
fn synthetic_quinn_0_11_14_is_vuln_keyed_to_exact_id() {
    let root = repo_root();
    let p = synthetic_policy(&root);
    let obs = observed(&[("quinn-proto", "0.11.14")], vec![quinn_fixture()]);
    let findings = evaluate_keyed(&p, &obs);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "SCA-VULN" && f.key == "RUSTSEC-2026-0185"),
        "quinn-proto 0.11.14 must be SCA-VULN keyed to RUSTSEC-2026-0185; got {}",
        render_findings(&findings)
    );
}

#[test]
fn synthetic_quinn_0_11_15_is_clean() {
    let root = repo_root();
    let p = synthetic_policy(&root);
    let obs = observed(&[("quinn-proto", "0.11.15")], vec![quinn_fixture()]);
    assert!(
        evaluate_keyed(&p, &obs).is_empty(),
        "quinn-proto 0.11.15 satisfies the patched range and must be clean"
    );
}

#[test]
fn synthetic_unmaintained_absent_then_present_in_ignore() {
    let root = repo_root();
    let adv = Advisory {
        id: "RUSTSEC-2024-0436".to_owned(),
        package: "paste".to_owned(),
        patched: vec![],
        unaffected: vec![],
        informational: Some("unmaintained".to_owned()),
    };
    let obs = observed(&[("paste", "1.0.15")], vec![adv]);

    // Absent from ignore → SCA-UNMAINTAINED (unmaintained_policy=all is the committed posture).
    let p = synthetic_policy(&root);
    let findings = evaluate_keyed(&p, &obs);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "SCA-UNMAINTAINED" && f.key == "RUSTSEC-2024-0436"),
        "unmaintained paste absent from ignore must be SCA-UNMAINTAINED; got {}",
        render_findings(&findings)
    );

    // Present in ignore → clean.
    let mut p2 = synthetic_policy(&root);
    p2["ignore"] =
        json!([{ "id": "RUSTSEC-2024-0436", "reason": "no drop-in", "remove_by": "2026-12-31" }]);
    assert!(
        evaluate_keyed(&p2, &obs).is_empty(),
        "an ignored, live-affected unmaintained crate must be clean"
    );
}

#[test]
fn synthetic_tampered_manifest_is_mirror_malformed() {
    let root = repo_root();
    let p = synthetic_policy(&root);
    let mut obs = observed(&[("quinn-proto", "0.11.15")], vec![quinn_fixture()]);
    obs["manifest"]["content_hash"] = json!("deadbeefdeadbeef");
    let findings = evaluate_keyed(&p, &obs);
    assert!(
        findings.iter().any(|f| f.code == "SCA-MIRROR-MALFORMED"),
        "a tampered manifest content_hash must be SCA-MIRROR-MALFORMED; got {}",
        render_findings(&findings)
    );
}
