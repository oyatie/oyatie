// #1504 repository-side capacity contract. Source declarations are parsed structurally; rollout,
// CNI enforcement, and cold-concurrency evidence remain external acceptance steps.
//
// RETIRED (2026-08-11): live scale sets tip-declare maxRunners=0. Synthetic packing
// cases below still exercise the evaluator. Historical: dual-worker general used
// distributed stack with maxRunners=4; live-postgres was maxRunners=1 on worker-2.
#![allow(clippy::expect_used, clippy::panic)]

use serde::Deserialize;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const GENERAL_VALUES: &str = "infra/arc/runner-scale-set-arm64-values.yaml";
const LIVE_POSTGRES_VALUES: &str = "infra/arc/runner-scale-set-live-postgres-arm64-values.yaml";
const QEMU_CILIUM_PATCH: &str = "infra/talos/qemu-cilium.patch.yaml";
const LOCAL_PATH_STORAGE: &str = "infra/gitops/local-path-storage.yaml";
const GENERAL_WORKERS: [&str; 2] = ["oya-talos-worker-1", "oya-talos-worker-2"];
/// Tip-declared retirement: both scale sets scale-to-zero (ARC fleet retired 2026-08-11).
const MAX_GENERAL_RUNNERS_THIS_SLICE: u64 = 0;
/// Spare GiB required when stacking multiple claims on one physical volume.
const STACK_RESERVE_GIB: u64 = 4;
/// General user volume size declared in Talos patches (fits 2×44Gi + reserve).
const GENERAL_VOLUME_GIB: u64 = 120;
/// Live-postgres user volume stays single-claim sized.
const LIVE_POSTGRES_VOLUME_GIB: u64 = 48;

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

