// :registry-drift gate — PR-owned committed faces == regenerated (PHASE-0-FIREWALL-PLAN §5.3).
// Re-runs the producer in --stdout (sandbox) mode and byte-diffs against the committed
// PR-owned accounting faces. Controller-owned/generated-output-control-plane faces are not PR
// merge surfaces: they must still be committed on the integration branch for ratchet consumers,
// and their producers must stay deterministic, but contributor PRs must not carry byte churn for
// those faces. The scm-facts face is now the ADR-0604 DE-COMMIT class (the last committed
// pure-derivation face, de-committed to kill the faces-serialization cascade): it has no committed
// copy to byte-compare, so it is validated by the REGENERATE-TWICE DETERMINISM canary (two fresh
// emitter runs must be byte-identical), matching the freshness gate's evaluate_face_determinism
// for the de-commit class. A non-deterministic emitter still fails here. ADR-0083 Tier-3:
// integration tests use unwrap/expect.
//
// HERMETIC: no `env!("CARGO")` (a compile-time cargo-only macro that breaks the buck2 build).
// The producer binary is resolved only from `OYA_CI_PRODUCER_BIN`; missing env fails closed so
// registry-drift cannot silently run `cargo run` inside a hermetic gate. Git-boundary emitter and
// codemod validation keep their documented local/CI boundary Cargo fallbacks below.
// The scm-facts face the producer consumes is materialized on demand (ADR-0604 de-commit class:
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
    root.join("ci/facade/artifact-inventory-registry")
}

/// The PR-owned committed generated faces and the `--face` name that regenerates each.
/// registry-drift byte-parity extends across the registry + ttl-policy + the GATE-1
/// decision-crosswalk + the GATE-4 enforcement-inventory/enforcement-liveness faces.
/// A PR-local hand-edit to any one fails this gate.
const BYTE_PARITY_FACES: [(&str, &str); 5] = [
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
];

/// Controller-owned faces are materialized on the integration branch for merge-base ratchet
/// consumers, but they are not contributor-PR byte-churn surfaces. The generated-output diff
/// policy rejects PR-local modifications to these faces; registry-drift therefore validates their
/// producer determinism and committed presence without comparing candidate-tree bytes to the
/// integration-branch snapshot.
const CONTROLLER_OWNED_FACES: [(&str, &str); 1] = [("gate-baseline.generated.json", "baseline")];

const SCM_FACTS_FACE: &str = "scm-facts.generated.json";
const ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS_ENV: &str =
    "OYA_CI_ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS";
const ENFORCEMENT_LIVENESS_CODEX_HOOKS_ENV: &str = "OYA_CI_ENFORCEMENT_LIVENESS_CODEX_HOOKS";
const ENFORCEMENT_LIVENESS_HOOKS_DIR_ENV: &str = "OYA_CI_ENFORCEMENT_LIVENESS_HOOKS_DIR";
const ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS: &str = ".claude/settings.json";
const ENFORCEMENT_LIVENESS_CODEX_HOOKS: &str = ".codex/hooks.json";
const ENFORCEMENT_LIVENESS_HOOKS_DIR: &str = "tools/hooks";

