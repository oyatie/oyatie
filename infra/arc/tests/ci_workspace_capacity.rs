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
    node: Option<String>,
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
        node: at(values, &["template", "spec", "nodeSelector"])
            .get("kubernetes.io/hostname")
            .and_then(Value::as_str)
            .map(str::to_owned),
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
        if runner.max_runners == 0 {
            return Err(format!(
                "{} has no admitted runner capacity",
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
        if runner.requested_gib * runner.max_runners > *physical_gib {
            return Err(format!(
                "{volume_name} cannot hold {} x {}GiB claims inside {physical_gib}GiB",
                runner.max_runners, runner.requested_gib
            ));
        }
    }
    Ok(())
}

#[test]
fn runner_cells_are_capacity_bounded_without_rootfs_fallback() {
    let root = repo_root();
    let runners = vec![
        runner_workspace(&yaml(&root, GENERAL_VALUES)),
        runner_workspace(&yaml(&root, LIVE_POSTGRES_VALUES)),
    ];
    for runner in &runners {
        assert_eq!(runner.mount_path, "/home/runner/_work");
    }
    assert_eq!(runners[0].max_runners, 1);
    assert_eq!(
        runners[0].node, None,
        "general set must span admitted ARM nodes"
    );
    assert_eq!(
        string_at(
            &yaml(&root, GENERAL_VALUES),
            &["template", "spec", "nodeSelector", "oya.io/ci-workspace"]
        ),
        "general"
    );
    assert_eq!(runners[1].max_runners, 1);
    assert_eq!(
        string_at(
            &yaml(&root, LIVE_POSTGRES_VALUES),
            &["template", "spec", "nodeSelector", "oya.io/ci-capacity"]
        ),
        "pg"
    );

    let general_values = yaml(&root, GENERAL_VALUES);
    let affinity = serde_json::to_string(at(
        &general_values,
        &[
            "template",
            "spec",
            "affinity",
            "podAntiAffinity",
            "requiredDuringSchedulingIgnoredDuringExecution",
        ],
    ))
    .expect("serialize required general-runner anti-affinity");
    assert!(affinity.contains("kubernetes.io/hostname"));
    assert!(affinity.contains("oya.io/ci-cell"));
    assert!(affinity.contains("general"));

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
    let configured_paths: BTreeSet<String> = config["nodePathMap"]
        .as_array()
        .expect("nodePathMap")
        .iter()
        .flat_map(|node| node["paths"].as_array().expect("nodePathMap paths"))
        .map(|path| path.as_str().expect("node path string").to_owned())
        .collect();
    assert_eq!(
        configured_paths,
        storage_paths.values().cloned().collect(),
        "every StorageClass path must be admitted by the fail-closed provisioner map"
    );

    let talos_documents: Vec<Value> = [
        "infra/talos/local/patches/ci-workspace-worker-1.yaml",
        "infra/talos/local/patches/ci-workspace-worker-2.yaml",
    ]
    .into_iter()
    .flat_map(|path| yaml_documents(&root, path))
    .collect();
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
                    .parse::<u64>()
                    .expect("Talos GiB size must be numeric"),
            )
        })
        .fold(BTreeMap::new(), |mut totals, (name, size)| {
            *totals.entry(name).or_default() += size;
            totals
        });

    validate_capacity_contract(&runners, &storage_paths, &filesystems_gib)
        .expect("live capacity declaration must be physically isolated");

    let provisioner = storage_documents
        .iter()
        .find(|document| is_kind(document, "Deployment"))
        .expect("workspace provisioner deployment");
    let provisioner_args = serde_json::to_string(provisioner).expect("serialize provisioner");
    assert!(provisioner_args.contains("oyatie.io/ci-workspace-local-path"));
    let storage_text = read(&root, "infra/arc/ci-workspace-storage.yaml");
    assert!(!storage_text.contains("DEFAULT_PATH_FOR_NON_LISTED_NODES"));
    assert_eq!(
        storage_text.matches("/dev/vdb*").count(),
        2,
        "setup and teardown must refuse Talos rootfs fallback"
    );
    assert!(storage_text.contains("refusing CI workspace outside"));
}

