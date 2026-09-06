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
#[path = "talos_inventory_tests.rs"]
mod tests;
