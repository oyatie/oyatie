//! Filesystem and Git-bound integration checks for the SCM facts emitter.
//!
//! These tests deliberately stay out of the small unit target: each crosses
//! a real filesystem or Git boundary that Buck2 must schedule independently.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use ci_path_resolver_adapters::MOVE_MANIFEST_PATH;
use ci_path_resolver_adapters::MOVE_MANIFEST_SCHEMA;
use ci_path_resolver_ports::{PathId, PathResolver};
use ci_scm_facts_snapshot::retirement::{
    CanonicalIgnoredGeneratedWriter, CanonicalRetirementFactsWriter,
    write_canonical_ignored_generated_file,
};
use ci_scm_facts_snapshot::{
    ADR_CENSUS_PARENT_RECEIPT_PATH, P2ParentReceipt, command_status_with_captured_stderr,
    command_status_with_timeout, discover_repo_root, dormant_p3_epoch_fingerprint,
    emit_adr_census_epoch_receipt, emit_adr_census_epoch_receipt_for_event, load_vocab_policy,
    output_path_resolver,
    retirement::{
        GENERATED_FACTS_PATH, RetirementMaterializationContext, emit_history_only_retirement_facts,
        historical_dev_push_context, visit_git_blobs, write_canonical_retirement_facts,
    },
    select_census_event_from_event, validate_adr_census_epoch_receipt_for_event,
    validate_census_event_transition,
};
use serde_json::json;
use sha2::Digest;

static NEXT_TEMP_REPO_ID: AtomicU64 = AtomicU64::new(0);

const PROTECTED: &str = "3333333333333333333333333333333333333333";
const CANDIDATE: &str = "5555555555555555555555555555555555555555";
const EPOCH_RECEIPT_PATH: &str =
    "ci/facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json";
const FIXED_P2_EPOCH_RECEIPT_SHA256: &str =
    "0f22621954fe0f7718a79616769bfe1ed4660851bab8890d69e98038080e2b0a";

#[test]
fn status_only_child_timeout_preserves_label_and_terminates_child() {
    let executable = std::env::current_exe().expect("resolve current integration-test executable");
    let root = temp_path("timeout-child");
    std::fs::create_dir(&root).expect("create timeout-child fixture root");
    let mut command = Command::new(executable);
    command
        .args(["--exact", "status_only_command_child_helper", "--nocapture"])
        .env("OYA_CI_COMMAND_CHILD_MODE", "timeout")
        .env("OYA_CI_COMMAND_CHILD_ROOT", &root);

    let release = root.join("release");
    let release_after_timeout = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(4));
        std::fs::write(release, b"release").expect("release timeout child after parent deadline");
    });
    let error = command_status_with_timeout(command, Duration::from_secs(3), "test bounded child")
        .expect_err("sleeping child must time out");
    assert!(error.contains("test bounded child timed out"), "{error}");
    release_after_timeout
        .join()
        .expect("timeout release helper must complete");
    assert!(
        root.join("started").is_file(),
        "timeout child must start before termination"
    );
    assert!(
        !root.join("survived").exists(),
        "timeout helper must not return while its direct child can survive"
    );
    std::fs::remove_dir_all(root).expect("remove timeout-child fixture root");
}

#[test]
fn status_only_child_discards_large_streams_and_preserves_nonzero_status() {
    let executable = std::env::current_exe().expect("resolve current integration-test executable");
    let mut command = Command::new(executable);
    command
        .args(["--exact", "status_only_command_child_helper", "--nocapture"])
        .env("OYA_CI_COMMAND_CHILD_MODE", "large-output");

    let status =
        command_status_with_timeout(command, Duration::from_secs(5), "test large-output child")
            .expect("large-output child must complete without pipe backpressure");
    assert_eq!(status.code(), Some(7));
}

#[test]
fn status_only_sustained_output_uses_no_parent_owned_storage() {
    let executable = std::env::current_exe().expect("resolve current integration-test executable");
    let root = temp_path("status-only-storage");
    std::fs::create_dir(&root).expect("create status-only storage fixture root");
    let mut command = Command::new(executable);
    command
        .args(["--exact", "status_only_command_probe_helper", "--nocapture"])
        .env("OYA_CI_COMMAND_PROBE_MODE", "sustained-output")
        .env("TMPDIR", &root)
        .env("TMP", &root)
        .env("TEMP", &root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut probe = command.spawn().expect("spawn status-only storage probe");
    let started = Instant::now();
    let mut maximum_storage = 0;
    let status = loop {
        maximum_storage = maximum_storage.max(directory_bytes(&root));
        if let Some(status) = probe.try_wait().expect("poll status-only storage probe") {
            break status;
        }
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "status-only storage probe must not hang on sustained output"
        );
        std::thread::sleep(Duration::from_millis(1));
    };
    maximum_storage = maximum_storage.max(directory_bytes(&root));
    assert!(status.success(), "status-only storage probe must pass");
    // This invariant is scoped to `command_status_with_timeout`, which supplies its own
    // null stderr and therefore allocates nothing. Stated plainly, because the name reads
    // broader than the coverage: that wrapper has NO production callers, so this assertion
    // covers no production path. Every caller that must be able to explain a nested failure
    // goes through `command_status_with_captured_stderr` instead, which deliberately DOES
    // take a temporary file — one child's stderr volume, outside any worktree, unlinked
    // before the call returns. What survives here is narrow and still worth keeping: proof
    // that the shared supervision loop costs nothing when a caller genuinely wants no sink.
    // It is not a repository-wide prohibition on capturing stderr, and it is NOT evidence
    // about the path production actually takes.
    assert_eq!(
        maximum_storage, 0,
        "status-only command supervision must never allocate parent-owned output storage"
    );
    std::fs::remove_dir_all(root).expect("remove status-only storage fixture root");
}

/// A nested child that fails must reach the caller with its own stderr, not just a status.
///
/// This is the regression that cost twice in one day: `dev` red for an hour on
/// `... failed with status exit status: 3` and nothing else. The marker is written ONLY to
/// the child's stderr, so this test fails outright if the sink goes back to `Stdio::null`.
#[test]
fn captured_stderr_reaches_the_caller_when_a_nested_child_fails() {
    let executable = std::env::current_exe().expect("resolve current integration-test executable");
    let mut command = Command::new(executable);
    command
        .args(["--exact", "status_only_command_child_helper", "--nocapture"])
        .env("OYA_CI_COMMAND_CHILD_MODE", "stderr-then-fail");

    let (status, suffix) =
        command_status_with_captured_stderr(command, Duration::from_secs(30), "test failing child")
            .expect("a child that exits non-zero is supervised successfully, not an error");
    assert_eq!(status.code(), Some(11));
    assert!(
        suffix.contains("NESTED-STDERR-MARKER"),
        "failing child's stderr must reach the caller, got: {suffix}"
    );
}

/// The timeout path must carry the capture too.
///
/// A timeout is where partial stderr is most diagnostic — "what was the child doing for ten
/// minutes?" is the whole question — and it is the path an early `?` most easily strands.
#[test]
fn captured_stderr_reaches_the_caller_when_a_nested_child_times_out() {
    let executable = std::env::current_exe().expect("resolve current integration-test executable");
    let root = temp_path("stderr-hang-child");
    std::fs::create_dir(&root).expect("create stderr-hang fixture root");
    let mut command = Command::new(executable);
    command
        .args(["--exact", "status_only_command_child_helper", "--nocapture"])
        .env("OYA_CI_COMMAND_CHILD_MODE", "stderr-then-hang")
        .env("OYA_CI_COMMAND_CHILD_ROOT", &root);

    let error =
        command_status_with_captured_stderr(command, Duration::from_secs(3), "test hanging child")
            .expect_err("a child that outlives its deadline must be an error");
    assert!(error.contains("test hanging child timed out"), "{error}");
    assert!(
        error.contains("NESTED-STDERR-MARKER"),
        "timed-out child's partial stderr must survive the early return, got: {error}"
    );
    // The child announces itself on stderr before hanging, so reaching the assertions above
    // proves the capture was flushed to the sink and read, not merely that the child started.
    assert!(
        root.join("started").is_file(),
        "hanging child must start before termination"
    );
    std::fs::remove_dir_all(root).expect("remove stderr-hang fixture root");
}

