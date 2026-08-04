// #1504 repository-side capacity contract. Source declarations are parsed structurally; rollout,
// CNI enforcement, and cold-concurrency evidence remain external acceptance steps.
#![allow(clippy::expect_used, clippy::panic)]

use serde::Deserialize;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const GENERAL_VALUES: &str = "infra/arc/runner-scale-set-arm64-values.yaml";
const LIVE_POSTGRES_VALUES: &str = "infra/arc/runner-scale-set-live-postgres-arm64-values.yaml";

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

fn yaml_documents(root: &Path, path: &str) -> Vec<Value> {
    serde_yaml::Deserializer::from_str(&read(root, path))
        .map(|document| {
            Value::deserialize(document).unwrap_or_else(|error| panic!("parse {path}: {error}"))
        })
        .collect()
}

fn yaml(root: &Path, path: &str) -> Value {
    serde_yaml::from_str(&read(root, path)).unwrap_or_else(|error| panic!("parse {path}: {error}"))
}

fn at<'a>(value: &'a Value, keys: &[&str]) -> &'a Value {
    keys.iter().fold(value, |current, key| {
        current
            .get(Value::String((*key).to_owned()))
            .unwrap_or_else(|| panic!("missing YAML path {}", keys.join(".")))
    })
}

fn string_at(value: &Value, keys: &[&str]) -> String {
    at(value, keys)
        .as_str()
        .unwrap_or_else(|| panic!("{} is not a string", keys.join(".")))
        .to_owned()
}

fn is_kind(value: &Value, expected: &str) -> bool {
    value
        .get(Value::String("kind".to_owned()))
        .and_then(Value::as_str)
        == Some(expected)
}

fn u64_at(value: &Value, keys: &[&str]) -> u64 {
    at(value, keys)
        .as_u64()
        .unwrap_or_else(|| panic!("{} is not an unsigned integer", keys.join(".")))
}

fn named<'a>(sequence: &'a Value, name: &str) -> &'a Value {
    sequence
        .as_sequence()
        .expect("expected YAML sequence")
        .iter()
        .find(|entry| string_at(entry, &["name"]) == name)
        .unwrap_or_else(|| panic!("missing named YAML entry {name}"))
}

#[derive(Debug, PartialEq, Eq)]
struct RunnerWorkspace {
    max_runners: u64,
    node: String,
    mount_path: String,
    storage_class: String,
    requested_gib: u64,
}

fn parse_gib(value: &str) -> u64 {
    value
        .strip_suffix("Gi")
        .unwrap_or_else(|| panic!("capacity is not expressed in Gi: {value}"))
        .parse()
        .unwrap_or_else(|error| panic!("invalid Gi capacity {value}: {error}"))
}

fn runner_workspace(values: &Value) -> RunnerWorkspace {
    let runner = named(at(values, &["template", "spec", "containers"]), "runner");
    let workspace_mounts: Vec<&Value> = at(runner, &["volumeMounts"])
        .as_sequence()
        .expect("runner volumeMounts must be a sequence")
        .iter()
        .filter(|mount| string_at(mount, &["name"]) == "workspace")
        .collect();
    assert_eq!(workspace_mounts.len(), 1, "runner must mount one workspace");

    let workspace = named(at(values, &["template", "spec", "volumes"]), "workspace");
    RunnerWorkspace {
        max_runners: u64_at(values, &["maxRunners"]),
        node: string_at(
            values,
            &["template", "spec", "nodeSelector", "kubernetes.io/hostname"],
        ),
        mount_path: string_at(workspace_mounts[0], &["mountPath"]),
        storage_class: string_at(
            workspace,
            &[
                "ephemeral",
                "volumeClaimTemplate",
                "spec",
                "storageClassName",
            ],
        ),
        requested_gib: parse_gib(&string_at(
            workspace,
            &[
                "ephemeral",
                "volumeClaimTemplate",
                "spec",
                "resources",
                "requests",
                "storage",
            ],
        )),
    }
}

