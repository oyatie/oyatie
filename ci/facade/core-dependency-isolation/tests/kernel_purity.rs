// ADR-0547 cloud-ci-kernel-purity: born-blocking self-test over TODAY's real *-kernel/*-core
// crates. The test collects each kernel's workspace-internal dep closure from the live tree and
// asserts:
//   * the live corpus is born-blocking GREEN — every *-kernel/*-core crate (and its path-dep
//     closure) is free of denylisted transient deps today, so any NEW kernel-with-transient-dep
//     fails closed on arrival;
//   * the scan actually found the kernel census (KP-EMPTY-SCAN floor is met), so a broken
//     glob/collect cannot pass as a silent false-green;
//   * the committed policy gate_id matches the crate contract.
// Filesystem RED fixtures (materialized under the OS temp dir at runtime, see new_temp_repo below)
// prove the collector resolves real manifests and BUCK files including target-cfg/build-deps
// placements and a closure leak through a local adapter. Pure-unit RED fixtures live in src/lib.rs.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ci_core_dependency_isolation::{Verdict, collect_kernel_deps, evaluate, evaluate_keyed};
use serde_json::Value;

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Verbatim from the sibling gates so collection runs from the
/// resolved repo root, not the `cargo test` CWD.
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

fn gate_dir(root: &Path) -> PathBuf {
    root.join("ci/facade/core-dependency-isolation")
}

fn load_json(path: &Path) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