#[test]
fn capacity_evaluator_rejects_overcommit_shared_paths_and_missing_physical_bounds() {
    let runner = |class: &str, max_runners| RunnerWorkspace {
        max_runners,
        node: Some("oya-talos-worker-2".to_owned()),
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
            &[runner("general", 3), runner("live", 1)],
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
        5
    );
    assert!(serialized.contains("0.0.0.0/0"));
    assert!(serialized.contains("10.0.0.0/8"));
    assert!(serialized.contains("oya-ci"));
    assert!(serialized.contains("oya-registry"));
    assert!(serialized.contains("oya-kms"));
    assert!(!serialized.contains("oya-data"));
}

#[test]
fn openbao_tls_and_github_identity_migration_is_exact_and_secret_free() {
    let root = repo_root();
    let base = read(&root, "infra/kms/openbao.k8s.yaml");
    assert!(!base.contains("8202"));
    assert!(!base.contains("openbao-server-tls"));

    let documents = yaml_documents(&root, "infra/kms/openbao-tls-migration.k8s.yaml");
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
        (
            "github-cas-reader-integrity-canary.json",
            "jason931225/oyatie/.github/workflows/cache-integrity-canary.yml@refs/heads/dev",
            "ci-cas-reader",
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
        assert_eq!(payload["bound_claims"]["repository_id"], "1236575706");
        assert_eq!(payload["bound_claims"]["repository_owner_id"], "56489493");
        assert_eq!(payload["bound_claims"]["repository_visibility"], "private");
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
    assert_eq!(runner_text.matches("optional\":true").count(), 2);
    assert!(!runner_text.contains("tls.key"));

    let public_ca = yaml_documents(&root, "infra/kms/openbao-public-ca.k8s.yaml");
    let openbao_namespaces: BTreeSet<String> = public_ca
        .iter()
        .filter(|config_map| {
            string_at(config_map, &["metadata", "name"]) == "openbao-offline-root-ca"
        })
        .map(|config_map| {
            assert!(is_kind(config_map, "ConfigMap"));
            assert_eq!(string_at(config_map, &["data", "ca.crt"]), "");
            string_at(config_map, &["metadata", "namespace"])
        })
        .collect();
    assert_eq!(
        openbao_namespaces,
        BTreeSet::from(["arc-runners".to_owned(), "external-secrets".to_owned()])
    );
    let nativelink_ca = public_ca
        .iter()
        .find(|config_map| string_at(config_map, &["metadata", "name"]) == "nativelink-server-ca")
        .expect("NativeLink server CA ConfigMap");
    assert_eq!(
        string_at(nativelink_ca, &["metadata", "namespace"]),
        "arc-runners"
    );

    let identity_text = serde_json::to_string(identity).unwrap();
    assert!(identity_text.contains("pki_cas_writer/issue/cas-writer"));
    assert!(identity_text.contains("pki_cas_reader/issue/cas-reader"));
    assert!(!identity_text.contains("pki_int/issue"));
    let nativelink = read(&root, "infra/nativelink/nativelink-cas.k8s.yaml");
    assert_eq!(nativelink.matches("/tls/ca-writer.crt").count(), 1);
    assert_eq!(nativelink.matches("/tls/ca-reader.crt").count(), 1);
    assert!(nativelink.contains(r#""socket_address": "0.0.0.0:50051""#));
    assert!(nativelink.contains(r#""socket_address": "0.0.0.0:50052""#));

    let runbook = read(&root, "infra/external-secrets/RUNBOOK.md");
    assert!(runbook.contains("There is no bootstrap controller in this slice"));
    assert!(runbook.contains("reader leaf against `:50051` fails"));
    assert!(runbook.contains("OYA_NATIVELINK_SERVER_CA_CERT"));
    assert!(runbook.contains("Do not apply the empty public-CA scaffold directly"));
    assert!(!runbook.contains("kubectl apply -f infra/kms/openbao-public-ca.k8s.yaml"));
    assert!(!runbook.contains("authenticated bootstrap controller"));
}
