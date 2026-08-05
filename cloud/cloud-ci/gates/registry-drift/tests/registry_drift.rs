// :registry-drift gate — materialized == regenerated (PHASE-0-FIREWALL-PLAN §5.3).
// Re-runs the producer in --stdout (sandbox) mode and byte-diffs against the materialized
// accounting faces, AND re-runs the scm-facts emitter and byte-diffs the committed
// scm-facts.generated.json (OYA-CI-HERMETIC-EXECUTION-DESIGN §1, Option C: the scm-facts face
// is byte-parity-protected exactly like the other faces). A hand-edit to any generated face —
// including scm-facts — fails this test. ADR-0083 Tier-3: integration tests use unwrap/expect.
//
// HERMETIC: no `env!("CARGO")` (a compile-time cargo-only macro that breaks the buck2 build).
// The producer + emitter binaries are resolved at RUNTIME:
//   - under buck2: from the `OYA_CI_PRODUCER_BIN` / `OYA_CI_EMITTER_BIN` env vars that the
//     `$(exe ...)` macro on the rust_test target populates with the buck2-built binary path;
//   - under cargo (local dev): the producer/emitter are invoked via the runtime `CARGO` env
//     var (`cargo run -p <crate>`), which cargo sets for integration tests — a RUNTIME read,
//     never a compile-time `env!`, so there is no cargo-specific compile-time coupling.
// The scm-facts face the producer consumes is the committed one (a declared input); the
// producer never calls git.
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

/// The materialized generated faces and the `--face` name that regenerates each. registry-drift
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

/// Run the scm-facts emitter to regenerate the scm-facts face to a temp path, returning its
/// bytes. Prefers the buck2-provided binary (`OYA_CI_EMITTER_BIN`), else `cargo run -p`.
fn regenerate_scm_facts(root: &Path) -> String {
    let out = std::env::temp_dir().join(format!(
        "oya-ci-scm-facts-regen-{}.json",
        std::process::id()
    ));
    let status = if let Ok(bin) = std::env::var("OYA_CI_EMITTER_BIN") {
        Command::new(resolve_bin(root, &bin))
            .args(["--repo-root"])
            .arg(root)
            .args(["--out"])
            .arg(&out)
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
            .current_dir(root)
            .status()
            .expect("cargo run emitter")
    };
    assert!(status.success(), "scm-facts emitter failed");
    let bytes = fs::read_to_string(&out).expect("read regenerated scm-facts");
    let _ = fs::remove_file(&out);
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

/// Regenerate each face in-memory (sandbox) and assert it byte-matches the materialized face.
#[test]
fn materialized_faces_equal_regenerated() {
    let root = repo_root();
    let dir = faces_dir(&root);

    for (file, face) in FACES {
        let materialized_path = dir.join(file);
        let materialized = fs::read_to_string(&materialized_path).unwrap_or_else(|e| {
            panic!(
                "materialized face missing at {} ({e}); run the producer to generate it",
                materialized_path.display()
            )
        });

        let regenerated = regenerate_face(&root, face);

        assert_eq!(
            materialized, regenerated,
            "REGISTRY DRIFT: materialized {file} != regenerated. \
             A generated face was hand-edited, or source changed without re-running the producer. \
             Re-run //cloud/cloud-ci/gates:oya-cloud-ci-accounting-registry-app-bin to regenerate."
        );
    }
}

/// Regenerate the scm-facts face (the single git boundary) and assert it byte-matches the
/// committed scm-facts.generated.json. A hand-edit to scm-facts — or a stale scm-facts vs the
/// real history — fails this test, identical to the other faces (OYA-CI-HERMETIC-EXECUTION-
/// DESIGN §1.4: scm-facts folds into the existing registry-drift tamper-evidence, no new trust
/// root).
#[test]
fn committed_scm_facts_equal_regenerated() {
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
    let committed_path = faces_dir(&root).join(SCM_FACTS_FACE);
    let committed = fs::read_to_string(&committed_path).unwrap_or_else(|e| {
        panic!(
            "committed scm-facts face missing at {} ({e}); run the scm-facts emitter to generate it",
            committed_path.display()
        )
    });

    let regenerated = regenerate_scm_facts(&root);

    assert_eq!(
        committed, regenerated,
        "SCM-FACTS DRIFT: committed {SCM_FACTS_FACE} != regenerated. \
         The scm-facts face was hand-edited, or git history advanced without re-running the \
         emitter. Re-run //cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app:oya-cloud-ci-scm-facts-emitter-app to regenerate."
    );
}