#[test]
fn live_kernel_corpus_is_born_blocking_pure() {
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("kernel-purity-policy.json"));

    let observed = collect_kernel_deps(&root, &policy)
        .expect("read-only kernel-dep collection should not need temp files or cleanup");
    let kernel_count = observed["kernel_crates_found"].as_u64().expect("count");
    // The floor comes from the policy (DATA), not a hardcoded literal, so ratcheting the floor is a
    // single reviewed policy edit. The 4 no_std cloud-kernel crates live in the workspace-excluded
    // nested workspace and are intentionally outside this count (documented in ADR-0547 D5).
    let floor = policy["min_expected_kernel_crates"]
        .as_u64()
        .expect("policy floor");
    assert!(
        kernel_count >= floor,
        "the live tree should carry at least the policy kernel-census floor ({floor}); got {kernel_count}"
    );

    let findings = evaluate_keyed(&policy, &observed);

    // Born-blocking: NO finding allowed on the live corpus. Surface the full set if any appears so
    // a regression names the offending kernel/dep rather than a bare count.
    assert!(
        findings.is_empty(),
        "kernel-purity is born-blocking green on the live corpus; got {} finding(s):\n{}",
        findings.len(),
        findings
            .iter()
            .map(|f| format!("  {} {}: {}", f.code, f.key, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);

    eprintln!(
        "KERNEL-PURITY live corpus: kernel_crates={kernel_count} findings=0 (born-blocking green)"
    );
}

#[test]
fn policy_gate_id_matches_the_crate_contract() {
    let root = repo_root();
    let policy = load_json(&gate_dir(&root).join("kernel-purity-policy.json"));
    assert_eq!(policy["gate_id"].as_str(), Some("cloud-ci-kernel-purity"));
}

// ---------------------------------------------------------------------------
// Filesystem fixture repos: prove collect_kernel_deps resolves real manifests + BUCK files from
// disk, including target-cfg/build-deps placements and a closure leak through a local adapter.
//
// The fixtures are MATERIALIZED under the OS temp dir at runtime (mirroring the
// oya-workspace-members-kernel test pattern) rather than committed as nested Cargo.toml files.
// A committed nested `[package]` Cargo.toml inside a workspace member's subtree would confuse
// cargo's workspace resolution; a runtime temp tree is hermetic and self-cleaning, and exercises
// the exact same collect_kernel_deps filesystem path.
// ---------------------------------------------------------------------------

use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempRepo {
    root: PathBuf,
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn new_temp_repo() -> TempRepo {
    let unique = format!(
        "oya-kp-fixture-{}-{}",
        std::process::id(),
        FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let root = std::env::temp_dir().join(unique);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture root");
    TempRepo { root }
}

fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create fixture parent dir");
    }
    std::fs::write(&path, contents).expect("write fixture file");
}

/// Write a root workspace manifest with the single-level glob the live repo uses.
fn write_root_manifest(root: &Path) {
    write_file(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"2\"\n",
    );
}

/// The committed denylist + globs + exceptions, but with the kernel-count floor lowered so the
/// small fixture repos do not trip KP-EMPTY-SCAN (the floor guards the LIVE corpus, not fixtures).
fn fixture_policy() -> Value {
    let live = repo_root();
    let mut policy = load_json(&gate_dir(&live).join("kernel-purity-policy.json"));
    policy["min_expected_kernel_crates"] = Value::from(1u64);
    policy
}

#[test]
fn red_repo_fixture_surfaces_every_violation_class_from_disk() {
    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);

    // fake-acme-kernel: kube under [target.'cfg(unix)'.dependencies] (Cargo) + sqlx in BUCK
    // rust_library deps (and kube in a rust_test block that must be IGNORED).
    write_file(
        root,
        "crates/fake-acme-kernel/Cargo.toml",
        "[package]\nname = \"fake-acme-kernel\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nserde = \"1\"\n\n\
         [target.'cfg(unix)'.dependencies]\nkube = \"0.99\"\n",
    );
    write_file(
        root,
        "crates/fake-acme-kernel/BUCK",
        "rust_library(\n    name = \"fake-acme-kernel\",\n    deps = [\n        \"third-party//:serde\",\n        \"third-party//:sqlx\",\n    ],\n)\n\n\
         rust_test(\n    name = \"fake-acme-kernel-unittest\",\n    deps = [\n        \"third-party//:k8s-openapi\",\n    ],\n)\n",
    );

    // fake-build-kernel: k8s-openapi under [build-dependencies].
    write_file(
        root,
        "crates/fake-build-kernel/Cargo.toml",
        "[package]\nname = \"fake-build-kernel\"\nversion = \"0.0.0\"\n\n\
         [build-dependencies]\nk8s-openapi = \"0.22\"\n",
    );

    // fake-leak-kernel -> fake-acme-adapter (local path dep) -> sqlx: a closure leak. The kernel
    // itself is directly clean; the transient is reached through the local adapter.
    write_file(
        root,
        "crates/fake-leak-kernel/Cargo.toml",
        "[package]\nname = \"fake-leak-kernel\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nserde = \"1\"\nfake-acme-adapter = { path = \"../fake-acme-adapter\" }\n",
    );
    write_file(
        root,
        "crates/fake-acme-adapter/Cargo.toml",
        "[package]\nname = \"fake-acme-adapter\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nsqlx = \"0.8\"\n",
    );

    let policy = fixture_policy();
    let observed = collect_kernel_deps(root, &policy).expect("collect red-repo fixture");
    let findings = evaluate_keyed(&policy, &observed);
    let by_code = |code: &str| -> BTreeSet<String> {
        findings
            .iter()
            .filter(|f| f.code == code)
            .map(|f| f.key.clone())
            .collect()
    };

    let cargo = by_code("KP-TRANSIENT-DEP-CARGO");
    let buck = by_code("KP-TRANSIENT-DEP-BUCK");
    assert!(
        cargo.contains("fake-acme-kernel:fake-acme-kernel:kube"),
        "kube under a target-cfg dependency table must be caught: {cargo:?}"
    );
    assert!(
        cargo.contains("fake-build-kernel:fake-build-kernel:k8s-openapi"),
        "k8s-openapi under [build-dependencies] must be caught: {cargo:?}"
    );
    assert!(
        buck.contains("fake-acme-kernel:fake-acme-kernel:sqlx"),
        "sqlx in the BUCK rust_library deps must be caught: {buck:?}"
    );
    assert!(
        !buck.iter().any(|key| key.ends_with(":k8s-openapi")),
        "k8s-openapi is only in a rust_test block and must be IGNORED: {buck:?}"
    );
    assert!(
        cargo.contains("fake-leak-kernel:fake-acme-adapter:sqlx"),
        "a transient reached through a local adapter path-dep closure must be caught: {cargo:?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

#[test]
fn green_repo_fixture_is_pure_including_core_arm_and_primitives() {
    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);

    // fake-acme-core exercises the *-core glob arm; the primitive deps must NOT false-positive.
    write_file(
        root,
        "crates/fake-acme-core/Cargo.toml",
        "[package]\nname = \"fake-acme-core\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nserde = \"1\"\naws-lc-rs = \"1\"\nlibc = \"0.2\"\nzeroize = \"1\"\n",
    );
    write_file(
        root,
        "crates/fake-pure-kernel/Cargo.toml",
        "[package]\nname = \"fake-pure-kernel\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nserde = \"1\"\ntokio = \"1\"\n",
    );
    write_file(
        root,
        "crates/fake-pure-kernel/BUCK",
        "rust_library(\n    name = \"fake-pure-kernel\",\n    deps = [\n        \"third-party//:serde\",\n        \"third-party//:tokio\",\n    ],\n)\n",
    );

    let policy = fixture_policy();
    let observed = collect_kernel_deps(root, &policy).expect("collect green-repo fixture");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings.is_empty(),
        "the green fixture must be pure; got {findings:#?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);
    assert!(
        observed["kernel_crates_found"].as_u64().unwrap() >= 2,
        "green fixture should enumerate its kernel + core crates"
    );
}

#[test]
fn target_cfg_build_dependencies_are_scanned_from_disk() {
    // A transient dep behind [target.'cfg(..)'.build-dependencies] is a legal rare placement that
    // must not false-green.
    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);
    write_file(
        root,
        "crates/fake-tcfg-kernel/Cargo.toml",
        "[package]\nname = \"fake-tcfg-kernel\"\nversion = \"0.0.0\"\n\n\
         [target.'cfg(unix)'.build-dependencies]\nk8s-openapi = \"0.22\"\n",
    );
    let policy = fixture_policy();
    let observed = collect_kernel_deps(root, &policy).expect("collect");
    let findings = evaluate_keyed(&policy, &observed);
    assert!(
        findings
            .iter()
            .any(|f| { f.code == "KP-TRANSIENT-DEP-CARGO" && f.key.ends_with(":k8s-openapi") }),
        "k8s-openapi under [target.*.build-dependencies] must be caught: {findings:#?}"
    );
}

