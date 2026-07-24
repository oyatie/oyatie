//! Filesystem and Git-bound integration checks for the SCM facts emitter.
//!
//! These tests deliberately stay out of the small unit target: each crosses
//! a real filesystem or Git boundary that Buck2 must schedule independently.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use ci_path_resolver_adapters::MOVE_MANIFEST_PATH;
use ci_path_resolver_adapters::MOVE_MANIFEST_SCHEMA;
use ci_path_resolver_ports::{PathId, PathResolver};
use ci_scm_facts_snapshot::retirement::CanonicalRetirementFactsWriter;
use ci_scm_facts_snapshot::{
    discover_repo_root, emit_fixed_adr_census_parent_receipt, load_vocab_policy,
    output_path_resolver,
    retirement::{
        GENERATED_FACTS_PATH, RetirementMaterializationContext, emit_history_only_retirement_facts,
        historical_dev_push_context, visit_git_blobs, write_canonical_retirement_facts,
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
        scm_event_base_ref: "refs/heads/dev",
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
    git_success(&root, ["config", "user.email", "scm-facts@example.test"]);
    git_success(&root, ["config", "user.name", "SCM Facts Integration"]);
    root
}

fn git_success<const N: usize>(root: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run Git fixture command");
    assert!(
        output.status.success(),
        "Git fixture command failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
}

fn git_stdout<const N: usize>(root: &Path, args: [&str; N]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run Git fixture command");
    assert!(
        output.status.success(),
        "Git fixture command failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    String::from_utf8(output.stdout)
        .expect("Git fixture output must be UTF-8")
        .trim()
        .to_owned()
}

fn commit_all(root: &Path, message: &str) -> String {
    git_success(root, ["add", "--all"]);
    git_success(root, ["commit", "--quiet", "-m", message]);
    git_stdout(root, ["rev-parse", "HEAD"])
}

fn write_control_plane(root: &Path) {
    let control_plane = root.join("registry/history-only-retirement/control-plane.json");
    std::fs::create_dir_all(control_plane.parent().expect("control-plane parent"))
        .expect("create control-plane parent");
    std::fs::write(control_plane, b"{}\n").expect("write control-plane fixture");
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

fn assert_git_blob_batch_recovers(root: &Path, blob_oid: &str, expected: &[u8]) {
    let mut visited = Vec::new();
    visit_git_blobs(root, &[blob_oid.to_owned()], &mut |oid, size, reader| {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .map_err(|error| format!("read recovery blob: {error}"))?;
        visited.push((oid.to_owned(), size, bytes));
        Ok(())
    })
    .expect("a valid batch call must succeed after an error");
    assert_eq!(
        visited,
        vec![(
            blob_oid.to_owned(),
            expected.len() as u64,
            expected.to_vec()
        )]
    );
}

#[test]
fn git_blob_batch_streams_requested_objects_with_exact_bytes() {
    let root = temp_git_repo("blob-batch");
    std::fs::write(root.join("first"), vec![b'x'; 128 * 1024]).expect("write large first blob");
    std::fs::write(root.join("second"), b"second body\0").expect("write second blob");
    commit_all(&root, "blob batch fixture");
    let first_oid = git_stdout(&root, ["rev-parse", "HEAD:first"]);
    let second_oid = git_stdout(&root, ["rev-parse", "HEAD:second"]);

    let mut visited = Vec::new();
    visit_git_blobs(
        &root,
        &[first_oid.clone(), second_oid.clone()],
        &mut |oid, size, reader| {
            if oid == first_oid {
                visited.push((oid.to_owned(), size, Vec::new()));
                return Ok(());
            }
            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .map_err(|error| format!("read streamed blob: {error}"))?;
            visited.push((oid.to_owned(), size, bytes));
            Ok(())
        },
    )
    .expect("stream exact blobs through the production Git batch boundary");

    assert_eq!(
        visited,
        vec![
            (first_oid, (128 * 1024) as u64, Vec::new()),
            (
                second_oid,
                b"second body\0".len() as u64,
                b"second body\0".to_vec()
            ),
        ]
    );
    std::fs::remove_dir_all(root).expect("remove blob batch fixture");
}

#[test]
fn git_blob_batch_fails_closed_and_recovers_after_each_error() {
    let root = temp_git_repo("blob-batch-errors");
    std::fs::write(root.join("blob"), b"exact body").expect("write blob");
    std::fs::create_dir(root.join("tree")).expect("create tree");
    std::fs::write(root.join("tree/child"), b"child").expect("write tree child");
    commit_all(&root, "blob batch errors fixture");
    let blob_oid = git_stdout(&root, ["rev-parse", "HEAD:blob"]);
    let tree_oid = git_stdout(&root, ["rev-parse", "HEAD:tree"]);

    let mut calls = 0;
    visit_git_blobs(&root, &[], &mut |_, _, _| {
        calls += 1;
        Ok(())
    })
    .expect("empty batch is a no-op");
    assert_eq!(calls, 0);

    let invalid = visit_git_blobs(&root, &["not-an-oid".to_owned()], &mut |_, _, _| Ok(()))
        .expect_err("invalid OID must fail before transport");
    assert!(
        invalid.contains("lowercase SHA-1"),
        "unexpected error: {invalid}"
    );
    assert_git_blob_batch_recovers(&root, &blob_oid, b"exact body");

    let missing = visit_git_blobs(
        &root,
        &["0000000000000000000000000000000000000000".to_owned()],
        &mut |_, _, _| Ok(()),
    )
    .expect_err("missing object header must fail");
    assert!(
        missing.contains("unexpected header"),
        "unexpected error: {missing}"
    );
    assert_git_blob_batch_recovers(&root, &blob_oid, b"exact body");

    let non_blob = visit_git_blobs(&root, &[tree_oid], &mut |_, _, _| Ok(()))
        .expect_err("tree object header must fail");
    assert!(
        non_blob.contains("unexpected header"),
        "unexpected error: {non_blob}"
    );
    assert_git_blob_batch_recovers(&root, &blob_oid, b"exact body");

    let visitor = visit_git_blobs(&root, &[blob_oid.clone()], &mut |_, _, _| {
        Err("visitor rejected body".to_owned())
    })
    .expect_err("visitor error must propagate");
    assert!(
        visitor.contains("visitor rejected body"),
        "unexpected error: {visitor}"
    );
    assert_git_blob_batch_recovers(&root, &blob_oid, b"exact body");

    std::fs::remove_dir_all(root).expect("remove blob batch error fixture");
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
    assert!(
        error.contains("not a real directory"),
        "unexpected intermediate-symlink error: {error}"
    );
    assert_eq!(
        std::fs::read(&target).expect("read outside target"),
        b"outside bytes"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[cfg(unix)]
#[test]
fn canonical_writer_stays_bound_to_open_parent_after_ancestor_swap() {
    use std::os::unix::fs::symlink;

    let root = temp_git_repo("canonical-output-parent-swap");
    configure_ignored_canonical_facts(&root);
    let original_ci = root.join("ci");
    let captured_output =
        original_ci.join("facade/scm-facts-snapshot/history-only-retirement-facts.generated.json");
    std::fs::create_dir_all(captured_output.parent().expect("captured output parent"))
        .expect("create canonical parent");
    let writer = CanonicalRetirementFactsWriter::open(&root)
        .expect("open canonical writer before the ancestor swap");
    std::fs::rename(&original_ci, root.join("ci-captured")).expect("move opened ancestor");

    let outside = root.join("outside");
    let target =
        outside.join("facade/scm-facts-snapshot/history-only-retirement-facts.generated.json");
    std::fs::create_dir_all(target.parent().expect("outside target parent"))
        .expect("create outside target parent");
    std::fs::write(&target, b"outside bytes").expect("write outside target");
    symlink(&outside, &original_ci).expect("swap canonical parent to symlink");

    writer
        .write(b"captured bytes")
        .expect("writer must finalize through its captured directory fd");
    assert_eq!(
        std::fs::read(&target).expect("read outside target"),
        b"outside bytes"
    );
    assert_eq!(
        std::fs::read(root.join(
            "ci-captured/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json"
        ))
        .expect("read captured output"),
        b"captured bytes"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[test]
fn historical_dev_push_context_accepts_exact_head_with_control_plane_and_one_parent() {
    let root = temp_git_repo("historical-dev-push-exact-head");
    std::fs::write(root.join("base.txt"), b"base\n").expect("write base fixture");
    let parent = commit_all(&root, "base");
    write_control_plane(&root);
    let head = commit_all(&root, "add control plane");

    assert_eq!(
        historical_dev_push_context(&root, &head).expect("accept exact one-parent head"),
        Some((head, parent))
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[test]
fn historical_dev_push_context_rejects_non_exact_head_alias() {
    let root = temp_git_repo("historical-dev-push-alias");
    std::fs::write(root.join("base.txt"), b"base\n").expect("write base fixture");
    commit_all(&root, "base");
    write_control_plane(&root);
    commit_all(&root, "add control plane");

    let error = historical_dev_push_context(&root, "HEAD")
        .expect_err("symbolic head alias must not satisfy immutable expected-head input");
    assert!(
        error.contains("does not resolve exactly"),
        "unexpected error: {error}"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[test]
fn historical_dev_push_context_rejects_root_commit_with_control_plane() {
    let root = temp_git_repo("historical-dev-push-root");
    write_control_plane(&root);
    let head = commit_all(&root, "root control plane");

    let error = historical_dev_push_context(&root, &head)
        .expect_err("control-plane root commit must not have an implicit protected parent");
    assert!(
        error.contains("exactly one parent"),
        "unexpected error: {error}"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[test]
fn historical_dev_push_context_rejects_merge_commit_with_control_plane() {
    let root = temp_git_repo("historical-dev-push-merge");
    std::fs::write(root.join("base.txt"), b"base\n").expect("write base fixture");
    commit_all(&root, "base");
    let primary_branch = git_stdout(&root, ["symbolic-ref", "--short", "HEAD"]);

    git_success(&root, ["checkout", "--quiet", "-b", "side"]);
    std::fs::write(root.join("side.txt"), b"side\n").expect("write side fixture");
    commit_all(&root, "side change");

    git_success(&root, ["checkout", "--quiet", &primary_branch]);
    write_control_plane(&root);
    commit_all(&root, "add control plane");
    git_success(
        &root,
        ["merge", "--quiet", "--no-ff", "side", "-m", "merge side"],
    );
    let merge_head = git_stdout(&root, ["rev-parse", "HEAD"]);

    let error = historical_dev_push_context(&root, &merge_head)
        .expect_err("control-plane merge commit must not choose an ambiguous protected parent");
    assert!(
        error.contains("exactly one parent"),
        "unexpected error: {error}"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[test]
fn historical_dev_push_context_returns_bootstrap_when_control_plane_is_absent() {
    let root = temp_git_repo("historical-dev-push-bootstrap");
    std::fs::write(root.join("base.txt"), b"base\n").expect("write base fixture");
    commit_all(&root, "base");
    std::fs::write(root.join("candidate.txt"), b"candidate\n").expect("write candidate fixture");
    let head = commit_all(&root, "candidate without control plane");

    assert_eq!(
        historical_dev_push_context(&root, &head).expect("bootstrap remains permitted"),
        None
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[test]
fn baseline_output_path_resolver_rejects_missing_move_manifest() {
    let absent_root = temp_repo_root("missing-resolver-manifest");
    let absent_error = output_path_resolver(&absent_root, true)
        .err()
        .expect("baseline resolver must reject an absent move manifest");
    assert!(
        absent_error.contains("move-manifest absent/unreadable"),
        "unexpected missing-manifest error: {absent_error}"
    );
    std::fs::remove_dir_all(absent_root).expect("remove absent-manifest fixture");
}

#[test]
fn vocab_policy_loads_filesystem_carve_out() {
    let root = temp_repo_root("vocab-policy");
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
    std::fs::remove_dir_all(root).expect("remove vocab-policy fixture");
}

#[test]
fn candidate_output_path_resolver_uses_current_canonical_path_without_manifest() {
    let root = temp_repo_root("candidate-resolver");
    assert_eq!(
        output_path_resolver(&root, false)
            .expect("candidate resolver")
            .candidate(PathId::ScmFactsFace),
        ci_path_resolver_ports::canonical_current(PathId::ScmFactsFace),
    );
    std::fs::remove_dir_all(root).expect("remove candidate-resolver fixture");
}

#[test]
fn baseline_output_path_resolver_uses_materialized_move_manifest() {
    let root = temp_repo_root("baseline-resolver");
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
        output_path_resolver(&root, true)
            .expect("baseline resolver")
            .candidate(PathId::ScmFactsFace),
        "relocated/scm-facts.generated.json",
    );
    std::fs::remove_dir_all(root).expect("remove baseline-resolver fixture");
}

#[test]
fn repository_discovery_finds_root_authority_pointer() {
    let root = discover_repo_root().expect("discover repository root");
    assert!(
        root.join("specs/root-hub-pointers.json").is_file(),
        "discovered repository root must contain the authority pointer"
    );
}

#[test]
fn fixed_adr_census_parent_receipt_emission_matches_builder_bytes() {
    let output_root = temp_repo_root("fixed-adr-census-receipt");
    let repo_root = discover_repo_root().expect("discover repository root");
    let first = ci_scm_facts_snapshot::build_fixed_adr_census_parent_receipt(&repo_root)
        .expect("build fixed receipt");
    let output = output_root.join("nested/receipt.generated.json");
    emit_fixed_adr_census_parent_receipt(&repo_root, &output).expect("emit fixed receipt");
    assert_eq!(std::fs::read(&output).expect("read emitted receipt"), first);
    std::fs::remove_dir_all(output_root).expect("remove receipt-output fixture");
}