fn try_at<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().try_fold(value, |current, key| {
        current.get(Value::String((*key).to_owned()))
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

fn object<'a>(documents: &'a [Value], kind: &str, name: &str) -> &'a Value {
    documents
        .iter()
        .find(|document| {
            is_kind(document, kind) && string_at(document, &["metadata", "name"]) == name
        })
        .unwrap_or_else(|| panic!("missing {kind}/{name}"))
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

#[derive(Debug, PartialEq, Eq, Clone)]
struct RunnerWorkspace {
    max_runners: u64,
    /// Optional kubernetes.io/hostname pin. General dual-worker leaves this unset.
    hostname_pin: Option<String>,
    arch: String,
    mount_path: String,
    storage_class: String,
    requested_gib: u64,
    /// True when template.spec has required hostname podAntiAffinity or
    /// DoNotSchedule topology spread on hostname.
    spreads_across_hostnames: bool,
}

fn parse_gib(value: &str) -> u64 {
    value
        .strip_suffix("Gi")
        .unwrap_or_else(|| panic!("capacity is not expressed in Gi: {value}"))
        .parse()
        .unwrap_or_else(|error| panic!("invalid Gi capacity {value}: {error}"))
}

fn has_hostname_spread(values: &Value) -> bool {
    let spec = at(values, &["template", "spec"]);

    if let Some(required) = try_at(
        spec,
        &[
            "affinity",
            "podAntiAffinity",
            "requiredDuringSchedulingIgnoredDuringExecution",
        ],
    )
    .and_then(Value::as_sequence)
    {
        if required.iter().any(|term| {
            string_at(term, &["topologyKey"]) == "kubernetes.io/hostname"
                && try_at(term, &["labelSelector"]).is_some()
        }) {
            return true;
        }
    }

    if let Some(spreads) = try_at(spec, &["topologySpreadConstraints"]).and_then(Value::as_sequence)
    {
        if spreads.iter().any(|constraint| {
            string_at(constraint, &["topologyKey"]) == "kubernetes.io/hostname"
                && string_at(constraint, &["whenUnsatisfiable"]) == "DoNotSchedule"
                && try_at(constraint, &["labelSelector"]).is_some()
        }) {
            return true;
        }
    }

    false
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
    let hostname_pin = try_at(
        values,
        &["template", "spec", "nodeSelector", "kubernetes.io/hostname"],
    )
    .and_then(Value::as_str)
    .map(str::to_owned);

    RunnerWorkspace {
        max_runners: u64_at(values, &["maxRunners"]),
        hostname_pin,
        arch: string_at(
            values,
            &["template", "spec", "nodeSelector", "kubernetes.io/arch"],
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
        spreads_across_hostnames: has_hostname_spread(values),
    }
}

/// path -> set of nodes that admit that path in nodePathMap.
type PathNodes = BTreeMap<String, BTreeSet<String>>;
/// (node, volume_name) -> physical GiB from Talos UserVolumeConfig.
type FilesystemsGib = BTreeMap<(String, String), u64>;

fn volume_name_from_path(path: &str) -> Result<&str, String> {
    path.strip_prefix("/var/mnt/")
        .ok_or_else(|| format!("workspace path {path} is outside Talos user volumes"))
}

/// Capacity contract:
/// - maxRunners > 1 when any of:
///   (a) distinct admitting nodes ≥ maxRunners (one claim per node) + hard hostname spread
///   (b) some single volume ≥ max_runners * requested + STACK_RESERVE (full stack)
///   (c) every admitting node ≥ ceil(max/n) * requested + STACK_RESERVE + hard hostname spread
/// - scale sets must not share the same workspace path
/// - every claim is 44Gi; general volumes are GENERAL_VOLUME_GIB in this slice
fn validate_capacity_contract(
    runners: &[RunnerWorkspace],
    storage_paths: &BTreeMap<String, String>,
    path_nodes: &PathNodes,
    filesystems_gib: &FilesystemsGib,
) -> Result<(), String> {
    let mut claimed_paths = BTreeSet::new();
    for runner in runners {
        if runner.requested_gib != 44 {
            return Err(format!("{} must request 44Gi", runner.storage_class));
        }
        if runner.arch != "arm64" {
            return Err(format!(
                "{} must pin kubernetes.io/arch=arm64",
                runner.storage_class
            ));
        }

        let path = storage_paths
            .get(&runner.storage_class)
            .ok_or_else(|| format!("{} has no storage path", runner.storage_class))?;
        if !claimed_paths.insert(path.clone()) {
            return Err(format!("workspace path {path} is shared across scale sets"));
        }

        let admitting = path_nodes
            .get(path)
            .ok_or_else(|| format!("workspace path {path} is not admitted"))?;
        if admitting.is_empty() {
            return Err(format!("workspace path {path} has no admitting nodes"));
        }

        let volume_name = volume_name_from_path(path)?;

        // Every admitting node must declare a physical user volume that can hold one claim.
        for node in admitting {
            let physical_gib = filesystems_gib
                .get(&(node.clone(), volume_name.to_owned()))
                .ok_or_else(|| format!("{path} has no Talos user volume on {node}"))?;
            if *physical_gib < runner.requested_gib {
                return Err(format!(
                    "{volume_name} on {node} is {physical_gib}Gi, below the {}Gi request",
                    runner.requested_gib
                ));
            }
        }

        if let Some(pin) = &runner.hostname_pin {
            if !admitting.contains(pin) {
                return Err(format!(
                    "{} runs on {pin} but {path} is not admitted there",
                    runner.storage_class
                ));
            }
        }

        // Scale-to-zero is the retired/decommissioned state (ARC fleet tip 2026-08-11).
        // Skip packing checks; storage/path isolation still validates above.
        if runner.max_runners == 0 {
            continue;
        }

        if runner.max_runners == 1 {
            // Single runner: hostname pin optional if exactly one admitting node; required
            // when the path is multi-admitted (live-postgres pins worker-2 while general
            // also admits worker-2).
            if runner.hostname_pin.is_none() && admitting.len() > 1 {
                return Err(format!(
                    "{} maxRunners=1 with multi-node admission must pin a hostname",
                    runner.storage_class
                ));
            }
            continue;
        }

        let n_nodes = admitting.len() as u64;
        // (a) one claim per node across enough nodes
        let multi_node_ok = n_nodes >= runner.max_runners
            && admitting.iter().all(|node| {
                filesystems_gib
                    .get(&(node.clone(), volume_name.to_owned()))
                    .copied()
                    .unwrap_or(0)
                    >= runner.requested_gib
            });

        // (b) some node can hold the entire maxRunners stack
        let full_stack_ok = admitting.iter().any(|node| {
            filesystems_gib
                .get(&(node.clone(), volume_name.to_owned()))
                .copied()
                .unwrap_or(0)
                >= runner.max_runners * runner.requested_gib + STACK_RESERVE_GIB
        });

        // (c) every node holds ceil(max/n) claims + reserve (distributed stack)
        let per_node_ceiling = runner.max_runners.div_ceil(n_nodes);
        let distributed_stack_ok = n_nodes > 0
            && admitting.iter().all(|node| {
                filesystems_gib
                    .get(&(node.clone(), volume_name.to_owned()))
                    .copied()
                    .unwrap_or(0)
                    >= per_node_ceiling * runner.requested_gib + STACK_RESERVE_GIB
            });

        if !multi_node_ok && !full_stack_ok && !distributed_stack_ok {
            return Err(format!(
                "{} allows maxRunners={} but neither distinct cells (≥ maxRunners), full stack ({} * {} + {} Gi), nor distributed stack (ceil={}/node) fit for {path}",
                runner.storage_class,
                runner.max_runners,
                runner.max_runners,
                runner.requested_gib,
                STACK_RESERVE_GIB,
                per_node_ceiling
            ));
        }

        // Paths that rely on multi-node or distributed packing need hard hostname spread
        // (local-path does not enforce PVC size).
        if (multi_node_ok || distributed_stack_ok)
            && !full_stack_ok
            && !runner.spreads_across_hostnames
        {
            return Err(format!(
                "{} maxRunners={} relies on multi-node/distributed packing but lacks required hostname anti-affinity or DoNotSchedule topology spread",
                runner.storage_class, runner.max_runners
            ));
        }

        if runner.hostname_pin.is_some() {
            return Err(format!(
                "{} maxRunners>1 must not pin kubernetes.io/hostname (anti-affinity needs free placement across admitting nodes)",
                runner.storage_class
            ));
        }
    }
    Ok(())
}

#[test]
fn cas_proof_cell_has_minimal_network_and_storage_prerequisites() {
    let root = repo_root();
    let patch = yaml(&root, QEMU_CILIUM_PATCH);
    assert_eq!(patch.as_mapping().expect("QEMU patch mapping").len(), 1);
    assert_eq!(
        string_at(&patch, &["cluster", "network", "cni", "name"]),
        "none"
    );
    assert_eq!(
        at(&patch, &["cluster", "proxy", "disabled"]).as_bool(),
        Some(true)
    );

    let documents = yaml_documents(&root, LOCAL_PATH_STORAGE);
    assert_eq!(documents.len(), 9);
    let namespace = object(&documents, "Namespace", "local-path-storage");
    assert_eq!(
        string_at(
            namespace,
            &["metadata", "labels", "pod-security.kubernetes.io/enforce"]
        ),
        "privileged"
    );

    let deployment = object(&documents, "Deployment", "local-path-provisioner");
    let container = named(
        at(deployment, &["spec", "template", "spec", "containers"]),
        "local-path-provisioner",
    );
    assert_eq!(
        string_at(container, &["image"]),
        "rancher/local-path-provisioner:v0.0.37@sha256:e757967a5ec338f6a9b371c5a9688bedaa8c3578ea3dd4db329ea0084be0a86f"
    );
    assert_eq!(
        string_at(container, &["readinessProbe", "httpGet", "path"]),
        "/ready"
    );
    assert_eq!(
        at(container, &["securityContext", "runAsNonRoot"]).as_bool(),
        Some(true)
    );
    assert_eq!(
        string_at(container, &["securityContext", "seccompProfile", "type"]),
        "RuntimeDefault"
    );

    let storage_class = object(&documents, "StorageClass", "local-path");
    assert_eq!(
        string_at(storage_class, &["provisioner"]),
        "rancher.io/local-path"
    );
    assert_eq!(
        string_at(storage_class, &["volumeBindingMode"]),
        "WaitForFirstConsumer"
    );

    let config_map = object(&documents, "ConfigMap", "local-path-config");
    let config: serde_json::Value =
        serde_json::from_str(&string_at(config_map, &["data", "config.json"]))
            .expect("parse local-path config.json");
    assert_eq!(
        config["nodePathMap"],
        serde_json::json!([{
            "node": "DEFAULT_PATH_FOR_NON_LISTED_NODES",
            "paths": ["/var/mnt/local-path"]
        }])
    );
    let helper: Value = serde_yaml::from_str(&string_at(config_map, &["data", "helperPod.yaml"]))
        .expect("parse local-path helper pod");
    assert_eq!(
        string_at(
            named(at(&helper, &["spec", "containers"]), "helper-pod"),
            &["image"]
        ),
        "mirror.gcr.io/library/busybox:1.37.0@sha256:9db7b59979c38555a39def84a31fb98b5296952f9e3afd4f6f11f05b07adfab0"
    );

    let manifest = read(&root, LOCAL_PATH_STORAGE);
    assert!(!manifest.contains("/opt/local-path-provisioner"));
    assert!(
        read(&root, "infra/gitops/bootstrap-sync.yaml")
            .contains("include: 'local-path-storage.yaml'")
    );
}

#[test]
fn two_scale_sets_are_structurally_bound_to_distinct_physical_filesystems() {
    let root = repo_root();
    let general_values = yaml(&root, GENERAL_VALUES);
    let live_values = yaml(&root, LIVE_POSTGRES_VALUES);
    let runners = vec![
        runner_workspace(&general_values),
        runner_workspace(&live_values),
    ];
    let general = &runners[0];
    let live = &runners[1];

    // RETIRED: both scale sets tip-declare maxRunners=0 (scale-to-zero).
    assert_eq!(general.max_runners, MAX_GENERAL_RUNNERS_THIS_SLICE);
    assert_eq!(live.max_runners, 0);
    assert_eq!(general.storage_class, "oya-ci-workspace-general");
    assert_eq!(live.storage_class, "oya-ci-workspace-live-postgres");
    // Historical topology / hostname pins may remain in the YAML tombstone; packing
    // is skipped when maxRunners=0 (see validate_capacity_contract).
    assert!(
        general.hostname_pin.is_none(),
        "general dual-worker must not pin kubernetes.io/hostname"
    );
    assert_eq!(
        live.hostname_pin.as_deref(),
        Some("oya-talos-worker-2")
    );

    for runner in &runners {
        assert_eq!(runner.mount_path, "/home/runner/_work");
        assert_eq!(runner.arch, "arm64");
        assert_eq!(runner.requested_gib, 44);
    }
    assert!(
        general_values
            .get(Value::String("listenerTemplate".to_owned()))
            .is_none(),
        "ARC 0.14.2 rejects a metadata-only listenerTemplate"
    );

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
    assert_eq!(
        string_at(config_map, &["metadata", "name"]),
        "local-path-config",
        "local-path-provisioner discovers helperPod.yaml through this canonical ConfigMap name"
    );
    let config: serde_json::Value =
        serde_json::from_str(&string_at(config_map, &["data", "config.json"]))
            .expect("parse local-path config.json");
    let mut path_nodes: PathNodes = BTreeMap::new();
    for mapping in config["nodePathMap"]
        .as_array()
        .expect("nodePathMap")
    {
        let node = mapping["node"].as_str().expect("node string").to_owned();
        for path in mapping["paths"].as_array().expect("node paths") {
            let path = path.as_str().expect("node path string").to_owned();
            path_nodes.entry(path).or_default().insert(node.clone());
        }
    }
    assert_eq!(
        path_nodes.get("/var/mnt/ci-workspace-general"),
        Some(
            &GENERAL_WORKERS
                .iter()
                .map(|node| (*node).to_owned())
                .collect::<BTreeSet<_>>()
        ),
        "general SC path must be admitted on both workers for dual-worker concurrency"
    );
    assert_eq!(
        path_nodes.get("/var/mnt/ci-workspace-live-postgres"),
        Some(&BTreeSet::from(["oya-talos-worker-2".to_owned()])),
        "live-postgres remains worker-2 only"
    );
    assert_eq!(
        path_nodes.keys().cloned().collect::<BTreeSet<_>>(),
        storage_paths.values().cloned().collect(),
        "the provisioner map and StorageClasses must admit the same paths"
    );

    let filesystems_gib: FilesystemsGib = [
        (
            "oya-talos-worker-1",
            "infra/talos/local/patches/ci-workspace-worker-1.yaml",
        ),
        (
            "oya-talos-worker-2",
            "infra/talos/local/patches/ci-workspace-worker-2.yaml",
        ),
    ]
    .into_iter()
    .flat_map(|(node, path)| {
        yaml_documents(&root, path)
            .into_iter()
            .filter_map(move |document| {
                is_kind(&document, "UserVolumeConfig").then(|| {
                    assert_eq!(
                        string_at(&document, &["provisioning", "diskSelector", "match"]),
                        "!system_disk && disk.dev_path == '/dev/vdb' && disk.size == 150u * GiB"
                    );
                    assert_eq!(string_at(&document, &["filesystem", "type"]), "xfs");
                    let min = string_at(&document, &["provisioning", "minSize"]);
                    let max = string_at(&document, &["provisioning", "maxSize"]);
                    assert_eq!(min, max, "Talos workspace volume must have a fixed size");
                    (
                        (node.to_owned(), string_at(&document, &["name"])),
                        min.strip_suffix("GiB")
                            .expect("Talos size must be GiB")
                            .parse()
                            .expect("Talos GiB size must be numeric"),
                    )
                })
            })
    })
    .collect();
    assert_eq!(
        filesystems_gib,
        BTreeMap::from([
            (
                (
                    "oya-talos-worker-1".to_owned(),
                    "ci-workspace-general".to_owned(),
                ),
                GENERAL_VOLUME_GIB,
            ),
            (
                (
                    "oya-talos-worker-2".to_owned(),
                    "ci-workspace-general".to_owned(),
                ),
                GENERAL_VOLUME_GIB,
            ),
            (
                (
                    "oya-talos-worker-2".to_owned(),
                    "ci-workspace-live-postgres".to_owned(),
                ),
                LIVE_POSTGRES_VOLUME_GIB,
            ),
        ]),
        "general volumes are 120Gi (2×44+reserve); live-postgres stays 48Gi"
    );

    validate_capacity_contract(&runners, &storage_paths, &path_nodes, &filesystems_gib)
        .expect("live capacity declaration must be physically isolated");

    let provisioner = storage_documents
        .iter()
        .find(|document| is_kind(document, "Deployment"))
        .expect("workspace provisioner deployment");
    let config_volume = named(
        at(provisioner, &["spec", "template", "spec", "volumes"]),
        "config",
    );
    assert_eq!(
        string_at(config_volume, &["configMap", "name"]),
        "local-path-config"
    );
    let provisioner_args = serde_json::to_string(provisioner).expect("serialize provisioner");
    assert!(provisioner_args.contains("oyatie.io/ci-workspace-local-path"));
    assert!(
        !read(&root, "infra/arc/ci-workspace-storage.yaml")
            .contains("DEFAULT_PATH_FOR_NON_LISTED_NODES")
    );
}

#[test]
fn capacity_evaluator_rejects_overcommit_shared_paths_and_missing_physical_bounds() {
    let general = |max_runners, hostname_pin: Option<&str>, spreads| RunnerWorkspace {
        max_runners,
        hostname_pin: hostname_pin.map(str::to_owned),
        arch: "arm64".to_owned(),
        mount_path: "/home/runner/_work".to_owned(),
        storage_class: "general".to_owned(),
        requested_gib: 44,
        spreads_across_hostnames: spreads,
    };
    let live = |max_runners| RunnerWorkspace {
        max_runners,
        hostname_pin: Some("oya-talos-worker-2".to_owned()),
        arch: "arm64".to_owned(),
        mount_path: "/home/runner/_work".to_owned(),
        storage_class: "live".to_owned(),
        requested_gib: 44,
        spreads_across_hostnames: false,
    };

    let dual_worker_filesystems = BTreeMap::from([
        (
            (
                "oya-talos-worker-1".to_owned(),
                "ci-workspace-general".to_owned(),
            ),
            GENERAL_VOLUME_GIB,
        ),
        (
            (
                "oya-talos-worker-2".to_owned(),
                "ci-workspace-general".to_owned(),
            ),
            GENERAL_VOLUME_GIB,
        ),
        (
            (
                "oya-talos-worker-2".to_owned(),
                "ci-workspace-live-postgres".to_owned(),
            ),
            LIVE_POSTGRES_VOLUME_GIB,
        ),
    ]);
    let dual_worker_paths = BTreeMap::from([
        (
            "general".to_owned(),
            "/var/mnt/ci-workspace-general".to_owned(),
        ),
        (
            "live".to_owned(),
            "/var/mnt/ci-workspace-live-postgres".to_owned(),
        ),
    ]);
    let dual_worker_nodes = BTreeMap::from([
        (
            "/var/mnt/ci-workspace-general".to_owned(),
            BTreeSet::from([
                "oya-talos-worker-1".to_owned(),
                "oya-talos-worker-2".to_owned(),
            ]),
        ),
        (
            "/var/mnt/ci-workspace-live-postgres".to_owned(),
            BTreeSet::from(["oya-talos-worker-2".to_owned()]),
        ),
    ]);

    // Happy path: dual-worker general max=4 with distributed stack + spread + live max=1.
    assert!(
        validate_capacity_contract(
            &[general(4, None, true), live(1)],
            &dual_worker_paths,
            &dual_worker_nodes,
            &dual_worker_filesystems
        )
        .is_ok()
    );
    // Still valid at max=2 one-per-node on 120Gi volumes.
    assert!(
        validate_capacity_contract(
            &[general(2, None, true), live(1)],
            &dual_worker_paths,
            &dual_worker_nodes,
            &dual_worker_filesystems
        )
        .is_ok()
    );

    // maxRunners=2 without multi-node admission and without stackable disk fails.
    let single_general_node = BTreeMap::from([
        (
            "/var/mnt/ci-workspace-general".to_owned(),
            BTreeSet::from(["oya-talos-worker-1".to_owned()]),
        ),
        (
            "/var/mnt/ci-workspace-live-postgres".to_owned(),
            BTreeSet::from(["oya-talos-worker-2".to_owned()]),
        ),
    ]);
    let single_node_fs = BTreeMap::from([
        (
            (
                "oya-talos-worker-1".to_owned(),
                "ci-workspace-general".to_owned(),
            ),
            48,
        ),
        (
            (
                "oya-talos-worker-2".to_owned(),
                "ci-workspace-live-postgres".to_owned(),
            ),
            48,
        ),
    ]);
    assert!(
        validate_capacity_contract(
            &[general(2, None, true), live(1)],
            &dual_worker_paths,
            &single_general_node,
            &single_node_fs
        )
        .is_err(),
        "48Gi cannot host 2×44Gi claims"
    );

    // Multi-node without anti-affinity/topology spread fails (even with 120Gi).
    assert!(
        validate_capacity_contract(
            &[general(4, None, false), live(1)],
            &dual_worker_paths,
            &dual_worker_nodes,
            &dual_worker_filesystems
        )
        .is_err()
    );

    // Hostname pin with maxRunners>1 fails.
    assert!(
        validate_capacity_contract(
            &[general(4, Some("oya-talos-worker-1"), true), live(1)],
            &dual_worker_paths,
            &dual_worker_nodes,
            &dual_worker_filesystems
        )
        .is_err()
    );

    // Shared path across scale sets fails.
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
            &[general(1, Some("oya-talos-worker-1"), false), live(1)],
            &shared_path,
            &dual_worker_nodes,
            &dual_worker_filesystems
        )
        .is_err()
    );

    // Missing physical volume fails.
    let missing_volume = BTreeMap::from([(
        (
            "oya-talos-worker-1".to_owned(),
            "ci-workspace-general".to_owned(),
        ),
        48,
    )]);
    assert!(
        validate_capacity_contract(
            &[general(1, Some("oya-talos-worker-1"), false), live(1)],
            &dual_worker_paths,
            &dual_worker_nodes,
            &missing_volume
        )
        .is_err()
    );

    // Pin to a node that does not admit the path fails.
    let live_only_on_worker_2 = BTreeMap::from([
        (
            "/var/mnt/ci-workspace-general".to_owned(),
            BTreeSet::from(["oya-talos-worker-1".to_owned()]),
        ),
        (
            "/var/mnt/ci-workspace-live-postgres".to_owned(),
            BTreeSet::from(["oya-talos-worker-2".to_owned()]),
        ),
    ]);
    assert!(
        validate_capacity_contract(
            &[
                general(1, Some("oya-talos-worker-2"), false),
                live(1)
            ],
            &dual_worker_paths,
            &live_only_on_worker_2,
            &single_node_fs
        )
        .is_err()
    );

    // Full stack on a single node may admit maxRunners>1 without multi-node admission
    // or anti-affinity when the volume holds the entire stack.
    let stackable_fs = BTreeMap::from([
        (
            (
                "oya-talos-worker-1".to_owned(),
                "ci-workspace-general".to_owned(),
            ),
            96, // 2*44 + 4 reserve
        ),
        (
            (
                "oya-talos-worker-2".to_owned(),
                "ci-workspace-live-postgres".to_owned(),
            ),
            48,
        ),
    ]);
    assert!(
        validate_capacity_contract(
            &[general(2, None, false), live(1)],
            &dual_worker_paths,
            &single_general_node,
            &stackable_fs
        )
        .is_ok(),
        "full-stack path: physical_gib >= max_runners * requested + reserve"
    );

    // Distributed stack: 120Gi on each of 2 nodes admits maxRunners=4 with spread
    // (ceil(4/2)=2 → 2*44+4=92 ≤ 120) but fails without spread.
    assert!(
        validate_capacity_contract(
            &[general(4, None, true), live(1)],
            &dual_worker_paths,
            &dual_worker_nodes,
            &dual_worker_filesystems
        )
        .is_ok(),
        "distributed stack path with topology spread"
    );
    assert!(
        validate_capacity_contract(
            &[general(4, None, false), live(1)],
            &dual_worker_paths,
            &dual_worker_nodes,
            &dual_worker_filesystems
        )
        .is_err(),
        "distributed stack without hard hostname spread must fail"
    );
}

