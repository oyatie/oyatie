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