#[test]
fn fix_removes_dead_transient_dep_and_turns_red_to_green() {
    // Automation-default end-to-end: a kernel declares a transient dep (kube) in Cargo.toml but
    // never references it in src -> the dep is dead -> plan_fixes + apply_fixes remove it
    // mechanically and the re-collected tree is GREEN. The BUCK file here carries NO kube edge,
    // so the ADR-0549 BUCK lane has nothing to do: it must stay byte-identical and report no
    // BUCK edit (no-op soundness).
    use ci_core_dependency_isolation::{apply_fixes, plan_fixes};

    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);

    // Dead dep: kube declared but src never mentions it.
    write_file(
        root,
        "crates/fake-dead-kernel/Cargo.toml",
        "[package]\nname = \"fake-dead-kernel\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nserde = \"1\"\nkube = \"0.99\"\n",
    );
    write_file(
        root,
        "crates/fake-dead-kernel/src/lib.rs",
        "// this kernel uses only serde\npub fn noop() {}\n",
    );
    write_file(
        root,
        "crates/fake-dead-kernel/BUCK",
        "rust_library(\n    name = \"fake-dead-kernel\",\n    deps = [\n        \"third-party//:serde\",\n    ],\n)\n",
    );
    let buck_before = std::fs::read(root.join("crates/fake-dead-kernel/BUCK")).unwrap();

    let policy = fixture_policy();
    let before = collect_kernel_deps(root, &policy).expect("collect before fix");
    assert_eq!(
        evaluate(&policy, &before).verdict,
        Verdict::Red,
        "dead transient dep must be RED before fix"
    );
    let fixes = plan_fixes(&policy, &before);
    assert_eq!(fixes.len(), 1, "exactly one auto-fix planned: {fixes:?}");
    assert_eq!(fixes[0].dep, "kube");

    let applied = apply_fixes(root, &fixes).expect("apply fixes");
    assert!(
        applied
            .iter()
            .any(|line| line.contains("Cargo.toml") && line.contains("kube")),
        "fix should report the Cargo.toml edit: {applied:?}"
    );
    assert!(
        !applied.iter().any(|line| line.contains("BUCK")),
        "the BUCK file carries no kube edge — no BUCK edit may be reported: {applied:?}"
    );

    // After the fix the kernel is pure.
    let after = collect_kernel_deps(root, &policy).expect("collect after fix");
    assert_eq!(
        evaluate(&policy, &after).verdict,
        Verdict::Green,
        "removing the dead dep must make the kernel green; findings: {:#?}",
        evaluate_keyed(&policy, &after)
    );
    // The Cargo.toml on disk no longer carries kube, but still carries serde.
    let cargo = std::fs::read_to_string(root.join("crates/fake-dead-kernel/Cargo.toml")).unwrap();
    assert!(!cargo.contains("kube ="), "kube line removed: {cargo}");
    assert!(cargo.contains("serde ="), "serde line preserved: {cargo}");
    // The BUCK file is byte-identical (no kube edge to remove — the remover is a no-op here).
    let buck_after = std::fs::read(root.join("crates/fake-dead-kernel/BUCK")).unwrap();
    assert_eq!(
        buck_before, buck_after,
        "BUCK file must be byte-identical after a no-op --fix"
    );
}