fn cleanup_stalled(now: u64, deletion_timestamp: Option<u64>) -> bool {
    deletion_timestamp.is_some_and(|started| now.saturating_sub(started) > 900)
}

#[test]
fn cleanup_alert_uses_deletion_delay_not_healthy_job_age() {
    let root = repo_root();
    let rules = yaml(&root, "infra/arc/ci-workspace-alerts.yaml");
    let alerts = at(&rules, &["spec", "groups"])
        .as_sequence()
        .expect("rule groups")
        .iter()
        .flat_map(|group| at(group, &["rules"]).as_sequence().expect("rules"))
        .collect::<Vec<_>>();
    let node_pressure = alerts
        .iter()
        .find(|rule| string_at(rule, &["alert"]) == "OyaCiWorkspaceNodeDiskPressure")
        .expect("node pressure alert");
    assert!(string_at(node_pressure, &["expr"]).contains("oya-talos-worker-(1|2)"));

    let cleanup = alerts
        .iter()
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
    assert_eq!(policies.len(), 3);
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
    assert!(!serialized.contains("oya-kms"));
    assert!(!serialized.contains("oya-data"));

    let openbao = policies
        .iter()
        .find(|policy| {
            string_at(policy, &["metadata", "name"]) == "general-ci-runner-openbao-egress"
        })
        .expect("general-cell OpenBao egress policy");
    assert_eq!(
        string_at(
            openbao,
            &["spec", "podSelector", "matchLabels", "oya.io/ci-cell"]
        ),
        "general"
    );
    let serialized = serde_json::to_string(openbao).expect("serialize OpenBao egress policy");
    assert!(serialized.contains("oya-kms") && serialized.contains("8202"));
    assert!(!serialized.contains("live-postgres"));
}

