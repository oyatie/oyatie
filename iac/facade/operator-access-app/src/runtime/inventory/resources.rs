use super::*;

pub(super) fn node(v: &Value) -> Result<Value, AccessError> {
    let taints = match v.pointer("/spec/taints") {
        None => vec![],
        Some(_) => array(v, "/spec/taints")?.iter().map(|t| Ok(json!({
            "key": text(t, "/key")?, "value": optional(t, "/value")?, "effect": text(t, "/effect")?,
        }))).collect::<Result<Vec<_>, AccessError>>()?,
    };
    let unschedulable = match v.pointer("/spec/unschedulable") {
        None => false,
        Some(value) => value.as_bool().ok_or(AccessError::DependencyFailed)?,
    };
    Ok(json!({"name": text(v, "/metadata/name")?,
        "resource_version": text(v, "/metadata/resourceVersion")?,
        "hostname": text(v, "/metadata/labels/kubernetes.io~1hostname")?,
        "architecture": text(v, "/status/nodeInfo/architecture")?,
        "unschedulable": unschedulable, "taints": taints,
        "allocatable": {"cpu": text(v, "/status/allocatable/cpu")?,
            "memory": text(v, "/status/allocatable/memory")?,
            "pods": text(v, "/status/allocatable/pods")?,
            "ephemeral_storage": optional(v, "/status/allocatable/ephemeral-storage")?}}))
}

pub(super) fn resource(v: &Value, kind: &str) -> Result<Value, AccessError> {
    let mut out = json!({"name": text(v, "/metadata/name")?,
        "uid": text(v, "/metadata/uid")?, "resource_version": text(v, "/metadata/resourceVersion")?});
    if kind != "PersistentVolume" {
        out["namespace"] = json!(text(v, "/metadata/namespace")?);
    }
    match kind {
        "PersistentVolume" => {
            out["phase"] = json!(text(v, "/status/phase")?);
            out["capacity"] = json!(text(v, "/spec/capacity/storage")?);
            for (name, path) in [
                ("storage_class", "/spec/storageClassName"),
                ("reclaim_policy", "/spec/persistentVolumeReclaimPolicy"),
                ("volume_mode", "/spec/volumeMode"),
                ("claim_namespace", "/spec/claimRef/namespace"),
                ("claim_name", "/spec/claimRef/name"),
                ("local_path", "/spec/local/path"),
                ("host_path", "/spec/hostPath/path"),
                ("csi_driver", "/spec/csi/driver"),
            ] {
                out[name] = optional(v, path)?;
            }
        }
        "PersistentVolumeClaim" => {
            out["phase"] = json!(text(v, "/status/phase")?);
            for (name, path) in [
                ("storage_class", "/spec/storageClassName"),
                ("volume", "/spec/volumeName"),
                ("requested_storage", "/spec/resources/requests/storage"),
                ("capacity", "/status/capacity/storage"),
            ] {
                out[name] = optional(v, path)?;
            }
        }
        "Pod" => {
            out["node"] = optional(v, "/spec/nodeName")?;
            out["phase"] = json!(text(v, "/status/phase")?);
            out["containers"] = Value::Array(
                array(v, "/spec/containers")?
                    .iter()
                    .map(|c| {
                        Ok(json!({
                            "name": text(c, "/name")?,
                            "cpu_request": optional(c, "/resources/requests/cpu")?,
                            "memory_request": optional(c, "/resources/requests/memory")?,
                            "cpu_limit": optional(c, "/resources/limits/cpu")?,
                            "memory_limit": optional(c, "/resources/limits/memory")?,
                        }))
                    })
                    .collect::<Result<_, AccessError>>()?,
            );
        }
        "Deployment" | "StatefulSet" | "DaemonSet" => {
            out["desired_replicas"] = count(
                v,
                if kind == "DaemonSet" {
                    "/status/desiredNumberScheduled"
                } else {
                    "/spec/replicas"
                },
            )?;
            out["ready_replicas"] = count(
                v,
                if kind == "DaemonSet" {
                    "/status/numberReady"
                } else {
                    "/status/readyReplicas"
                },
            )?;
        }
        _ => return Err(AccessError::DependencyFailed),
    }
    Ok(out)
}