#[test]
fn fix_is_table_aware_and_does_not_corrupt_manifest() {
    // BLOCKER-2 + CRITICAL-1 regression (amended by ADR-0549): --fix must:
    //   (a) remove a plain [dependencies] dead dep;
    //   (b) NOT remove a same-named [dev-dependencies] line;
    //   (c) NOT remove a dep that is `optional = true` and wired via `dep:X` in [features]
    //       (CRITICAL-1: doing so would leave a dangling feature entry cargo rejects);
    //   (d) remove the dead rust_library kube edge from BUCK via the oya-buck-syntax-kernel
    //       sound parser + fixer harness (ADR-0549 closes the FRIC-1781200001 refusal-only
    //       descope) while the rust_test kube edge — out of detect scope — SURVIVES.
    //
    // This fixture uses TWO separate kernels:
    //   fake-plain-kernel: plain dead kube dep (no optional, no features) — IS auto-fixed.
    //   fake-feat-kernel:  optional+features-wired kube dep — IS NOT auto-fixed (CRITICAL-1).
    use ci_core_dependency_isolation::{apply_fixes, plan_fixes};

    let repo = new_temp_repo();
    let root = &repo.root;

    // Root workspace manifest covers both members.
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/fake-plain-kernel\", \"crates/fake-feat-kernel\"]\n",
    )
    .unwrap();

    // fake-plain-kernel: dead kube in [dependencies], no features wiring.
    write_file(
        root,
        "crates/fake-plain-kernel/Cargo.toml",
        "[package]\nname = \"fake-plain-kernel\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nserde = \"1\"\nkube = \"0.99\"\n\n\
         [dev-dependencies]\nkube = \"0.99\"\n",
    );
    write_file(
        root,
        "crates/fake-plain-kernel/src/lib.rs",
        "pub fn x() {}\n",
    );
    write_file(
        root,
        "crates/fake-plain-kernel/BUCK",
        "rust_library(\n    name = \"fake-plain-kernel\",\n    deps = [\n        \"third-party//:kube\",\n    ],\n)\n\n\
         rust_test(\n    name = \"fake-plain-kernel-unittest\",\n    deps = [\n        \"third-party//:kube\",\n    ],\n)\n",
    );

    // fake-feat-kernel: kube is optional=true AND wired via dep:kube in [features].
    // CRITICAL-1: this dep must NOT be auto-removed — doing so would leave `k8s = ["dep:kube"]`
    // dangling and cargo would reject the manifest.
    write_file(
        root,
        "crates/fake-feat-kernel/Cargo.toml",
        "[package]\nname = \"fake-feat-kernel\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nserde = \"1\"\nkube = { version = \"0.99\", optional = true }\n\n\
         [features]\nk8s = [\"dep:kube\"]\n",
    );
    write_file(
        root,
        "crates/fake-feat-kernel/src/lib.rs",
        "// this crate references nothing transient\npub fn x() {}\n",
    );
    write_file(
        root,
        "crates/fake-feat-kernel/BUCK",
        "rust_library(\n    name = \"fake-feat-kernel\",\n    deps = [],\n)\n",
    );

    let policy = fixture_policy();
    let observed = collect_kernel_deps(root, &policy).expect("collect");

    // ADR-0549: the dead rust_library kube edge in BUCK is now AUTO-FIXABLE — the sound parser +
    // fixer harness make the removal mechanical, citing the shared kernel in the next action.
    let findings = evaluate_keyed(&policy, &observed);
    let buck_finding = findings
        .iter()
        .find(|f| f.code == "KP-TRANSIENT-DEP-BUCK" && f.key.ends_with(":kube"))
        .expect("BUCK kube finding");
    assert!(
        buck_finding.auto_fixable,
        "a mechanically dead BUCK edge is auto-fixable under ADR-0549: {buck_finding:?}"
    );
    assert!(
        buck_finding.next_action.contains("oya-buck-syntax-kernel"),
        "the BUCK auto-fix action must cite the fixer harness: {buck_finding:?}"
    );

    let fixes = plan_fixes(&policy, &observed);

    // Only fake-plain-kernel's Cargo kube is auto-fixable; fake-feat-kernel's kube is
    // feature-backed and the BUCK edges never plan a fix.
    assert_eq!(
        fixes.len(),
        1,
        "exactly one auto-fix (fake-plain-kernel only): {fixes:?}"
    );
    assert_eq!(
        fixes[0].member_path, "crates/fake-plain-kernel",
        "fix targets the plain kernel: {fixes:?}"
    );
    apply_fixes(root, &fixes).expect("apply");

    // fake-plain-kernel: [dependencies] kube removed, [dev-dependencies] kube preserved.
    let plain_cargo =
        std::fs::read_to_string(root.join("crates/fake-plain-kernel/Cargo.toml")).unwrap();
    assert!(
        !plain_cargo.contains("\nkube = \"0.99\"\n") || plain_cargo.contains("[dev-dependencies]"),
        "the [dependencies] kube line should be removed: {plain_cargo}"
    );
    assert!(
        plain_cargo.contains("[dev-dependencies]\nkube = \"0.99\""),
        "dev-dep kube preserved: {plain_cargo}"
    );

    // fake-plain-kernel BUCK: the rust_library kube edge is REMOVED (sound parser + harness);
    // the rust_test kube edge is out of detect scope and survives untouched.
    let plain_buck = std::fs::read_to_string(root.join("crates/fake-plain-kernel/BUCK")).unwrap();
    assert_eq!(
        plain_buck.matches("third-party//:kube").count(),
        1,
        "exactly the rust_test kube edge survives: {plain_buck}"
    );
    assert!(
        plain_buck.contains("rust_test"),
        "the rust_test block must be structurally intact: {plain_buck}"
    );

    // fake-feat-kernel: manifest UNCHANGED — feature-backed optional dep must not be touched.
    let feat_cargo =
        std::fs::read_to_string(root.join("crates/fake-feat-kernel/Cargo.toml")).unwrap();
    assert!(
        feat_cargo.contains("kube = { version = \"0.99\", optional = true }"),
        "feature-backed optional kube must NOT be removed (CRITICAL-1): {feat_cargo}"
    );
    assert!(
        feat_cargo.contains("k8s = [\"dep:kube\"]"),
        "features entry must survive: {feat_cargo}"
    );
}

