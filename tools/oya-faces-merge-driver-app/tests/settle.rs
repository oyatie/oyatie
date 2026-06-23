//! Integration tests for the AUTHORITATIVE post-merge settle (`run_post_merge_settle`), run with a
//! fake [`RegenPort`] so NO buck2 is needed. These are the task-#125 acceptance + fail-closed proof:
//!
//!   - GREEN: simulate a faces merge conflict on a real temp git repo, run the settle, assert the
//!     faces are written + staged faces-only and the freshness gate sees them settled (the merged
//!     faces == a from-scratch materialize of the merged tree), exit Ok.
//!   - FAIL-CLOSED (regen): a regen failure leaves the committed faces UNTOUCHED and returns Err.
//!   - FAIL-CLOSED (determinism/drift): a second-regeneration byte mismatch returns Err and never
//!     stages a face.
//!   - FAIL-CLOSED (dirty tree): an uncommitted non-face change refuses the settle (the post-merge-
//!     commit invariant) and returns Err.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use oya_cloud_ci_freshness_app::generated_face_paths;
use oya_faces_merge_driver_app::{
    Buck2RegenAdapter, ControlPlane, FacesMergeError, FacesMergeErrorKind, PRODUCER_TARGET, RegenPort,
    run_post_merge_settle,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-faces-settle-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture root");
    root
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run git");
    assert!(output.status.success(), "git {args:?} failed");
    String::from_utf8(output.stdout).expect("git stdout utf8")
}

/// A minimal control-plane manifest declaring the 7 accounting faces as regeneratable, so
/// `ControlPlane::load` succeeds. Mirrors the real manifest's relevant fields only (the closed-schema
/// gate is not run here — the settle only reads `artifacts[].{path,merge_policy,generator}`).
fn write_control_plane(root: &Path) {
    let registry_dir = root.join("registry");
    std::fs::create_dir_all(&registry_dir).expect("create registry dir");
    let mut artifacts = String::new();
    for path in generated_face_paths() {
        // Use the accounting producer target so each face is both regeneratable AND settle-capable
        // (the realistic shape; the driver's decline predicate is scoped to settle-capable).
        artifacts.push_str(&format!(
            "{{\"artifact_id\": \"{path}\", \"path\": \"{path}\", \"merge_policy\": \
             \"never-manual-merge-regenerate-from-source-tree\", \"generator\": \
             {{\"generator_target\": \"{PRODUCER_TARGET}\"}}}},",
        ));
    }
    let artifacts = artifacts.trim_end_matches(',');
    std::fs::write(
        registry_dir.join("generated-artifact-control-plane.json"),
        format!("{{\"artifacts\": [{artifacts}]}}\n"),
    )
    .expect("write control plane");
}

/// Seed a temp git repo with committed non-face content + the root-hub marker + the 7 committed
/// "old" faces. The post-merge settle regenerates the faces from this committed tree.
fn init_repo() -> PathBuf {
    let root = fixture_root();
    git(&root, &["init"]);
    git(&root, &["config", "user.name", "Oyatie Test"]);
    git(&root, &["config", "user.email", "oyatie-test@example.com"]);
    git(&root, &["config", "commit.gpgsign", "false"]);

    // root-hub marker so `discover_repo_root` (and any future caller) resolves the root.
    std::fs::create_dir_all(root.join("specs")).expect("create specs");
    std::fs::write(root.join("specs/root-hub-pointers.json"), "{}\n").expect("write marker");
    std::fs::write(root.join("README.md"), "content v1\n").expect("write content");
    write_control_plane(&root);

    for path in generated_face_paths() {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().expect("face parent")).expect("create face parent");
        std::fs::write(path, "stale conflicted face\n").expect("write face");
    }
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "seed"]);
    root
}

/// A fake [`RegenPort`] that writes deterministic "regenerated" bytes for each face under the faces
/// dir (the shape the real adapter produces), so the settle runs with NO buck2. Configurable to
/// simulate a regen failure or a determinism/drift mismatch.
struct FakeRegen {
    /// If true, `regenerate` returns Err without touching the tree (simulate a producer failure).
    fail_regen: bool,
    /// If Some, `verify_determinism` returns a Drift error for that face name (simulate non-determinism).
    drift_face: Option<String>,
    /// repo_roots `regenerate` was called with (so a test can assert it ran / did not).
    regen_calls: RefCell<Vec<PathBuf>>,
}

impl FakeRegen {
    fn ok() -> Self {
        Self {
            fail_regen: false,
            drift_face: None,
            regen_calls: RefCell::new(Vec::new()),
        }
    }
}

const FACES_DIR: &str = "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app";

fn regenerated_bytes(file_name: &str) -> String {
    format!("regenerated authoritative bytes for {file_name}\n")
}

impl RegenPort for FakeRegen {
    fn regenerate(&self, repo_root: &Path) -> Result<Vec<(String, String)>, FacesMergeError> {
        self.regen_calls.borrow_mut().push(repo_root.to_path_buf());
        if self.fail_regen {
            return Err(FacesMergeError::new(
                FacesMergeErrorKind::Regen,
                "simulated producer failure",
            ));
        }
        let mut faces = Vec::new();
        for path in generated_face_paths() {
            let file_name = Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .expect("face name")
                .to_owned();
            let bytes = regenerated_bytes(&file_name);
            let full = repo_root.join(FACES_DIR).join(&file_name);
            std::fs::create_dir_all(full.parent().expect("parent")).expect("create faces dir");
            std::fs::write(&full, &bytes).expect("write regenerated face");
            faces.push((file_name, bytes));
        }
        faces.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(faces)
    }

