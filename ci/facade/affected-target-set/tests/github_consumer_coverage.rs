// `.github/**` synthetic-seed completeness gate (ADR-0554 synthetic_dependencies).
//
// WHY THIS EXISTS. `.github/**` has no buck2 `owner()` — it is a whole-tree-scanner INPUT, not a
// declared graph src — so every `.github` touch used to escalate the affected-set lane to the
// FULL workspace tier. Two cheaper shapes were tried and both were wrong:
//
//   1. Declaring the class INERT (`[]`). That is a FALSE GREEN: `[]` asserts "not a buck2 graph
//      input", but a dozen gate tests READ `.github/**` at runtime by climbing from cwd to a
//      repo-root marker. A workflow-only PR resolved to `NoGraphTargets` and walked straight past
//      the no-new-shell ratchet. Reverted.
//   2. Making `.github` OWNED (`export_file` + `$(location)`). Subtler, also wrong: `resolve()`
//      short-circuits on a non-empty `owner()` — `seeds.extend(owners); continue;` — and never
//      reaches `synthetic_seeds()`. Owning these paths permanently DISABLES the synthetic
//      declaration for them and narrows the cone to the export's own rdeps. Abandoned.
//
// The right mechanism is the one the engine was designed for: a NON-EMPTY
// `synthetic_dependencies[".github/**"]` naming the real consumers' seed targets — the
// `docs/ideas/archive/**` shape (empty `owner()`, real SEED).
//
// WHY THE GATE, NOT JUST THE LIST. A hand-maintained seed list rots: consumer N+1 lands unwired,
// the cone silently narrows, nothing goes red. That is the SAME class of hand-maintained safety
// property that produced the reverted `[]` declaration, so the list is not acceptable without a
// mechanical completeness check. `scan_path_literal_consumers` derives the candidate set FROM THE
// TREE and this gate fails closed when a package reads `.github/**` but is absent from the
// declaration. The kernel stays repo-neutral (ADR-0548 R0): the scanned class is a PARAMETER, and
// `.github` — a repo fact — lives here in the gate test, not in the library.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ci_affected_target_set::{
    Change, Decision, PathLiteralConsumer, Policy, plan_changes, resolve,
    scan_path_literal_consumers,
};

/// The whole-tree-scanner path class this gate accounts for — the repo fact the neutral kernel
/// takes as a parameter, and the `synthetic_dependencies` key it maps to (`.github/**`).
const GITHUB_CLASS_DIR: &str = ".github";

/// A representative `.github` path used to interrogate the SHIPPED policy end to end.
const PROBE_PATH: &str = ".github/workflows/oya-ci-required.yml";

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Same anchor the gate tests under `ci/facade/` use, so this
/// gate resolves the tree identically to the consumers it is accounting for.
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

fn shipped_policy(root: &Path) -> Policy {
    let path = root.join("ci/facade/affected-target-set/affected-set-policy.json");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    Policy::from_json(&raw).expect("shipped affected-set policy must parse")
}

/// Run the SHIPPED policy through the real kernel for a lone `.github` change with an EMPTY
/// owner map — the exact production shape (`owner()` is empty for `.github/**` by definition) —
/// and return the seed set the declaration contributes.
fn resolve_github_probe(root: &Path) -> Decision {
    let policy = shipped_policy(root);
    let plan = plan_changes(&[Change::Present(PROBE_PATH.to_owned())], &policy);
    resolve(&plan, &BTreeMap::new(), &policy)
}

/// `root//ci/facade/x:` / `//ci/facade/x:` -> `ci/facade/x`, so declared seed target patterns
/// compare against the repo-root-relative package dirs the tree scan yields.
fn seed_package(seed: &str) -> String {
    let without_cell = seed.split_once("//").map_or(seed, |(_, rest)| rest);
    without_cell
        .split_once(':')
        .map_or(without_cell, |(pkg, _)| pkg)
        .to_owned()
}

// ───────────────── end-to-end: the declaration actually reaches the verdict ─────────────────

