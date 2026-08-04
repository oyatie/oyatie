// Live-corpus born-accounting liveness gate (ADR-0568 slice 3b invocation surface).
//
// WHY A GATE AND NOT A PRODUCER/CLI: `register_crate` MUTATES the hand-authored SSOTs (OWNERS,
// workspace member globs, the capability registry, an ADR's governed-path block, the catalog, the
// reachability registry). A materializer-invoked producer is the wrong shape — the materializer
// re-derives GENERATED faces and must never rewrite human merge surfaces — and a human-typed CLI is
// forbidden outright. The read-only half (`plan_registration`) is therefore the invocation surface:
// this gate drives it against the real checkout, where an EMPTY plan is the assertion that the
// subject crate is fully born-accounted and every loader agrees with the live SSOTs.
//
// Until this landed, `register_crate` had no caller outside its own tempdir fixtures: a loader that
// silently stopped matching the real repo shape (capability registry format, OWNERS resolution, the
// ADR governed-surfaces fence, member globs) was invisible to CI.
//
// ADR-0083 Tier-3: integration tests assert with unwrap/expect/panic.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use ci_crate_registration::{list_tracked_paths, plan_registration};
use oya_crate_registrar_kernel::{CrateRole, RegisterCrateRequest};

/// Corpus FLOOR. `git ls-files` currently returns ~19_000 tracked paths; a collector that degrades
/// to a near-empty universe must fail LOUD here rather than read as clean somewhere downstream.
const MIN_TRACKED_PATHS: usize = 15_000;

/// The subject crate: already fully born-accounted under ADR-0568, so its live plan is empty.
const SUBJECT_DIR: &str = "libs/oya-crate-registrar-kernel";

/// A real ADR that exists but does NOT enumerate the subject's governed paths — the RED fixture's
/// only mutation.
const UNRELATED_ADR: &str = "ADR-0600";

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

fn subject_request(owning_adr: &str) -> RegisterCrateRequest {
    RegisterCrateRequest {
        crate_dir: SUBJECT_DIR.to_owned(),
        // meta home the closed capability registry absorbs `libs/oya-crate-registrar-*` into.
        capability: "build/".to_owned(),
        owning_adr: owning_adr.to_owned(),
        owner: "cloud-ci-platform".to_owned(),
        role: CrateRole::Kernel,
        has_lib: true,
        has_test_code: true,
        catalog: None,
        extra_governed_paths: Vec::new(),
    }
}

// FLOOR: a broken tracked-path collector cannot certify anything.
#[test]
fn live_tracked_path_corpus_meets_the_floor() {
    let root = repo_root();
    let tracked = list_tracked_paths(&root).expect("git ls-files over the live checkout");
    assert!(
        tracked.len() >= MIN_TRACKED_PATHS,
        "tracked-path corpus collapsed to {} (floor {MIN_TRACKED_PATHS}) — the loader is broken, \
         not the repo",
        tracked.len()
    );
    assert!(
        tracked.iter().any(|p| p == &format!("{SUBJECT_DIR}/Cargo.toml")),
        "subject crate is not in the tracked universe — the gate would assert over nothing"
    );
}

// GREEN: the orchestrator's read-only planner, driven over the live corpus, agrees that the subject
// crate is fully born-accounted. Any SSOT regression (a deleted OWNERS file, a dropped capability
// mapping, a member glob narrowed past the dir, a governed path removed from ADR-0568) turns this
// RED — and so does a loader that stops understanding the live file formats.
#[test]
fn live_registration_plan_for_an_already_registered_crate_is_empty() {
    let root = repo_root();
    let plan = plan_registration(&root, &subject_request("ADR-0568"))
        .expect("planning an already-registered crate must not be refused");
    assert!(
        plan.edits.is_empty(),
        "{SUBJECT_DIR} is not fully born-accounted against the live SSOTs; outstanding edits: {:?}",
        plan.edits
    );
}

// RED FIXTURE: the same live corpus, one field changed — an owning ADR that does not enumerate the
// subject's governed paths. The planner MUST notice. Without this, an "always empty plan" loader
// bug would read as a permanent green above.
#[test]
fn live_planner_discriminates_an_unaccounted_governed_path_set() {
    let root = repo_root();
    assert!(
        adr_exists(&root, UNRELATED_ADR),
        "{UNRELATED_ADR} must exist — a MISSING ADR fails for the wrong reason"
    );
    let plan = plan_registration(&root, &subject_request(UNRELATED_ADR))
        .expect("planning against an unrelated ADR must plan, not refuse");
    assert!(
        !plan.edits.is_empty(),
        "planner emitted no edits for a crate whose governed paths {UNRELATED_ADR} does not \
         enumerate — the ADR governed-surfaces loader is inert"
    );
}

fn adr_exists(root: &Path, adr: &str) -> bool {
    let dir = root.join("docs/decisions");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.filter_map(Result::ok).any(|entry| {
        entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with(&format!("{adr}-")))
    })
}
