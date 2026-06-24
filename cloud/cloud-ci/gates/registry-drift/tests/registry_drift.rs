// :registry-drift gate — committed == regenerated (PHASE-0-FIREWALL-PLAN §5.3).
// Re-runs the producer in --stdout (sandbox) mode and byte-diffs against the committed
// accounting faces. The scm-facts face is now the ADR-0597 DE-COMMIT class (the last committed
// pure-derivation face, de-committed to kill the faces-serialization cascade): it has no
// committed copy to byte-compare, so it is validated by the REGENERATE-TWICE DETERMINISM canary
// (two fresh emitter runs must be byte-identical), matching the freshness gate's
// evaluate_face_determinism for the de-commit class. A non-deterministic emitter still fails
// here. ADR-0083 Tier-3: integration tests use unwrap/expect.
//
// HERMETIC: no `env!("CARGO")` (a compile-time cargo-only macro that breaks the buck2 build).
// The producer + emitter binaries are resolved at RUNTIME:
//   - under buck2: from the `OYA_CI_PRODUCER_BIN` / `OYA_CI_EMITTER_BIN` env vars that the
//     `$(exe ...)` macro on the rust_test target populates with the buck2-built binary path;
//   - under cargo (local dev): the producer/emitter are invoked via the runtime `CARGO` env
//     var (`cargo run -p <crate>`), which cargo sets for integration tests — a RUNTIME read,
//     never a compile-time `env!`, so there is no cargo-specific compile-time coupling.
// The scm-facts face the producer consumes is materialized on demand (ADR-0597 de-commit class:
// it is no longer tracked in git; CI/local materialize writes it to the faces dir before the
// producer reads it); the producer never calls git.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn faces_dir(root: &Path) -> PathBuf {
    root.join("cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app")
}

/// The committed generated faces and the `--face` name that regenerates each. registry-drift
/// extends across ALL of them: the registry + ttl-policy + the GATE-1 decision-crosswalk +
/// the GATE-4 enforcement-inventory/enforcement-liveness faces + the GO-LIVE gate-baseline (the accepted-debt
/// ratchet). A hand-edit to any one fails this gate. The baseline being byte-diff-protected
/// here is what makes laundering debt into the baseline tamper-evident: a hand-edit to widen
/// the accepted-violation set is itself registry_drift RED.
const FACES: [(&str, &str); 6] = [
    ("accounting-registry.generated.json", "registry"),
    ("ttl-policy.generated.json", "ttl-policy"),
    ("decision-crosswalk.generated.json", "decision-crosswalk"),
    (
        "enforcement-inventory.generated.json",
        "enforcement-inventory",
    ),
    (
        "enforcement-liveness.generated.json",
        "enforcement-liveness",
    ),
    ("gate-baseline.generated.json", "baseline"),
];

const SCM_FACTS_FACE: &str = "scm-facts.generated.json";

/// The committed reorg move-manifest face (task #64). Byte-bound to the codemod's deterministic
/// output exactly like the accounting faces + scm-facts: a hand-forged manifest row is
/// registry_drift RED before the firewall consumes it (the anti-forgery binding for the
/// rename-aware path-keyed baseline relabel).
const MOVE_MANIFEST_FACE: &str = "specs/reorg/move-manifest.generated.json";

