use super::*;

mod resources;

const COLLECTIONS: [(&str, &str); 6] = [
    ("persistentvolumes", "PersistentVolume"),
    ("persistentvolumeclaims", "PersistentVolumeClaim"),
    ("pods", "Pod"),
    ("deployments.apps", "Deployment"),
    ("statefulsets.apps", "StatefulSet"),
    ("daemonsets.apps", "DaemonSet"),
];

pub(super) fn collect(
    p: &Profile,
    server: &str,
    kube: &[u8],
    node: &Value,
) -> Result<Value, AccessError> {
    collect_with(node, |resource| {
        let output = run(
            "kubectl",
            &strings(&[
                "--kubeconfig",
                "/dev/stdin",
                "--server",
                server,
                "--tls-server-name",
                &p.private_ip,
                "--request-timeout=15s",
                "get",
                resource,
                "--all-namespaces",
                "--chunk-size=0",
                "-o",
                "json",
            ]),
            kube,
            false,
        )?;
        serde_json::from_slice(&output).map_err(|_| AccessError::DependencyFailed)
    })
}

pub(super) fn collect_with(
    node: &Value,
    mut fetch: impl FnMut(&str) -> Result<Value, AccessError>,
) -> Result<Value, AccessError> {
    let node = resources::node(node)?;
    let mut lists = serde_json::Map::new();
    for (resource, kind) in COLLECTIONS {
        lists.insert(resource.into(), project_list(&fetch(resource)?, kind)?);
    }
    Ok(json!({
        "schema": 1,
        "consistency": "sequential_observations_not_atomic",
        "deployment_readiness": "Unknown",
        "talos_storage": {"status": "Unknown", "reason": "resource_wire_schema_not_qualified"},
        "node": node, "collections": lists,
        "scheduling_assessment": "not_computed",
    }))
}

pub(super) fn project_list(value: &Value, kind: &str) -> Result<Value, AccessError> {
    if text(value, "/kind")? != format!("{kind}List")
        || value
            .pointer("/metadata/continue")
            .is_some_and(|v| v.as_str() != Some(""))
    {
        return Err(AccessError::DependencyFailed);
    }
    let revision = text(value, "/metadata/resourceVersion")?;
    let mut identities = std::collections::BTreeSet::new();
    let items = array(value, "/items")?
        .iter()
        .map(|item| {
            if text(item, "/kind")? != kind {
                return Err(AccessError::DependencyFailed);
            }
            let projected = resources::resource(item, kind)?;
            let identity = (
                projected["namespace"].clone().to_string(),
                projected["name"].clone().to_string(),
            );
            if !identities.insert(identity) {
                return Err(AccessError::DependencyFailed);
            }
            Ok(projected)
        })
        .collect::<Result<Vec<_>, AccessError>>()?;
    Ok(json!({"resource_version": revision, "items": items}))
}

pub(super) fn text<'a>(v: &'a Value, path: &str) -> Result<&'a str, AccessError> {
    v.pointer(path)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control))
        .ok_or(AccessError::DependencyFailed)
}

fn optional(v: &Value, path: &str) -> Result<Value, AccessError> {
    match v.pointer(path) {
        None | Some(Value::Null) => Ok(Value::Null),
        Some(Value::String(s)) if s.is_empty() => Ok(json!("")),
        _ => Ok(json!(text(v, path)?)),
    }
}

fn array<'a>(v: &'a Value, path: &str) -> Result<&'a Vec<Value>, AccessError> {
    v.pointer(path)
        .and_then(Value::as_array)
        .ok_or(AccessError::DependencyFailed)
}

fn count(v: &Value, path: &str) -> Result<Value, AccessError> {
    match v.pointer(path) {
        None | Some(Value::Null) => Ok(Value::Null),
        Some(value) if value.as_u64().is_some() => Ok(value.clone()),
        _ => Err(AccessError::DependencyFailed),
    }
}
