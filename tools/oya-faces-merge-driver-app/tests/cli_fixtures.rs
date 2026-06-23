//! Process-level fixtures for the `oya-faces-merge-driver` binary.
//!
//!   - `driver %O %A %B %P` on a DECLARED regeneratable face: cosmetic resolve writes theirs over %A
//!     and exits 0 (the post-merge settle is authoritative — this value is provisional).
//!   - `driver` on a NON-declared face: declines (exit 1) and leaves %A byte-untouched, so git keeps
//!     the conflict (the driver must never resolve a non-face surface).
//!   - wrong argument count / unknown subcommand: exit 2.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn driver_bin() -> PathBuf {
    // buck2 injects `$(location :…)` as a path RELATIVE to the repo root (the test's launch cwd).
    // The tests `current_dir(temp)` into a fixture repo, so resolve to absolute against the launch
    // cwd FIRST (before any chdir) — otherwise the relative path is not found from the temp dir.
    let raw = if let Ok(path) = std::env::var("OYA_FACES_MERGE_DRIVER") {
        PathBuf::from(path)
    } else {
        match option_env!("CARGO_BIN_EXE_oya-faces-merge-driver") {
            Some(path) => PathBuf::from(path),
            None => panic!("missing OYA_FACES_MERGE_DRIVER"),
        }
    };
    if raw.is_absolute() {
        raw
    } else {
        std::env::current_dir().expect("cwd").join(raw)
    }
}

fn unique_repo(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-faces-driver-cli-{label}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create repo");
    root
}

const FACE_PATH: &str =
    "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/accounting-registry.generated.json";

/// Write the root-hub marker + a control-plane manifest declaring FACE_PATH as a SETTLE-CAPABLE face
/// (its generator target is the accounting producer), so the binary's root discovery + control-plane
/// load succeed and the per-file driver accepts FACE_PATH when cwd is `root`.
fn seed_repo(root: &Path) {
    std::fs::create_dir_all(root.join("specs")).expect("specs");
    std::fs::write(root.join("specs/root-hub-pointers.json"), "{}\n").expect("marker");
    std::fs::create_dir_all(root.join("registry")).expect("registry");
    std::fs::write(
        root.join("registry/generated-artifact-control-plane.json"),
        format!(
            "{{\"artifacts\": [{{\"artifact_id\": \"f\", \"path\": \"{FACE_PATH}\", \
             \"merge_policy\": \"never-manual-merge-regenerate-from-source-tree\", \"generator\": \
             {{\"generator_target\": \"{}\"}}}}]}}\n",
            oya_faces_merge_driver_app::PRODUCER_TARGET
        ),
    )
    .expect("write control plane");
}

fn run_driver(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(driver_bin())
        .arg("driver")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run driver")
}

#[test]
fn cosmetic_resolve_writes_theirs_over_ours_and_exits_0() {
    let root = unique_repo("resolve");
    seed_repo(&root);
    let ancestor = root.join("base.tmp");
    let ours = root.join("ours.tmp");
    let theirs = root.join("theirs.tmp");
    std::fs::write(&ancestor, "base\n").unwrap();
    std::fs::write(&ours, "ours bytes\n").unwrap();
    std::fs::write(&theirs, "theirs bytes\n").unwrap();

    let output = run_driver(
        &root,
        &[
            ancestor.to_str().unwrap(),
            ours.to_str().unwrap(),
            theirs.to_str().unwrap(),
            FACE_PATH,
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    // %A now holds theirs (cosmetic) — the settle will overwrite it authoritatively.
    assert_eq!(std::fs::read_to_string(&ours).unwrap(), "theirs bytes\n");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn non_declared_face_declines_exit_1_and_leaves_ours_untouched() {
    let root = unique_repo("decline");
    seed_repo(&root);
    let ancestor = root.join("base.tmp");
    let ours = root.join("ours.tmp");
    let theirs = root.join("theirs.tmp");
    std::fs::write(&ancestor, "base\n").unwrap();
    std::fs::write(&ours, "ours bytes\n").unwrap();
    std::fs::write(&theirs, "theirs bytes\n").unwrap();

    let output = run_driver(
        &root,
        &[
            ancestor.to_str().unwrap(),
            ours.to_str().unwrap(),
            theirs.to_str().unwrap(),
            "some/other/file.txt",
        ],
    );
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    // %A is byte-untouched: git falls back to a normal conflict.
    assert_eq!(std::fs::read_to_string(&ours).unwrap(), "ours bytes\n");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn wrong_argument_count_exits_2() {
    let root = unique_repo("badargs");
    seed_repo(&root);
    let output = run_driver(&root, &["only-one-arg"]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn unknown_subcommand_exits_2() {
    let output = Command::new(driver_bin())
        .arg("frobnicate")
        .output()
        .expect("run driver");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
}

#[test]
fn no_subcommand_exits_2() {
    let output = Command::new(driver_bin()).output().expect("run driver");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
}