#[test]
fn fix_refuses_every_feature_reference_syntax_h1_h4_manifest_byte_identical() {
    // CRITICAL-A layer 1, end-to-end (round-3 fixtures H1–H4): a denylisted dep that is referenced
    // in [features] in ANY syntax must end in REFUSAL — plan_fixes schedules nothing, apply_fixes
    // is a no-op, and every manifest is byte-identical afterwards.
    //   H1: non-optional kube + k8s = ["kube/client"]      (sub-feature path, weak ref)
    //   H2: optional kube     + k8s = ["kube?/client"]     (optional-activation `?` syntax)
    //   H3: optional kube     + full = ["kube"]            (bare dep name)
    //   H4: optional kube under [target.'cfg(unix)'.dependencies] + k8s = ["dep:kube"]
    use ci_core_dependency_isolation::{apply_fixes, plan_fixes};

    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);

    let manifests: &[(&str, &str)] = &[
        (
            "crates/fake-h1-kernel/Cargo.toml",
            "[package]\nname = \"fake-h1-kernel\"\nversion = \"0.0.0\"\n\n\
             [dependencies]\nkube = \"0.99\"\n\n\
             [features]\nk8s = [\"kube/client\"]\n",
        ),
        (
            "crates/fake-h2-kernel/Cargo.toml",
            "[package]\nname = \"fake-h2-kernel\"\nversion = \"0.0.0\"\n\n\
             [dependencies]\nkube = { version = \"0.99\", optional = true }\n\n\
             [features]\nk8s = [\"kube?/client\"]\n",
        ),
        (
            "crates/fake-h3-kernel/Cargo.toml",
            "[package]\nname = \"fake-h3-kernel\"\nversion = \"0.0.0\"\n\n\
             [dependencies]\nkube = { version = \"0.99\", optional = true }\n\n\
             [features]\nfull = [\"kube\"]\n",
        ),
        (
            "crates/fake-h4-kernel/Cargo.toml",
            "[package]\nname = \"fake-h4-kernel\"\nversion = \"0.0.0\"\n\n\
             [target.'cfg(unix)'.dependencies]\nkube = { version = \"0.99\", optional = true }\n\n\
             [features]\nk8s = [\"dep:kube\"]\n",
        ),
    ];
    for (path, contents) in manifests {
        write_file(root, path, contents);
        // src never mentions kube — every dep is otherwise mechanically dead.
        let src = Path::new(path).parent().unwrap().join("src/lib.rs");
        write_file(root, src.to_str().unwrap(), "pub fn noop() {}\n");
    }

    let policy = fixture_policy();
    let observed = collect_kernel_deps(root, &policy).expect("collect");
    let findings = evaluate_keyed(&policy, &observed);

    // All four kernels are RED (the transient dep IS a violation)…
    for kernel in [
        "fake-h1-kernel",
        "fake-h2-kernel",
        "fake-h3-kernel",
        "fake-h4-kernel",
    ] {
        let finding = findings
            .iter()
            .find(|f| {
                f.code == "KP-TRANSIENT-DEP-CARGO" && f.key.starts_with(&format!("{kernel}:"))
            })
            .unwrap_or_else(|| panic!("{kernel} must have a kube finding: {findings:#?}"));
        // …but NONE is auto-fixable: every feature-reference syntax demotes to design-action.
        assert!(
            !finding.auto_fixable,
            "{kernel}: feature-referenced kube must NOT be auto-fixable: {finding:?}"
        );
        assert!(
            finding.next_action.contains("[features]"),
            "{kernel}: remediation must name the feature-backed reason (LOW-F): {finding:?}"
        );
    }

    // Refusal end-state: nothing planned, apply is a no-op, manifests byte-identical.
    let fixes = plan_fixes(&policy, &observed);
    assert!(
        fixes.is_empty(),
        "no fix may be planned for feature-referenced deps: {fixes:?}"
    );
    let before: Vec<Vec<u8>> = manifests
        .iter()
        .map(|(path, _)| std::fs::read(root.join(path)).unwrap())
        .collect();
    let applied = apply_fixes(root, &fixes).expect("apply (no-op)");
    assert!(applied.is_empty(), "apply must be a no-op: {applied:?}");
    for ((path, _), pre) in manifests.iter().zip(before) {
        let post = std::fs::read(root.join(path)).unwrap();
        assert_eq!(pre, post, "{path} must be byte-identical after refusal");
    }
}