    fn verify_determinism(
        &self,
        _repo_root: &Path,
        _first: &[(String, String)],
    ) -> Result<(), FacesMergeError> {
        if let Some(face) = &self.drift_face {
            return Err(FacesMergeError::new(
                FacesMergeErrorKind::Drift,
                format!("simulated non-determinism on {face}"),
            ));
        }
        Ok(())
    }
}

#[test]
fn settle_resolves_conflicted_faces_and_stages_faces_only() {
    let root = init_repo();
    let regen = FakeRegen::ok();

    let faces = run_post_merge_settle(&root, &regen).expect("settle must succeed");
    assert_eq!(faces.len(), generated_face_paths().len());

    // The faces were regenerated + staged; the on-disk bytes are the authoritative regeneration,
    // NOT the stale conflicted bytes.
    for path in generated_face_paths() {
        let on_disk = std::fs::read_to_string(root.join(&path)).expect("read settled face");
        let file_name = Path::new(&path).file_name().unwrap().to_str().unwrap();
        assert_eq!(on_disk, regenerated_bytes(file_name), "face {path} is authoritative");
    }

    // Only the generated faces are staged (the freshness engine refuses any non-face staged path).
    let staged: std::collections::BTreeSet<String> =
        git_output(&root, &["diff", "--cached", "--name-only"])
            .lines()
            .map(str::to_owned)
            .collect();
    let face_paths: std::collections::BTreeSet<String> = generated_face_paths().into_iter().collect();
    assert_eq!(staged, face_paths, "only faces are staged");

    // The regen ran against the repo root.
    assert_eq!(regen.regen_calls.borrow().as_slice(), &[root.clone()]);
}

#[test]
fn settle_fails_closed_on_regen_failure_and_leaves_committed_faces_untouched() {
    let root = init_repo();
    let regen = FakeRegen {
        fail_regen: true,
        drift_face: None,
        regen_calls: RefCell::new(Vec::new()),
    };

    let err = run_post_merge_settle(&root, &regen).expect_err("regen failure must fail closed");
    assert_eq!(err.kind(), FacesMergeErrorKind::Regen);

    // The committed faces are byte-untouched (no partial/guessed face written) and nothing is staged.
    for path in generated_face_paths() {
        assert_eq!(
            std::fs::read_to_string(root.join(&path)).expect("read face"),
            "stale conflicted face\n",
            "face {path} must be untouched on a fail-closed regen"
        );
    }
    assert!(
        git_output(&root, &["diff", "--cached", "--name-only"]).trim().is_empty(),
        "nothing staged on a fail-closed regen"
    );
}

#[test]
fn settle_fails_closed_on_determinism_drift_and_does_not_stage() {
    let root = init_repo();
    let regen = FakeRegen {
        fail_regen: false,
        drift_face: Some("accounting-registry.generated.json".to_owned()),
        regen_calls: RefCell::new(Vec::new()),
    };

    let err = run_post_merge_settle(&root, &regen).expect_err("drift must fail closed");
    assert_eq!(err.kind(), FacesMergeErrorKind::Drift);

    // The determinism check runs BEFORE the settle engine stages anything, so no face is staged even
    // though `regenerate` wrote the working-tree bytes (the commit/stage never happened).
    assert!(
        git_output(&root, &["diff", "--cached", "--name-only"]).trim().is_empty(),
        "drift must not stage a face"
    );
}

#[test]
fn settle_fails_closed_on_dirty_non_face_tree() {
    let root = init_repo();
    // An uncommitted non-face change: the post-merge-commit invariant must refuse the settle.
    std::fs::write(root.join("README.md"), "uncommitted content v2\n").expect("dirty tracked file");
    let regen = FakeRegen::ok();

    let err = run_post_merge_settle(&root, &regen).expect_err("dirty tree must fail closed");
    assert_eq!(err.kind(), FacesMergeErrorKind::Settle);
    // The regen never ran (the clean-tree check fails first).
    assert!(regen.regen_calls.borrow().is_empty(), "regen must not run on a dirty tree");
}

#[test]
fn settle_fails_closed_when_control_plane_missing() {
    let root = fixture_root();
    git(&root, &["init"]);
    git(&root, &["config", "user.name", "Oyatie Test"]);
    git(&root, &["config", "user.email", "oyatie-test@example.com"]);
    // No control-plane manifest -> load fails closed BEFORE touching the tree.
    let regen = FakeRegen::ok();
    let err = run_post_merge_settle(&root, &regen).expect_err("missing control plane fails closed");
    assert_eq!(err.kind(), FacesMergeErrorKind::ControlPlane);
}

#[test]
fn buck2_regen_adapter_is_the_production_port() {
    // Compile-time proof that the production adapter implements the port (no buck2 invoked).
    let _adapter: &dyn RegenPort = &Buck2RegenAdapter;
}

#[test]
fn control_plane_loads_the_committed_manifest() {
    let root = init_repo();
    let cp = ControlPlane::load(&root).expect("load control plane");
    assert!(cp.is_regeneratable_face(
        "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/accounting-registry.generated.json"
    ));
}
