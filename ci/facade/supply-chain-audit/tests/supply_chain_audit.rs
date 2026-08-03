//! cloud-ci-supply-chain-audit live-corpus self-test (owned RustSec advisory scan).
//!
//! Legs:
//!   0. LIVE COVERAGE: DERIVE the tree's `Cargo.lock` corpus by walking it and assert
//!      `policy.lockfile_paths` equals it. The gate scans exactly what that array names, so the array
//!      is the coverage boundary; asserting it against a derivation (rather than trusting review) is
//!      what stops a newly added workspace from being silently unscanned. Paired with a synthetic
//!      RED/GREEN leg over a temp repo, and a leg proving a vulnerable pin present ONLY in a
//!      non-root declared lockfile is actually scanned.
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

use oya_advisory_mirror_kernel::{Advisory, canonical_hash};
use ci_supply_chain_audit::{
    GATE_ID, collect, configured_lockfiles, discover_lockfiles, evaluate_keyed, render_findings,
};
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
    root.join(
        "ci/facade/supply-chain-audit/supply-chain-audit-policy.json",
    )
}

fn load_policy(root: &Path) -> Value {
    let text = std::fs::read_to_string(policy_path(root)).expect("read committed policy");
    serde_json::from_str(&text).expect("parse committed policy")
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

/// LIVE COVERAGE: the declared lockfile corpus must equal the one DERIVED from this tree.
///
/// This is the leg that makes the multi-lockfile fix stay fixed. The gate scans exactly
/// `policy.lockfile_paths`, so that array is the coverage boundary; a `Cargo.lock` missing from it is
/// a workspace whose pins are never matched against the advisory mirror. Rather than trusting review
/// to keep the array current, the corpus is derived by walking the tree and asserted equal here — a
/// fourth workspace added tomorrow REDs until it is declared.
#[test]
fn live_lockfile_corpus_is_derived_and_fully_declared() {
    let root = repo_root();
    let policy = load_policy(&root);

    let declared = configured_lockfiles(&policy)
        .expect("committed policy must declare lockfile_paths as an array of strings");
    let derived = discover_lockfiles(&root).expect("walk the tree for Cargo.lock files");

    assert!(
        !derived.is_empty(),
        "the derivation found NO Cargo.lock in {}; a walk that reports nothing must never be read \
         as full coverage",
        root.display()
    );
    assert_eq!(
        declared,
        derived,
        "policy.lockfile_paths must equal the lockfile corpus derived from the tree.\n  \
         undeclared (present on disk, UNSCANNED): {:?}\n  \
         stale (declared, no file behind it): {:?}",
        derived.iter().filter(|p| !declared.contains(p)).collect::<Vec<_>>(),
        declared.iter().filter(|p| !derived.contains(p)).collect::<Vec<_>>(),
    );

    // And the same thing through the gate's own predicate, so the assertion is what BLOCKS — not
    // just what this test compares.
    let observed = collect(&root, &policy).expect("collect live corpus");
    let coverage: Vec<String> = evaluate_keyed(&policy, &observed)
        .iter()
        .filter(|f| f.code.starts_with("SCA-LOCKFILE-"))
        .map(|f| format!("{} {}", f.code, f.key))
        .collect();
    assert!(coverage.is_empty(), "live coverage findings: {coverage:?}");
}

/// RED PROOF on a violating input: a lockfile that exists in the tree but is not declared.
///
/// Runs the REAL walk over a synthetic repo, so it also pins the walk itself — if a future skip-list
/// edit stopped descending into ordinary directories, the derivation would silently under-report and
/// this leg fails.
#[test]
fn an_undeclared_lockfile_in_the_tree_is_uncovered_then_green_once_declared() {
    let repo = TempRepo::new("lockfile-coverage");
    repo.write("Cargo.lock", MINIMAL_LOCK);
    repo.write("mirror/advisories.json", "[]");
    repo.write(
        "mirror/mirror-manifest.json",
        &format!(
            "{{\"content_hash\":\"{}\",\"advisory_count\":0}}",
            canonical_hash(&[])
        ),
    );
    let base = json!({
        "gate_id": GATE_ID,
        "mirror_dir": "mirror",
        "unmaintained_policy": "all",
        "min_advisories": 0,
        "ignore": [],
    });

    // Declaring only the root while a nested workspace exists is the exact pre-fix state.
    repo.write("fourth-workspace/Cargo.lock", MINIMAL_LOCK);
    let mut narrow = base.clone();
    narrow["lockfile_paths"] = json!(["Cargo.lock"]);
    let observed = collect(&repo.root, &narrow).expect("collect synthetic repo");
    let findings = evaluate_keyed(&narrow, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "SCA-LOCKFILE-UNCOVERED" && f.key == "fourth-workspace/Cargo.lock"),
        "an undeclared lockfile must be SCA-LOCKFILE-UNCOVERED; got {}",
        render_findings(&findings)
    );

    // Declaring it closes the hole.
    let mut full = base.clone();
    full["lockfile_paths"] = json!(["Cargo.lock", "fourth-workspace/Cargo.lock"]);
    let observed = collect(&repo.root, &full).expect("collect synthetic repo");
    assert!(
        evaluate_keyed(&full, &observed).is_empty(),
        "a declaration equal to the derived corpus must be GREEN"
    );

    // Removing the file without shrinking the declaration is the other direction.
    std::fs::remove_file(repo.root.join("fourth-workspace/Cargo.lock")).expect("remove lockfile");
    let observed = collect(&repo.root, &full).expect("collect synthetic repo");
    let findings = evaluate_keyed(&full, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "SCA-LOCKFILE-ABSENT" && f.key == "fourth-workspace/Cargo.lock"),
        "a declared-but-missing lockfile must be SCA-LOCKFILE-ABSENT; got {}",
        render_findings(&findings)
    );
}