/// Run the producer to regenerate a single face to stdout. Prefers the buck2-provided binary
/// (`OYA_CI_PRODUCER_BIN`), else falls back to `cargo run -p` via the RUNTIME `CARGO` env var.
fn regenerate_face(root: &Path, face: &str) -> String {
    let scm_facts = faces_dir(root).join(SCM_FACTS_FACE);
    let output = if let Ok(bin) = std::env::var("OYA_CI_PRODUCER_BIN") {
        Command::new(resolve_bin(root, &bin))
            .args(["--repo-root"])
            .arg(root)
            .args(["--scm-facts"])
            .arg(&scm_facts)
            .args(["--stdout", "--face", face])
            .current_dir(root)
            .output()
            .expect("run producer binary")
    } else {
        Command::new(cargo())
            .args([
                "run",
                "--quiet",
                "-p",
                "oya-cloud-ci-accounting-registry-app",
                "--",
                "--repo-root",
            ])
            .arg(root)
            .args(["--scm-facts"])
            .arg(&scm_facts)
            .args(["--stdout", "--face", face])
            .current_dir(root)
            .output()
            .expect("cargo run producer")
    };
    assert!(
        output.status.success(),
        "producer failed for face {face}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("producer stdout utf8")
}

/// The committed per-PR move plan (task #64), if any: a MOVE PR commits exactly one
/// `specs/reorg/<capability>-move-plan.json`. The codemod's `manifest` subcommand derives the
/// move-manifest from (this plan + the candidate tracked tree), so the regen here MUST pass the
/// same `--plan` the materialize pipeline does, or `committed != regenerated` would falsely RED a
/// real move PR (and falsely GREEN-empty a forged manifest). With no plan (a no-move PR) the
/// manifest is the canonical EMPTY identity manifest. The glob is sorted for determinism; the
/// first match is used (exactly one plan per move PR).
fn committed_move_plan(root: &Path) -> Option<PathBuf> {
    let dir = root.join("specs/reorg");
    let mut plans: Vec<PathBuf> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with("-move-plan.json"))
        })
        .collect();
    plans.sort();
    plans.into_iter().next()
}

/// Run the reorg codemod `manifest` subcommand to regenerate the move-manifest face to a temp
/// path, returning its bytes (task #64). Prefers the buck2-provided binary
/// (`OYA_CI_CODEMOD_BIN`), else `cargo run -p`. Passes the committed move plan via `--plan` (same
/// as the materialize pipeline) so committed==regenerated holds for a real move PR; with no plan
/// it emits the canonical EMPTY manifest. Reads `git ls-files`, so this is a git-boundary regen
/// (the caller gates it to the boundary context, identical to scm-facts).
fn regenerate_move_manifest(root: &Path) -> String {
    let out = std::env::temp_dir().join(format!(
        "oya-ci-move-manifest-regen-{}.json",
        std::process::id()
    ));
    let plan = committed_move_plan(root);
    let status = if let Ok(bin) = std::env::var("OYA_CI_CODEMOD_BIN") {
        let mut cmd = Command::new(resolve_bin(root, &bin));
        cmd.args(["manifest", "--repo-root"]).arg(root);
        if let Some(plan) = &plan {
            cmd.args(["--plan"]).arg(plan);
        }
        cmd.args(["--out"])
            .arg(&out)
            .current_dir(root)
            .status()
            .expect("run codemod binary")
    } else {
        let mut cmd = Command::new(cargo());
        cmd.args([
            "run",
            "--quiet",
            "-p",
            "oya-reorg-codemod-app",
            "--",
            "manifest",
            "--repo-root",
        ])
        .arg(root);
        if let Some(plan) = &plan {
            cmd.args(["--plan"]).arg(plan);
        }
        cmd.args(["--out"])
            .arg(&out)
            .current_dir(root)
            .status()
            .expect("cargo run codemod")
    };
    assert!(status.success(), "reorg codemod manifest failed");
    let bytes = fs::read_to_string(&out).expect("read regenerated move-manifest");
    let _ = fs::remove_file(&out);
    bytes
}