#[test]
fn status_only_command_probe_helper() {
    if std::env::var("OYA_CI_COMMAND_PROBE_MODE").as_deref() != Ok("sustained-output") {
        return;
    }
    let executable = std::env::current_exe().expect("resolve current integration-test executable");
    let mut command = Command::new(executable);
    command
        .args(["--exact", "status_only_command_child_helper", "--nocapture"])
        .env("OYA_CI_COMMAND_CHILD_MODE", "sustained-output");
    let status = command_status_with_timeout(
        command,
        Duration::from_secs(5),
        "test sustained-output child",
    )
    .expect("sustained-output child must complete without storage or backpressure");
    assert!(status.success());
}

#[test]
fn status_only_command_child_helper() {
    match std::env::var("OYA_CI_COMMAND_CHILD_MODE").as_deref() {
        Ok("timeout") => {
            let root = PathBuf::from(
                std::env::var_os("OYA_CI_COMMAND_CHILD_ROOT")
                    .expect("timeout child root must be supplied"),
            );
            std::fs::write(root.join("started"), b"started").expect("write started sentinel");
            while !root.join("release").exists() {
                std::thread::sleep(Duration::from_millis(10));
            }
            std::fs::write(root.join("survived"), b"survived").expect("write survivor sentinel");
        }
        Ok("stderr-then-fail") => {
            let mut stderr = std::io::stderr();
            stderr
                .write_all(b"NESTED-STDERR-MARKER: the child explained itself here\n")
                .and_then(|()| stderr.flush())
                .expect("write helper stderr");
            std::process::exit(11);
        }
        Ok("stderr-then-hang") => {
            let root = PathBuf::from(
                std::env::var_os("OYA_CI_COMMAND_CHILD_ROOT")
                    .expect("hanging child root must be supplied"),
            );
            std::fs::write(root.join("started"), b"started").expect("write started sentinel");
            let mut stderr = std::io::stderr();
            stderr
                .write_all(b"NESTED-STDERR-MARKER: still working when the deadline passed\n")
                .and_then(|()| stderr.flush())
                .expect("write helper stderr");
            // Self-bounded: a supervisor that failed to terminate this must not leave a
            // process running for the life of the test runner.
            std::thread::sleep(Duration::from_secs(60));
        }
        Ok("large-output") => {
            let payload = vec![b'x'; 256 * 1024];
            let mut stdout = std::io::stdout();
            stdout
                .write_all(&payload)
                .and_then(|()| stdout.write_all(b"stdout-complete\n"))
                .and_then(|()| stdout.flush())
                .expect("write helper stdout");
            let mut stderr = std::io::stderr();
            stderr
                .write_all(&payload)
                .and_then(|()| stderr.write_all(b"stderr-complete\n"))
                .and_then(|()| stderr.flush())
                .expect("write helper stderr");
            std::process::exit(7);
        }
        Ok("sustained-output") => {
            let payload = vec![b'x'; 64 * 1024];
            let mut stdout = std::io::stdout();
            let mut stderr = std::io::stderr();
            for _ in 0..1_024 {
                stdout
                    .write_all(&payload)
                    .expect("write sustained helper stdout");
                stderr
                    .write_all(&payload)
                    .expect("write sustained helper stderr");
            }
            stdout.flush().expect("flush sustained helper stdout");
            stderr.flush().expect("flush sustained helper stderr");
        }
        _ => {}
    }
}

fn directory_bytes(root: &Path) -> u64 {
    let mut total = 0;
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let entries = std::fs::read_dir(path).expect("read status-only storage fixture directory");
        for entry in entries {
            let entry = entry.expect("read status-only storage fixture entry");
            let metadata = entry
                .metadata()
                .expect("inspect status-only storage fixture entry");
            if metadata.is_dir() {
                pending.push(entry.path());
            } else {
                total += metadata.len();
            }
        }
    }
    total
}

fn temp_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "oya-scm-facts-integration-{label}-{}-{}",
        std::process::id(),
        NEXT_TEMP_REPO_ID.fetch_add(1, Ordering::Relaxed)
    ))
}

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
    configure_fixture_repo(&root);
    root
}