/// Run the producer to regenerate a single face to stdout. The producer binary must be
/// provided by `OYA_CI_PRODUCER_BIN`; missing env fails closed so hermetic gates cannot silently
/// fall back to Cargo.
fn regenerate_face(root: &Path, face: &str) -> String {
    let scm_facts = faces_dir(root).join(SCM_FACTS_FACE);
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let bin = producer_binary(root, producer_bin.as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let mut command = Command::new(bin);
    command
        .args(["--repo-root"])
        .arg(root)
        .args(["--scm-facts"])
        .arg(&scm_facts);
    append_declared_enforcement_liveness_corpus_args(&mut command, root);
    let output = command
        .args(["--stdout", "--face", face])
        .current_dir(root)
        .output()
        .expect("run producer binary");
    assert!(
        output.status.success(),
        "producer failed for face {face}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("producer stdout utf8")
}

fn producer_binary(root: &Path, producer_bin: Option<&str>) -> Result<PathBuf, String> {
    let Some(bin) = producer_bin else {
        return Err(
            "FAIL-CLOSED: missing OYA_CI_PRODUCER_BIN; Cargo fallback is forbidden".to_owned(),
        );
    };
    Ok(resolve_bin(root, bin))
}

#[test]
fn producer_binary_env_is_required_for_hermetic_gate() {
    let err = producer_binary(Path::new("/repo"), None)
        .expect_err("missing OYA_CI_PRODUCER_BIN must fail closed");
    assert!(err.contains("OYA_CI_PRODUCER_BIN"));
}

fn append_declared_enforcement_liveness_corpus_args(command: &mut Command, root: &Path) {
    append_enforcement_liveness_corpus_paths(
        command,
        &declared_corpus_file(
            root,
            ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS_ENV,
            ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS,
            "settings.json",
        ),
        &declared_corpus_file(
            root,
            ENFORCEMENT_LIVENESS_CODEX_HOOKS_ENV,
            ENFORCEMENT_LIVENESS_CODEX_HOOKS,
            "hooks.json",
        ),
        &declared_corpus_path(
            root,
            ENFORCEMENT_LIVENESS_HOOKS_DIR_ENV,
            ENFORCEMENT_LIVENESS_HOOKS_DIR,
        ),
    );
}

fn declared_corpus_file(
    root: &Path,
    env_key: &str,
    fallback_rel: &str,
    file_name: &str,
) -> PathBuf {
    let path = declared_corpus_path(root, env_key, fallback_rel);
    if path.is_file() {
        return path;
    }
    let nested = path.join(file_name);
    if nested.is_file() {
        return nested;
    }
    path
}

fn declared_corpus_path(root: &Path, env_key: &str, fallback_rel: &str) -> PathBuf {
    declared_corpus_path_from_env(
        root,
        env_key,
        fallback_rel,
        std::env::var("OYA_CI_PRODUCER_BIN").is_ok(),
        std::env::var(env_key).ok().as_deref(),
    )
}

fn declared_corpus_path_from_env(
    root: &Path,
    env_key: &str,
    fallback_rel: &str,
    buck_backed_producer: bool,
    env_value: Option<&str>,
) -> PathBuf {
    if let Some(value) = env_value {
        return resolve_bin(root, value);
    }
    assert!(
        !buck_backed_producer,
        "FAIL-CLOSED: buck-backed registry-drift producer invocation is missing declared corpus env {env_key}"
    );
    root.join(fallback_rel)
}

fn append_enforcement_liveness_corpus_paths(
    command: &mut Command,
    claude_settings: &Path,
    codex_hooks: &Path,
    hooks_dir: &Path,
) {
    command
        .arg("--enforcement-liveness-claude-settings")
        .arg(claude_settings)
        .arg("--enforcement-liveness-codex-hooks")
        .arg(codex_hooks)
        .arg("--enforcement-liveness-hooks-dir")
        .arg(hooks_dir);
}

fn append_manifest_args(command: &mut Command, root: &Path, out: &Path) {
    command
        .args(["manifest", "--repo-root"])
        .arg(root)
        .arg("--out")
        .arg(out);
}

/// Run the reorg codemod `manifest` subcommand to regenerate the move-manifest face to a temp
/// path, returning its bytes (task #64). Prefers the buck2-provided binary
/// (`OYA_CI_CODEMOD_BIN`), else `cargo run -p`. The codemod owns active-plan selection; callers do
/// not pass `--plan`, so landed and PARKED plans cannot override its single-active fail-closed
/// policy. Reads `git ls-files`, so this is a git-boundary regen (the caller gates it to the
/// boundary context, identical to scm-facts). `pass` discriminates the temp output path so the
/// determinism canary can regenerate twice in one process without the two passes colliding on the
/// same temp file.
fn regenerate_move_manifest(root: &Path, pass: u32) -> String {
    let out = std::env::temp_dir().join(format!(
        "oya-ci-move-manifest-regen-{}-{pass}.json",
        std::process::id()
    ));
    let status = if let Ok(bin) = std::env::var("OYA_CI_CODEMOD_BIN") {
        let mut cmd = Command::new(resolve_bin(root, &bin));
        append_manifest_args(&mut cmd, root, &out);
        cmd.current_dir(root).status().expect("run codemod binary")
    } else {
        let mut cmd = Command::new(cargo());
        cmd.args(["run", "--quiet", "-p", "oya-reorg-codemod-app", "--"]);
        append_manifest_args(&mut cmd, root, &out);
        cmd.current_dir(root).status().expect("cargo run codemod")
    };
    assert!(status.success(), "reorg codemod manifest failed");
    let bytes = fs::read_to_string(&out).expect("read regenerated move-manifest");
    let _ = fs::remove_file(&out);
    bytes
}

#[test]
fn registry_drift_delegates_move_plan_selection_to_codemod() {
    let mut command = Command::new("codemod");
    append_manifest_args(
        &mut command,
        Path::new("/repo"),
        Path::new("/tmp/move-manifest.json"),
    );

    let args: Vec<String> = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        args,
        [
            "manifest",
            "--repo-root",
            "/repo",
            "--out",
            "/tmp/move-manifest.json",
        ]
    );
    assert!(!args.iter().any(|arg| arg == "--plan"));
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

#[test]
fn producer_regeneration_declares_enforcement_liveness_corpus_args() {
    let mut command = Command::new("/tmp/producer");
    append_enforcement_liveness_corpus_paths(
        &mut command,
        Path::new("/repo/.claude/settings.json"),
        Path::new("/repo/.codex/hooks.json"),
        Path::new("/repo/tools/hooks"),
    );

    let args: Vec<String> = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    assert!(args.windows(2).any(|pair| {
        pair == [
            "--enforcement-liveness-claude-settings",
            "/repo/.claude/settings.json",
        ]
    }));
    assert!(args.windows(2).any(|pair| {
        pair == [
            "--enforcement-liveness-codex-hooks",
            "/repo/.codex/hooks.json",
        ]
    }));
    assert!(
        args.windows(2)
            .any(|pair| { pair == ["--enforcement-liveness-hooks-dir", "/repo/tools/hooks",] })
    );
}

#[test]
fn buck_backed_registry_drift_requires_declared_corpus_env() {
    let panic = std::panic::catch_unwind(|| {
        declared_corpus_path_from_env(
            Path::new("/repo"),
            "MISSING_CORPUS_ENV",
            "fallback",
            true,
            None,
        );
    })
    .expect_err("buck-backed missing corpus env must fail closed");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");
    assert!(message.contains("FAIL-CLOSED"));
    assert!(message.contains("MISSING_CORPUS_ENV"));
}

/// Regenerate each PR-owned face in-memory (sandbox) and assert it byte-matches the committed
/// face.
#[test]
fn committed_faces_equal_regenerated() {
    let root = repo_root();
    let dir = faces_dir(&root);

    for (file, face) in BYTE_PARITY_FACES {
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
             Re-run //ci/facade:oya-cloud-ci-accounting-registry-app-bin to regenerate."
        );
    }
}

/// Controller-owned faces must remain committed for integration-branch ratchet consumers, and
/// their producers must remain deterministic, but contributor PRs must not be forced to commit
/// their regenerated bytes. This is the registry-drift counterpart to the generated-output diff
/// policy's "not PR merge surfaces" rule.
#[test]
fn controller_owned_faces_regenerate_deterministically() {
    let root = repo_root();
    let dir = faces_dir(&root);

    for (file, face) in CONTROLLER_OWNED_FACES {
        let committed_path = dir.join(file);
        assert!(
            committed_path.is_file(),
            "controller-owned face missing at {}; the integration branch must keep this \
             materialized snapshot for merge-base ratchet consumers",
            committed_path.display()
        );

        let first = regenerate_face(&root, face);
        let second = regenerate_face(&root, face);

        assert!(
            !first.trim().is_empty(),
            "controller-owned face {file} regenerated empty output"
        );
        assert_eq!(
            first, second,
            "CONTROLLER-OWNED FACE NON-DETERMINISTIC: two fresh emissions of {file} differ. \
             Contributor PRs do not own generated byte churn for this face, so deterministic \
             regeneration is the integrity canary."
        );
    }
}

/// Regenerate the scm-facts face (the single git boundary) TWICE and assert the two emissions are
/// byte-identical (ADR-0604 de-commit-class determinism canary). scm-facts is no longer tracked in
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
         The scm-facts emitter must be a pure function of the tracked tree (ADR-0604 de-commit \
         class: there is no committed copy, so regenerate-twice determinism is the integrity \
         canary). A non-deterministic emitter is a hard failure."
    );
}