/// Run the scm-facts emitter to regenerate the scm-facts face to a temp path, returning its
/// bytes. Prefers the buck2-provided binary (`OYA_CI_EMITTER_BIN`), else `cargo run -p`.
/// `pass` discriminates the temp output path so the determinism canary can regenerate twice in
/// one process without the two passes colliding on the same temp file.
fn regenerate_scm_facts(root: &Path, pass: u32) -> String {
    let out = std::env::temp_dir().join(format!(
        "oya-ci-scm-facts-regen-{}-{pass}.json",
        std::process::id()
    ));
    // Route the volatile snapshot to a temp path too (ADR-0552): this regeneration exists
    // only to derive the de-commit-class stable face — a test action must never write the
    // checkout's materialized scm-volatile-facts snapshot.
    let volatile_out = std::env::temp_dir().join(format!(
        "oya-ci-scm-volatile-facts-regen-{}-{pass}.json",
        std::process::id()
    ));
    let status = if let Ok(bin) = std::env::var("OYA_CI_EMITTER_BIN") {
        Command::new(resolve_bin(root, &bin))
            .args(["--repo-root"])
            .arg(root)
            .args(["--out"])
            .arg(&out)
            .args(["--volatile-out"])
            .arg(&volatile_out)
            .current_dir(root)
            .status()
            .expect("run emitter binary")
    } else {
        Command::new(cargo())
            .args([
                "run",
                "--quiet",
                "-p",
                "oya-cloud-ci-scm-facts-emitter-app",
                "--",
                "--repo-root",
            ])
            .arg(root)
            .args(["--out"])
            .arg(&out)
            .args(["--volatile-out"])
            .arg(&volatile_out)
            .current_dir(root)
            .status()
            .expect("cargo run emitter")
    };
    assert!(status.success(), "scm-facts emitter failed");
    let bytes = fs::read_to_string(&out).expect("read regenerated scm-facts");
    let _ = fs::remove_file(&out);
    let _ = fs::remove_file(&volatile_out);
    bytes
}

/// The runtime `CARGO` env var (set by cargo for integration tests). NOT `env!("CARGO")` —
/// that is a compile-time macro that only cargo populates, which breaks the buck2 build.
fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

/// `$(exe ...)` yields a path relative to the test CWD (the project root) under buck2; make it
/// absolute against the resolved repo root so the test can exec it.
fn resolve_bin(root: &Path, bin: &str) -> PathBuf {
    let p = PathBuf::from(bin);
    if p.is_absolute() { p } else { root.join(p) }
}

/// Regenerate each face in-memory (sandbox) and assert it byte-matches the committed face.
#[test]
fn committed_faces_equal_regenerated() {
    let root = repo_root();
    let dir = faces_dir(&root);

    for (file, face) in FACES {
        let committed_path = dir.join(file);
        let committed = fs::read_to_string(&committed_path).unwrap_or_else(|e| {
            panic!(
                "committed face missing at {} ({e}); run the producer to generate it",
                committed_path.display()
            )
        });

        let regenerated = regenerate_face(&root, face);

        assert_eq!(
            committed, regenerated,
            "REGISTRY DRIFT: committed {file} != regenerated. \
             A generated face was hand-edited, or source changed without re-running the producer. \
             Re-run //cloud/cloud-ci/gates:oya-cloud-ci-accounting-registry-app-bin to regenerate."
        );
    }
}