/// Give a fixture repository a committer identity and no background Git maintenance.
///
/// With default config, `git commit` spawns `git maintenance run --auto --quiet --detach`, which
/// daemonizes and keeps writing under `<root>/.git/objects` (its `maintenance.lock`, then any
/// repack output) after `git commit` has already returned. Teardown's single-pass
/// `remove_dir_all` reads each directory once and never retries, so an entry the daemon creates
/// between that read and the `rmdir` surfaces as a spurious `DirectoryNotEmpty`. Disabling
/// maintenance per fixture removes the concurrent writer instead of retrying around it;
/// `gc.auto` covers Git versions old enough to run `git gc --auto` directly.
fn configure_fixture_repo(root: &Path) {
    git_success(root, ["config", "user.email", "scm-facts@example.test"]);
    git_success(root, ["config", "user.name", "SCM Facts Integration"]);
    git_success(root, ["config", "maintenance.auto", "false"]);
    git_success(root, ["config", "gc.auto", "0"]);
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

fn head_topology(root: &Path) -> Vec<String> {
    git_stdout(root, ["rev-list", "--parents", "-n", "1", "HEAD"])
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

fn source_candidate_revision(root: &Path) -> String {
    let topology = head_topology(root);
    match topology.as_slice() {
        [evaluated, _parent] => evaluated.clone(),
        [evaluated, protected, subject] => {
            select_census_event_from_event(
                root,
                protected,
                evaluated,
                "pull_request",
                "refs/pull/1/merge",
                "dev",
                subject,
            )
            .expect("two-parent source checkout must be an exact synthetic PR topology");
            subject.clone()
        }
        _ => panic!(
            "source checkout must be a linear candidate or exact synthetic PR merge: {topology:?}"
        ),
    }
}

fn commit_all(root: &Path, message: &str) -> String {
    git_success(root, ["add", "--all"]);
    git_success(root, ["commit", "--quiet", "-m", message]);
    git_stdout(root, ["rev-parse", "HEAD"])
}

fn commit_paths_allow_empty(root: &Path, paths: &[&str], message: &str) -> String {
    let output = Command::new("git")
        .args(["add", "--"])
        .args(paths)
        .current_dir(root)
        .output()
        .expect("stage bounded Git fixture paths");
    assert!(
        output.status.success(),
        "Git fixture path staging failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    git_success(root, ["commit", "--quiet", "--allow-empty", "-m", message]);
    git_stdout(root, ["rev-parse", "HEAD"])
}

const P3_PROTECTED_SOURCE_PATHS: &[&str] = &[
    "ci/facade/scm-facts-snapshot/BUCK",
    "ci/facade/scm-facts-snapshot/Cargo.toml",
    "ci/facade/scm-facts-snapshot/src/bin/adr-census-epoch-receipt-gate.rs",
    "ci/facade/scm-facts-snapshot/src/lib.rs",
    "ci/facade/scm-facts-snapshot/src/main.rs",
    "ci/facade/scm-facts-snapshot/src/retirement.rs",
    "governance/corpus/doc-parser/BUCK",
    "governance/corpus/doc-parser/Cargo.toml",
    "governance/corpus/doc-parser/src/lib.rs",
    "governance/corpus/doc-parser/tests/fixtures/adr-heading-reference.md",
    "governance/corpus/doc-parser/tests/fixtures/adversarial-exfil.md",
    "governance/corpus/work-area-tree-kernel/BUCK",
    "governance/corpus/work-area-tree-kernel/Cargo.toml",
    "governance/corpus/work-area-tree-kernel/src/lib.rs",
    "specs/adr-census-epoch-control-plane.schema.json",
    "specs/adr-census-epoch-receipt.schema.json",
];

fn write_fixture_file(root: &Path, path: &str, bytes: &[u8]) {
    let destination = root.join(path);
    std::fs::create_dir_all(destination.parent().expect("fixture file parent"))
        .expect("create fixture file parent");
    std::fs::write(destination, bytes).expect("write fixture file");
}

fn write_census_control(root: &Path, epoch: &str) {
    write_fixture_file(
        root,
        "registry/adr-census-epoch/control-plane.json",
        format!(
            "{{
  \"$schema\": \"https://docs.oyatie.com/schemas/adr-census-epoch-control-plane.schema.json\",
  \"schema_version\": 1,
  \"canonical_name\": \"adr-census-epoch-control-plane\",
  \"receipt_path\": \"ci/facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json\",
  \"active_epoch\": \"{epoch}\"
}}
"
        )
        .as_bytes(),
    );
}

fn p3_identity_fixture(label: &str) -> PathBuf {
    let source_root = discover_repo_root().expect("discover source repository root");
    let root = temp_git_repo(label);
    for path in P3_PROTECTED_SOURCE_PATHS {
        write_fixture_file(
            &root,
            path,
            &std::fs::read(source_root.join(path)).expect("read protected source fixture"),
        );
    }
    write_fixture_file(
        &root,
        "docs/decisions/ADR-0700-ci-admission-live-apex.md",
        &std::fs::read(source_root.join(
            "docs/decisions/ADR-0700-ci-admission-live-apex.md",
        ))
        .expect("read selected ADR fixture"),
    );
    write_fixture_file(&root, "docs/README.md", b"unselected documentation\n");
    write_fixture_file(
        &root,
        "docs/decisions/nested/ADR-9999-unselected.md",
        b"nested and therefore unselected\n",
    );
    write_fixture_file(
        &root,
        "registry/adr-census-epoch/control-plane.json",
        br#"{
  "$schema": "https://docs.oyatie.com/schemas/adr-census-epoch-control-plane.schema.json",
  "schema_version": 1,
  "canonical_name": "adr-census-epoch-control-plane",
  "receipt_path": "ci/facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json",
  "active_epoch": "P2"
}
"#,
    );
    write_fixture_file(&root, ".buckconfig", b"[cells]\n  root = .\n");
    write_fixture_file(
        &root,
        "rust-toolchain.toml",
        b"[toolchain]\nchannel = \"stable\"\n",
    );
    write_fixture_file(&root, "third-party/BUCK", b"# third-party\n");
    write_fixture_file(&root, "toolchains/BUCK", b"# toolchains\n");
    configure_ignored_epoch_receipt(&root);
    commit_all(&root, "seed P3 identity fixture");
    root
}