#[test]
fn fix_refuses_optional_dep_whose_implicit_feature_a_sibling_requests() {
    // MED-X1 (reviewer-reproduced vector): an `optional = true` dep with ZERO src references and
    // NO mention in its own [features] still exports an IMPLICIT cargo feature named after the
    // dep. A sibling workspace member requests it via `features = ["kube"]` on its path dep.
    // Layer 1 (collect_features_referenced_deps) misses it — the OWNING manifest never references
    // it; layer 2 (`cargo metadata --no-deps`) misses it — no cross-member feature resolution.
    // The sound bound: optional deps are NEVER auto-fixable. End state must be REFUSAL: nothing
    // planned, every manifest byte-identical, and the gate never reports "passed" on this tree.
    use ci_core_dependency_isolation::{apply_fixes_with_validator, plan_fixes, render_findings};

    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);

    // The kernel: optional kube, NO [features] table at all, src never mentions kube.
    write_file(
        root,
        "crates/fake-implicit-kernel/Cargo.toml",
        "[package]\nname = \"fake-implicit-kernel\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nserde = \"1\"\nkube = { version = \"0.99\", optional = true }\n",
    );
    write_file(
        root,
        "crates/fake-implicit-kernel/src/lib.rs",
        "pub fn noop() {}\n",
    );

    // The sibling: requests the kernel's IMPLICIT `kube` feature on its path dep.
    write_file(
        root,
        "crates/fake-sibling-adapter/Cargo.toml",
        "[package]\nname = \"fake-sibling-adapter\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nfake-implicit-kernel = { path = \"../fake-implicit-kernel\", features = [\"kube\"] }\n",
    );
    write_file(
        root,
        "crates/fake-sibling-adapter/src/lib.rs",
        "pub fn noop() {}\n",
    );

    let policy = fixture_policy();
    let observed = collect_kernel_deps(root, &policy).expect("collect");
    let findings = evaluate_keyed(&policy, &observed);
    let f = findings
        .iter()
        .find(|f| {
            f.code == "KP-TRANSIENT-DEP-CARGO"
                && f.key == "fake-implicit-kernel:fake-implicit-kernel:kube"
        })
        .unwrap_or_else(|| panic!("kube finding expected: {findings:#?}"));
    assert!(
        !f.auto_fixable,
        "optional dep must NOT be auto-fixable even with no own-manifest [features] mention: {f:?}"
    );
    assert!(
        f.next_action.contains("implicit"),
        "remediation must explain the implicit-feature export: {f:?}"
    );

    // Refusal end-state. The injected Ok validator models exactly what `cargo metadata --no-deps`
    // reports for this tree: cross-member feature requests are NOT resolved, so layer 2 is blind
    // here — if classification were unsound the fix would be applied and the manifests would
    // diverge below.
    let fixes = plan_fixes(&policy, &observed);
    assert!(
        fixes.is_empty(),
        "no fix may be planned for an optional dep: {fixes:?}"
    );
    let kernel_path = root.join("crates/fake-implicit-kernel/Cargo.toml");
    let sibling_path = root.join("crates/fake-sibling-adapter/Cargo.toml");
    let pre_kernel = std::fs::read(&kernel_path).unwrap();
    let pre_sibling = std::fs::read(&sibling_path).unwrap();
    let applied = apply_fixes_with_validator(root, &fixes, |_| Ok(())).expect("apply (no-op)");
    assert!(applied.is_empty(), "apply must be a no-op: {applied:?}");
    assert_eq!(
        pre_kernel,
        std::fs::read(&kernel_path).unwrap(),
        "kernel manifest byte-identical"
    );
    assert_eq!(
        pre_sibling,
        std::fs::read(&sibling_path).unwrap(),
        "sibling manifest byte-identical"
    );

    // Never "passed": the tree stays RED with a design-action, not a false-green.
    let after = collect_kernel_deps(root, &policy).expect("re-collect");
    assert_eq!(evaluate(&policy, &after).verdict, Verdict::Red);
    let rendered = render_findings(&evaluate_keyed(&policy, &after));
    assert!(
        !rendered.contains("passed"),
        "the gate must never print 'passed' for this tree: {rendered}"
    );
    assert!(
        rendered.contains("DESIGN ACTIONS"),
        "design action must be reported: {rendered}"
    );
}