/// Regenerate the scm-facts face (the single git boundary) TWICE and assert the two emissions are
/// byte-identical (ADR-0597 de-commit-class determinism canary). scm-facts is no longer tracked in
/// git, so there is no committed copy to byte-compare; with byte-parity-to-committed retired, the
/// regenerate-twice determinism check is the integrity canary that keeps derive-on-demand sound —
/// a non-deterministic emitter must hard-fail here rather than silently green. This is the
/// registry-drift analog of the freshness gate's `evaluate_face_determinism` (OYA-CI-HERMETIC-
/// EXECUTION-DESIGN §1.4: scm-facts folds into the existing registry-drift tamper-evidence, no new
/// trust root).
#[test]
fn scm_facts_regenerates_deterministically() {
    // The scm-facts emitter is the SINGLE out-of-graph git boundary (OYA-CI-HERMETIC-EXECUTION-
    // DESIGN §1.5): git is allowed to run in the CI scm-facts-regen pre-step and in local cargo
    // dev, but NEVER inside a hermetic buck2 action (no ambient git in the action graph — an RBE
    // worker has no `.git`, and a shallow checkout collapses history non-deterministically, PM1).
    // So this regen-validation runs ONLY from a git-bearing boundary context:
    //   - under cargo (the `CARGO` env var is set by cargo for integration tests), and
    //   - in the CI scm-facts-regen pre-step (which sets `OYA_CI_SCM_FACTS_REGEN=1`).
    // When run as a sandboxed buck2 action (neither set), it SKIPS — git is intentionally out of
    // the action graph; the producer-faces drift check above stays fully hermetic. This is the
    // boundary doctrine, not a `local_only` / cargo-fallback escape: the SAME logic runs at the
    // out-of-graph boundary on every runner.
    let regen_boundary = std::env::var_os("CARGO").is_some()
        || std::env::var("OYA_CI_SCM_FACTS_REGEN").as_deref() == Ok("1");
    if !regen_boundary {
        eprintln!(
            "scm-facts regen-validation SKIPPED: not a git boundary context (run via cargo or \
             the CI scm-facts-regen pre-step with OYA_CI_SCM_FACTS_REGEN=1). The hermetic \
             producer-faces drift check ran; git stays out of the buck2 action graph."
        );
        return;
    }

    let root = repo_root();
    let first = regenerate_scm_facts(&root, 1);
    let second = regenerate_scm_facts(&root, 2);

    assert_eq!(
        first, second,
        "SCM-FACTS NON-DETERMINISTIC: two fresh emissions of {SCM_FACTS_FACE} differ. \
         The scm-facts emitter must be a pure function of the tracked tree (ADR-0597 de-commit \
         class: there is no committed copy, so regenerate-twice determinism is the integrity \
         canary). A non-deterministic emitter is a hard failure."
    );
}

/// Regenerate the reorg move-manifest face (task #64) via the codemod `manifest` subcommand and
/// assert it byte-matches the committed `specs/reorg/move-manifest.generated.json`. This extends
/// the committed==regenerated coverage to the move-manifest: a hand-forged manifest row is
/// registry_drift RED before the firewall's rename-aware relabel consumes it (the anti-forgery
/// binding). The codemod reads `git ls-files`, so — exactly like the scm-facts emitter — this
/// runs ONLY at a git boundary (cargo dev / CI regen pre-step with OYA_CI_SCM_FACTS_REGEN=1) and
/// SKIPS inside a hermetic buck2 action (no `.git` on an RBE worker).
#[test]
fn committed_move_manifest_equals_regenerated() {
    let regen_boundary = std::env::var_os("CARGO").is_some()
        || std::env::var("OYA_CI_SCM_FACTS_REGEN").as_deref() == Ok("1");
    if !regen_boundary {
        eprintln!(
            "move-manifest regen-validation SKIPPED: not a git boundary context (run via cargo \
             or the CI regen pre-step with OYA_CI_SCM_FACTS_REGEN=1). git stays out of the buck2 \
             action graph."
        );
        return;
    }

    let root = repo_root();
    let committed_path = root.join(MOVE_MANIFEST_FACE);
    let committed = fs::read_to_string(&committed_path).unwrap_or_else(|e| {
        panic!(
            "committed move-manifest face missing at {} ({e}); run the codemod manifest \
             subcommand to generate it",
            committed_path.display()
        )
    });

    let regenerated = regenerate_move_manifest(&root);

    assert_eq!(
        committed, regenerated,
        "MOVE-MANIFEST DRIFT: committed {MOVE_MANIFEST_FACE} != regenerated. \
         The move-manifest face was hand-edited, or the committed move plan / candidate tree \
         changed without re-running the codemod. Re-run \
         //tools/oya-reorg-codemod-app:oya-reorg-codemod manifest to regenerate."
    );
}
