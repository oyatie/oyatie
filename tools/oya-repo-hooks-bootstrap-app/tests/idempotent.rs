//! Bootstrap idempotency fixtures (task #125): run `bootstrap` twice on a temp git repo and assert
//! the merge driver + hooks + .gitattributes glob are activated, and the SECOND run is a no-op.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use oya_repo_hooks_bootstrap_app::{HOOKS_DIR, bootstrap};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture_repo() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-bootstrap-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create repo");
    git(&root, &["init"]);
    git(&root, &["config", "user.name", "Oyatie Test"]);
    git(&root, &["config", "user.email", "oyatie-test@example.com"]);
    // Control-plane manifest declaring two SETTLE-CAPABLE faces (producer + emitter targets — the
    // universality surface the glob block is derived from).
    std::fs::create_dir_all(root.join("registry")).expect("registry");
    std::fs::write(
        root.join("registry/generated-artifact-control-plane.json"),
        format!(
            "{{\"artifacts\": [\
             {{\"artifact_id\": \"a\", \"path\": \"out/a.generated.json\", \"merge_policy\": \
             \"never-manual-merge-regenerate-from-source-tree\", \"generator\": {{\"generator_target\": \"{producer}\"}}}},\
             {{\"artifact_id\": \"b\", \"path\": \"out/b.generated.json\", \"merge_policy\": \
             \"never-manual-merge-regenerate-from-source-tree\", \"generator\": {{\"generator_target\": \"{emitter}\"}}}}\
             ]}}\n",
            producer = oya_faces_merge_driver_app::PRODUCER_TARGET,
            emitter = oya_faces_merge_driver_app::EMITTER_TARGET,
        ),
    )
    .expect("write control plane");
    // A pre-existing hand-authored .gitattributes line, to prove the block is appended without
    // clobbering existing entries.
    std::fs::write(root.join(".gitattributes"), "Cargo.lock merge=cargo-lock\n").expect("seed gitattributes");
    root
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run git");
    assert!(output.status.success(), "git {args:?} failed");
}

fn git_config(root: &Path, key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--local", "--get", key])
        .current_dir(root)
        .output()
        .expect("run git config");
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim_end_matches('\n').to_owned())
    } else {
        None
    }
}

#[test]
fn bootstrap_is_idempotent_and_activates_driver_hooks_and_gitattributes() {
    let root = fixture_repo();
    let driver = Path::new("/opt/oya/bin/oya-faces-merge-driver");

    // First run: everything is configured.
    let first = bootstrap(&root, driver).expect("first bootstrap");
    assert!(first.merge_driver_configured, "merge driver configured on first run");
    assert!(first.hooks_path_configured, "hooksPath configured on first run");
    assert_eq!(
        first.hooks_installed,
        vec!["post-checkout".to_owned(), "post-merge".to_owned(), "post-rewrite".to_owned()],
        "all three hooks installed on first run"
    );
    assert!(first.gitattributes_updated, "gitattributes block written on first run");
    assert!(!first.is_noop());

    // The git config matches the control-plane-derived values.
    assert_eq!(
        git_config(&root, "merge.oya-faces.driver").as_deref(),
        Some("/opt/oya/bin/oya-faces-merge-driver driver %O %A %B %P")
    );
    assert_eq!(git_config(&root, "core.hooksPath").as_deref(), Some(HOOKS_DIR));

    // The hooks exist + are executable; the settle hook calls the driver settle subcommand.
    let post_merge = root.join(HOOKS_DIR).join("post-merge");
    let body = std::fs::read_to_string(&post_merge).expect("read post-merge");
    assert!(body.contains("oya-faces-merge-driver settle"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&post_merge).unwrap().permissions().mode();
        assert!(mode & 0o111 != 0, "post-merge must be executable");
    }

    // The .gitattributes block is present AND the prior hand-authored line is preserved.
    let attrs = std::fs::read_to_string(root.join(".gitattributes")).expect("read gitattributes");
    assert!(attrs.contains("Cargo.lock merge=cargo-lock"), "prior line preserved");
    assert!(attrs.contains("out/a.generated.json merge=oya-faces"));
    assert!(attrs.contains("out/b.generated.json merge=oya-faces"));

    // Second run: a complete no-op (idempotent).
    let second = bootstrap(&root, driver).expect("second bootstrap");
    assert!(second.is_noop(), "second bootstrap must be a no-op: {second:?}");

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn bootstrap_fails_closed_without_control_plane() {
    let root = std::env::temp_dir().join(format!(
        "oya-bootstrap-noplan-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create repo");
    git(&root, &["init"]);
    let err = bootstrap(&root, Path::new("/opt/oya/bin/oya-faces-merge-driver"))
        .expect_err("missing control plane must fail closed");
    assert!(err.to_string().contains("control-plane"), "{err}");
    let _ = std::fs::remove_dir_all(root);
}