fn p3_history_fixture(label: &str) -> PathBuf {
    let source_root = discover_repo_root().expect("discover source repository root");
    let source_candidate = source_candidate_revision(&source_root);
    let root = temp_path(label);
    let output = Command::new("git")
        .args(["clone", "--quiet", "--shared", "--no-checkout"])
        .arg(&source_root)
        .arg(&root)
        .output()
        .expect("clone P3 history fixture");
    assert!(
        output.status.success(),
        "clone P3 history fixture failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    configure_fixture_repo(&root);
    git_success(
        &root,
        ["checkout", "--quiet", "--detach", &source_candidate],
    );
    for path in P3_PROTECTED_SOURCE_PATHS {
        write_fixture_file(
            &root,
            path,
            &std::fs::read(source_root.join(path)).expect("read protected source fixture"),
        );
    }
    commit_paths_allow_empty(
        &root,
        P3_PROTECTED_SOURCE_PATHS,
        "bind current P3 policy to historical P2 ancestry",
    );
    root
}

/// Copy the pre-step's materialized P2 parent receipt into a fixture clone.
///
/// Deliberately `expect`s rather than skipping: an active-P2 emission cannot be exercised at all
/// without these bytes, and a test that quietly passed because the pre-step never ran would be a
/// false green strictly worse than a red.
fn copy_materialized_p2_parent_receipt(root: &Path) -> PathBuf {
    let source = discover_repo_root()
        .expect("discover source repository root")
        .join(ADR_CENSUS_PARENT_RECEIPT_PATH);
    let destination = root.join(ADR_CENSUS_PARENT_RECEIPT_PATH);
    std::fs::create_dir_all(destination.parent().expect("parent receipt face parent"))
        .expect("create parent receipt face directory");
    std::fs::copy(&source, &destination).unwrap_or_else(|error| {
        panic!(
            "the out-of-graph pre-step must have materialized {}: {error}",
            source.display()
        )
    });
    destination
}

fn mutate_fixture_and_commit(root: &Path, path: &str, mutation: &[u8]) {
    let destination = root.join(path);
    let mut bytes = std::fs::read(&destination).expect("read fixture mutation target");
    bytes.extend_from_slice(mutation);
    std::fs::write(destination, bytes).expect("write fixture mutation target");
    commit_all(root, "mutate P3 identity input");
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

fn configure_ignored_epoch_receipt(root: &Path) {
    std::fs::write(root.join(".gitignore"), "**/*.generated.json\n")
        .expect("ignore canonical ADR census epoch receipt output");
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

/// A git FAULT on the ignore probe must not be reported as a policy violation.
///
/// `git check-ignore --quiet` answers 0 = ignored, 1 = NOT ignored, 128 = git itself failed.
/// The probe used to test `!= Some(0)`, collapsing 1 and 128, so a stale `index.lock` or a
/// broken repository reached the consuming gate as "must be ignored and untracked" — a wrong
/// answer ABOUT THE CANDIDATE TREE, manufactured from a fault in the tool asking the question.
/// This gates `producer-regen`, so that wrong answer reds every job downstream of it.
///
/// The fixture is a directory holding a `.git` REGULAR FILE containing garbage: git stops its
/// upward repository walk at any `.git` entry, so this faults with exit 128 no matter where the
/// temp directory sits — including inside a real repository's tree, which a "no repository
/// above here" fixture cannot promise (buck2 may root TMPDIR under `buck-out`, where the
/// enclosing repo would answer 1 and the test would silently stop testing anything).
#[cfg(unix)]
#[test]
fn ignore_probe_distinguishes_a_git_fault_from_a_policy_violation() {
    let root = temp_path("ignore-probe-fault");
    std::fs::create_dir(&root).expect("create ignore-probe fault fixture root");
    std::fs::write(root.join(".git"), b"not a gitfile").expect("write broken gitfile");

    let error = CanonicalIgnoredGeneratedWriter::open(&root, Path::new(EPOCH_RECEIPT_PATH))
        .map(|_| ())
        .expect_err("a faulting ignore probe must fail closed");

    assert!(
        error.contains("check ignored generated output boundary exited with Some(128)"),
        "a git fault must be reported as a fault, got: {error}"
    );
    assert!(
        !error.contains("must be ignored and untracked"),
        "a git fault must NOT be reported as a policy violation, got: {error}"
    );
    // Also proves the capture is wired on THIS probe: the fault is only ever explained on
    // git's stderr, so an error carrying it could not have come from a discarded sink.
    assert!(
        error.contains("child stderr"),
        "the fault must carry git's own explanation, got: {error}"
    );
    std::fs::remove_dir_all(root).expect("remove ignore-probe fault fixture root");
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

#[cfg(unix)]
#[test]
fn epoch_receipt_leaf_symlink_is_rejected_without_touching_target() {
    use std::os::unix::fs::symlink;

    let root = temp_git_repo("epoch-receipt-leaf-symlink");
    configure_ignored_epoch_receipt(&root);
    let output = root.join(EPOCH_RECEIPT_PATH);
    std::fs::create_dir_all(output.parent().expect("epoch receipt parent"))
        .expect("create epoch receipt parent");
    let outside = root.join("outside.json");
    std::fs::write(&outside, b"outside bytes").expect("write outside target");
    symlink(&outside, &output).expect("link epoch receipt output");

    let error = write_canonical_ignored_generated_file(
        &root,
        Path::new(EPOCH_RECEIPT_PATH),
        b"replacement",
    )
    .expect_err("epoch receipt leaf symlink must fail closed");
    assert!(error.contains("must be a regular file"));
    assert_eq!(
        std::fs::read(&outside).expect("read outside target"),
        b"outside bytes"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[cfg(unix)]
#[test]
fn epoch_receipt_intermediate_symlink_is_rejected_without_touching_target() {
    use std::os::unix::fs::symlink;

    let root = temp_git_repo("epoch-receipt-intermediate-symlink");
    configure_ignored_epoch_receipt(&root);
    let outside = root.join("outside");
    std::fs::create_dir(&outside).expect("create outside directory");
    let target =
        outside.join("facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json");
    std::fs::create_dir_all(target.parent().expect("outside target parent"))
        .expect("create outside target parent");
    std::fs::write(&target, b"outside bytes").expect("write outside target");
    symlink(&outside, root.join("ci")).expect("link intermediate directory");

    write_canonical_ignored_generated_file(&root, Path::new(EPOCH_RECEIPT_PATH), b"replacement")
        .expect_err("epoch receipt intermediate symlink must fail closed");
    assert_eq!(
        std::fs::read(&target).expect("read outside target"),
        b"outside bytes"
    );
    std::fs::remove_dir_all(root).expect("remove integration fixture");
}

#[cfg(unix)]
#[test]
fn epoch_receipt_writer_stays_bound_to_open_parent_after_ancestor_swap() {
    use std::os::unix::fs::symlink;

    let root = temp_git_repo("epoch-receipt-parent-swap");
    configure_ignored_epoch_receipt(&root);
    let original_ci = root.join("ci");
    let captured_output = original_ci
        .join("facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json");
    std::fs::create_dir_all(captured_output.parent().expect("captured output parent"))
        .expect("create canonical parent");
    let writer = CanonicalIgnoredGeneratedWriter::open(&root, Path::new(EPOCH_RECEIPT_PATH))
        .expect("open epoch receipt writer before the ancestor swap");
    std::fs::rename(&original_ci, root.join("ci-captured")).expect("move opened ancestor");

    let outside = root.join("outside");
    let target =
        outside.join("facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json");
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
            "ci-captured/facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json"
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
fn active_p2_epoch_emission_preserves_the_fixed_historical_receipt() {
    let output_root = temp_repo_root("active-p2-adr-census-epoch");
    let repo_root = discover_repo_root().expect("discover repository root");
    let output = output_root.join("nested/adr-census-epoch-receipt.generated.json");
    let topology = head_topology(&repo_root);
    let event = match topology.as_slice() {
        [_evaluated, _parent] => None,
        [evaluated, protected, subject] => {
            let selection = select_census_event_from_event(
                &repo_root,
                protected,
                evaluated,
                "pull_request",
                "refs/pull/1/merge",
                "dev",
                subject,
            )
            .expect("synthetic PR checkout must select its exact subject");
            Some(
                validate_census_event_transition(&selection)
                    .expect("synthetic PR checkout must preserve a pointer-only P2 transition"),
            )
        }
        _ => panic!(
            "source checkout must be a linear candidate or exact synthetic PR merge: {topology:?}"
        ),
    };
    match event.as_ref() {
        Some(event) => {
            emit_adr_census_epoch_receipt_for_event(event, &output, P2ParentReceipt::Materialized)
                .expect("emit event-bound active P2 epoch receipt")
        }
        None => emit_adr_census_epoch_receipt(&repo_root, &output, P2ParentReceipt::Materialized)
            .expect("emit local active P2 epoch receipt"),
    }
    let receipt = std::fs::read(&output).expect("read active P2 receipt");
    assert_eq!(
        format!("{:x}", sha2::Sha256::digest(&receipt)),
        FIXED_P2_EPOCH_RECEIPT_SHA256
    );
    let value: serde_json::Value = serde_json::from_slice(&receipt).expect("parse receipt");
    assert_eq!(
        value["outer_sha256"],
        "c3c4195f440fbf7825101dcf303fea9d8aec9d2ce7a77bd3ec25d8411dfdf528"
    );
    assert_eq!(
        value["receipt"]["canonical_digest"],
        "7a8eb3848e3b5d1dd148595b5210f2a059fac582db9e5607cf54be2f502b24d8"
    );
    assert_eq!(
        value["receipt"]["aggregate_fold"],
        "2aeb7459f61b6f216b4eee75164bcfb85e405bbe8ca74cf180e5492b09c99507"
    );
    let mut tampered = receipt;
    tampered[0] ^= 1;
    std::fs::write(&output, tampered).expect("write tampered P2 receipt");
    match event.as_ref() {
        Some(event) => {
            assert!(validate_adr_census_epoch_receipt_for_event(event, &output).is_err())
        }
        None => assert!(
            ci_scm_facts_snapshot::validate_adr_census_epoch_receipt(&repo_root, &output).is_err()
        ),
    }
    std::fs::remove_dir_all(output_root).expect("remove epoch receipt fixture");
}

#[test]
fn synthetic_pull_request_merge_selects_exact_subject_and_rejects_topology_laundering() {
    let root = temp_git_repo("synthetic-pr-census-candidate");
    write_fixture_file(&root, "seed", b"base\n");
    let protected = commit_all(&root, "protected base");
    git_success(&root, ["checkout", "-b", "subject"]);
    write_fixture_file(&root, "subject", b"candidate\n");
    let subject = commit_all(&root, "subject");
    git_success(&root, ["checkout", "--detach", &protected]);
    git_success(&root, ["merge", "--no-ff", "--no-edit", &subject]);
    let evaluated = git_stdout(&root, ["rev-parse", "HEAD"]);

    let _selection = select_census_event_from_event(
        &root,
        &protected,
        &evaluated,
        "pull_request",
        "refs/pull/1376/merge",
        "dev",
        &subject,
    )
    .expect("synthetic PR merge must select its exact subject");
    assert!(
        select_census_event_from_event(
            &root,
            &subject,
            &evaluated,
            "pull_request",
            "refs/pull/1376/merge",
            "dev",
            &protected,
        )
        .is_err(),
        "swapped parents must fail closed"
    );
    let third = git_stdout(&root, ["commit-tree", "HEAD^{tree}", "-p", &subject]);
    let extra = git_stdout(
        &root,
        [
            "commit-tree",
            "HEAD^{tree}",
            "-p",
            &third,
            "-p",
            &subject,
            "-p",
            &protected,
        ],
    );
    git_success(&root, ["reset", "--hard", &extra]);
    assert!(
        select_census_event_from_event(
            &root,
            &protected,
            &extra,
            "pull_request",
            "refs/pull/1376/merge",
            "dev",
            &subject,
        )
        .is_err(),
        "extra-parent synthetic merge must fail closed"
    );
    std::fs::remove_dir_all(root).expect("remove synthetic PR fixture");
}

#[test]
fn dev_push_merge_is_not_an_eligible_census_candidate() {
    let root = temp_git_repo("push-merge-census-candidate");
    write_fixture_file(&root, "seed", b"base\n");
    let protected = commit_all(&root, "protected base");
    git_success(&root, ["checkout", "-b", "subject"]);
    write_fixture_file(&root, "subject", b"candidate\n");
    let subject = commit_all(&root, "subject");
    git_success(&root, ["checkout", "--detach", &protected]);
    git_success(&root, ["merge", "--no-ff", "--no-edit", &subject]);
    let evaluated = git_stdout(&root, ["rev-parse", "HEAD"]);
    assert!(
        select_census_event_from_event(
            &root,
            &protected,
            &evaluated,
            "push",
            "refs/heads/dev",
            "refs/heads/dev",
            &evaluated,
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).expect("remove push merge fixture");
}

#[test]
fn linear_dev_push_selects_exact_evaluated_commit() {
    let root = temp_git_repo("linear-push-census-candidate");
    write_fixture_file(&root, "seed", b"base\n");
    let protected = commit_all(&root, "protected base");
    write_fixture_file(&root, "candidate", b"candidate\n");
    let evaluated = commit_all(&root, "linear candidate");

    let selection = select_census_event_from_event(
        &root,
        &protected,
        &evaluated,
        "push",
        "refs/heads/dev",
        "refs/heads/dev",
        &evaluated,
    )
    .expect("linear dev push must select its exact evaluated commit");
    validate_census_event_transition(&selection)
        .expect("linear dev push must retain one validated event boundary");
    std::fs::remove_dir_all(root).expect("remove linear push fixture");
}

#[test]
fn merge_group_selects_exact_evaluated_commit() {
    let root = temp_git_repo("merge-group-census-candidate");
    write_fixture_file(&root, "seed", b"base\n");
    let protected = commit_all(&root, "protected base");
    git_success(&root, ["checkout", "-b", "subject"]);
    write_fixture_file(&root, "subject", b"candidate\n");
    let subject = commit_all(&root, "subject");
    git_success(&root, ["checkout", "--detach", &protected]);
    git_success(&root, ["merge", "--no-ff", "--no-edit", &subject]);
    let evaluated = git_stdout(&root, ["rev-parse", "HEAD"]);

    let selection = select_census_event_from_event(
        &root,
        &protected,
        &evaluated,
        "merge_group",
        "refs/heads/gh-readonly-queue/dev/pr-1376",
        "refs/heads/dev",
        &evaluated,
    )
    .expect("merge group must select its exact evaluated commit");
    validate_census_event_transition(&selection)
        .expect("merge group must retain one validated event boundary");
    std::fs::remove_dir_all(root).expect("remove merge-group fixture");
}

#[test]
fn candidate_wide_pointer_transition_rejects_a_prior_policy_commit() {
    let root = temp_git_repo("candidate-wide-pointer-laundering");
    write_census_control(&root, "P2");
    let protected = commit_all(&root, "protected P2");
    git_success(&root, ["checkout", "-b", "subject"]);
    write_fixture_file(&root, "policy.txt", b"candidate policy\n");
    commit_all(&root, "change candidate policy");
    write_census_control(&root, "P3");
    let subject = commit_all(&root, "activate P3 pointer");
    git_success(&root, ["checkout", "--detach", &protected]);
    git_success(&root, ["merge", "--no-ff", "--no-edit", &subject]);
    let evaluated = git_stdout(&root, ["rev-parse", "HEAD"]);

    let selection = select_census_event_from_event(
        &root,
        &protected,
        &evaluated,
        "pull_request",
        "refs/pull/1376/merge",
        "dev",
        &subject,
    )
    .expect("synthetic PR event must select the candidate");
    let error = validate_census_event_transition(&selection)
        .expect_err("policy plus pointer candidate must fail closed");
    assert!(
        error.contains(
            "ADR census epoch candidate transition may change only the control-plane pointer"
        ),
        "{error}"
    );
    std::fs::remove_dir_all(root).expect("remove pointer-laundering fixture");
}

#[test]
fn candidate_wide_pointer_only_transition_accepts_the_exact_delta() {
    let root = temp_git_repo("candidate-wide-pointer-only");
    write_census_control(&root, "P2");
    let protected = commit_all(&root, "protected P2");
    git_success(&root, ["checkout", "-b", "subject"]);
    write_census_control(&root, "P3");
    let subject = commit_all(&root, "activate P3 pointer");
    git_success(&root, ["checkout", "--detach", &protected]);
    git_success(&root, ["merge", "--no-ff", "--no-edit", &subject]);
    let evaluated = git_stdout(&root, ["rev-parse", "HEAD"]);

    let selection = select_census_event_from_event(
        &root,
        &protected,
        &evaluated,
        "pull_request",
        "refs/pull/1376/merge",
        "dev",
        &subject,
    )
    .expect("synthetic PR event must select the candidate");
    validate_census_event_transition(&selection)
        .expect("exact pointer-only candidate transition must remain eligible");
    std::fs::remove_dir_all(root).expect("remove pointer-only fixture");
}

#[test]
fn synthetic_pr_p3_receipt_uses_evaluated_tree_while_subject_history_stays_pointer_only() {
    let root = p3_history_fixture("synthetic-pr-p3-evaluated-tree");
    let stale_base = git_stdout(&root, ["rev-parse", "HEAD"]);
    git_success(&root, ["checkout", "-b", "subject"]);
    write_census_control(&root, "P3");
    let subject = commit_all(&root, "activate P3 pointer from stale base");

    git_success(&root, ["checkout", "--detach", &stale_base]);
    let mutated_path = "ci/facade/scm-facts-snapshot/src/lib.rs";
    mutate_fixture_and_commit(
        &root,
        mutated_path,
        b"\n// protected P3 identity update after subject branch\n",
    );
    let protected = git_stdout(&root, ["rev-parse", "HEAD"]);
    git_success(&root, ["merge", "--no-ff", "--no-edit", &subject]);
    let evaluated = git_stdout(&root, ["rev-parse", "HEAD"]);

    let selection = select_census_event_from_event(
        &root,
        &protected,
        &evaluated,
        "pull_request",
        "refs/pull/1376/merge",
        "dev",
        &subject,
    )
    .expect("synthetic PR event must retain subject history and evaluated content");
    let validated = validate_census_event_transition(&selection)
        .expect("pointer-only subject transition must remain eligible after protected rebasing");

    let output_root = temp_path("synthetic-pr-p3-evaluated-receipt");
    std::fs::create_dir(&output_root).expect("create external P3 receipt output root");
    let output = output_root.join("receipt.json");
    emit_adr_census_epoch_receipt_for_event(&validated, &output, P2ParentReceipt::Materialized)
        .expect("event-bound P3 receipt must materialize");
    validate_adr_census_epoch_receipt_for_event(&validated, &output)
        .expect("event-bound P3 receipt must validate");

    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).expect("read event-bound P3 receipt"))
            .expect("parse event-bound P3 receipt");
    let source = value
        .pointer("/receipt/core/execution/source_closure/producer_gate")
        .and_then(serde_json::Value::as_array)
        .expect("P3 producer/gate source closure")
        .iter()
        .find(|row| row["path"].as_str() == Some(mutated_path))
        .expect("mutated protected source must remain in the P3 closure");
    let evaluated_blob = git_stdout(&root, ["rev-parse", &format!("{evaluated}:{mutated_path}")]);
    let subject_blob = git_stdout(&root, ["rev-parse", &format!("{subject}:{mutated_path}")]);
    assert_ne!(
        evaluated_blob, subject_blob,
        "fixture must distinguish evaluated protected content from the stale subject"
    );
    assert_eq!(
        source["blob_oid"], evaluated_blob,
        "P3 content identity must derive from the evaluated candidate tree"
    );

    std::fs::remove_dir_all(output_root).expect("remove external P3 receipt output");
    std::fs::remove_dir_all(root).expect("remove stale-subject P3 fixture");
}

#[test]
fn synthetic_pr_p3_to_p2_pointer_only_rollback_emits_the_fixed_receipt() {
    let root = p3_history_fixture("synthetic-pr-p3-to-p2-rollback");
    // The fixture is a bare-ish clone, so it carries no untracked face. Rolling back to P2 makes
    // this emission a P2 one, which CONSUMES the pre-step's parent receipt — copy it in from the
    // source checkout, and fail loudly (not skip) when the pre-step has not run.
    copy_materialized_p2_parent_receipt(&root);
    write_census_control(&root, "P3");
    let protected = commit_all(&root, "activate protected P3 pointer");

    git_success(&root, ["checkout", "-b", "subject"]);
    write_census_control(&root, "P2");
    let subject = commit_all(&root, "roll back to P2 pointer");
    git_success(&root, ["checkout", "--detach", &protected]);
    git_success(&root, ["merge", "--no-ff", "--no-edit", &subject]);
    let evaluated = git_stdout(&root, ["rev-parse", "HEAD"]);

    let selection = select_census_event_from_event(
        &root,
        &protected,
        &evaluated,
        "pull_request",
        "refs/pull/1376/merge",
        "dev",
        &subject,
    )
    .expect("synthetic PR rollback must select its exact subject");
    let validated = validate_census_event_transition(&selection)
        .expect("pointer-only P3 to P2 rollback must remain eligible");

    let output_root = temp_path("synthetic-pr-p3-to-p2-receipt");
    std::fs::create_dir(&output_root).expect("create external P2 rollback receipt root");
    let output = output_root.join("receipt.json");
    emit_adr_census_epoch_receipt_for_event(&validated, &output, P2ParentReceipt::Materialized)
        .expect("event-bound P2 rollback receipt must materialize");
    validate_adr_census_epoch_receipt_for_event(&validated, &output)
        .expect("event-bound P2 rollback receipt must validate");
    assert_eq!(
        format!(
            "{:x}",
            sha2::Sha256::digest(std::fs::read(&output).expect("read rollback receipt"))
        ),
        FIXED_P2_EPOCH_RECEIPT_SHA256,
        "P3 to P2 rollback must restore the fixed historical receipt"
    );

    std::fs::remove_dir_all(output_root).expect("remove external P2 rollback receipt");
    std::fs::remove_dir_all(root).expect("remove P3 to P2 rollback fixture");
}

/// The in-graph P2 contract, pinned in all three states so "it passed" cannot mean "it checked
/// nothing": absent face -> named failure, wrong bytes -> named failure, real bytes -> the fixed
/// receipt. Written as one test on ONE fixture so the green leg is the same emission as the two
/// red ones, which is what makes them evidence rather than assertion.
#[test]
fn active_p2_emission_consumes_the_out_of_graph_parent_receipt_and_never_skips() {
    let root = p3_history_fixture("p2-parent-receipt-contract");
    let output_root = temp_path("p2-parent-receipt-contract-out");
    std::fs::create_dir(&output_root).expect("create parent-receipt contract output root");
    let output = output_root.join("receipt.json");

    let absent = emit_adr_census_epoch_receipt(&root, &output, P2ParentReceipt::Materialized)
        .expect_err("an absent parent receipt face must fail the emission, never skip it");
    assert!(
        absent.contains("read materialized exact historical P2 parent receipt"),
        "{absent}"
    );
    assert!(
        absent.contains("oya-cloud-ci-materialize-generated-faces-bin"),
        "the failure must name the out-of-graph pre-step that produces it: {absent}"
    );
    assert!(
        !output.exists(),
        "a failed emission must leave no receipt behind"
    );

    let face = copy_materialized_p2_parent_receipt(&root);
    let mut tampered = std::fs::read(&face).expect("read copied parent receipt face");
    tampered[0] ^= 1;
    std::fs::write(&face, tampered).expect("tamper the parent receipt face");
    let corrupted = emit_adr_census_epoch_receipt(&root, &output, P2ParentReceipt::Materialized)
        .expect_err("a tampered parent receipt face must fail the emission");
    assert!(
        corrupted.contains("exact historical P2 whole-file digest differs"),
        "{corrupted}"
    );

    copy_materialized_p2_parent_receipt(&root);
    emit_adr_census_epoch_receipt(&root, &output, P2ParentReceipt::Materialized)
        .expect("the restored parent receipt face must emit");
    assert_eq!(
        format!(
            "{:x}",
            sha2::Sha256::digest(std::fs::read(&output).expect("read restored receipt"))
        ),
        FIXED_P2_EPOCH_RECEIPT_SHA256,
        "the same emission that went red twice must produce the fixed receipt from real bytes"
    );

    std::fs::remove_dir_all(output_root).expect("remove parent-receipt contract output");
    std::fs::remove_dir_all(root).expect("remove parent-receipt contract fixture");
}

#[test]
fn synthetic_pr_p3_to_p2_rollback_rejects_a_prior_policy_change() {
    let root = p3_history_fixture("synthetic-pr-p3-to-p2-laundering");
    write_census_control(&root, "P3");
    let protected = commit_all(&root, "activate protected P3 pointer");

    git_success(&root, ["checkout", "-b", "subject"]);
    write_fixture_file(&root, "policy.txt", b"candidate policy\n");
    commit_all(&root, "change candidate policy before rollback");
    write_census_control(&root, "P2");
    let subject = commit_all(&root, "roll back to P2 pointer");
    git_success(&root, ["checkout", "--detach", &protected]);
    git_success(&root, ["merge", "--no-ff", "--no-edit", &subject]);
    let evaluated = git_stdout(&root, ["rev-parse", "HEAD"]);

    let selection = select_census_event_from_event(
        &root,
        &protected,
        &evaluated,
        "pull_request",
        "refs/pull/1376/merge",
        "dev",
        &subject,
    )
    .expect("synthetic PR rollback must select its exact subject");
    let error = validate_census_event_transition(&selection)
        .expect_err("policy plus rollback candidate must fail closed");
    assert!(
        error.contains(
            "ADR census epoch candidate transition may change only the control-plane pointer"
        ),
        "{error}"
    );

    std::fs::remove_dir_all(root).expect("remove P3 to P2 laundering fixture");
}

#[test]
fn active_control_linear_push_matches_local_receipt_bytes() {
    let root = p3_history_fixture("active-control-linear-push");
    let protected = git_stdout(&root, ["rev-parse", "HEAD"]);
    write_census_control(&root, "P3");
    let evaluated = commit_all(&root, "activate P3 pointer by linear push");

    let selection = select_census_event_from_event(
        &root,
        &protected,
        &evaluated,
        "push",
        "refs/heads/dev",
        "refs/heads/dev",
        &evaluated,
    )
    .expect("active-control linear push must select its exact evaluated commit");
    let validated = validate_census_event_transition(&selection)
        .expect("active-control linear push must validate its pointer-only transition");

    let output_root = temp_path("active-control-linear-push-receipts");
    std::fs::create_dir(&output_root).expect("create external push receipt root");
    let event_output = output_root.join("event.json");
    let local_output = output_root.join("local.json");
    emit_adr_census_epoch_receipt_for_event(
        &validated,
        &event_output,
        P2ParentReceipt::Materialized,
    )
    .expect("event-bound push receipt must materialize");
    emit_adr_census_epoch_receipt(&root, &local_output, P2ParentReceipt::Materialized)
        .expect("local receipt at the evaluated commit must materialize");
    assert_eq!(
        std::fs::read(&event_output).expect("read event-bound push receipt"),
        std::fs::read(&local_output).expect("read local push receipt"),
        "event-bound and local receipt paths must agree at the same immutable commit"
    );
    validate_adr_census_epoch_receipt_for_event(&validated, &event_output)
        .expect("event-bound push receipt must validate");
    ci_scm_facts_snapshot::validate_adr_census_epoch_receipt(&root, &local_output)
        .expect("local push receipt must validate");

    std::fs::remove_dir_all(output_root).expect("remove external push receipts");
    std::fs::remove_dir_all(root).expect("remove active-control push fixture");
}

#[test]
fn active_p3_merge_group_emits_from_the_exact_evaluated_tree() {
    let root = p3_history_fixture("active-p3-merge-group");
    write_census_control(&root, "P3");
    let protected = commit_all(&root, "activate protected P3 pointer");

    git_success(&root, ["checkout", "-b", "subject"]);
    mutate_fixture_and_commit(
        &root,
        "docs/README.md",
        b"\nmerge-group candidate documentation\n",
    );
    let subject = git_stdout(&root, ["rev-parse", "HEAD"]);
    git_success(&root, ["checkout", "--detach", &protected]);
    git_success(&root, ["merge", "--no-ff", "--no-edit", &subject]);
    let evaluated = git_stdout(&root, ["rev-parse", "HEAD"]);

    let selection = select_census_event_from_event(
        &root,
        &protected,
        &evaluated,
        "merge_group",
        "refs/heads/gh-readonly-queue/dev/pr-1376",
        "refs/heads/dev",
        &evaluated,
    )
    .expect("active P3 merge group must select its exact evaluated commit");
    let validated = validate_census_event_transition(&selection)
        .expect("active P3 merge group must retain one validated event boundary");

    let output_root = temp_path("active-p3-merge-group-receipt");
    std::fs::create_dir(&output_root).expect("create external merge-group receipt root");
    let output = output_root.join("receipt.json");
    emit_adr_census_epoch_receipt_for_event(&validated, &output, P2ParentReceipt::Materialized)
        .expect("event-bound merge-group receipt must materialize");
    validate_adr_census_epoch_receipt_for_event(&validated, &output)
        .expect("event-bound merge-group receipt must validate");

    std::fs::remove_dir_all(output_root).expect("remove external merge-group receipt");
    std::fs::remove_dir_all(root).expect("remove active P3 merge-group fixture");
}

#[test]
fn candidate_event_rejects_a_synthetic_tree_with_a_different_pointer() {
    let root = temp_git_repo("candidate-synthetic-pointer-divergence");
    write_census_control(&root, "P2");
    let protected = commit_all(&root, "protected P2");
    git_success(&root, ["checkout", "-b", "subject"]);
    write_census_control(&root, "P3");
    let subject = commit_all(&root, "activate P3 pointer");
    let evaluated = git_stdout(
        &root,
        [
            "commit-tree",
            &format!("{protected}^{{tree}}"),
            "-p",
            &protected,
            "-p",
            &subject,
        ],
    );
    git_success(&root, ["reset", "--hard", &evaluated]);

    let selection = select_census_event_from_event(
        &root,
        &protected,
        &evaluated,
        "pull_request",
        "refs/pull/1376/merge",
        "dev",
        &subject,
    )
    .expect("synthetic PR topology must select the subject before pointer comparison");
    let error = validate_census_event_transition(&selection)
        .expect_err("evaluated and selected pointers must agree");
    assert!(
        error.contains("history and content control-plane pointers differ"),
        "{error}"
    );
    std::fs::remove_dir_all(root).expect("remove synthetic pointer divergence fixture");
}

#[test]
fn root_commit_p3_control_reaches_named_bootstrap_shape_failure() {
    let root = temp_git_repo("root-p3-control");
    write_fixture_file(
        &root,
        "registry/adr-census-epoch/control-plane.json",
        br#"{
  "$schema": "https://docs.oyatie.com/schemas/adr-census-epoch-control-plane.schema.json",
  "schema_version": 1,
  "canonical_name": "adr-census-epoch-control-plane",
  "receipt_path": "ci/facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json",
  "active_epoch": "P3"
}
"#,
    );
    commit_all(&root, "root P3 control");

    let output = root.join("out/adr-census-epoch-receipt.generated.json");
    let error = emit_adr_census_epoch_receipt(&root, &output, P2ParentReceipt::Materialized)
        .expect_err("a root P3 control must fail the bootstrap shape rule");
    std::fs::remove_dir_all(&root).expect("remove root P3 control fixture");

    assert!(
        error.contains("only the first P2 bootstrap may introduce the ADR census epoch control"),
        "{error}"
    );
    assert!(!error.contains("rev-parse"), "{error}");
}

#[test]
fn dormant_p3_identity_rejects_a_unicode_direct_adr_path_after_raw_tree_parsing() {
    let root = p3_identity_fixture("unicode-direct-adr");
    let source = root
        .join("docs/decisions/ADR-0700-ci-admission-live-apex.md");
    let unicode_path = "docs/decisions/ADR-0002-résumé.md";
    write_fixture_file(
        &root,
        unicode_path,
        &std::fs::read(source).expect("read valid direct ADR fixture"),
    );
    commit_all(&root, "add Unicode direct ADR fixture");

    let error = dormant_p3_epoch_fingerprint(&root)
        .expect_err("a Unicode direct ADR path must fail the selector's named ASCII contract");
    std::fs::remove_dir_all(&root).expect("remove Unicode direct ADR fixture");

    assert!(
        error.contains("P3 selector path must be ASCII"),
        "unexpected error: {error}"
    );
}

#[test]
fn adr_census_epoch_receipt_schema_types_every_digest_and_oid_pattern() {
    let repo_root = discover_repo_root().expect("discover repository root");
    let schema_path = repo_root.join("specs/adr-census-epoch-receipt.schema.json");
    let schema: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&schema_path).expect("read ADR census epoch receipt schema"),
    )
    .expect("parse ADR census epoch receipt schema");

    for path in [
        "/properties/outer_sha256",
        "/$defs/receipt/properties/canonical_digest",
        "/$defs/core/properties/corpus_content_digest",
        "/$defs/execution/properties/action_source_set_digest",
        "/$defs/execution/properties/producer_gate_source_set_digest",
        "/$defs/execution/properties/toolchain_input_digest",
        "/$defs/producer_gate_source/properties/blob_oid",
        "/$defs/producer_gate_source/properties/sha256",
        "/$defs/toolchain_source/properties/blob_oid",
        "/$defs/toolchain_source/properties/sha256",
        "/$defs/parser/properties/source_set_digest",
        "/$defs/parser_source/properties/blob_oid",
        "/$defs/parser_source/properties/sha256",
        "/$defs/source/properties/blob_oid",
        "/$defs/source/properties/sha256",
    ] {
        let property = schema
            .pointer(path)
            .unwrap_or_else(|| panic!("schema property must exist at {path}"));
        assert!(
            property.get("pattern").is_some(),
            "schema property must retain its format guard at {path}"
        );
        assert_eq!(
            property.get("type").and_then(serde_json::Value::as_str),
            Some("string"),
            "numeric and boolean values must be rejected before pattern evaluation at {path}"
        );
    }

    for removed_path in [
        "/$defs/core/properties/decisions_tree",
        "/$defs/core/properties/docs_tree",
    ] {
        assert!(
            schema.pointer(removed_path).is_none(),
            "P3 projected core must not carry broad tree identity at {removed_path}"
        );
    }
    let protected = schema
        .pointer("/$defs/execution_source_closure/properties/producer_gate")
        .expect("protected source closure schema");
    assert_eq!(protected["minItems"], 16);
    assert_eq!(protected["maxItems"], 16);
    let paths = schema
        .pointer("/$defs/producer_gate_source/properties/path/enum")
        .and_then(serde_json::Value::as_array)
        .expect("protected source path enum");
    for required_path in [
        "specs/adr-census-epoch-control-plane.schema.json",
        "specs/adr-census-epoch-receipt.schema.json",
    ] {
        assert!(
            paths
                .iter()
                .any(|value| value.as_str() == Some(required_path)),
            "P3 protected source closure must bind {required_path}"
        );
    }
}

#[test]
fn census_epoch_artifact_rows_use_the_rust_buck_gate() {
    let repo_root = discover_repo_root().expect("discover repository root");
    let registry: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo_root.join("registry/artifact-capabilities-registry.json"))
            .expect("read artifact capabilities registry"),
    )
    .expect("parse artifact capabilities registry");
    let artifacts = registry["rows"]
        .as_array()
        .expect("artifact capabilities registry must contain artifacts");
    let expected_command = "buck2 test //ci/facade/scm-facts-snapshot:ci-scm-facts-snapshot-gate";

    for artifact_id in [
        "adr-census-epoch-control-plane-schema",
        "adr-census-epoch-receipt-schema",
        "adr-census-epoch-control-plane",
    ] {
        let matching = artifacts
            .iter()
            .filter(|artifact| artifact["artifact_id"].as_str() == Some(artifact_id))
            .collect::<Vec<_>>();
        assert_eq!(
            matching.len(),
            1,
            "{artifact_id} must be registered exactly once"
        );
        assert_eq!(
            matching[0]
                .pointer("/capability_overrides/verification/command")
                .and_then(serde_json::Value::as_str),
            Some(expected_command),
            "{artifact_id} must use the Rust/Buck census gate"
        );
    }
}

#[test]
fn census_epoch_owners_is_adr_justified_and_not_hand_registered() {
    let repo_root = discover_repo_root().expect("discover repository root");
    let owners_path = "registry/adr-census-epoch/OWNERS";
    let adr = std::fs::read_to_string(
        repo_root.join("docs/decisions/ADR-0700-ci-admission-live-apex.md"),
    )
    .expect("read ADR-0624");
    assert!(
        adr.contains(owners_path),
        "ADR-0624 must name the census epoch ownership marker as an implementation surface"
    );

    let reachability: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo_root.join("specs/reachability-registry.json"))
            .expect("read reachability registry"),
    )
    .expect("parse reachability registry");
    let matching = reachability["registered"]
        .as_array()
        .expect("reachability registrations must be an array")
        .iter()
        .filter(|row| row["prefix"].as_str() == Some(owners_path))
        .collect::<Vec<_>>();
    assert!(
        matching.is_empty(),
        "OWNERS files are accounted by construction and must not regain hand-written reachability rows"
    );
}