/// The load-bearing behavioural proof. With an EMPTY owner map (production reality for
/// `.github/**`), the SHIPPED policy must decide `Affected` with a non-empty seed set.
///
/// Both reverted attempts are pinned as explicit non-outcomes here: `NoGraphTargets` is the
/// `[]`-inert false green, and `Full` is the ~120-minute escalation the declaration exists to
/// avoid. Nothing in this test is a fixture — it reads the real policy file.
#[test]
fn github_path_with_no_owner_resolves_to_affected_through_the_shipped_policy() {
    let root = repo_root();
    match resolve_github_probe(&root) {
        Decision::Affected { seeds } => {
            assert!(
                !seeds.is_empty(),
                "the `.github/**` synthetic declaration must contribute seeds; an empty seed set \
                 is the reverted `[]`-inert false green wearing a different hat"
            );
        }
        Decision::NoGraphTargets => panic!(
            "`{PROBE_PATH}` resolved to NoGraphTargets — the `.github/**` class is declared INERT \
             again. That is the reverted PR #1389 false green: a workflow-only PR would bypass \
             the no-new-shell ratchet and every other whole-tree gate that reads `.github/**`."
        ),
        other => panic!(
            "`{PROBE_PATH}` must resolve to Affected via the synthetic declaration, got {other:?}"
        ),
    }
}

/// `.github/**` overlaps the `**/*.md` declaration (issue/PR templates live there).
/// `synthetic_seeds` UNIONS every matching pattern, so neither class may shadow the other — a
/// `.github/ISSUE_TEMPLATE/*.md` edit still has to reach every `.github/**` consumer, AND it must
/// additionally carry the citation-closure gate that `**/*.md` now seeds. Pinned because the
/// opposite semantics (first-match-wins, or one class dominating) would silently reintroduce the
/// PR #1389 hole for exactly the file class the templates live in.
#[test]
fn a_markdown_file_under_github_unions_both_classes_and_is_shadowed_by_neither() {
    const CITATION_GATE: &str =
        "root//governance/check/adr-citation-closure:check-adr-citation-closure-gate";
    let root = repo_root();
    let policy = shipped_policy(&root);
    let plan = plan_changes(
        &[Change::Present(".github/ISSUE_TEMPLATE/bug_report.md".to_owned())],
        &policy,
    );
    let Decision::Affected { seeds } = resolve(&plan, &BTreeMap::new(), &policy) else {
        panic!("a `.github` markdown change must still resolve to Affected, not be declared inert");
    };
    let Decision::Affected { seeds: yml_seeds } = resolve_github_probe(&root) else {
        panic!("`{PROBE_PATH}` must resolve to Affected");
    };
    let mut expected = yml_seeds;
    expected.push(CITATION_GATE.to_owned());
    expected.sort();
    expected.dedup();
    assert_eq!(
        seeds, expected,
        "a `.github` markdown change must union the `.github/**` seeds with the `**/*.md` \
         citation-closure seed; neither class may shadow the other"
    );
}

/// The single most important seed: the no-new-shell ratchet. Its policy declares
/// `.github/workflows` + `.github/actions` as scan roots and its frozen baseline is keyed
/// per-file, so a `.github` change it does not see is a merge-authority hole, not a slow lane.
#[test]
fn the_no_new_shell_ratchet_is_in_the_github_cone() {
    let root = repo_root();
    let Decision::Affected { seeds } = resolve_github_probe(&root) else {
        panic!("`{PROBE_PATH}` must resolve to Affected");
    };
    let packages: BTreeSet<String> = seeds.iter().map(|s| seed_package(s)).collect();
    assert!(
        packages.contains("ci/facade/automation-language-policy"),
        "the rust-first automation-language ratchet (the no-new-shell gate, whose scan roots ARE \
         `.github/workflows` + `.github/actions`) must be seeded by a `.github/**` change; \
         got {packages:?}"
    );
}

// ─────────────────────────── completeness: the list cannot rot ───────────────────────────