#[test]
fn cas_network_proof_is_dark_credential_free_and_narrow() {
    let root = repo_root();
    let documents = yaml_documents(&root, "infra/arc/cas-network-proof.k8s.yaml");
    assert_eq!(documents.len(), 2);
    assert_eq!(
        at(
            object(&documents, "ServiceAccount", "cas-network-proof-probe"),
            &["automountServiceAccountToken"]
        )
        .as_bool(),
        Some(false)
    );
    assert!(!documents.iter().any(|document| matches!(
        string_at(document, &["kind"]).as_str(),
        "Role" | "RoleBinding" | "ClusterRole" | "ClusterRoleBinding"
    )));

    let job = object(&documents, "Job", "cas-network-proof-probe");
    assert_eq!(at(job, &["spec", "suspend"]).as_bool(), Some(true));
    assert_eq!(u64_at(job, &["spec", "backoffLimit"]), 0);
    assert_eq!(
        string_at(job, &["spec", "template", "spec", "serviceAccountName"]),
        "cas-network-proof-probe"
    );
    assert_eq!(
        at(
            job,
            &["spec", "template", "spec", "automountServiceAccountToken"]
        )
        .as_bool(),
        Some(false)
    );

    let runner = yaml(&root, GENERAL_VALUES);
    assert_eq!(
        at(job, &["spec", "template", "metadata", "labels"]),
        at(&runner, &["template", "metadata", "labels"]),
        "the proof source must exercise the exact general ARC policy identity"
    );
    let probe = named(
        at(job, &["spec", "template", "spec", "containers"]),
        "probe",
    );
    let runner_container = named(at(&runner, &["template", "spec", "containers"]), "runner");
    assert_eq!(
        string_at(probe, &["image"]),
        string_at(runner_container, &["image"]),
        "reuse the reviewed runner image rather than adding a probe dependency"
    );
    let script = at(probe, &["args"])
        .as_sequence()
        .expect("probe args")
        .first()
        .and_then(Value::as_str)
        .expect("probe script");
    for required in [
        "http://${target}:8200/v1/sys/health",
        "https://${target}:8202/v1/sys/health",
        "plaintext_8200=connection_failed",
        "tls_8202_health=success",
        "--cacert /etc/openbao/ca/ca.crt",
    ] {
        assert!(script.contains(required), "missing proof step {required}");
    }
    assert!(!script.contains("plaintext_8200=denied"));
    let serialized_job = serde_json::to_string(job).expect("serialize proof Job");
    assert!(!serialized_job.contains("secretKeyRef"));
    assert!(!serialized_job.contains("projected"));
    assert!(!serialized_job.contains("client-key"));
    let manifest = read(&root, "infra/arc/cas-network-proof.k8s.yaml");
    assert!(!manifest.contains("cas-network-proof-verifier"));
    assert!(!manifest.contains("pods/log"));
    assert!(!read(&root, "infra/gitops/values.yaml").contains("cas-network-proof.k8s.yaml"));

    let cilium = yaml(&root, "infra/talos/cilium-values.yaml");
    assert_eq!(string_at(&cilium, &["policyEnforcementMode"]), "default");
    let runbook = read(&root, "infra/external-secrets/RUNBOOK.md");
    for required in [
        "If Flannel is live",
        "rebuild/reprovision the disposable cell",
        "conditions.ready",
        "conditions.serving",
        "conditions.terminating",
        "verdict == \"DROPPED\"",
        "drop_reason_desc == \"POLICY_DENIED\"",
        "must have both",
        "destination TCP port `8200`",
        "destination IP must equal an address from",
        "healthy OpenBao EndpointSlice validated in step 2",
        "plaintext_8200=connection_failed",
        "tls_8202_health=success",
        "no reviewed,",
        "digest-pinned Hubble CLI image",
        "adds no verifier ServiceAccount, RBAC, `pods/log`",
    ] {
        assert!(
            runbook.contains(required),
            "missing runbook invariant {required}"
        );
    }
}

