// gate-registration completeness meta-test (ADR-0515 D2; CICD-DESIGN-PLAN Stage 1B + Pre-mortem
// Scenario-1c "silent-skip false-green" sibling acceptance test).
//
// INVARIANT: every gate crate directory under `cloud/cloud-ci/gates/` — EXCEPT the producer
// (`oya-cloud-ci-accounting-registry-app`, the rust_binary that EMITS the faces, not a gate
// lane) — MUST be registered as a job lane in `.github/workflows/oya-ci-required.yml`, the
// single canonical `oya-ci-required` fan-in. A new gate cannot be added without registering
// it in the required workflow; an in-tree-but-unregistered gate fails this test (it would be a
// silent false-green one level below the workflow's `needs:` fan-in).
//
// It is a pure filesystem + text gate: it reads the gates dir and greps the workflow yaml. No
// network, no GitHub API — runnable in any presubmit. Keep it deterministic and surface-all
// (it collects EVERY unregistered gate, not just the first).
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

/// The producer crate is NOT a gate lane — it is the rust_binary that EMITS the accounting
/// faces (registered in the workflow via `cargo run -p ...`, not `cargo test -p ...`). It is
/// the single intentional exclusion from the gate-registration invariant.
const PRODUCER_CRATE: &str = "oya-cloud-ci-accounting-registry-app";

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Mirrors the helper in `firewall.rs` so both meta-gates
/// resolve the root identically.
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

fn gates_dir(root: &Path) -> PathBuf {
    root.join("cloud/cloud-ci/gates")
}

fn workflow_path(root: &Path) -> PathBuf {
    root.join(".github/workflows/oya-ci-required.yml")
}

/// Every directory directly under `cloud/cloud-ci/gates/` that is a crate (has a Cargo.toml).
fn gate_crate_dirs(gates: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(gates)
        .unwrap_or_else(|e| panic!("read gates dir {}: {e}", gates.display()))
        .map(|entry| entry.expect("dir entry").path())
        .filter(|p| p.is_dir() && p.join("Cargo.toml").is_file())
        .map(|p| {
            p.file_name()
                .expect("dir file_name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

#[test]
fn every_gate_crate_is_registered_in_oya_ci_required_workflow() {
    let root = repo_root();
    let gates = gates_dir(&root);
    let wf = workflow_path(&root);

    let workflow = fs::read_to_string(&wf)
        .unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));

    let crates = gate_crate_dirs(&gates);
    assert!(
        !crates.is_empty(),
        "found no gate crates under {} — the gates dir layout changed",
        gates.display()
    );

    // The producer must be present in-tree (sanity: the exclusion is real, not a typo).
    assert!(
        crates.iter().any(|c| c == PRODUCER_CRATE),
        "producer crate {PRODUCER_CRATE} not found under {} — update PRODUCER_CRATE if it was \
         renamed",
        gates.display()
    );

    // Surface-all: collect EVERY unregistered gate, then assert the set is empty.
    let mut missing: Vec<String> = Vec::new();
    for crate_dir in &crates {
        if crate_dir == PRODUCER_CRATE {
            continue;
        }
        // A gate is "registered" iff the workflow references it as a cargo package via
        // `-p <crate>` (the package name equals the directory name for all gates). This binds
        // the registration to an actual `cargo test -p <crate>` lane, not a stray mention.
        let token = format!("-p {crate_dir}");
        if !workflow.contains(&token) {
            missing.push(crate_dir.clone());
        }
    }

    assert!(
        missing.is_empty(),
        "gate crate(s) present under {} but NOT registered as a `-p <crate>` lane in {}: {:?}\n\
         A new gate must be added as a job in the oya-ci-required fan-in (and to the fan-in \
         job's `needs:`), or the required check is a silent false-green.",
        gates.display(),
        wf.display(),
        missing
    );
}

/// Defense-in-depth: every registered gate must ALSO be wired into the fan-in job's `needs:`
/// list (a `cargo test -p <crate>` lane that the `oya-ci-required` job does not depend on
/// would never gate the required context). We assert each non-producer gate's directory name
/// appears somewhere after the `oya-ci-required:` job header in a `needs:` context.
#[test]
fn every_gate_lane_is_a_dependency_of_the_fan_in_job() {
    let root = repo_root();
    let gates = gates_dir(&root);
    let wf = workflow_path(&root);
    let workflow = fs::read_to_string(&wf)
        .unwrap_or_else(|e| panic!("read workflow {}: {e}", wf.display()));

    // Isolate the text of the fan-in job (from its `oya-ci-required:` header to EOF). The
    // fan-in is the last job in the file by construction.
    let fan_in_anchor = "\n  oya-ci-required:";
    let idx = workflow
        .find(fan_in_anchor)
        .expect("fan-in job `oya-ci-required:` not found in workflow");
    let fan_in_block = &workflow[idx..];
    assert!(
        fan_in_block.contains("needs:"),
        "fan-in job `oya-ci-required` has no `needs:` — it must depend on every gate lane"
    );

    // Map each non-producer gate crate to its expected job-name token in the fan-in `needs:`.
    // Job names in this workflow embed the gate identity (e.g. `gate-total-accounting`,
    // `gate-registry-drift`). We assert the gate's short identity appears in the fan-in block.
    let crates = gate_crate_dirs(&gates);
    let mut missing: Vec<String> = Vec::new();
    for crate_dir in &crates {
        if crate_dir == PRODUCER_CRATE {
            continue;
        }
        // Strip the `oya-cloud-ci-` prefix and `-app` suffix to get the gate's short identity
        // used in the job names; `registry-drift` is already short.
        let short = crate_dir
            .strip_prefix("oya-cloud-ci-")
            .unwrap_or(crate_dir)
            .strip_suffix("-app")
            .map(str::to_owned)
            .unwrap_or_else(|| crate_dir.clone());
        if !fan_in_block.contains(&short) {
            missing.push(format!("{crate_dir} (looked for `{short}` in fan-in needs)"));
        }
    }

    assert!(
        missing.is_empty(),
        "gate lane(s) not wired into the `oya-ci-required` fan-in job's `needs:` in {}: {:?}\n\
         Every gate lane must be a dependency of the fan-in or its failure cannot make the \
         required context red.",
        wf.display(),
        missing
    );
}