/// Surface-all completeness: EVERY package the tree scan derives as a `.github/**` consumer must
/// appear in the declaration. Extra declared seeds are legal (over-seeding costs build time;
/// under-seeding is a merge-authority hole), so this is a subset assertion, not equality.
#[test]
fn every_derived_github_consumer_is_declared_in_the_synthetic_seed_list() {
    let root = repo_root();
    let Decision::Affected { seeds } = resolve_github_probe(&root) else {
        panic!("`{PROBE_PATH}` must resolve to Affected");
    };
    let declared: BTreeSet<String> = seeds.iter().map(|s| seed_package(s)).collect();

    let consumers = scan_path_literal_consumers(&root, GITHUB_CLASS_DIR)
        .expect("scan the repo for `.github` consumers");
    assert!(
        !consumers.is_empty(),
        "the derivation found NO `.github` consumers at all — the scanner is broken (it would \
         rubber-stamp any declaration, including the reverted inert one)"
    );

    let undeclared: Vec<&PathLiteralConsumer> = consumers
        .iter()
        .filter(|c| !declared.contains(&c.package))
        .collect();
    assert!(
        undeclared.is_empty(),
        "buck2 package(s) name a repo-root-relative `.github` path but are NOT seeded by \
         `synthetic_dependencies[\".github/**\"]` in \
         ci/facade/affected-target-set/affected-set-policy.json — a `.github`-only PR would skip \
         them, exactly the PR #1389 false-green class. Add each package (as a `root//<dir>:` \
         package pattern) to that list:\n{}",
        undeclared
            .iter()
            .map(|c| format!("  root//{}:   (evidence: {})", c.package, c.evidence))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ─────────────────────────── RED fixture: the detector really fires ───────────────────────────

struct TestDir(PathBuf);

impl TestDir {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "oya-github-consumer-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, rel: &str, body: &str) {
        let path = self.0.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        fs::write(&path, body).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// RED fixture: a NEW gate crate that reads `.github/**` and was never added to the declaration.
/// This is the drift the completeness gate exists to catch; if the scanner misses it, the seed
/// list is decorative.
#[test]
fn an_undeclared_new_consumer_is_detected() {
    let dir = TestDir::new("red");
    dir.write("ci/facade/newcomer/BUCK", "rust_test(\n    name = \"newcomer-gate\",\n)\n");
    dir.write(
        "ci/facade/newcomer/tests/newcomer.rs",
        "fn main() { let _ = root.join(\".github/workflows/oya-ci-required.yml\"); }\n",
    );

    let found =
        scan_path_literal_consumers(dir.path(), GITHUB_CLASS_DIR).expect("scan fixture tree");
    assert_eq!(
        found,
        vec![PathLiteralConsumer {
            package: "ci/facade/newcomer".to_owned(),
            evidence: "ci/facade/newcomer/tests/newcomer.rs".to_owned(),
        }],
        "a new rust_test package naming a `.github` path MUST be surfaced as an undeclared consumer"
    );
}

/// GREEN counterparts — the three ways a `.github` string is NOT a verdict-flipping consumer.
/// Without these the RED fixture proves only that the scanner matches a substring.
#[test]
fn non_consumers_are_not_surfaced() {
    let dir = TestDir::new("green");

    // 1. Data-only package (no `rust_test`): `specs/` and `registry/` are full of `.github`
    //    strings and cannot produce a green verdict, so they are not consumers.
    dir.write("specs/BUCK", "export_file(\n    name = \"spec\",\n)\n");
    dir.write("specs/thing.json", "{ \"workflow\": \".github/workflows/x.yml\" }\n");

    // 2. Prose mention: a doc comment / JSON `_comment` names the path without quote-anchoring
    //    it, so it is not a path literal the package resolves.
    dir.write("ci/facade/prose/BUCK", "rust_test(\n    name = \"prose\",\n)\n");
    dir.write(
        "ci/facade/prose/src/lib.rs",
        "//! Replaces the inline shell in `.github/workflows/oya-ci-required.yml`.\n",
    );

    // 3. No enclosing buildfile: not a buck2 package at all, so it has no target to seed.
    dir.write("evidence/report.json", "{ \"path\": \".github/workflows/x.yml\" }\n");

    // 4. `.github` itself is the SUBJECT of the declaration, never a consumer of it — not even
    //    when it carries its own rust_test-bearing buildfile.
    dir.write(".github/BUCK", "rust_test(\n    name = \"self\",\n)\n");
    dir.write(".github/tooling/config.json", "{ \"self\": \".github/workflows/x.yml\" }\n");

    assert_eq!(
        scan_path_literal_consumers(dir.path(), GITHUB_CLASS_DIR).expect("scan fixture tree"),
        Vec::new(),
        "only quote-anchored `.github` path literals inside a rust_test-bearing buck2 package \
         are consumers"
    );
}