#[test]
fn openbao_tls_and_github_identity_migration_is_exact_and_secret_free() {
    let root = repo_root();
    let base = read(&root, "infra/kms/openbao.k8s.yaml");
    assert!(!base.contains("8202"));
    assert!(!base.contains("openbao-server-tls"));

    let documents = yaml_documents(&root, "infra/kms/openbao-tls-migration.k8s.yaml");
    let base_documents = yaml_documents(&root, "infra/kms/openbao.k8s.yaml");
    let document_identities: Vec<(String, String)> = documents
        .iter()
        .map(|document| {
            (
                string_at(document, &["kind"]),
                string_at(document, &["metadata", "name"]),
            )
        })
        .collect();
    let identities: BTreeSet<(String, String)> = document_identities.iter().cloned().collect();
    assert_eq!(
        document_identities.len(),
        identities.len(),
        "the migration file must not duplicate a resource identity"
    );
    let mut expected_identities: BTreeSet<(String, String)> = base_documents
        .iter()
        .map(|document| {
            (
                string_at(document, &["kind"]),
                string_at(document, &["metadata", "name"]),
            )
        })
        .collect();
    assert!(expected_identities.remove(&("ConfigMap".to_owned(), "openbao-config".to_owned())));
    expected_identities.insert((
        "ConfigMap".to_owned(),
        "openbao-tls-migration-config".to_owned(),
    ));
    assert_eq!(
        identities, expected_identities,
        "the migration file must remain a complete Argo replacement"
    );
    for (kind, name) in [
        ("Namespace", "oya-kms"),
        ("PersistentVolumeClaim", "openbao-data"),
    ] {
        assert_eq!(
            object(&documents, kind, name),
            object(&base_documents, kind, name),
            "the GitOps source switch must not prune or mutate {kind}/{name}"
        );
    }
    for (kind, name) in [
        ("Deployment", "openbao"),
        ("Service", "openbao"),
        ("NetworkPolicy", "openbao-ingress"),
    ] {
        assert_eq!(
            at(object(&documents, kind, name), &["metadata"]),
            at(object(&base_documents, kind, name), &["metadata"]),
            "the migration must retain {kind}/{name} identity metadata"
        );
    }
    let deployment = documents
        .iter()
        .find(|document| is_kind(document, "Deployment"))
        .expect("OpenBao Deployment");
    let container = named(
        at(deployment, &["spec", "template", "spec", "containers"]),
        "openbao",
    );
    assert_eq!(
        string_at(container, &["image"]),
        "ghcr.io/openbao/openbao@sha256:5b2486ab0fb90bbc788cc345b0a08616dfb375873ee8be5df3a2fd4d378a67e0"
    );

    let migration = documents
        .iter()
        .find(|document| {
            is_kind(document, "ConfigMap")
                && string_at(document, &["metadata", "name"]) == "openbao-tls-migration-config"
        })
        .expect("TLS migration ConfigMap");
    let hcl = string_at(migration, &["data", "openbao.hcl"]);
    for required in [
        "0.0.0.0:8200",
        "0.0.0.0:8201",
        "0.0.0.0:8202",
        "0.0.0.0:8203",
        "tls13",
    ] {
        assert!(
            hcl.contains(required),
            "missing TLS migration declaration {required}"
        );
    }
    let deployment_text = serde_json::to_string(deployment).expect("serialize deployment");
    assert!(deployment_text.contains("openbao-server-tls"));
    assert!(deployment_text.contains("/openbao/tls"));
    assert!(deployment_text.contains("containerPort\":8202"));
    assert!(deployment_text.contains("containerPort\":8203"));

    let identity_documents = yaml_documents(&root, "infra/kms/openbao-ci-identity.k8s.yaml");
    let identity = identity_documents
        .iter()
        .find(|document| {
            is_kind(document, "ConfigMap")
                && string_at(document, &["metadata", "name"]) == "openbao-ci-identity-contract"
        })
        .expect("CI identity contract");
    let data = at(identity, &["data"]);
    for key in [
        "pki-cas-writer.json",
        "pki-cas-reader.json",
        "ci-cas-writer.hcl",
        "ci-cas-reader.hcl",
    ] {
        assert!(
            data.get(key).and_then(Value::as_str).is_some(),
            "missing {key}"
        );
    }
    assert!(
        !serde_json::to_string(identity)
            .unwrap()
            .contains("re-client")
    );
    for (role, workflow, policy) in [
        (
            "github-cas-writer-dev-push.json",
            "jason931225/oyatie/.github/workflows/oya-ci-required.yml@refs/heads/dev",
            "ci-cas-writer",
        ),
    ] {
        let payload: serde_json::Value = serde_json::from_str(
            data.get(role)
                .and_then(Value::as_str)
                .expect("role payload"),
        )
        .expect("role JSON");
        assert_eq!(payload["bound_audiences"][0], "oya-openbao");
        assert_eq!(payload["user_claim"], "workflow_ref");
        assert_eq!(payload["bound_claims"]["workflow_ref"], workflow);
        assert_eq!(
            payload["bound_claims"]["job_workflow_ref"],
            workflow
        );
        assert_eq!(payload["bound_claims"]["repository_id"], "1236575706");
        assert_eq!(payload["bound_claims"]["repository_owner_id"], "56489493");
        assert_eq!(payload["bound_claims"]["repository_visibility"], "public");
        assert_eq!(payload["bound_claims"]["runner_environment"], "self-hosted");
        assert_eq!(
            payload["bound_claims"]["sub"],
            "repo:jason931225/oyatie:ref:refs/heads/dev"
        );
        assert!(
            payload["bound_claims"]
                .get("repository_owner_type")
                .is_none()
        );
        assert_eq!(payload["token_policies"][0], policy);
        assert_eq!(payload["token_max_ttl"], "5m");
        if role == "github-cas-writer-dev-push.json" {
            assert_eq!(payload["bound_claims"]["ref"], "refs/heads/dev");
            assert_eq!(payload["bound_claims"]["event_name"], "push");
        } else {
            assert_eq!(payload["bound_claims"]["ref"], "refs/heads/dev");
            assert_eq!(payload["bound_claims"]["event_name"][0], "schedule");
            assert_eq!(
                payload["bound_claims"]["event_name"][1],
                "workflow_dispatch"
            );
        }
    }
    for role in ["pki-cas-writer.json", "pki-cas-reader.json"] {
        let payload: serde_json::Value = serde_json::from_str(
            data.get(role)
                .and_then(Value::as_str)
                .expect("PKI role payload"),
        )
        .expect("PKI role JSON");
        assert_eq!(payload["max_ttl"], "3h");
        assert_eq!(payload["ttl"], "3h");
        assert_eq!(payload["require_cn"], false);
        assert_eq!(payload["allow_ip_sans"], false);
        assert_eq!(payload["allow_localhost"], false);
        assert_eq!(payload["allowed_domains"].as_array().unwrap().len(), 0);
        assert_eq!(payload["allowed_uri_sans"].as_array().unwrap().len(), 1);
        assert_eq!(payload["client_flag"], true);
        assert_eq!(payload["server_flag"], false);
    }
    assert!(
        !serde_json::to_string(identity)
            .unwrap()
            .contains("PRIVATE KEY")
    );

    let stores = yaml_documents(
        &root,
        "infra/external-secrets/clustersecretstore-openbao-oya-tls-migration.yaml",
    );
    assert!(
        !read(
            &root,
            "infra/external-secrets/clustersecretstore-openbao-oya.yaml"
        )
        .contains("8202")
    );
    let migration_stores: Vec<&Value> = stores
        .iter()
        .filter(|store| {
            is_kind(store, "ClusterSecretStore")
                && string_at(store, &["metadata", "name"]).ends_with("tls-migration")
        })
        .collect();
    assert_eq!(migration_stores.len(), 3);
    for store in migration_stores {
        assert_eq!(
            string_at(store, &["spec", "provider", "vault", "server"]),
            "https://openbao.oya-kms.svc:8202"
        );
        assert_eq!(
            string_at(store, &["spec", "provider", "vault", "caProvider", "name"]),
            "openbao-offline-root-ca"
        );
    }

    let runner = yaml(&root, GENERAL_VALUES);
    let runner_text = serde_json::to_string(&runner).expect("serialize runner values");
    assert!(runner_text.contains("openbao-offline-root-ca"));
    assert!(runner_text.contains("/etc/openbao/ca"));
    assert!(runner_text.contains("nativelink-server-ca"));
    assert!(runner_text.contains("/etc/nativelink/ca"));
    // W2: optional mTLS client leaf secret (nativelink-client-reader) joins the two
    // optional CA ConfigMaps. Still secret-free in git — only empty optional mounts.
    assert!(runner_text.contains("nativelink-client-reader"));
    assert!(runner_text.contains("/etc/nativelink/client"));
    assert!(runner_text.contains("OYA_CACHE_TLS_CLIENT_CERT"));
    assert_eq!(
        runner_text.matches("optional\":true").count(),
        3,
        "exactly three optional mounts: openbao CA, nativelink server CA, nativelink client leaf"
    );
    assert!(
        !runner_text.contains("PRIVATE KEY") && !runner_text.contains("BEGIN CERTIFICATE"),
        "runner values must stay secret-free (no PEM material)"
    );
}
