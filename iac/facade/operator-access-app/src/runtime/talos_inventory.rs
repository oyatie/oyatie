use super::*;
use inventory::text;

pub(super) fn collect(
    p: &Profile,
    endpoint: &str,
    credentials: &[u8],
) -> Result<Value, AccessError> {
    let mut result = json!({"status": "Observed", "schema": "talos_resource_json_v1.13.8"});
    for (resource, kind) in [
        ("disks", "Disks.block.talos.dev"),
        ("volumestatus", "VolumeStatuses.block.talos.dev"),
    ] {
        let output = run(
            "talosctl",
            &strings(&[
                "--talosconfig",
                "/dev/stdin",
                "--endpoints",
                endpoint,
                "--nodes",
                &p.private_ip,
                "get",
                resource,
                "--namespace",
                "runtime",
                "--output",
                "json",
            ]),
            credentials,
            false,
        )?;
        result[resource] = project(&output, &p.private_ip, kind)?;
    }
    Ok(result)
}

fn project(bytes: &[u8], node: &str, kind: &str) -> Result<Value, AccessError> {
    let mut items = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for value in serde_json::Deserializer::from_slice(bytes).into_iter::<Value>() {
        let value = value.map_err(|_| AccessError::DependencyFailed)?;
        if text(&value, "/node")? != node
            || text(&value, "/metadata/namespace")? != "runtime"
            || text(&value, "/metadata/type")? != kind
        {
            return Err(AccessError::TargetMismatch);
        }
        let id = text(&value, "/metadata/id")?;
        if !ids.insert(id.to_string()) {
            return Err(AccessError::DependencyFailed);
        }
        let version = value
            .pointer("/metadata/version")
            .ok_or(AccessError::DependencyFailed)?;
        if version.as_u64().is_none() {
            text(&value, "/metadata/version")?;
        }
        let mut item = json!({"id": id, "version": version});
        let fields: &[&str] = match kind {
            "Disks.block.talos.dev" => {
                item["size_bytes"] = json!(
                    value["spec"]["size"]
                        .as_u64()
                        .ok_or(AccessError::DependencyFailed)?
                );
                item["readonly"] = json!(
                    value["spec"]["readonly"]
                        .as_bool()
                        .ok_or(AccessError::DependencyFailed)?
                );
                item["dev_path"] = json!(text(&value, "/spec/dev_path")?);
                if let Some(symlinks) = value.pointer("/spec/symlinks") {
                    let links = symlinks.as_array().ok_or(AccessError::DependencyFailed)?;
                    item["symlinks"] = Value::Array(
                        links
                            .iter()
                            .map(|link| Ok(json!(text(link, "")?)))
                            .collect::<Result<_, AccessError>>()?,
                    );
                }
                &["serial", "wwid", "model", "transport"]
            }
            "VolumeStatuses.block.talos.dev" => {
                item["phase"] = json!(text(&value, "/spec/phase")?);
                item["type"] = json!(text(&value, "/spec/type")?);
                if let Some(size) = value.pointer("/spec/size") {
                    item["size_bytes"] = json!(size.as_u64().ok_or(AccessError::DependencyFailed)?);
                }
                &[
                    "location",
                    "mountLocation",
                    "parentLocation",
                    "parentID",
                    "filesystem",
                ]
            }
            _ => return Err(AccessError::DependencyFailed),
        };
        for field in fields {
            let path = format!("/spec/{field}");
            if value.pointer(&path).is_some() {
                item[*field] = json!(text(&value, &path)?);
            }
        }
        items.push(item);
    }
    if items.is_empty() {
        return Err(AccessError::DependencyFailed);
    }
    Ok(Value::Array(items))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disk() -> Value {
        json!({"node": "10.0.0.227", "metadata": {"namespace": "runtime", "type": "Disks.block.talos.dev", "id": "vda", "version": 1},
            "spec": {"dev_path": "/dev/vda", "size": 100, "readonly": false, "serial": "disk-id", "symlinks": ["/dev/disk/by-id/disk-id"], "secret": "SECRET_SENTINEL"}})
    }

    #[test]
    fn disk_stream_projects_only_validated_storage_fields() {
        let bytes = serde_json::to_vec(&disk()).unwrap();
        let report = project(&bytes, "10.0.0.227", "Disks.block.talos.dev").unwrap();
        assert_eq!(report[0]["size_bytes"], 100);
        assert!(!report.to_string().contains("SECRET_SENTINEL"));
    }

    #[test]
    fn disk_stream_refuses_wrong_node_truncation_duplicates_and_empty() {
        let bytes = serde_json::to_vec(&disk()).unwrap();
        assert!(project(&bytes, "other", "Disks.block.talos.dev").is_err());
        for invalid in [
            vec![],
            bytes[..bytes.len() - 1].to_vec(),
            [bytes.clone(), bytes].concat(),
        ] {
            assert!(project(&invalid, "10.0.0.227", "Disks.block.talos.dev").is_err());
        }
    }

    #[test]
    fn directory_volume_omits_size_and_never_returns_encryption_metadata() {
        let value = json!({"node": "10.0.0.227", "metadata": {"namespace": "runtime", "type": "VolumeStatuses.block.talos.dev", "id": "ETCD", "version": "3"},
            "spec": {"phase": "ready", "type": "directory", "configuredEncryptionKeys": ["SECRET_SENTINEL"]}});
        let report = project(
            &serde_json::to_vec(&value).unwrap(),
            "10.0.0.227",
            "VolumeStatuses.block.talos.dev",
        )
        .unwrap();
        assert_eq!(report[0]["phase"], "ready");
        assert!(report[0].get("size_bytes").is_none());
        assert!(!report.to_string().contains("SECRET_SENTINEL"));
    }
}