/// The declared corpus is scanned in FULL: a vulnerable pin that exists only in a non-root lockfile
/// must still be found. Without this, "declare every lockfile" could be satisfied while the scan
/// still only read the first one.
#[test]
fn a_vulnerable_pin_in_a_non_root_lockfile_is_scanned() {
    let repo = TempRepo::new("nonroot-scan");
    repo.write("Cargo.lock", MINIMAL_LOCK);
    repo.write(
        "nested/Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"quinn-proto\"\nversion = \"0.11.14\"\n",
    );
    let advisories = vec![quinn_fixture()];
    repo.write(
        "mirror/advisories.json",
        &serde_json::to_string(&advisories).unwrap(),
    );
    repo.write(
        "mirror/mirror-manifest.json",
        &format!(
            "{{\"content_hash\":\"{}\",\"advisory_count\":1}}",
            canonical_hash(&advisories)
        ),
    );
    let policy = json!({
        "gate_id": GATE_ID,
        "lockfile_paths": ["Cargo.lock", "nested/Cargo.lock"],
        "mirror_dir": "mirror",
        "unmaintained_policy": "all",
        "min_advisories": 0,
        "ignore": [],
    });
    let observed = collect(&repo.root, &policy).expect("collect synthetic repo");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "SCA-VULN" && f.key == "RUSTSEC-2026-0185"),
        "a vulnerable pin present ONLY in a non-root declared lockfile must be scanned; got {}",
        render_findings(&findings)
    );
}

/// A `[[package]]`-free lockfile the gate is asked to scan. Nothing in the workspace should hold one,
/// but the corpus is now derived from the tree, so the parse path must name WHICH file broke rather
/// than fail with an unattributable "Cargo.lock: ..." message.
const MINIMAL_LOCK: &str = "version = 4\n\n[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\n";

struct TempRepo {
    root: PathBuf,
}

impl TempRepo {
    fn new(name: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir()
            .join(format!("oya-sca-{name}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&root).expect("create temp repo");
        Self { root }
    }

    fn write(&self, rel: &str, contents: &str) {
        let path = self.root.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(path, contents).expect("write fixture");
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
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
        legacy_keyless_policy
            .contains("https://github.com/jason931225/oyatie/.github/workflows/.+@refs/(heads/dev|tags/v.+)")
            && !legacy_keyless_policy.contains(&broad_github_workflow),
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
        "cloud/cloud-iac/iac/k8s/kubewarden/verify-image-signatures-policy.yaml",
        "cloud/cloud-iac/iac/k8s/kubewarden/verification-config.yaml",
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
        cli_usage.contains("--manifest <cloud/cloud-iac/manifest.json>")
            && cli_usage.contains("--chart-root <cloud/cloud-iac/iac/k8s/helm>")
            && cli_usage.contains("--kubewarden-root <cloud/cloud-iac/iac/k8s/kubewarden>"),
        "dev-cli help must advertise live cloud/cloud-iac supply-chain admission paths"
    );

    let checklist =
        std::fs::read_to_string(root.join("docs/checklists/release-readiness-checklist.md"))
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
            && lane_index.contains("current dependency/advisory authority is `cloud-ci-supply-chain-audit`")
            && !lane_index.contains("cargo run -p oya-governance-cargo-vet"),
        "the canonical lane index must not present cargo-vet as live CI authority"
    );
}

/// The committed policy with the live-corpus DATA neutralized: empty ignore (so a synthetic 1-record
/// observation does not go stale), a zero floor (so synthetic observations do not trip underflow),
/// and an empty declared lockfile corpus paired with the empty derived corpus [`observed`] reports
/// (so the coverage assertion is neutral here — it has its own live + synthetic legs below).
/// The gate_id + unmaintained_policy are the REAL committed ones.
fn synthetic_policy(root: &Path) -> Value {
    let mut p = load_policy(root);
    p["ignore"] = json!([]);
    p["min_advisories"] = json!(0);
    p["lockfile_paths"] = json!([]);
    p
}

fn observed(locked: &[(&str, &str)], advisories: Vec<Advisory>) -> Value {
    let hash = canonical_hash(&advisories);
    let count = advisories.len();
    json!({
        "locked": locked.iter().map(|(n, v)| json!({"name": n, "version": v})).collect::<Vec<_>>(),
        "discovered_lockfiles": [],
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
