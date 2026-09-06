use super::*;

#[test]
fn fixed_api_paths_preserve_server_lists_without_arbitrary_raw_access() {
    assert_eq!(
        inventory::api_path("persistentvolumes"),
        Ok("/api/v1/persistentvolumes")
    );
    assert_eq!(
        inventory::api_path("deployments.apps"),
        Ok("/apis/apps/v1/deployments")
    );
    for refused in [
        "secrets",
        "/api/v1/secrets",
        "pods?watch=true",
        "pods/status",
    ] {
        assert_eq!(
            inventory::api_path(refused),
            Err(AccessError::DependencyFailed)
        );
    }
}

#[test]
fn generic_printer_lists_are_refused_but_typed_api_items_inherit_kind() {
    let pod = json!({"metadata": {"name": "cache", "namespace": "build", "uid": "u", "resourceVersion": "7"},
        "spec": {"containers": [{"name": "cache"}]}, "status": {"phase": "Running"}});
    let generic =
        json!({"kind": "List", "metadata": {"resourceVersion": ""}, "items": [pod.clone()]});
    assert!(inventory::project_list(&generic, "Pod").is_err());
    assert!(inventory::project_list(&list("Pod", vec![pod.clone()]), "Pod").is_ok());
    let mut wrong = pod;
    wrong["kind"] = json!("Secret");
    assert!(inventory::project_list(&list("Pod", vec![wrong]), "Pod").is_err());
}

fn node() -> Value {
    json!({"metadata": {"name": "seed", "resourceVersion": "1", "labels": {"kubernetes.io/hostname": "seed"}},
        "spec": {}, "status": {"nodeInfo": {"architecture": "arm64"},
        "allocatable": {"cpu": "4", "memory": "24Gi", "pods": "110"}}})
}

fn list(kind: &str, items: Vec<Value>) -> Value {
    json!({"kind": format!("{kind}List"), "metadata": {"resourceVersion": "7"}, "items": items})
}

#[test]
fn inventory_refuses_partial_malformed_and_duplicate_lists() {
    for value in [
        json!({}),
        json!({"kind": "PodList", "items": []}),
        json!({"kind": "PodList", "metadata": {"resourceVersion": "7", "continue": "next"}, "items": []}),
    ] {
        assert!(inventory::project_list(&value, "Pod").is_err());
    }
    let pod = json!({"kind": "Pod", "metadata": {"name": "cache", "namespace": "build", "uid": "u", "resourceVersion": "7"},
        "spec": {"containers": [{"name": "cache"}]}, "status": {"phase": "Running"}});
    assert!(inventory::project_list(&list("Pod", vec![pod.clone(), pod]), "Pod").is_err());
}

#[test]
fn inventory_projects_allowlisted_fields_without_workload_credentials() {
    let pod = json!({"kind": "Pod", "metadata": {"name": "cache", "namespace": "build", "uid": "u", "resourceVersion": "7",
        "annotations": {"private": "SECRET_SENTINEL"}},
        "spec": {"nodeName": "seed", "containers": [{"name": "cache", "env": [{"name": "TOKEN", "value": "SECRET_SENTINEL"}],
        "args": ["SECRET_SENTINEL"], "resources": {"requests": {"cpu": "100m", "memory": "64Mi"}}}]},
        "status": {"phase": "Running", "message": "SECRET_SENTINEL"}});
    let report = inventory::project_list(&list("Pod", vec![pod]), "Pod").unwrap();
    assert!(!report.to_string().contains("SECRET_SENTINEL"));
    assert_eq!(report["items"][0]["containers"][0]["cpu_request"], "100m");
}

#[test]
fn failed_collection_never_returns_partial_inventory() {
    let mut calls = 0;
    let result = inventory::collect_with(&node(), |_| {
        calls += 1;
        if calls == 1 {
            Ok(list("PersistentVolume", vec![]))
        } else {
            Err(AccessError::Timeout)
        }
    });
    assert_eq!(result, Err(AccessError::Timeout));
    assert_eq!(calls, 2);
}

#[test]
fn complete_kubernetes_inventory_still_blocks_unqualified_talos_storage() {
    let mut observed = Vec::new();
    let report = inventory::collect_with(&node(), |resource| {
        observed.push(resource.to_string());
        let kind = match resource {
            "persistentvolumes" => "PersistentVolume",
            "persistentvolumeclaims" => "PersistentVolumeClaim",
            "pods" => "Pod",
            "deployments.apps" => "Deployment",
            "statefulsets.apps" => "StatefulSet",
            "daemonsets.apps" => "DaemonSet",
            _ => panic!("unapproved command"),
        };
        Ok(list(kind, vec![]))
    })
    .unwrap();
    assert_eq!(observed.len(), 6);
    assert_eq!(report["deployment_readiness"], "Unknown");
    assert_eq!(report["talos_storage"]["status"], "Unknown");
    assert_eq!(report["node"]["hostname"], "seed");
}

#[test]
fn missing_node_scheduling_facts_fail_closed() {
    let result = inventory::collect_with(&json!({}), |resource| {
        let kind = match resource {
            "persistentvolumes" => "PersistentVolume",
            "persistentvolumeclaims" => "PersistentVolumeClaim",
            "pods" => "Pod",
            "deployments.apps" => "Deployment",
            "statefulsets.apps" => "StatefulSet",
            _ => "DaemonSet",
        };
        Ok(list(kind, vec![]))
    });
    assert_eq!(result, Err(AccessError::DependencyFailed));
}