#[test]
fn dormant_p3_identity_is_bounded_to_selected_inputs() {
    let unchanged_inputs = [
        (
            "pointer-only-p2-to-p3",
            "registry/adr-census-epoch/control-plane.json",
            None,
        ),
        (
            "unrelated-docs",
            "docs/README.md",
            Some(b"unrelated\n".as_slice()),
        ),
        (
            "nested-unselected-decision",
            "docs/decisions/nested/ADR-9999-unselected.md",
            Some(b"nested\n".as_slice()),
        ),
    ];
    for (label, path, mutation) in unchanged_inputs {
        let root = p3_identity_fixture(label);
        let baseline = dormant_p3_epoch_fingerprint(&root).expect("build baseline P3 fingerprint");
        if let Some(mutation) = mutation {
            mutate_fixture_and_commit(&root, path, mutation);
        } else {
            std::fs::write(
                root.join(path),
                br#"{
  "$schema": "https://docs.oyatie.com/schemas/adr-census-epoch-control-plane.schema.json",
  "schema_version": 1,
  "canonical_name": "adr-census-epoch-control-plane",
  "receipt_path": "ci/facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json",
  "active_epoch": "P3"
}
"#,
            )
            .expect("write pointer-only P3 transition");
            commit_all(&root, "pointer-only P2 to P3");
        }
        assert_eq!(
            dormant_p3_epoch_fingerprint(&root).expect("rebuild unchanged P3 fingerprint"),
            baseline,
            "{label} must not perturb the dormant P3 projection bytes"
        );
        std::fs::remove_dir_all(root).expect("remove unchanged identity fixture");
    }

    for (label, path) in [
        (
            "direct-adr",
            "docs/decisions/ADR-0700-ci-admission-live-apex.md",
        ),
        ("producer-gate", "ci/facade/scm-facts-snapshot/src/main.rs"),
        (
            "control-schema",
            "specs/adr-census-epoch-control-plane.schema.json",
        ),
        (
            "receipt-schema",
            "specs/adr-census-epoch-receipt.schema.json",
        ),
        ("toolchain", ".buckconfig"),
    ] {
        let root = p3_identity_fixture(label);
        let baseline = dormant_p3_epoch_fingerprint(&root).expect("build baseline P3 fingerprint");
        mutate_fixture_and_commit(&root, path, b"\nP3 identity mutation\n");
        assert_ne!(
            dormant_p3_epoch_fingerprint(&root).expect("rebuild changed P3 fingerprint"),
            baseline,
            "{label} must change the P3 identity after recompilation"
        );
        std::fs::remove_dir_all(root).expect("remove changed identity fixture");
    }

    let root = p3_identity_fixture("parser-source-mismatch");
    mutate_fixture_and_commit(
        &root,
        "governance/corpus/doc-parser/src/lib.rs",
        b"\nP3 parser source mismatch\n",
    );
    let error = dormant_p3_epoch_fingerprint(&root)
        .expect_err("a Git parser source mismatch must fail closed before identity is claimed");
    assert!(error.contains("parser source set is invalid"), "{error}");
    std::fs::remove_dir_all(root).expect("remove parser source mismatch fixture");
}

#[test]
fn adr_0515_chronology_names_the_complete_live_amendment_and_epoch_gate_boundary() {
    let repo_root = discover_repo_root().expect("discover repository root");
    let adr =
        std::fs::read_to_string(repo_root.join(
            "docs/decisions/ADR-0700-ci-admission-live-apex.md",
        ))
        .expect("read ADR-0515");
    assert!(adr.contains(
        "amended_by: [ADR-0516, ADR-0519, ADR-0526, ADR-0527, ADR-0528, ADR-0529, ADR-0530, ADR-0624, ADR-0639]"
    ));
    assert!(
        !adr.contains("adr-census-parent-receipt-gate.rs"),
        "ADR-0515 must not name the retired parent-only gate as live"
    );
    let normalized_adr = adr.split_whitespace().collect::<Vec<_>>().join(" ");
    for required_statement in [
        "adr-census-epoch-receipt-gate",
        "P2 remains active",
        "P3 remains dormant",
        "sole protected `oya-ci-required` context",
        "does not authorize planning dispatch",
    ] {
        assert!(
            normalized_adr.contains(required_statement),
            "ADR-0515 must retain the live epoch-gate boundary: {required_statement}"
        );
    }
}
