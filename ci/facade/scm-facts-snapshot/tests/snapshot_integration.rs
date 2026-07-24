//! Filesystem and Git-bound integration checks for the SCM facts emitter.
//!
//! These tests deliberately stay out of the small unit target: each crosses
//! a real filesystem or Git boundary that Buck2 must schedule independently.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use ci_path_resolver_adapters::MOVE_MANIFEST_PATH;
use ci_path_resolver_adapters::MOVE_MANIFEST_SCHEMA;
use ci_path_resolver_ports::{PathId, PathResolver};
use ci_scm_facts_snapshot::{
    discover_repo_root, emit_fixed_adr_census_parent_receipt, load_vocab_policy,
    output_path_resolver,
    retirement::{
        GENERATED_FACTS_PATH, RetirementMaterializationContext, emit_history_only_retirement_facts,
        write_canonical_retirement_facts,
    },
};
use serde_json::json;

static NEXT_TEMP_REPO_ID: AtomicU64 = AtomicU64::new(0);

const PROTECTED: &str = "3333333333333333333333333333333333333333";
const CANDIDATE: &str = "5555555555555555555555555555555555555555";

fn context() -> RetirementMaterializationContext<'static> {
    RetirementMaterializationContext {
        control_plane_path: "registry/history-only-retirement/control-plane.json",
        protected_base_commit: PROTECTED,
        evaluated_commit: CANDIDATE,
        scm_event_name: "push",
        scm_event_ref: "refs/heads/dev",
        subject_commit: CANDIDATE,
    }
}

fn temp_git_repo(label: &str) -> PathBuf {
    let id = NEXT_TEMP_REPO_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "oya-scm-facts-integration-{label}-{}-{id}",
        std::process::id()
    ));
    std::fs::create_dir(&root).expect("create isolated SCM facts integration repository");
    let status = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&root)
        .status()
        .expect("run git init");
    assert!(status.success(), "git init must succeed");
    root
}

fn temp_repo_root(test_name: &str) -> PathBuf {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "oya-scm-facts-integration-{test_name}-{}-{suffix}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).expect("create integration fixture root");
    root
}

fn configure_ignored_canonical_facts(root: &Path) {
    std::fs::write(
        root.join(".gitignore"),
        format!("/{GENERATED_FACTS_PATH}\n"),
    )
    .expect("ignore canonical retirement facts output");
}

