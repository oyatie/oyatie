// #1504 repository-side capacity contract. These tests deliberately inspect source declarations:
// rollout and cold-concurrency evidence are external acceptance steps and must not be inferred from
// a successful render.
#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        assert!(dir.pop(), "failed to locate repository root");
    }
    panic!("failed to locate repository root");
}

fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn assert_contains_all(text: &str, required: &[&str], surface: &str) {
    for needle in required {
        assert!(
            text.contains(needle),
            "{surface} must contain the capacity invariant `{needle}`"
        );
    }
}

#[test]
fn talos_patch_allocates_only_the_blank_worker_disk_and_bounds_the_filesystem() {
    let root = repo_root();
    let patch = read(
        &root,
        "infra/talos/local/patches/ci-workspace-worker-2.yaml",
    );

    assert_contains_all(
        &patch,
        &[
            "kind: UserVolumeConfig",
            "name: ci-workspace",
            "!system_disk",
            "disk.dev_path == '/dev/vdb'",
            "disk.size == 150u * GiB",
            "minSize: 96GiB",
            "maxSize: 96GiB",
            "oya.io/ci-workspace: \"true\"",
        ],
        "Talos CI workspace patch",
    );
    assert!(
        !patch.contains("oya.cell/ci") && !patch.contains("oya.cell/role"),
        "a dedicated workspace disk on the current mixed worker must not be represented as a dedicated CI cell"
    );
}

#[test]
fn dedicated_provisioner_cannot_claim_the_existing_stateful_storage_class_or_other_nodes() {
    let root = repo_root();
    let storage = read(&root, "infra/arc/ci-workspace-storage.yaml");

    assert_contains_all(
        &storage,
        &[
            "name: oya-ci-workspace",
            "provisioner: oyatie.io/ci-workspace-local-path",
            "--provisioner-name",
            "oyatie.io/ci-workspace-local-path",
            "\"node\":\"oya-talos-worker-2\"",
            "\"paths\":[\"/var/mnt/ci-workspace\"]",
            "volumeBindingMode: WaitForFirstConsumer",
            "reclaimPolicy: Delete",
        ],
        "dedicated CI local-path provisioner",
    );
    assert!(
        !storage.contains("DEFAULT_PATH_FOR_NON_LISTED_NODES"),
        "CI workspace provisioning must fail closed outside the explicitly listed worker"
    );
    assert!(
        !storage.contains("provisioner: rancher.io/local-path\n"),
        "CI workspace provisioning must not compete for the existing stateful local-path class"
    );
}

#[test]
fn runner_workspace_is_ephemeral_node_pinned_and_serialized() {
    let root = repo_root();
    let runner = read(&root, "infra/arc/runner-scale-set-arm64-values.yaml");

    assert_contains_all(
        &runner,
        &[
            "maxRunners: 1",
            "kubernetes.io/hostname: oya-talos-worker-2",
            "oya.io/ci-workspace: \"true\"",
            "mountPath: /home/runner/_work",
            "ephemeral:",
            "storageClassName: oya-ci-workspace",
            "storage: 44Gi",
        ],
        "ARC arm64 runner values",
    );

    let live_postgres = read(
        &root,
        "infra/arc/runner-scale-set-live-postgres-arm64-values.yaml",
    );
    assert_contains_all(
        &live_postgres,
        &[
            "maxRunners: 1",
            "kubernetes.io/hostname: oya-talos-worker-2",
            "oya.io/ci-workspace: \"true\"",
            "mountPath: /home/runner/_work",
            "ephemeral:",
            "storageClassName: oya-ci-workspace",
            "storage: 44Gi",
        ],
        "ARC live-PostgreSQL runner values",
    );
}

#[test]
fn capacity_alerts_cover_disk_workspace_eviction_cleanup_and_queue_delay() {
    let root = repo_root();
    let alerts = read(&root, "infra/arc/ci-workspace-alerts.yaml");

    assert_contains_all(
        &alerts,
        &[
            "kube_node_status_condition",
            "kubelet_volume_stats_available_bytes",
            "container_fs_usage_bytes",
            "reason=\"Evicted\"",
            "kube_persistentvolumeclaim_created",
            "gha_job_startup_duration_seconds_bucket",
        ],
        "CI workspace alert rules",
    );
}
