use super::AccessError;

#[derive(Clone, Copy, Debug)]
pub(super) enum Stage {
    Command,
    Decode,
    Projection,
}

pub(super) fn at<T>(
    resource: &'static str,
    stage: Stage,
    result: Result<T, AccessError>,
) -> Result<T, AccessError> {
    result.inspect_err(|error| {
        eprintln!(
            "operator_access_inventory_failure resource={resource} stage={stage:?} cause={error}"
        )
    })
}

pub(super) fn invalid(path: &str, expected: &'static str) -> AccessError {
    let field = field(path);
    eprintln!("operator_access_inventory_field field={field} expected={expected}");
    AccessError::DependencyFailed
}

fn field(path: &str) -> &'static str {
    match path {
        "/kind" => "kind",
        "/items" => "items",
        "/node" => "node",
        "/metadata/type" => "resource_type",
        "/metadata/id" => "resource_id",
        "/metadata/version" => "resource_version",
        "/metadata/resourceVersion" => "list_resource_version",
        "/metadata/namespace" => "namespace",
        "/metadata/name" => "name",
        "/metadata/uid" => "uid",
        "/spec/dev_path" => "device_path",
        "/spec/size" => "size",
        "/spec/readonly" => "readonly",
        "/spec/secureBoot" => "secure_boot",
        "/spec/bootedWithUKI" => "booted_with_uki",
        "/spec/moduleSignatureEnforced" => "module_signature_enforced",
        "/spec/phase" | "/status/phase" => "phase",
        "/spec/type" => "volume_type",
        "/metadata/continue" => "continuation",
        "/spec/containers" => "containers",
        "/status/allocatable/cpu" => "allocatable_cpu",
        "/status/allocatable/memory" => "allocatable_memory",
        "/status/allocatable/pods" => "allocatable_pods",
        "/metadata/labels/kubernetes.io~1hostname" => "hostname",
        "/status/nodeInfo/architecture" => "architecture",
        _ => "allowlisted_resource_attribute",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostic_field_labels_never_echo_untrusted_input() {
        assert_eq!(field("SECRET_SENTINEL"), "allowlisted_resource_attribute");
        assert_eq!(field("/kind"), "kind");
    }
}