/// Regenerate the reorg move-manifest face (task #64) TWICE via the codemod `manifest` subcommand
/// and assert the two emissions are byte-identical (ADR-0614 de-commit-class determinism canary,
/// mirroring [`scm_facts_regenerates_deterministically`]). ADR-0614 amends ADR-0563 and de-commits
/// move-manifest: it is no longer tracked in git, so there is no committed copy to byte-compare.
/// With byte-parity-to-committed retired, the regenerate-twice determinism check is the integrity
/// canary that keeps derive-on-demand sound — a NON-DETERMINISTIC codemod (which would silently
/// forge a different bijection on the materialize leg vs the relabel-read leg) must hard-fail here
/// rather than green. The codemod reads `git ls-files`, so — exactly like the scm-facts emitter —
/// this runs ONLY at a git boundary (cargo dev / CI regen pre-step with OYA_CI_SCM_FACTS_REGEN=1)
/// and SKIPS inside a hermetic buck2 action (no `.git` on an RBE worker).
#[test]
fn move_manifest_regenerates_deterministically() {
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
    let first = regenerate_move_manifest(&root, 1);
    let second = regenerate_move_manifest(&root, 2);

    assert_eq!(
        first, second,
        "MOVE-MANIFEST NON-DETERMINISTIC: two fresh codemod `manifest` emissions differ. \
         move-manifest is de-committed (ADR-0614): there is no committed copy, so regenerate-twice \
         determinism is the integrity canary. The codemod must be a pure function of the committed \
         move plan(s) x candidate tracked tree. A non-deterministic generator is a hard failure. \
         Re-run //tools/oya-reorg-codemod-app:oya-reorg-codemod manifest to reproduce."
    );
}