#[test]
fn rollback_restores_original_when_same_manifest_is_edited_twice() {
    // LOW-X3: TWO dead transient deps in ONE manifest produce two fixes against the SAME file.
    // On semantic-revalidation failure the rollback must restore the ORIGINAL pre-image — an
    // insertion-order restore would leave the file at the intermediate one-edit state.
    use ci_core_dependency_isolation::{apply_fixes_with_validator, plan_fixes};

    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);
    write_file(
        root,
        "crates/fake-twice-kernel/Cargo.toml",
        "[package]\nname = \"fake-twice-kernel\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nserde = \"1\"\nkube = \"0.99\"\nsqlx = \"0.8\"\n",
    );
    write_file(
        root,
        "crates/fake-twice-kernel/src/lib.rs",
        "pub fn noop() {}\n",
    );

    let policy = fixture_policy();
    let observed = collect_kernel_deps(root, &policy).expect("collect");
    let fixes = plan_fixes(&policy, &observed);
    assert_eq!(
        fixes.len(),
        2,
        "both dead deps planned against the same manifest: {fixes:?}"
    );
    assert!(
        fixes
            .iter()
            .all(|f| f.member_path == "crates/fake-twice-kernel"),
        "both fixes target the same member: {fixes:?}"
    );

    let manifest_path = root.join("crates/fake-twice-kernel/Cargo.toml");
    let pre = std::fs::read(&manifest_path).unwrap();
    let result = apply_fixes_with_validator(root, &fixes, |_| Err("injected failure".to_owned()));
    assert!(
        result.is_err(),
        "semantic failure must surface as an error: {result:?}"
    );
    let post = std::fs::read(&manifest_path).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&pre),
        String::from_utf8_lossy(&post),
        "rollback must restore the ORIGINAL manifest (both kube and sqlx lines), not the intermediate one-edit state"
    );
}