#[test]
fn emitter_rejects_canonical_generated_facts_path_when_tracked() {
    let root = temp_git_repo("tracked-output");
    let output_path = root.join(GENERATED_FACTS_PATH);
    std::fs::create_dir_all(output_path.parent().expect("output parent")).expect("mkdir");
    std::fs::write(
        root.join(".gitignore"),
        format!("/{GENERATED_FACTS_PATH}\n"),
    )
    .expect("write gitignore");
    std::fs::write(&output_path, b"{}\n").expect("write generated facts fixture");
    let status = Command::new("git")
        .args(["add", "-f", "--", GENERATED_FACTS_PATH])
        .current_dir(&root)
        .status()
        .expect("force-add generated facts fixture");
    assert!(
        status.success(),
        "force-add canonical generated facts fixture"
    );

    let error =
        emit_history_only_retirement_facts(&root, &context(), Path::new(GENERATED_FACTS_PATH))
            .expect_err("tracked generated facts must fail closed");
    assert!(
        error.contains("must be ignored and untracked"),
        "unexpected error: {error}"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[test]
fn emitter_rejects_lexical_retirement_facts_path_escapes() {
    let root = temp_git_repo("lexical-output-escape");
    let error = emit_history_only_retirement_facts(&root, &context(), Path::new("../outside.json"))
        .expect_err("retirement facts must accept only their canonical repo-relative path");
    assert!(
        error.contains("exact canonical repo-relative"),
        "unexpected error: {error}"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[cfg(unix)]
#[test]
fn output_symlinks_are_rejected_without_touching_targets() {
    use std::os::unix::fs::symlink;

    let root = temp_git_repo("output-symlink");
    configure_ignored_canonical_facts(&root);
    let output = root.join(GENERATED_FACTS_PATH);
    std::fs::create_dir_all(output.parent().expect("output parent")).expect("mkdir");
    let outside = root.join("outside.json");
    std::fs::write(&outside, b"outside bytes").expect("write outside target");
    symlink(&outside, &output).expect("link output");

    let error = write_canonical_retirement_facts(&root, b"replacement")
        .expect_err("output symlink must fail closed");
    assert!(error.contains("must be a regular file"));
    assert_eq!(
        std::fs::read(&outside).expect("read outside target"),
        b"outside bytes"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[cfg(unix)]
#[test]
fn intermediate_output_symlinks_are_rejected_without_touching_targets() {
    use std::os::unix::fs::symlink;

    let root = temp_git_repo("intermediate-output-symlink");
    configure_ignored_canonical_facts(&root);
    let outside = root.join("outside");
    std::fs::create_dir(&outside).expect("create outside directory");
    let target =
        outside.join("facade/scm-facts-snapshot/history-only-retirement-facts.generated.json");
    std::fs::create_dir_all(target.parent().expect("outside target parent"))
        .expect("create outside target parent");
    std::fs::write(&target, b"outside bytes").expect("write outside target");
    symlink(&outside, root.join("ci")).expect("link intermediate directory");

    let error = write_canonical_retirement_facts(&root, b"replacement")
        .expect_err("intermediate symlink must fail closed");
    assert!(error.contains("not a real directory"));
    assert_eq!(
        std::fs::read(&target).expect("read outside target"),
        b"outside bytes"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[cfg(unix)]
#[test]
fn canonical_writer_rejects_parent_directory_swapped_to_symlink() {
    use std::os::unix::fs::symlink;

    let root = temp_git_repo("canonical-output-parent-swap");
    configure_ignored_canonical_facts(&root);
    let original_ci = root.join("ci");
    std::fs::create_dir_all(&original_ci).expect("create original canonical parent");
    std::fs::rename(&original_ci, root.join("ci-before-swap")).expect("move original parent");

    let outside = root.join("outside");
    let target =
        outside.join("facade/scm-facts-snapshot/history-only-retirement-facts.generated.json");
    std::fs::create_dir_all(target.parent().expect("outside target parent"))
        .expect("create outside target parent");
    std::fs::write(&target, b"outside bytes").expect("write outside target");
    symlink(&outside, &original_ci).expect("swap canonical parent to symlink");

    let error = write_canonical_retirement_facts(&root, b"replacement")
        .expect_err("swapped canonical parent must fail closed");
    assert!(
        error.contains("not a real directory"),
        "unexpected error: {error}"
    );
    assert_eq!(
        std::fs::read(&target).expect("read outside target"),
        b"outside bytes"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[test]
fn filesystem_backed_resolvers_and_receipt_output_remain_deterministic() {
    let absent_root = temp_repo_root("missing-resolver-manifest");
    let absent_error = output_path_resolver(&absent_root, true)
        .expect_err("baseline resolver must reject an absent move manifest");
    assert!(
        absent_error.contains("move-manifest absent/unreadable"),
        "unexpected missing-manifest error: {absent_error}"
    );
    std::fs::remove_dir_all(absent_root).expect("remove absent-manifest fixture");

    let root = temp_repo_root("resolver");
    std::fs::write(
        root.join("oya-ci.toml"),
        "\n[[vocab.carve_outs]]\nkind = \"line_contains_ci\"\nvalue = \"structural-marker\"\nexempt_stems = [\"alpha\"]\n",
    )
    .expect("write config");
    let policy = load_vocab_policy(&root).expect("load vocab policy");
    assert!(
        policy
            .carve_outs
            .iter()
            .any(|rule| rule.value == "structural-marker")
    );

    let manifest_path = root.join(MOVE_MANIFEST_PATH);
    std::fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("mkdir");
    std::fs::write(
        &manifest_path,
        serde_json::to_vec(&json!({
            "schema": MOVE_MANIFEST_SCHEMA,
            "files": [{"old_path": "ci/facade/artifact-inventory-registry/scm-facts.generated.json", "new_path": "relocated/scm-facts.generated.json"}],
            "crate_dirs": [], "crate_idents": []
        }))
        .expect("serialize manifest"),
    )
    .expect("write manifest");
    assert_eq!(
        output_path_resolver(&root, false)
            .expect("candidate resolver")
            .candidate(PathId::ScmFactsFace),
        ci_path_resolver_ports::canonical_current(PathId::ScmFactsFace),
    );
    assert_eq!(
        output_path_resolver(&root, true)
            .expect("baseline resolver")
            .candidate(PathId::ScmFactsFace),
        "relocated/scm-facts.generated.json",
    );

    let repo_root = discover_repo_root().expect("discover repository root");
    let first = ci_scm_facts_snapshot::build_fixed_adr_census_parent_receipt(&repo_root)
        .expect("build fixed receipt");
    let output = root.join("nested/receipt.generated.json");
    emit_fixed_adr_census_parent_receipt(&repo_root, &output).expect("emit fixed receipt");
    assert_eq!(std::fs::read(&output).expect("read emitted receipt"), first);
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}