fn validate_capacity_contract(
    runners: &[RunnerWorkspace],
    storage_paths: &BTreeMap<String, String>,
    filesystems_gib: &BTreeMap<String, u64>,
) -> Result<(), String> {
    let mut claimed_paths = BTreeSet::new();
    for runner in runners {
        if runner.max_runners != 1 {
            return Err(format!(
                "{} allows more than one runner",
                runner.storage_class
            ));
        }
        if runner.requested_gib != 44 {
            return Err(format!("{} must request 44Gi", runner.storage_class));
        }
        let path = storage_paths
            .get(&runner.storage_class)
            .ok_or_else(|| format!("{} has no storage path", runner.storage_class))?;
        if !claimed_paths.insert(path.clone()) {
            return Err(format!("workspace path {path} is shared across scale sets"));
        }
        let volume_name = path
            .strip_prefix("/var/mnt/")
            .ok_or_else(|| format!("workspace path {path} is outside Talos user volumes"))?;
        let physical_gib = filesystems_gib
            .get(volume_name)
            .ok_or_else(|| format!("{path} has no Talos user volume"))?;
        if *physical_gib != 48 {
            return Err(format!("{volume_name} must be physically capped at 48GiB"));
        }
    }
    Ok(())
}

#[test]
fn two_scale_sets_are_structurally_bound_to_distinct_physical_filesystems() {
    let root = repo_root();
    let runners = vec![
        runner_workspace(&yaml(&root, GENERAL_VALUES)),
        runner_workspace(&yaml(&root, LIVE_POSTGRES_VALUES)),
    ];
    for runner in &runners {
        assert_eq!(runner.node, "oya-talos-worker-2");
        assert_eq!(runner.mount_path, "/home/runner/_work");
    }

    let storage_documents = yaml_documents(&root, "infra/arc/ci-workspace-storage.yaml");
    let storage_paths: BTreeMap<String, String> = storage_documents
        .iter()
        .filter(|document| is_kind(document, "StorageClass"))
        .map(|document| {
            assert_eq!(
                string_at(document, &["provisioner"]),
                "oyatie.io/ci-workspace-local-path"
            );
            (
                string_at(document, &["metadata", "name"]),
                string_at(document, &["parameters", "nodePath"]),
            )
        })
        .collect();
    let config_map = storage_documents
        .iter()
        .find(|document| is_kind(document, "ConfigMap"))
        .expect("workspace provisioner ConfigMap");
    let config: serde_json::Value =
        serde_json::from_str(&string_at(config_map, &["data", "config.json"]))
            .expect("parse local-path config.json");
    let configured_paths: BTreeSet<String> = config["nodePathMap"][0]["paths"]
        .as_array()
        .expect("nodePathMap paths")
        .iter()
        .map(|path| path.as_str().expect("node path string").to_owned())
        .collect();
    assert_eq!(
        configured_paths,
        storage_paths.values().cloned().collect(),
        "every StorageClass path must be admitted by the fail-closed provisioner map"
    );

    let talos_documents = yaml_documents(
        &root,
        "infra/talos/local/patches/ci-workspace-worker-2.yaml",
    );
    let filesystems_gib: BTreeMap<String, u64> = talos_documents
        .iter()
        .filter(|document| is_kind(document, "UserVolumeConfig"))
        .map(|document| {
            assert_eq!(
                string_at(document, &["provisioning", "diskSelector", "match"]),
                "!system_disk && disk.dev_path == '/dev/vdb' && disk.size == 150u * GiB"
            );
            let min = string_at(document, &["provisioning", "minSize"]);
            let max = string_at(document, &["provisioning", "maxSize"]);
            assert_eq!(min, max, "Talos workspace volume must have a fixed size");
            (
                string_at(document, &["name"]),
                min.strip_suffix("GiB")
                    .expect("Talos size must be GiB")
                    .parse()
                    .expect("Talos GiB size must be numeric"),
            )
        })
        .collect();

    validate_capacity_contract(&runners, &storage_paths, &filesystems_gib)
        .expect("live capacity declaration must be physically isolated");

    let provisioner = storage_documents
        .iter()
        .find(|document| is_kind(document, "Deployment"))
        .expect("workspace provisioner deployment");
    let provisioner_args = serde_json::to_string(provisioner).expect("serialize provisioner");
    assert!(provisioner_args.contains("oyatie.io/ci-workspace-local-path"));
    assert!(
        !read(&root, "infra/arc/ci-workspace-storage.yaml")
            .contains("DEFAULT_PATH_FOR_NON_LISTED_NODES")
    );
}