#[test]
fn fix_rolls_back_all_preimages_when_semantic_revalidation_fails() {
    // CRITICAL-A layer 2, end-to-end: when the post-edit semantic revalidation fails, ALL edited
    // manifests are restored byte-identically from their pre-images and the error instructs the
    // operator to treat the findings as design-actions, carrying the validator's error text.
    // The validator is injected so the rollback path is deterministic (no dependency on a cargo
    // binary in the test environment); apply_fixes wires the same path to `cargo metadata`.
    use ci_core_dependency_isolation::{apply_fixes_with_validator, plan_fixes};

    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);
    write_file(
        root,
        "crates/fake-rollback-kernel/Cargo.toml",
        "[package]\nname = \"fake-rollback-kernel\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nserde = \"1\"\nkube = \"0.99\"\n",
    );
    write_file(
        root,
        "crates/fake-rollback-kernel/src/lib.rs",
        "pub fn noop() {}\n",
    );
    // ADR-0549: the BUCK lane is active too — its edit must also roll back to the pre-image.
    write_file(
        root,
        "crates/fake-rollback-kernel/BUCK",
        "rust_library(\n    name = \"fake-rollback-kernel\",\n    deps = [\n        \"third-party//:serde\",\n        \"third-party//:kube\",\n    ],\n)\n",
    );

    let policy = fixture_policy();
    let observed = collect_kernel_deps(root, &policy).expect("collect");
    let fixes = plan_fixes(&policy, &observed);
    assert_eq!(
        fixes.len(),
        1,
        "the dead kube dep is planned for removal: {fixes:?}"
    );

    let manifest_path = root.join("crates/fake-rollback-kernel/Cargo.toml");
    let buck_path = root.join("crates/fake-rollback-kernel/BUCK");
    let pre = std::fs::read(&manifest_path).unwrap();
    let pre_buck = std::fs::read(&buck_path).unwrap();
    let result = apply_fixes_with_validator(root, &fixes, |_| {
        Err(
            "error: feature `k8s` includes `dep:kube`, but `kube` is not a dependency (injected)"
                .to_owned(),
        )
    });
    let err = result.expect_err("semantic failure must surface as an error");
    let message = err.to_string();
    assert!(
        message.contains("rolled back"),
        "error must state the rollback: {message}"
    );
    assert!(
        message.contains("DESIGN ACTION"),
        "error must reclassify as design-action: {message}"
    );
    assert!(
        message.contains("injected"),
        "error must carry the cargo error text: {message}"
    );
    let post = std::fs::read(&manifest_path).unwrap();
    assert_eq!(
        pre, post,
        "manifest must be restored byte-identically after rollback"
    );
    let post_buck = std::fs::read(&buck_path).unwrap();
    assert_eq!(
        pre_buck, post_buck,
        "BUCK must be restored byte-identically after rollback (ADR-0549)"
    );
}

#[test]
fn fix_leaves_live_build_dependency_in_place() {
    // BLOCKER-1 regression: a build-dep USED in build.rs must compile-survive --fix (never removed).
    use ci_core_dependency_isolation::{apply_fixes, plan_fixes};

    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);
    write_file(
        root,
        "crates/fake-build-live-kernel/Cargo.toml",
        "[package]\nname = \"fake-build-live-kernel\"\nversion = \"0.0.0\"\n\n\
         [build-dependencies]\nk8s-openapi = \"0.22\"\n",
    );
    // build.rs uses the build-dep — outside src/, the file BLOCKER-1 said was ignored.
    write_file(
        root,
        "crates/fake-build-live-kernel/build.rs",
        "fn main() { let _ = k8s_openapi::VERSION; }\n",
    );

    let policy = fixture_policy();
    let observed = collect_kernel_deps(root, &policy).expect("collect");
    // Still RED (a build-dep transient is a violation), but NOT auto-fixable.
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
    let fixes = plan_fixes(&policy, &observed);
    assert!(
        fixes.is_empty(),
        "a build-dep is never auto-fixed: {fixes:?}"
    );
    let applied = apply_fixes(root, &fixes).expect("apply (no-op)");
    assert!(applied.is_empty());
    let cargo =
        std::fs::read_to_string(root.join("crates/fake-build-live-kernel/Cargo.toml")).unwrap();
    assert!(
        cargo.contains("k8s-openapi ="),
        "build-dep left in place: {cargo}"
    );
}

#[test]
fn fix_leaves_used_transient_dep_in_place() {
    // A USED transient dep is a design action, not auto-fixable: --fix must not touch it.
    use ci_core_dependency_isolation::{apply_fixes, plan_fixes};

    let repo = new_temp_repo();
    let root = &repo.root;
    write_root_manifest(root);
    write_file(
        root,
        "crates/fake-live-kernel/Cargo.toml",
        "[package]\nname = \"fake-live-kernel\"\nversion = \"0.0.0\"\n\n\
         [dependencies]\nsqlx = \"0.8\"\n",
    );
    write_file(
        root,
        "crates/fake-live-kernel/src/lib.rs",
        "use sqlx::Pool;\npub fn uses_sqlx(_p: &Pool<sqlx::Postgres>) {}\n",
    );

    let policy = fixture_policy();
    let observed = collect_kernel_deps(root, &policy).expect("collect");
    let fixes = plan_fixes(&policy, &observed);
    assert!(
        fixes.is_empty(),
        "a used transient dep must not be auto-fixed: {fixes:?}"
    );
    let applied = apply_fixes(root, &fixes).expect("apply (no-op)");
    assert!(applied.is_empty());
    let cargo = std::fs::read_to_string(root.join("crates/fake-live-kernel/Cargo.toml")).unwrap();
    assert!(cargo.contains("sqlx ="), "used dep left in place: {cargo}");
}
