use super::*;
use diagnostics::{Stage, at, invalid};
use inventory::text;

pub(super) fn collect(
    p: &Profile,
    endpoint: &str,
    credentials: &[u8],
) -> Result<Value, AccessError> {
    collect_resources(
        p,
        endpoint,
        credentials,
        &[
            ("disks", "Disks.block.talos.dev"),
            ("volumestatus", "VolumeStatuses.block.talos.dev"),
        ],
    )
}

pub(super) fn collect_security(
    p: &Profile,
    endpoint: &str,
    credentials: &[u8],
) -> Result<Value, AccessError> {
    collect_resources(
        p,
        endpoint,
        credentials,
        &[("securitystate", "SecurityStates.talos.dev")],
    )
}

fn collect_resources(
    p: &Profile,
    endpoint: &str,
    credentials: &[u8],
    resources: &[(&'static str, &'static str)],
) -> Result<Value, AccessError> {
    let mut result = json!({"status": "Observed", "schema": "talos_resource_json_v1.13.8"});
    for &(resource, kind) in resources {
        let output = at(
            resource,
            Stage::Command,
            run(
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
            ),
        )?;
        result[resource] = at(
            resource,
            Stage::Projection,
            project(&output, &p.private_ip, kind),
        )?;
    }
    Ok(result)
}

fn project(bytes: &[u8], node: &str, kind: &str) -> Result<Value, AccessError> {
    let mut items = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for value in serde_json::Deserializer::from_slice(bytes).into_iter::<Value>() {
        let value = value.map_err(|_| invalid("/items", "complete_json_stream"))?;
        if text(&value, "/node")? != node
            || text(&value, "/metadata/namespace")? != "runtime"
            || text(&value, "/metadata/type")? != kind
        {
            invalid("/node", "matching_node_namespace_type");
            return Err(AccessError::TargetMismatch);
        }
        let id = text(&value, "/metadata/id")?;
        if !ids.insert(id.to_string()) {
            return Err(invalid("/metadata/id", "unique_identity"));
        }
        let version = value
            .pointer("/metadata/version")
            .ok_or_else(|| invalid("/metadata/version", "version"))?;
        if version.as_u64().is_none() {
            text(&value, "/metadata/version")?;
        }
        let mut item = json!({"id": id, "version": version});
        let fields: &[&str] = match kind {
            "SecurityStates.talos.dev" => {
                if id != "securitystate" {
                    return Err(invalid("/metadata/id", "securitystate_identity"));
                }
                let security_version = version
                    .as_u64()
                    .or_else(|| {
                        version.as_str().and_then(|version| {
                            version
                                .parse::<u64>()
                                .ok()
                                .filter(|parsed| parsed.to_string() == version)
                        })
                    })
                    .ok_or_else(|| invalid("/metadata/version", "unsigned_integer"))?;
                item["version"] = json!(security_version);
                item["secureBoot"] = json!(
                    value["spec"]["secureBoot"]
                        .as_bool()
                        .ok_or_else(|| invalid("/spec/secureBoot", "boolean"))?
                );
                for field in ["bootedWithUKI", "moduleSignatureEnforced"] {
                    let path = format!("/spec/{field}");
                    item[field] = json!(match value.pointer(&path) {
                        None => false,
                        Some(value) => value.as_bool().ok_or_else(|| invalid(&path, "boolean"))?,
                    });
                }
                &[]
            }
            "Disks.block.talos.dev" => {
                item["size_bytes"] = json!(
                    value["spec"]["size"]
                        .as_u64()
                        .ok_or_else(|| invalid("/spec/size", "unsigned_integer"))?
                );
                item["readonly"] = json!(
                    value["spec"]["readonly"]
                        .as_bool()
                        .ok_or_else(|| invalid("/spec/readonly", "boolean"))?
                );
                item["dev_path"] = json!(text(&value, "/spec/dev_path")?);
                if let Some(symlinks) = value.pointer("/spec/symlinks") {
                    let links = symlinks
                        .as_array()
                        .ok_or_else(|| invalid("/spec/symlinks", "array"))?;
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
                    item["size_bytes"] = json!(
                        size.as_u64()
                            .ok_or_else(|| invalid("/spec/size", "unsigned_integer"))?
                    );
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
        return Err(invalid("/items", "nonempty_stream"));
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

    fn security() -> Value {
        json!({"node": "10.0.0.227", "metadata": {"namespace": "runtime", "type": "SecurityStates.talos.dev", "id": "securitystate", "version": 1},
            "spec": {"secureBoot": false, "secret": "SECRET_SENTINEL", "ukiSigningKeyFingerprint": "SECRET_SENTINEL", "pcrSigningKeyFingerprint": "SECRET_SENTINEL", "selinuxState": "SECRET_SENTINEL"}})
    }

    fn project_security(value: &Value) -> Result<Value, AccessError> {
        project(
            &serde_json::to_vec(value).unwrap(),
            "10.0.0.227",
            "SecurityStates.talos.dev",
        )
    }

    #[test]
    fn security_projects_only_booleans_and_qualified_omitted_false_fields() {
        let mut value = security();
        let report = project_security(&value).unwrap();
        assert_eq!(
            report[0],
            json!({"id": "securitystate", "version": 1, "secureBoot": false, "bootedWithUKI": false, "moduleSignatureEnforced": false})
        );
        assert!(!report.to_string().contains("SECRET_SENTINEL"));
        for field in ["secureBoot", "bootedWithUKI", "moduleSignatureEnforced"] {
            value["spec"][field] = json!(true);
        }
        let report = project_security(&value).unwrap();
        for field in ["secureBoot", "bootedWithUKI", "moduleSignatureEnforced"] {
            assert_eq!(report[0][field], true);
        }
    }

    #[test]
    fn security_refuses_missing_required_and_malformed_boolean_fields() {
        for path in [
            "/spec/secureBoot",
            "/metadata/version",
            "/metadata/id",
            "/node",
        ] {
            let mut value = security();
            *value.pointer_mut(path).unwrap() = Value::Null;
            assert!(project_security(&value).is_err());
        }
        let mut value = security();
        value["spec"].as_object_mut().unwrap().remove("secureBoot");
        assert!(project_security(&value).is_err());
        for field in ["secureBoot", "bootedWithUKI", "moduleSignatureEnforced"] {
            for invalid in [
                Value::Null,
                json!("SECRET_SENTINEL"),
                json!(0),
                json!([]),
                json!({}),
            ] {
                let mut value = security();
                value["spec"][field] = invalid;
                assert!(project_security(&value).is_err());
            }
        }
    }

    #[test]
    fn security_refuses_wrong_target_duplicates_empty_and_partial_streams() {
        for path in [
            "/node",
            "/metadata/namespace",
            "/metadata/type",
            "/metadata/id",
            "/metadata/version",
        ] {
            let mut value = security();
            *value.pointer_mut(path).unwrap() = json!("SECRET_SENTINEL");
            assert!(project_security(&value).is_err());
        }
        let bytes = serde_json::to_vec(&security()).unwrap();
        for invalid in [
            vec![],
            bytes[..bytes.len() - 1].to_vec(),
            [bytes.clone(), bytes].concat(),
        ] {
            assert!(project(&invalid, "10.0.0.227", "SecurityStates.talos.dev").is_err());
        }
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