#[test]
fn capacity_evaluator_rejects_overcommit_shared_paths_and_missing_physical_bounds() {
    let runner = |class: &str, max_runners| RunnerWorkspace {
        max_runners,
        node: "oya-talos-worker-2".to_owned(),
        mount_path: "/home/runner/_work".to_owned(),
        storage_class: class.to_owned(),
        requested_gib: 44,
    };
    let filesystems = BTreeMap::from([
        ("ci-workspace-general".to_owned(), 48),
        ("ci-workspace-live-postgres".to_owned(), 48),
    ]);
    let distinct_paths = BTreeMap::from([
        (
            "general".to_owned(),
            "/var/mnt/ci-workspace-general".to_owned(),
        ),
        (
            "live".to_owned(),
            "/var/mnt/ci-workspace-live-postgres".to_owned(),
        ),
    ]);
    assert!(
        validate_capacity_contract(
            &[runner("general", 2), runner("live", 1)],
            &distinct_paths,
            &filesystems
        )
        .is_err()
    );

    let shared_path = BTreeMap::from([
        (
            "general".to_owned(),
            "/var/mnt/ci-workspace-general".to_owned(),
        ),
        (
            "live".to_owned(),
            "/var/mnt/ci-workspace-general".to_owned(),
        ),
    ]);
    assert!(
        validate_capacity_contract(
            &[runner("general", 1), runner("live", 1)],
            &shared_path,
            &filesystems
        )
        .is_err()
    );

    let missing_volume = BTreeMap::from([("ci-workspace-general".to_owned(), 48)]);
    assert!(
        validate_capacity_contract(
            &[runner("general", 1), runner("live", 1)],
            &distinct_paths,
            &missing_volume
        )
        .is_err()
    );
}

fn cleanup_stalled(now: u64, deletion_timestamp: Option<u64>) -> bool {
    deletion_timestamp.is_some_and(|started| now.saturating_sub(started) > 900)
}

#[test]
fn cleanup_alert_uses_deletion_delay_not_healthy_job_age() {
    let root = repo_root();
    let rules = yaml(&root, "infra/arc/ci-workspace-alerts.yaml");
    let cleanup = at(&rules, &["spec", "groups"])
        .as_sequence()
        .expect("rule groups")
        .iter()
        .flat_map(|group| at(group, &["rules"]).as_sequence().expect("rules"))
        .find(|rule| string_at(rule, &["alert"]) == "OyaCiWorkspaceCleanupStalled")
        .expect("cleanup alert");
    let expression = string_at(cleanup, &["expr"]);
    assert!(expression.contains("kube_persistentvolumeclaim_deletion_timestamp"));
    assert!(expression.contains("> 900"));
    assert!(!expression.contains("kube_persistentvolumeclaim_created"));

    assert!(!cleanup_stalled(20_000, None), "healthy long-running job");
    assert!(!cleanup_stalled(20_000, Some(19_500)), "recent deletion");
    assert!(cleanup_stalled(20_000, Some(19_000)), "stalled deletion");
}

#[test]
fn runner_network_policy_is_kubernetes_native_and_fail_closed() {
    let root = repo_root();
    let policies = yaml_documents(&root, "infra/arc/live-postgres-runner-network-policy.yaml");
    assert_eq!(policies.len(), 2);
    for policy in &policies {
        assert_eq!(string_at(policy, &["apiVersion"]), "networking.k8s.io/v1");
        assert_eq!(string_at(policy, &["kind"]), "NetworkPolicy");
    }
    let egress = policies
        .iter()
        .find(|policy| string_at(policy, &["metadata", "name"]) == "ci-runners-egress-allowlist")
        .expect("runner egress policy");
    let serialized = serde_json::to_string(egress).expect("serialize egress policy");
    assert_eq!(
        at(egress, &["spec", "policyTypes"])
            .as_sequence()
            .expect("policyTypes")
            .as_slice(),
        &[Value::String("Egress".to_owned())]
    );
    assert_eq!(
        at(egress, &["spec", "egress"])
            .as_sequence()
            .expect("egress rules")
            .len(),
        4
    );
    assert!(serialized.contains("0.0.0.0/0"));
    assert!(serialized.contains("10.0.0.0/8"));
    assert!(serialized.contains("oya-ci"));
    assert!(serialized.contains("oya-registry"));
    assert!(!serialized.contains("oya-data"));
}
