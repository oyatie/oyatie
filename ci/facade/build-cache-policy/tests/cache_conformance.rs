// ADR-0560 cache-wiring conformance gate: live-corpus self-test over the REAL
// policy + license + overlays + canary workflow (the slice-1 instalment of the
// ADR-0556-named cache-policy-conformance successor; asserting the FULL live CI
// cache configuration against the policy remains that gate's scope).
//
// Proves mechanically, on every PR:
//   1. the dark-wiring guarantee — while specs/cache-warm-license.json is
//      unlicensed, EVERY class resolves bypass (today's builds are untouched);
//   2. the cold-required floor — the four ADR-0556 one-way cold classes resolve
//      bypass even under a licensed fixture (pinned here as a ratchet: dropping
//      one from the policy DATA goes RED and requires superseding ADR-0556);
//   3. the kill-switch works — flipping the license fixture flips warm classes
//      between bypass and their classified modes, and never the cold ones;
//   4. the overlays parse, select the cache execution platform, set the posture
//      their name claims, and carry NO keyed identity material;
//   5. the root .buckconfig stays clean of any RE/cache section;
//   6. the canary workflow exists, is scheduled, restores no actions/cache, and
//      wires the cold proof (assert-cold) + structured record.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::{
    collections::{BTreeSet, HashSet},
    path::{Component, Path, PathBuf},
};

use ci_build_cache_policy as app;
use serde::Deserialize;
use serde_json::{Value, json};
use serde_yaml::Value as YamlValue;
use sha2::{Digest, Sha256};

const CANARY_WORKFLOW_PATH: &str = ".github/workflows/cache-integrity-canary.yml";
const CANARY_SCHEDULE_WORKFLOW_PATH: &str = ".github/workflows/cache-integrity-canary-schedule.yml";
const REQUIRED_WORKFLOW_PATH: &str = ".github/workflows/oya-ci-required.yml";
const NATIVELINK_MANIFEST_PATH: &str = "storage/adapters/nativelink/nativelink-cas.k8s.yaml";
const EXTERNAL_SECRETS_RUNBOOK_PATH: &str = "infra/external-secrets/RUNBOOK.md";
const EXTERNAL_SECRET_STORE_PATH: &str =
    "infra/external-secrets/clustersecretstore-openbao-oya.yaml";
const RUNNER_NETWORK_POLICY_PATH: &str = "infra/arc/live-postgres-runner-network-policy.yaml";
const RUNNER_VALUES_PATH: &str = "infra/arc/runner-scale-set-arm64-values.yaml";
const COLD_REQUIRED_FLOOR: [&str; 4] = [
    "release-production-image",
    "integrity-canary",
    "untrusted-author-presubmit",
    "provenance-attestation",
];

fn repo_root() -> PathBuf {
    let cwd = std::env::current_dir().expect("current_dir");
    app::repo_root_from(&cwd).expect("failed to locate repo root from test current_dir")
}

fn kubernetes_tcp_port(entry: &YamlValue, context: &str) -> Result<u64, String> {
    if entry.get("endPort").is_some() {
        return Err(format!(
            "{context} must not widen a cache port with endPort"
        ));
    }
    if let Some(protocol) = entry.get("protocol") {
        match protocol.as_str() {
            Some("TCP") => {}
            Some(other) => {
                return Err(format!("{context} protocol `{other}` is not TCP"));
            }
            None => {
                return Err(format!("{context} protocol is not a string"));
            }
        }
    }
    entry["port"]
        .as_u64()
        .ok_or_else(|| format!("{context} port is missing or non-numeric"))
}

fn validated_service_port(service: &YamlValue, role: &str) -> Result<u64, String> {
    let entry = service["spec"]["ports"]
        .as_sequence()
        .and_then(|ports| ports.first())
        .ok_or_else(|| format!("{role} Service has no ports"))?;
    let port = kubernetes_tcp_port(entry, &format!("{role} Service"))?;
    let target_port = entry["targetPort"]
        .as_u64()
        .ok_or_else(|| format!("{role} Service targetPort is missing or non-numeric"))?;
    if target_port != port {
        return Err(format!(
            "{role} Service targetPort {target_port} disagrees with port {port}"
        ));
    }
    Ok(port)
}

fn validate_service_exposure(service: &YamlValue, role: &str) -> Result<(), String> {
    if !matches!(service["spec"]["type"].as_str(), None | Some("ClusterIP")) {
        return Err(format!("{role} Service must remain cluster-internal"));
    }
    for field in [
        "externalIPs",
        "externalName",
        "externalTrafficPolicy",
        "loadBalancerClass",
        "loadBalancerIP",
    ] {
        if service["spec"].get(field).is_some() {
            return Err(format!(
                "{role} Service must not declare external field `{field}`"
            ));
        }
    }
    let ports = service["spec"]["ports"]
        .as_sequence()
        .ok_or_else(|| format!("{role} Service has no ports"))?;
    if ports.iter().any(|port| port.get("nodePort").is_some()) {
        return Err(format!("{role} Service must not declare a nodePort"));
    }
    Ok(())
}

fn metadata_namespace<'a>(document: &'a YamlValue, context: &str) -> Result<&'a str, String> {
    document["metadata"]["namespace"]
        .as_str()
        .ok_or_else(|| format!("{context} metadata.namespace is missing or non-string"))
}

fn validate_unique_kubernetes_identities(documents: &[YamlValue]) -> Result<(), String> {
    let mut identities = BTreeSet::new();
    for document in documents {
        let kind = document["kind"]
            .as_str()
            .ok_or_else(|| "Kubernetes document kind is missing or non-string".to_string())?;
        let name = document["metadata"]["name"]
            .as_str()
            .ok_or_else(|| format!("Kubernetes {kind} metadata.name is missing or non-string"))?;
        let namespace = document["metadata"]["namespace"].as_str().unwrap_or("");
        let identity = (kind.to_string(), namespace.to_string(), name.to_string());
        if !identities.insert(identity.clone()) {
            return Err(format!(
                "duplicate Kubernetes identity `{}/{}/{}`",
                identity.0, identity.1, identity.2
            ));
        }
    }
    Ok(())
}

fn common_service_namespace(services: &[(&str, &YamlValue)]) -> Result<String, String> {
    let mut expected = None;
    for (role, service) in services {
        let namespace = metadata_namespace(service, &format!("{role} Service"))?;
        match expected.as_deref() {
            None => expected = Some(namespace.to_owned()),
            Some(previous) if previous == namespace => {}
            Some(previous) => {
                return Err(format!(
                    "{role} Service namespace `{namespace}` disagrees with `{previous}`"
                ));
            }
        }
    }
    expected.ok_or_else(|| "no cache Services were supplied".to_string())
}

fn validate_service_selector(
    service: &YamlValue,
    deployment: &YamlValue,
    role: &str,
) -> Result<(), String> {
    let selector = service["spec"]["selector"]
        .as_mapping()
        .ok_or_else(|| format!("{role} Service selector is missing or non-mapping"))?;
    if selector.is_empty() {
        return Err(format!("{role} Service selector is empty"));
    }
    let deployment_selector = deployment["spec"]["selector"]["matchLabels"]
        .as_mapping()
        .ok_or_else(|| {
            "NativeLink Deployment selector.matchLabels is missing or non-mapping".to_string()
        })?;
    if selector != deployment_selector {
        return Err(format!(
            "{role} Service selector {selector:?} disagrees with Deployment selector \
             {deployment_selector:?}"
        ));
    }
    Ok(())
}

fn validate_workload_selectors(
    deployment: &YamlValue,
    ingress_policy: &YamlValue,
) -> Result<(), String> {
    let deployment_selector_spec = deployment["spec"]["selector"]
        .as_mapping()
        .ok_or_else(|| "NativeLink Deployment selector is missing or non-mapping".to_string())?;
    if deployment_selector_spec.len() != 1 {
        return Err("NativeLink Deployment selector must use only exact matchLabels".to_string());
    }
    let deployment_selector = deployment["spec"]["selector"]["matchLabels"]
        .as_mapping()
        .ok_or_else(|| {
            "NativeLink Deployment selector.matchLabels is missing or non-mapping".to_string()
        })?;
    if deployment_selector.is_empty() {
        return Err("NativeLink Deployment selector.matchLabels is empty".to_string());
    }
    let pod_labels = deployment["spec"]["template"]["metadata"]["labels"]
        .as_mapping()
        .ok_or_else(|| "NativeLink Deployment pod labels are missing or non-mapping".to_string())?;
    for (key, value) in deployment_selector {
        if pod_labels.get(key) != Some(value) {
            return Err(format!(
                "NativeLink Deployment selector {key:?}={value:?} does not match pod labels"
            ));
        }
    }
    let ingress_selector_spec = ingress_policy["spec"]["podSelector"]
        .as_mapping()
        .ok_or_else(|| {
            "NativeLink ingress policy podSelector is missing or non-mapping".to_string()
        })?;
    if ingress_selector_spec.len() != 1 {
        return Err(
            "NativeLink ingress policy podSelector must use only exact matchLabels".to_string(),
        );
    }
    let ingress_selector = ingress_policy["spec"]["podSelector"]["matchLabels"]
        .as_mapping()
        .ok_or_else(|| {
            "NativeLink ingress policy podSelector.matchLabels is missing or non-mapping"
                .to_string()
        })?;
    if ingress_selector != deployment_selector {
        return Err(format!(
            "NativeLink ingress target selector {ingress_selector:?} disagrees with Deployment \
             selector {deployment_selector:?}"
        ));
    }
    Ok(())
}

fn validate_deployment_topology(deployment: &YamlValue) -> Result<(), String> {
    if deployment["spec"]["replicas"].as_u64() != Some(1) {
        return Err("NativeLink Deployment must run exactly one replica".to_string());
    }
    if deployment["spec"]["strategy"]["type"].as_str() != Some("Recreate") {
        return Err("NativeLink Deployment strategy must be `Recreate`".to_string());
    }
    Ok(())
}

fn validate_ops_health_binding(ops: &Value, deployment: &YamlValue) -> Result<(), String> {
    if ops["listener"]["http"]["socket_address"].as_str() != Some("0.0.0.0:50061") {
        return Err("NativeLink ops listener must bind `0.0.0.0:50061`".to_string());
    }
    let container = deployment["spec"]["template"]["spec"]["containers"]
        .as_sequence()
        .and_then(|containers| {
            containers
                .iter()
                .find(|container| container["name"].as_str() == Some("nativelink"))
        })
        .ok_or_else(|| "NativeLink Deployment has no `nativelink` container".to_string())?;
    let ops_port = container["ports"]
        .as_sequence()
        .and_then(|ports| {
            ports
                .iter()
                .find(|port| port["name"].as_str() == Some("ops"))
        })
        .ok_or_else(|| "NativeLink container has no named `ops` port".to_string())?;
    if ops_port["containerPort"].as_u64() != Some(50061)
        || !matches!(
            ops_port.get("protocol").and_then(YamlValue::as_str),
            None | Some("TCP")
        )
    {
        return Err("NativeLink container `ops` port must be TCP 50061".to_string());
    }
    for probe in ["readinessProbe", "livenessProbe"] {
        let http_get = &container[probe]["httpGet"];
        if http_get["port"].as_str() != Some("ops") || http_get["path"].as_str() != Some("/status")
        {
            return Err(format!(
                "NativeLink {probe} must request `/status` on named port `ops`"
            ));
        }
    }
    Ok(())
}

fn validate_external_secret_store(
    external_secret: &YamlValue,
    cluster_store: &YamlValue,
) -> Result<(), String> {
    if cluster_store["kind"].as_str() != Some("ClusterSecretStore") {
        return Err("OpenBao store document is not a ClusterSecretStore".to_string());
    }
    let store_name = cluster_store["metadata"]["name"]
        .as_str()
        .ok_or_else(|| "OpenBao ClusterSecretStore name is missing or non-string".to_string())?;
    if external_secret["spec"]["secretStoreRef"]["kind"].as_str() != Some("ClusterSecretStore")
        || external_secret["spec"]["secretStoreRef"]["name"].as_str() != Some(store_name)
    {
        return Err(format!(
            "NativeLink ExternalSecret must reference ClusterSecretStore `{store_name}`"
        ));
    }
    Ok(())
}

fn validate_network_policy_direction(
    policy: &YamlValue,
    expected: &str,
    context: &str,
) -> Result<(), String> {
    let directions = policy["spec"]["policyTypes"]
        .as_sequence()
        .ok_or_else(|| format!("{context} policyTypes are missing or non-sequence"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| format!("{context} policyType is non-string"))
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if directions != BTreeSet::from([expected]) {
        return Err(format!(
            "{context} policyTypes {directions:?} must be exactly `{expected}`"
        ));
    }
    Ok(())
}

fn validate_runner_role_labels(runner_values: &YamlValue, roles: &[&str]) -> Result<(), String> {
    let labels = runner_values["template"]["metadata"]["labels"]
        .as_mapping()
        .ok_or_else(|| "runner template labels are missing or non-mapping".to_string())?;
    if labels
        .get(YamlValue::String("oya.io/ci-cell".to_string()))
        .and_then(YamlValue::as_str)
        != Some("general")
    {
        return Err("runner template must identify the `general` CI cell".to_string());
    }
    for role in roles {
        let label = format!("oya.io/nativelink-cas-{role}");
        if labels
            .get(YamlValue::String(label.clone()))
            .and_then(YamlValue::as_str)
            != Some("true")
        {
            return Err(format!(
                "runner template does not carry required cache role label `{label}`"
            ));
        }
    }
    Ok(())
}

fn validate_listener_tls(server: &Value, role: &str) -> Result<(), String> {
    let expected_client_ca = match role {
        "writer" => "/tls/ca-writer.crt",
        "reader" => "/tls/ca-reader.crt",
        _ => return Err(format!("unknown cache role `{role}`")),
    };
    let tls = server["listener"]["http"]["tls"]
        .as_object()
        .ok_or_else(|| format!("{role} listener TLS is missing or non-object"))?;
    for (field, expected) in [
        ("cert_file", "/tls/tls.crt"),
        ("key_file", "/tls/tls.key"),
        ("client_ca_file", expected_client_ca),
    ] {
        if tls.get(field).and_then(Value::as_str) != Some(expected) {
            return Err(format!(
                "{role} listener TLS `{field}` must be `{expected}`"
            ));
        }
    }
    Ok(())
}

fn validate_server_names(servers: &[Value]) -> Result<(), String> {
    let names = servers
        .iter()
        .map(|server| {
            server["name"]
                .as_str()
                .ok_or_else(|| "NativeLink server name is missing or non-string".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let unique = names.iter().copied().collect::<BTreeSet<_>>();
    let expected = BTreeSet::from(["ops", "reader", "writer"]);
    if names.len() != expected.len() || unique.len() != names.len() || unique != expected {
        return Err(format!(
            "NativeLink servers {names:?} must be exactly one each of {expected:?}"
        ));
    }
    Ok(())
}

fn validate_cache_store_bindings(server: &Value, role: &str) -> Result<(), String> {
    for (service, field, expected) in [
        ("cas", "cas_store", "CAS_MAIN_STORE"),
        ("bytestream", "cas_store", "CAS_MAIN_STORE"),
        ("ac", "ac_store", "AC_MAIN_STORE"),
    ] {
        let instances = server["services"][service]
            .as_array()
            .ok_or_else(|| format!("{role} {service} instances are missing or non-array"))?;
        if instances.is_empty() {
            return Err(format!("{role} {service} instances are empty"));
        }
        for instance in instances {
            if instance[field].as_str() != Some(expected) {
                return Err(format!(
                    "{role} {service} `{field}` must reference `{expected}`"
                ));
            }
        }
    }
    Ok(())
}

fn validate_deployment_runtime_binding(
    external_secret: &YamlValue,
    deployment: &YamlValue,
    service_namespace: &str,
    role_ports: &[(&str, u64)],
) -> Result<(), String> {
    let external_secret_namespace =
        metadata_namespace(external_secret, "NativeLink ExternalSecret")?;
    let deployment_namespace = metadata_namespace(deployment, "NativeLink Deployment")?;
    if external_secret_namespace != deployment_namespace {
        return Err(format!(
            "NativeLink ExternalSecret namespace `{external_secret_namespace}` disagrees with \
             Deployment namespace `{deployment_namespace}`"
        ));
    }
    if deployment_namespace != service_namespace {
        return Err(format!(
            "NativeLink Deployment namespace `{deployment_namespace}` disagrees with Service \
             namespace `{service_namespace}`"
        ));
    }
    let secret_name = external_secret["spec"]["target"]["name"]
        .as_str()
        .ok_or_else(|| {
            "NativeLink ExternalSecret target name is missing or non-string".to_string()
        })?;
    if external_secret["spec"]["target"]["creationPolicy"].as_str() != Some("Owner") {
        return Err("NativeLink ExternalSecret target creationPolicy must be `Owner`".to_string());
    }
    let external_data = external_secret["spec"]["data"]
        .as_sequence()
        .ok_or_else(|| "NativeLink ExternalSecret has no data mappings".to_string())?;
    let secret_keys = external_data
        .iter()
        .map(|entry| {
            entry["secretKey"].as_str().ok_or_else(|| {
                "NativeLink ExternalSecret secretKey is missing or non-string".to_string()
            })
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let required_secret_keys =
        BTreeSet::from(["tls.crt", "tls.key", "ca-writer.crt", "ca-reader.crt"]);
    if external_data.len() != required_secret_keys.len() || secret_keys != required_secret_keys {
        return Err(format!(
            "NativeLink ExternalSecret must have exactly one mapping for every TLS file \
             {required_secret_keys:?}; observed {secret_keys:?}"
        ));
    }
    for (secret_key, property) in [
        ("tls.crt", "server-cert"),
        ("tls.key", "server-key"),
        ("ca-writer.crt", "writer-client-ca"),
        ("ca-reader.crt", "reader-client-ca"),
    ] {
        let mapping = external_data
            .iter()
            .find(|entry| entry["secretKey"].as_str() == Some(secret_key))
            .ok_or_else(|| format!("NativeLink ExternalSecret has no `{secret_key}` mapping"))?;
        if mapping["remoteRef"]["key"].as_str() != Some("oya/ci/nativelink-cas-tls")
            || mapping["remoteRef"]["property"].as_str() != Some(property)
        {
            return Err(format!(
                "NativeLink ExternalSecret `{secret_key}` must map \
                 `oya/ci/nativelink-cas-tls` property `{property}`"
            ));
        }
    }
    let volumes = deployment["spec"]["template"]["spec"]["volumes"]
        .as_sequence()
        .ok_or_else(|| "NativeLink Deployment has no volumes".to_string())?;
    let tls_volume = volumes
        .iter()
        .find(|volume| volume["name"].as_str() == Some("tls"))
        .ok_or_else(|| "NativeLink Deployment has no `tls` volume".to_string())?;
    if tls_volume["secret"]["secretName"].as_str() != Some(secret_name) {
        return Err(format!(
            "NativeLink Deployment TLS volume does not reference ExternalSecret target \
             `{secret_name}`"
        ));
    }
    if tls_volume["secret"].get("items").is_some()
        || tls_volume["secret"]["optional"].as_bool() == Some(true)
    {
        return Err(
            "NativeLink Deployment TLS volume must project every required non-optional key"
                .to_string(),
        );
    }

    let containers = deployment["spec"]["template"]["spec"]["containers"]
        .as_sequence()
        .ok_or_else(|| "NativeLink Deployment has no containers".to_string())?;
    let container = containers
        .iter()
        .find(|container| container["name"].as_str() == Some("nativelink"))
        .ok_or_else(|| "NativeLink Deployment has no `nativelink` container".to_string())?;
    let mounts = container["volumeMounts"]
        .as_sequence()
        .ok_or_else(|| "NativeLink container has no volume mounts".to_string())?;
    let tls_mount = mounts
        .iter()
        .find(|mount| mount["name"].as_str() == Some("tls"))
        .ok_or_else(|| "NativeLink container has no `tls` volume mount".to_string())?;
    if tls_mount["mountPath"].as_str() != Some("/tls")
        || tls_mount["readOnly"].as_bool() != Some(true)
    {
        return Err("NativeLink `tls` volume must be mounted read-only at `/tls`".to_string());
    }
    if tls_mount.get("subPath").is_some() || tls_mount.get("subPathExpr").is_some() {
        return Err("NativeLink `tls` mount must project the complete Secret".to_string());
    }

    let declared_ports = container["ports"]
        .as_sequence()
        .ok_or_else(|| "NativeLink container has no declared ports".to_string())?
        .iter()
        .map(|entry| {
            entry["containerPort"]
                .as_u64()
                .ok_or_else(|| "NativeLink containerPort is missing or non-numeric".to_string())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    for (role, port) in role_ports {
        if !declared_ports.contains(port) {
            return Err(format!(
                "{role} Service target port {port} is not declared by the NativeLink container"
            ));
        }
    }
    Ok(())
}

fn validate_deployment_config_binding(
    config_map: &YamlValue,
    deployment: &YamlValue,
) -> Result<(), String> {
    let config_name = config_map["metadata"]["name"]
        .as_str()
        .ok_or_else(|| "NativeLink ConfigMap name is missing or non-string".to_string())?;
    let config_namespace = metadata_namespace(config_map, "NativeLink ConfigMap")?;
    let deployment_namespace = metadata_namespace(deployment, "NativeLink Deployment")?;
    if config_namespace != deployment_namespace {
        return Err(format!(
            "NativeLink Deployment namespace `{deployment_namespace}` disagrees with ConfigMap \
             namespace `{config_namespace}`"
        ));
    }
    if config_map["data"]["cas.json"].as_str().is_none() {
        return Err("NativeLink ConfigMap does not supply data.cas.json".to_string());
    }

    let volumes = deployment["spec"]["template"]["spec"]["volumes"]
        .as_sequence()
        .ok_or_else(|| "NativeLink Deployment has no volumes".to_string())?;
    let config_volume = volumes
        .iter()
        .find(|volume| volume["name"].as_str() == Some("config"))
        .ok_or_else(|| "NativeLink Deployment has no `config` volume".to_string())?;
    if config_volume["configMap"]["name"].as_str() != Some(config_name) {
        return Err(format!(
            "NativeLink Deployment config volume does not reference ConfigMap `{config_name}`"
        ));
    }
    if config_volume["configMap"].get("items").is_some() {
        return Err(
            "NativeLink Deployment config volume must not remap the `cas.json` key".to_string(),
        );
    }
    if config_volume["configMap"]["optional"].as_bool() == Some(true) {
        return Err("NativeLink Deployment config volume must not be optional".to_string());
    }

    let containers = deployment["spec"]["template"]["spec"]["containers"]
        .as_sequence()
        .ok_or_else(|| "NativeLink Deployment has no containers".to_string())?;
    let container = containers
        .iter()
        .find(|container| container["name"].as_str() == Some("nativelink"))
        .ok_or_else(|| "NativeLink Deployment has no `nativelink` container".to_string())?;
    let args = container["args"]
        .as_sequence()
        .ok_or_else(|| "NativeLink container args are missing or non-sequence".to_string())?;
    if args.as_slice() != [YamlValue::String("/etc/nativelink/cas.json".to_string())] {
        return Err("NativeLink container must select only `/etc/nativelink/cas.json`".to_string());
    }
    let mounts = container["volumeMounts"]
        .as_sequence()
        .ok_or_else(|| "NativeLink container has no volume mounts".to_string())?;
    let config_mount = mounts
        .iter()
        .find(|mount| mount["name"].as_str() == Some("config"))
        .ok_or_else(|| "NativeLink container has no `config` volume mount".to_string())?;
    if config_mount["mountPath"].as_str() != Some("/etc/nativelink")
        || config_mount["readOnly"].as_bool() != Some(true)
    {
        return Err(
            "NativeLink `config` volume must be mounted read-only at `/etc/nativelink`".to_string(),
        );
    }
    if config_mount.get("subPath").is_some() || config_mount.get("subPathExpr").is_some() {
        return Err("NativeLink `config` mount must project the complete ConfigMap".to_string());
    }
    Ok(())
}

fn validate_deployment_config_digest(
    config_text: &str,
    deployment: &YamlValue,
) -> Result<(), String> {
    let expected = format!("{:x}", Sha256::digest(config_text.as_bytes()));
    let observed =
        deployment["spec"]["template"]["metadata"]["annotations"]["oya.io/config-sha256"]
            .as_str()
            .ok_or_else(|| "NativeLink pod template lacks `oya.io/config-sha256`".to_string())?;
    if observed != expected {
        return Err(format!(
            "NativeLink pod-template config digest `{observed}` disagrees with `{expected}`"
        ));
    }
    Ok(())
}

fn validate_deployment_data_binding(
    config: &Value,
    pvc: &YamlValue,
    deployment: &YamlValue,
) -> Result<(), String> {
    let pvc_name = pvc["metadata"]["name"]
        .as_str()
        .ok_or_else(|| "NativeLink PVC name is missing or non-string".to_string())?;
    let pvc_namespace = metadata_namespace(pvc, "NativeLink PVC")?;
    let deployment_namespace = metadata_namespace(deployment, "NativeLink Deployment")?;
    if pvc_namespace != deployment_namespace {
        return Err(format!(
            "NativeLink PVC namespace `{pvc_namespace}` disagrees with Deployment namespace \
             `{deployment_namespace}`"
        ));
    }
    let access_modes = pvc["spec"]["accessModes"]
        .as_sequence()
        .ok_or_else(|| "NativeLink PVC accessModes are missing or non-sequence".to_string())?
        .iter()
        .map(|mode| {
            mode.as_str()
                .ok_or_else(|| "NativeLink PVC accessMode is non-string".to_string())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if access_modes != BTreeSet::from(["ReadWriteOnce"]) {
        return Err("NativeLink PVC must use exactly the ReadWriteOnce access mode".to_string());
    }

    let volumes = deployment["spec"]["template"]["spec"]["volumes"]
        .as_sequence()
        .ok_or_else(|| "NativeLink Deployment has no volumes".to_string())?;
    let data_volumes = volumes
        .iter()
        .filter(|volume| volume["name"].as_str() == Some("data"))
        .collect::<Vec<_>>();
    if data_volumes.len() != 1 {
        return Err("NativeLink Deployment must have exactly one `data` volume".to_string());
    }
    let data_volume = data_volumes[0];
    if data_volume["persistentVolumeClaim"]["claimName"].as_str() != Some(pvc_name)
        || data_volume["persistentVolumeClaim"]["readOnly"].as_bool() == Some(true)
    {
        return Err(format!(
            "NativeLink Deployment data volume must mount writable PVC `{pvc_name}`"
        ));
    }

    let container = deployment["spec"]["template"]["spec"]["containers"]
        .as_sequence()
        .and_then(|containers| {
            containers
                .iter()
                .find(|container| container["name"].as_str() == Some("nativelink"))
        })
        .ok_or_else(|| "NativeLink Deployment has no `nativelink` container".to_string())?;
    let mounts = container["volumeMounts"]
        .as_sequence()
        .ok_or_else(|| "NativeLink container has no volume mounts".to_string())?;
    let data_mounts = mounts
        .iter()
        .filter(|mount| mount["name"].as_str() == Some("data"))
        .collect::<Vec<_>>();
    if data_mounts.len() != 1 {
        return Err("NativeLink container must have exactly one `data` mount".to_string());
    }
    let data_mount = data_mounts[0];
    if data_mount["mountPath"].as_str() != Some("/data")
        || data_mount["readOnly"].as_bool() == Some(true)
        || data_mount.get("subPath").is_some()
        || data_mount.get("subPathExpr").is_some()
    {
        return Err(
            "NativeLink `data` volume must be mounted writable at `/data` without a subPath"
                .to_string(),
        );
    }

    let stores = config["stores"]
        .as_array()
        .ok_or_else(|| "NativeLink stores are missing or non-array".to_string())?;
    for (name, pointer, content_path, temp_path) in [
        (
            "CAS_MAIN_STORE",
            "/verify/backend/fast_slow/slow/filesystem",
            "/data/cas-content",
            "/data/cas-tmp",
        ),
        (
            "AC_MAIN_STORE",
            "/fast_slow/slow/filesystem",
            "/data/ac-content",
            "/data/ac-tmp",
        ),
    ] {
        let store = stores
            .iter()
            .find(|store| store["name"].as_str() == Some(name))
            .ok_or_else(|| format!("NativeLink store `{name}` is missing"))?;
        let filesystem = store
            .pointer(pointer)
            .ok_or_else(|| format!("NativeLink store `{name}` has no slow filesystem"))?;
        if filesystem["content_path"].as_str() != Some(content_path)
            || filesystem["temp_path"].as_str() != Some(temp_path)
        {
            return Err(format!(
                "NativeLink store `{name}` must use `{content_path}` and `{temp_path}`"
            ));
        }
    }
    Ok(())
}

fn role_ingress_ports(
    policy: &YamlValue,
    role: &str,
    expected_port: u64,
    runner_namespace: &str,
) -> Result<BTreeSet<u64>, String> {
    let label = format!("oya.io/nativelink-cas-{role}");
    let rules = policy["spec"]["ingress"]
        .as_sequence()
        .ok_or_else(|| "NativeLink ingress policy has no ingress rules".to_string())?;
    let mut ports = BTreeSet::new();
    for rule in rules {
        let declared = rule["ports"]
            .as_sequence()
            .ok_or_else(|| "NativeLink ingress rule has no ports".to_string())?;
        let declared_ports = declared
            .iter()
            .map(|port| kubernetes_tcp_port(port, "NativeLink ingress rule"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        let peers = rule["from"]
            .as_sequence()
            .ok_or_else(|| "NativeLink ingress rule has no source peers".to_string())?;
        let matches_role = peers.iter().any(|peer| {
            peer["podSelector"]["matchLabels"][label.as_str()].as_str() == Some("true")
        });
        if !matches_role {
            if declared_ports.contains(&expected_port) {
                return Err(format!(
                    "{role} ingress port {expected_port} is exposed to a non-role peer"
                ));
            }
            continue;
        }
        for peer in peers {
            let pod_selector = peer["podSelector"]
                .as_mapping()
                .ok_or_else(|| format!("{role} ingress peer has no podSelector"))?;
            let labels = peer["podSelector"]["matchLabels"]
                .as_mapping()
                .ok_or_else(|| format!("{role} ingress peer has no podSelector labels"))?;
            if pod_selector.len() != 1
                || labels.len() != 1
                || peer["podSelector"]["matchLabels"][label.as_str()].as_str() != Some("true")
            {
                return Err(format!(
                    "{role} ingress peer must use only the exclusive role label `{label}`"
                ));
            }
            let namespace_selector = peer
                .get("namespaceSelector")
                .and_then(YamlValue::as_mapping)
                .ok_or_else(|| format!("{role} ingress peer has no namespaceSelector"))?;
            let namespace_labels = peer["namespaceSelector"]["matchLabels"]
                .as_mapping()
                .ok_or_else(|| {
                    format!("{role} ingress peer has no namespaceSelector.matchLabels")
                })?;
            let selects_runner_namespace = namespace_selector.len() == 1
                && namespace_labels.len() == 1
                && peer["namespaceSelector"]["matchLabels"]["kubernetes.io/metadata.name"].as_str()
                    == Some(runner_namespace);
            if !selects_runner_namespace {
                return Err(format!(
                    "{role} ingress peer namespaceSelector does not admit runner namespace \
                     `{runner_namespace}`"
                ));
            }
        }
        ports.extend(declared_ports);
    }
    if ports.is_empty() {
        Err(format!(
            "NativeLink ingress policy has no rule for role `{role}`"
        ))
    } else {
        Ok(ports)
    }
}

fn validate_runner_egress_selector(policy: &YamlValue) -> Result<(), String> {
    let selector = policy["spec"]["podSelector"]
        .as_mapping()
        .ok_or_else(|| "runner egress policy podSelector is missing or non-mapping".to_string())?;
    let expressions = policy["spec"]["podSelector"]["matchExpressions"]
        .as_sequence()
        .ok_or_else(|| {
            "runner egress policy podSelector.matchExpressions is missing or non-sequence"
                .to_string()
        })?;
    if selector.len() != 1 || expressions.len() != 1 {
        return Err(
            "runner egress policy must select cells with one `oya.io/ci-cell` expression"
                .to_string(),
        );
    }
    let expression = &expressions[0];
    if expression["key"].as_str() != Some("oya.io/ci-cell")
        || expression["operator"].as_str() != Some("In")
    {
        return Err("runner egress policy must use `oya.io/ci-cell In (...)`".to_string());
    }
    let values = expression["values"]
        .as_sequence()
        .ok_or_else(|| "runner egress cell values are missing or non-sequence".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "runner egress cell value is non-string".to_string())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let required = BTreeSet::from(["general", "live-postgres"]);
    if !required.is_subset(&values) {
        return Err(format!(
            "runner egress cell values {values:?} are missing {required:?}"
        ));
    }
    Ok(())
}

fn rule_grants_cache_port(rule: &YamlValue, cache_ports: &BTreeSet<u64>) -> bool {
    let Some(ports) = rule["ports"].as_sequence() else {
        return true;
    };
    if ports.is_empty() {
        return true;
    }
    ports.iter().any(|entry| {
        if !matches!(entry["protocol"].as_str(), None | Some("TCP")) {
            return false;
        }
        let Some(start) = entry["port"].as_u64() else {
            return true;
        };
        let end = entry["endPort"].as_u64().unwrap_or(start);
        cache_ports
            .iter()
            .any(|port| *port >= start && *port <= end)
    })
}

fn destination_may_include_namespace(peer: &YamlValue, namespace: &str) -> bool {
    if let Some(selector) = peer["namespaceSelector"].as_mapping() {
        let labels = peer["namespaceSelector"]["matchLabels"].as_mapping();
        return selector.len() != 1
            || labels.is_none_or(|labels| labels.len() != 1)
            || peer["namespaceSelector"]["matchLabels"]["kubernetes.io/metadata.name"].as_str()
                == Some(namespace);
    }
    if let Some(ip_block) = peer["ipBlock"].as_mapping() {
        let excluded = peer["ipBlock"]["except"]
            .as_sequence()
            .map(|ranges| {
                ranges
                    .iter()
                    .filter_map(YamlValue::as_str)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let required_private_exclusions = BTreeSet::from([
            "10.0.0.0/8",
            "100.64.0.0/10",
            "127.0.0.0/8",
            "169.254.0.0/16",
            "172.16.0.0/12",
            "192.168.0.0/16",
        ]);
        return ip_block["cidr"].as_str() != Some("0.0.0.0/0")
            || !required_private_exclusions.is_subset(&excluded);
    }
    peer.get("podSelector").is_none()
}

fn namespace_egress_ports(
    policy: &YamlValue,
    namespace: &str,
    cache_ports: &BTreeSet<u64>,
) -> Result<BTreeSet<u64>, String> {
    let rules = policy["spec"]["egress"]
        .as_sequence()
        .ok_or_else(|| "runner egress policy has no egress rules".to_string())?;
    let mut ports = BTreeSet::new();
    for rule in rules {
        let peers = rule["to"]
            .as_sequence()
            .ok_or_else(|| "runner egress rule has no destination peers".to_string())?;
        if peers.is_empty() {
            return Err("runner egress rule has no destination peers".to_string());
        }
        let matches_namespace = peers.iter().any(|peer| {
            peer["namespaceSelector"]["matchLabels"]["kubernetes.io/metadata.name"].as_str()
                == Some(namespace)
        });
        if !matches_namespace {
            if peers
                .iter()
                .any(|peer| destination_may_include_namespace(peer, namespace))
                && rule_grants_cache_port(rule, cache_ports)
            {
                return Err(format!(
                    "runner egress grants cache ports to a destination outside `{namespace}`"
                ));
            }
            continue;
        }
        if peers.len() != 1 {
            return Err(format!(
                "runner egress rule for `{namespace}` must have exactly one destination"
            ));
        }
        let peer = &peers[0];
        let peer_fields = peer
            .as_mapping()
            .ok_or_else(|| format!("runner egress destination for `{namespace}` is non-mapping"))?;
        let namespace_selector = peer["namespaceSelector"].as_mapping().ok_or_else(|| {
            format!("runner egress destination for `{namespace}` has no namespaceSelector")
        })?;
        let namespace_labels = peer["namespaceSelector"]["matchLabels"]
            .as_mapping()
            .ok_or_else(|| {
                format!(
                    "runner egress destination for `{namespace}` has no namespaceSelector labels"
                )
            })?;
        if peer_fields.len() != 1
            || namespace_selector.len() != 1
            || namespace_labels.len() != 1
            || peer["namespaceSelector"]["matchLabels"]["kubernetes.io/metadata.name"].as_str()
                != Some(namespace)
        {
            return Err(format!(
                "runner egress destination must use only namespace `{namespace}`"
            ));
        }
        let declared = rule["ports"]
            .as_sequence()
            .ok_or_else(|| format!("runner egress rule for `{namespace}` has no ports"))?;
        for port in declared {
            ports.insert(kubernetes_tcp_port(
                port,
                &format!("runner egress rule for `{namespace}`"),
            )?);
        }
    }
    if ports.is_empty() {
        Err(format!(
            "runner egress policy has no rule for namespace `{namespace}`"
        ))
    } else {
        Ok(ports)
    }
}

fn validate_endpoint_network_policy_ports(
    ingress_policy: &YamlValue,
    runner_egress_policy: &YamlValue,
    service_namespace: &str,
    role_ports: &[(&str, u64)],
) -> Result<(), String> {
    validate_network_policy_direction(ingress_policy, "Ingress", "NativeLink ingress policy")?;
    validate_network_policy_direction(runner_egress_policy, "Egress", "runner egress policy")?;
    let ingress_namespace = metadata_namespace(ingress_policy, "NativeLink ingress policy")?;
    if ingress_namespace != service_namespace {
        return Err(format!(
            "NativeLink ingress policy namespace `{ingress_namespace}` disagrees with Service \
             namespace `{service_namespace}`"
        ));
    }
    let expected_egress = role_ports
        .iter()
        .map(|(_, port)| *port)
        .collect::<BTreeSet<_>>();
    validate_runner_egress_selector(runner_egress_policy)?;
    let runner_namespace = metadata_namespace(runner_egress_policy, "runner egress policy")?;
    for (role, port) in role_ports {
        let actual = role_ingress_ports(ingress_policy, role, *port, runner_namespace)?;
        let expected = BTreeSet::from([*port]);
        if actual != expected {
            return Err(format!(
                "{role} ingress ports {actual:?} disagree with endpoint port {port}"
            ));
        }
    }
    let actual_egress =
        namespace_egress_ports(runner_egress_policy, service_namespace, &expected_egress)?;
    if !expected_egress.is_subset(&actual_egress) {
        return Err(format!(
            "runner egress ports {actual_egress:?} are missing endpoint ports \
             {expected_egress:?}"
        ));
    }
    Ok(())
}

fn validate_server_san_preflight(
    runbook: &str,
    expected_sans: &HashSet<String>,
) -> Result<(), String> {
    let san_block = runbook
        .split_once("cat >\"$tmp/server-sans.expected\" <<'EOF'\n")
        .and_then(|(_, suffix)| suffix.split_once("\nEOF"))
        .map(|(block, _)| block)
        .ok_or_else(|| "exact server SAN preflight block is missing".to_string())?;
    let actual_sans = san_block.lines().map(str::to_owned).collect::<HashSet<_>>();
    if &actual_sans != expected_sans {
        return Err(format!(
            "server SAN preflight {actual_sans:?} disagrees with endpoint DATA {expected_sans:?}"
        ));
    }
    Ok(())
}

fn licensed_fixture() -> Value {
    json!({ "warm_reads_licensed": true, "reason": "conformance fixture", "licensed_by_canary_run": "fixture" })
}

fn invocation_record_fixture(
    cache_hit_rate: f64,
    action_hits: u64,
    local: u64,
    remote: u64,
) -> Value {
    json!({
        "cache_hit_rate": cache_hit_rate,
        "run_action_cache_count": action_hits,
        "run_local_count": local,
        "run_remote_count": remote,
        "run_skipped_count": 0,
        "cache_upload_attempt_count": 0,
        "cache_upload_count": 0,
        "dep_file_upload_attempt_count": 0,
        "dep_file_upload_count": 0,
        "run_remote_dep_file_cache_count": 0,
        "re_upload_bytes": 0,
        "re_download_bytes": if action_hits > 0 { 1024 } else { 0 },
        "exit_result_name": "SUCCESS",
        "run_command_failure_count": 0,
        "errors": [],
        "daemon_connection_failure": false,
        "last_snapshot": {
            "re_action_cache_started": action_hits,
            "re_action_cache_finished_successfully": action_hits,
            "re_action_cache_finished_with_error": 0,
            "re_upload_bytes": 0,
            "re_uploads_started": 0,
            "re_uploads_finished_successfully": 0,
            "re_uploads_finished_with_error": 0,
            "re_download_bytes": if action_hits > 0 { 1024 } else { 0 },
            "re_downloads_started": action_hits,
            "re_downloads_finished_successfully": action_hits,
            "re_downloads_finished_with_error": 0,
            "re_executes_started": 0,
            "re_executes_finished_successfully": 0,
            "re_executes_finished_with_error": 0,
            "re_write_action_results_started": 0,
            "re_write_action_results_finished_successfully": 0,
            "re_write_action_results_finished_with_error": 0,
            "re_get_digest_expirations_started": 0,
            "re_get_digest_expirations_finished_successfully": 0,
            "re_get_digest_expirations_finished_with_error": 0,
            "re_materializes_started": 0,
            "re_materializes_finished_successfully": 0,
            "re_materializes_finished_with_error": 0
        },
    })
}

#[derive(Debug)]
enum GlobToken {
    Literal(char),
    AnyCharacter,
    Star,
    CharacterClass {
        negated: bool,
        ranges: Vec<(char, char)>,
    },
}

fn parse_glob_segment(pattern: &str) -> Result<Vec<GlobToken>, String> {
    let characters: Vec<char> = pattern.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < characters.len() {
        match characters[index] {
            '*' => {
                tokens.push(GlobToken::Star);
                index += 1;
            }
            '?' => {
                tokens.push(GlobToken::AnyCharacter);
                index += 1;
            }
            '\\' => {
                index += 1;
                let Some(character) = characters.get(index).copied() else {
                    return Err(format!("trailing glob escape in {pattern:?}"));
                };
                tokens.push(GlobToken::Literal(character));
                index += 1;
            }
            '[' => {
                index += 1;
                let negated = matches!(characters.get(index), Some('!' | '^'));
                if negated {
                    index += 1;
                }
                let mut ranges = Vec::new();
                if characters.get(index) == Some(&']') {
                    ranges.push((']', ']'));
                    index += 1;
                }
                while index < characters.len() && characters[index] != ']' {
                    let start = if characters[index] == '\\' {
                        index += 1;
                        characters.get(index).copied().ok_or_else(|| {
                            format!("trailing character-class escape in {pattern:?}")
                        })?
                    } else {
                        characters[index]
                    };
                    index += 1;

                    if characters.get(index) == Some(&'-')
                        && characters.get(index + 1).is_some_and(|value| *value != ']')
                    {
                        index += 1;
                        let end = if characters[index] == '\\' {
                            index += 1;
                            characters.get(index).copied().ok_or_else(|| {
                                format!("trailing character-class range escape in {pattern:?}")
                            })?
                        } else {
                            characters[index]
                        };
                        index += 1;
                        if start > end {
                            return Err(format!("reversed character-class range in {pattern:?}"));
                        }
                        ranges.push((start, end));
                    } else {
                        ranges.push((start, start));
                    }
                }
                if characters.get(index) != Some(&']') || ranges.is_empty() {
                    return Err(format!(
                        "unterminated or empty character class in {pattern:?}"
                    ));
                }
                index += 1;
                tokens.push(GlobToken::CharacterClass { negated, ranges });
            }
            character => {
                tokens.push(GlobToken::Literal(character));
                index += 1;
            }
        }
    }

    Ok(tokens)
}

fn add_star_epsilon_closure(tokens: &[GlobToken], states: &mut HashSet<usize>) {
    loop {
        let additions: Vec<usize> = states
            .iter()
            .filter_map(|position| {
                matches!(tokens.get(*position), Some(GlobToken::Star)).then_some(position + 1)
            })
            .filter(|position| !states.contains(position))
            .collect();
        if additions.is_empty() {
            return;
        }
        states.extend(additions);
    }
}

fn glob_segment_matches(pattern: &str, target: &str) -> Result<bool, String> {
    let tokens = parse_glob_segment(pattern)?;
    let mut states = HashSet::from([0]);
    add_star_epsilon_closure(&tokens, &mut states);

    for character in target.chars() {
        let mut next = HashSet::new();
        for position in &states {
            match tokens.get(*position) {
                Some(GlobToken::Star) => {
                    next.insert(*position);
                }
                Some(GlobToken::Literal(expected)) if *expected == character => {
                    next.insert(position + 1);
                }
                Some(GlobToken::AnyCharacter) => {
                    next.insert(position + 1);
                }
                Some(GlobToken::CharacterClass { negated, ranges }) => {
                    let listed = ranges
                        .iter()
                        .any(|(start, end)| *start <= character && character <= *end);
                    if listed != *negated {
                        next.insert(position + 1);
                    }
                }
                _ => {}
            }
        }
        states = next;
        add_star_epsilon_closure(&tokens, &mut states);
    }

    Ok(states.contains(&tokens.len()))
}

fn cache_path_candidate_archives_checkout(candidate: &str) -> Result<bool, String> {
    let mut relative = strip_workspace_expression(candidate)
        .map(|suffix| suffix.trim_start_matches('/'))
        .unwrap_or(candidate)
        .trim_end_matches('/');
    while let Some(stripped) = relative.strip_prefix("./") {
        relative = stripped;
    }

    if relative.contains("${{") || relative.contains("}}") {
        return Err(format!(
            "unresolved dynamic expression controls the cache path: {candidate:?}"
        ));
    }

    if relative.is_empty() || relative == "." {
        return Ok(true);
    }

    let first_component = relative
        .split('/')
        .find(|component| !component.is_empty() && *component != ".")
        .unwrap_or(relative);
    glob_segment_matches(first_component, "buck-out")
}

fn strip_workspace_expression(candidate: &str) -> Option<&str> {
    let expression = candidate.strip_prefix("${{")?;
    let end = expression.find("}}")?;
    let name: String = expression[..end]
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect();
    let suffix = &expression[end + 2..];
    (name.eq_ignore_ascii_case("github.workspace")
        && (suffix.is_empty() || suffix.starts_with('/') || suffix.starts_with('\\')))
    .then_some(suffix)
}

fn cache_path_archives_checkout(raw_path: &str) -> Result<bool, String> {
    let normalized = raw_path.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("empty include pattern".to_owned());
    }
    let bytes = normalized.as_bytes();
    let windows_drive_path = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if normalized.starts_with('/') || normalized.starts_with('\\') || windows_drive_path {
        return Err(format!(
            "absolute actions/cache paths cannot prove exclusion of runner-local buck-out: {raw_path:?}"
        ));
    }
    if normalized.starts_with('~') {
        const SAFE_TILDE_ROOTS: [&str; 2] = ["~/.rustup/toolchains", "~/.rustup/update-hashes"];
        let proven_safe = SAFE_TILDE_ROOTS.iter().any(|root| {
            normalized == *root
                || normalized
                    .strip_prefix(root)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        });
        if !proven_safe {
            return Err(format!(
                "unproven tilde-expanded actions/cache path can reach the runner checkout: {raw_path:?}"
            ));
        }
    }
    if normalized
        .split(['/', '\\'])
        .any(|component| component == "..")
    {
        return Err(format!(
            "relative parent segments are not supported by @actions/glob: {raw_path:?}"
        ));
    }

    // On Linux/macOS a backslash escapes glob metacharacters; on Windows it can
    // also be a separator. Reject if either supported interpretation reaches the
    // checkout's runner-local `buck-out` root.
    cache_path_candidate_archives_checkout(&normalized).and_then(|archives| {
        if archives || !normalized.contains('\\') {
            Ok(archives)
        } else {
            cache_path_candidate_archives_checkout(&normalized.replace('\\', "/"))
        }
    })
}

fn included_cache_pattern(raw_path: &str) -> Result<Option<&str>, String> {
    let mut pattern = raw_path.trim();
    if pattern.is_empty() || pattern.starts_with('#') {
        return Ok(None);
    }
    let mut excluded = false;
    while let Some(remainder) = pattern.strip_prefix('!') {
        excluded = !excluded;
        pattern = remainder.trim();
    }
    if excluded {
        cache_path_archives_checkout(pattern)?;
        return Ok(None);
    }
    if pattern.is_empty() {
        return Err(format!("empty actions/cache include pattern {raw_path:?}"));
    }
    Ok(Some(pattern))
}

fn action_steps(doc: &YamlValue) -> Vec<(&str, &[YamlValue])> {
    let mut scopes = Vec::new();
    if let Some(jobs) = doc.get("jobs").and_then(YamlValue::as_mapping) {
        for (job_name, job) in jobs {
            if let Some(steps) = job.get("steps").and_then(YamlValue::as_sequence) {
                scopes.push((
                    job_name.as_str().unwrap_or("<non-string-job>"),
                    steps.as_slice(),
                ));
            }
        }
    }
    if let Some(steps) = doc
        .get("runs")
        .and_then(|runs| runs.get("steps"))
        .and_then(YamlValue::as_sequence)
    {
        scopes.push(("<composite-action>", steps.as_slice()));
    }
    scopes
}

fn local_action_file(repo_root: &Path, action_name: &str) -> Result<Option<PathBuf>, String> {
    let Some(relative) = action_name
        .strip_prefix("./")
        .or_else(|| action_name.strip_prefix(".\\"))
    else {
        return Ok(None);
    };
    if action_name.contains('\\') {
        return Err(format!(
            "backslash-bearing local action paths are host-ambiguous; use portable `./` slash syntax: {action_name:?}"
        ));
    }
    let relative = Path::new(relative);
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "local action path escapes the repository: {action_name:?}"
        ));
    }
    let action_dir = repo_root.join(relative);
    for file_name in ["action.yml", "action.yaml"] {
        let candidate = action_dir.join(file_name);
        if let Ok(metadata) = candidate.symlink_metadata() {
            if metadata.file_type().is_file() {
                return Ok(Some(candidate));
            }
            return Err(format!(
                "local action metadata is not a regular file: {}",
                candidate.display()
            ));
        }
    }
    Err(format!(
        "local action {action_name:?} has no action.yml or action.yaml"
    ))
}

fn local_reusable_workflow_file(repo_root: &Path, reference: &str) -> Result<PathBuf, String> {
    let Some(relative) = reference.strip_prefix("./") else {
        return Err(format!(
            "external job-level reusable workflow is not proven cache-safe: {reference:?}"
        ));
    };
    let relative = Path::new(relative);
    if !relative.starts_with(".github/workflows")
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || !matches!(
            relative
                .extension()
                .and_then(|extension| extension.to_str()),
            Some("yml" | "yaml")
        )
    {
        return Err(format!(
            "invalid same-repository reusable workflow reference: {reference:?}"
        ));
    }
    let workflow_file = repo_root.join(relative);
    match workflow_file.symlink_metadata() {
        Ok(metadata) if metadata.file_type().is_file() => Ok(workflow_file),
        Ok(_) => Err(format!(
            "reusable workflow is not a regular file: {}",
            workflow_file.display()
        )),
        Err(error) => Err(format!(
            "cannot resolve reusable workflow {}: {error}",
            workflow_file.display()
        )),
    }
}

fn inspect_local_yaml_document(
    repo_root: &Path,
    path: PathBuf,
    visited_local_documents: &mut HashSet<PathBuf>,
    violations: &mut Vec<String>,
) {
    if !visited_local_documents.insert(path.clone()) {
        return;
    }
    match std::fs::read_to_string(&path) {
        Ok(text) => match serde_yaml::from_str(&text) {
            Ok(doc) => inspect_actions_cache_steps(
                Some(repo_root),
                &path.display().to_string(),
                &doc,
                visited_local_documents,
                violations,
            ),
            Err(error) => violations.push(format!(
                "{}: malformed local workflow/action YAML: {error}",
                path.display()
            )),
        },
        Err(error) => violations.push(format!(
            "{}: cannot read local workflow/action: {error}",
            path.display()
        )),
    }
}

fn inspect_actions_cache_steps(
    repo_root: Option<&Path>,
    source: &str,
    doc: &YamlValue,
    visited_local_documents: &mut HashSet<PathBuf>,
    violations: &mut Vec<String>,
) {
    if let Some(jobs) = doc.get("jobs").and_then(YamlValue::as_mapping) {
        for (job_name, job) in jobs {
            let scope = job_name.as_str().unwrap_or("<non-string-job>");
            let Some(reference_value) = job.get("uses") else {
                continue;
            };
            let Some(reference) = reference_value.as_str() else {
                violations.push(format!(
                    "{source}:{scope}: non-string reusable workflow reference {reference_value:?}"
                ));
                continue;
            };
            let Some(repo_root) = repo_root else {
                violations.push(format!(
                    "{source}:{scope}: cannot verify reusable workflow {reference:?} without a repository root"
                ));
                continue;
            };
            match local_reusable_workflow_file(repo_root, reference) {
                Ok(path) => inspect_local_yaml_document(
                    repo_root,
                    path,
                    visited_local_documents,
                    violations,
                ),
                Err(error) => violations.push(format!("{source}:{scope}: {error}")),
            }
        }
    }

    for (scope, steps) in action_steps(doc) {
        for step in steps {
            let Some(action) = step.get("uses").and_then(YamlValue::as_str) else {
                continue;
            };
            let local_action = action.starts_with("./") || action.starts_with(".\\");
            let action_name = if local_action {
                action
            } else {
                action.split('@').next().unwrap_or(action)
            };
            let action_name_lower = if local_action {
                action_name.to_ascii_lowercase()
            } else {
                action_name
                    .split(['/', '\\'])
                    .filter(|segment| !segment.is_empty())
                    .collect::<Vec<_>>()
                    .join("/")
                    .to_ascii_lowercase()
            };
            if matches!(
                action_name_lower.as_str(),
                "actions/cache" | "actions/cache/restore" | "actions/cache/save"
            ) {
                let step_name = step
                    .get("name")
                    .and_then(YamlValue::as_str)
                    .unwrap_or("<unnamed-step>");
                let Some(path) = step.get("with").and_then(|with| with.get("path")) else {
                    continue;
                };
                let mut raw_paths = Vec::new();
                match path {
                    YamlValue::String(value) => raw_paths.extend(value.lines()),
                    YamlValue::Sequence(values) => {
                        for value in values {
                            match value.as_str() {
                                Some(value) => raw_paths.extend(value.lines()),
                                None => violations.push(format!(
                                    "{source}:{scope}/{step_name}: non-string actions/cache path {value:?}"
                                )),
                            }
                        }
                    }
                    value => violations.push(format!(
                        "{source}:{scope}/{step_name}: non-string actions/cache path {value:?}"
                    )),
                }

                for raw_path in raw_paths {
                    match included_cache_pattern(raw_path) {
                        Ok(None) => {}
                        Ok(Some(include)) => match cache_path_archives_checkout(include) {
                            Ok(true) => violations.push(format!(
                                "{source}:{scope}/{step_name}: {action_name} archives forbidden path {raw_path:?}"
                            )),
                            Ok(false) => {}
                            Err(error) => violations.push(format!(
                                "{source}:{scope}/{step_name}: malformed actions/cache path {raw_path:?}: {error}"
                            )),
                        },
                        Err(error) => violations.push(format!(
                            "{source}:{scope}/{step_name}: malformed actions/cache path: {error}"
                        )),
                    }
                }
                continue;
            }

            let Some(repo_root) = repo_root else {
                continue;
            };
            match local_action_file(repo_root, action_name) {
                Ok(Some(action_file)) => inspect_local_yaml_document(
                    repo_root,
                    action_file,
                    visited_local_documents,
                    violations,
                ),
                Ok(None) => {}
                Err(error) => violations.push(format!("{source}:{scope}: {error}")),
            }
        }
    }
}

fn actions_cache_buck_out_violations(
    repo_root: Option<&Path>,
    source: &str,
    workflow: &str,
) -> Vec<String> {
    let doc: YamlValue = serde_yaml::from_str(workflow).expect("parse workflow YAML");
    let mut violations = Vec::new();
    let mut visited_local_documents = HashSet::new();
    if let Some(repo_root) = repo_root {
        let source_path = Path::new(source);
        if !source_path.is_absolute()
            && source_path
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
        {
            let candidate = repo_root.join(source_path);
            if candidate
                .symlink_metadata()
                .is_ok_and(|metadata| metadata.file_type().is_file())
            {
                visited_local_documents.insert(candidate);
            }
        }
    }
    inspect_actions_cache_steps(
        repo_root,
        source,
        &doc,
        &mut visited_local_documents,
        &mut violations,
    );
    violations
}

#[test]
fn policy_and_license_parse_and_the_default_is_fail_closed() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("load real cache-warmth policy");
    let license = app::load_license(&root).expect("load real cache-warm license");

    let default = &policy["default_for_unlisted_classes"];
    assert_eq!(default["warmth"], "cold", "unlisted default must be cold");
    assert_eq!(default["cache_read"], false);
    assert_eq!(default["cache_write"], false);

    assert_eq!(
        app::canary_class(&policy),
        Some("integrity-canary"),
        "the policy must name its canary trust anchor"
    );
    assert!(license["warm_reads_licensed"].is_boolean());
}

#[test]
fn dark_wiring_guarantee_under_the_real_license() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("policy");
    let license = app::load_license(&root).expect("license");
    let licensed = license["warm_reads_licensed"].as_bool().unwrap();

    let classes: Vec<String> = policy["build_classes"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .chain(std::iter::once("not-a-classified-class".to_string()))
        .collect();

    for class in &classes {
        let r = app::resolve(&policy, &license, class).expect("resolve");
        if !licensed {
            assert_eq!(
                r.mode,
                app::CacheMode::Bypass,
                "DARK-WIRING VIOLATION: class `{class}` resolved `{}` while \
                 warm_reads_licensed=false — no lane may touch the cache today",
                r.mode
            );
        } else {
            // Once the license flips (CAS bring-up + first GREEN canary), warm
            // classes must match their classified posture and cold stays bypass.
            let entry = policy["build_classes"].get(class);
            let warm = entry
                .map(|e| e["warmth"] == "warm" && e["cache_read"] == true)
                .unwrap_or(false)
                && class != "integrity-canary";
            assert_eq!(r.mode != app::CacheMode::Bypass, warm, "class `{class}`");
        }
    }
}

#[test]
fn cold_required_floor_holds_even_under_a_licensed_fixture() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("policy");
    let license = licensed_fixture();
    for class in COLD_REQUIRED_FLOOR {
        let entry = policy["build_classes"]
            .get(class)
            .unwrap_or_else(|| panic!("ADR-0556 one-way cold class `{class}` missing from policy"));
        assert_eq!(entry["warmth"], "cold", "`{class}` left the cold floor");
        assert_eq!(entry["cache_read"], false, "`{class}` gained cache_read");
        assert_eq!(entry["cache_write"], false, "`{class}` gained cache_write");
        let r = app::resolve(&policy, &license, class).expect("resolve");
        assert_eq!(
            r.mode,
            app::CacheMode::Bypass,
            "one-way floor: `{class}` must bypass even when warm is licensed"
        );
    }
}

#[test]
fn kill_switch_flips_warm_classes_and_only_warm_classes() {
    let root = repo_root();
    let policy = app::load_policy(&root).expect("policy");
    let unlicensed = json!({ "warm_reads_licensed": false, "reason": "fixture" });
    let licensed = licensed_fixture();

    let mut saw_warm = false;
    for (class, entry) in policy["build_classes"].as_object().unwrap() {
        let off = app::resolve(&policy, &unlicensed, class).unwrap().mode;
        let on = app::resolve(&policy, &licensed, class).unwrap().mode;
        assert_eq!(off, app::CacheMode::Bypass);
        if entry["warmth"] == "warm" && entry["cache_read"] == true && class != "integrity-canary" {
            saw_warm = true;
            let expected = if entry["cache_write"] == true {
                app::CacheMode::WarmReadWrite
            } else {
                app::CacheMode::WarmReadOnly
            };
            assert_eq!(on, expected, "licensed warm class `{class}`");
        } else {
            assert_eq!(
                on,
                app::CacheMode::Bypass,
                "cold class `{class}` must stay bypass"
            );
        }
    }
    assert!(
        saw_warm,
        "policy carries no warm-eligible class — fixture rot?"
    );
}

#[test]
fn overlays_parse_select_the_cache_platform_and_carry_no_identity() {
    let root = repo_root();
    let endpoints = app::load_endpoint_profile(&root).expect("load endpoint profile");
    for mode in [app::CacheMode::WarmReadWrite, app::CacheMode::WarmReadOnly] {
        let binding = app::cache_mode_binding(&endpoints, mode).expect("warm mode binding");
        let path = binding.overlay_path;
        let uploads = binding.allows_uploads.to_string();
        let text =
            std::fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let cfg = app::parse_buckconfig(&text);

        let build = cfg
            .get("build")
            .unwrap_or_else(|| panic!("{path}: no [build]"));
        assert_eq!(
            build["execution_platforms"], "toolchains//cache:cache-platform",
            "{path} must select the cache execution platform"
        );

        let oya = cfg
            .get("oya_cache")
            .unwrap_or_else(|| panic!("{path}: no [oya_cache]"));
        assert_eq!(oya["remote_cache_enabled"], "true", "{path}");
        assert_eq!(oya["allow_cache_uploads"], uploads, "{path}");

        let default_upload = cfg
            .get("buck2")
            .and_then(|section| section.get("default_allow_cache_upload"))
            .map(String::as_str);
        if binding.allows_uploads {
            assert_eq!(
                default_upload,
                Some("true"),
                "{path}: locally executed writer actions must opt into Buck2 cache uploads"
            );
        } else {
            assert_ne!(
                default_upload,
                Some("true"),
                "{path}: the reader overlay must never enable cache uploads"
            );
        }

        let re = cfg
            .get("buck2_re_client")
            .unwrap_or_else(|| panic!("{path}: no [buck2_re_client]"));
        assert_eq!(re["tls"], "true", "{path}: keyed transport is TLS-only");
        for key in ["engine_address", "cas_address", "action_cache_address"] {
            assert_eq!(
                re[key],
                app::RE_ADDRESS_TOKEN,
                "{path}: {key} must be materialized from endpoint DATA"
            );
        }
        assert_eq!(re["instance_name"], app::INSTANCE_NAME_TOKEN, "{path}");
        assert!(
            !re.contains_key("tls_client_cert"),
            "{path}: the keyed identity must come from secret-mounted env at emit time, \
             never from the checked-in overlay"
        );
        assert!(
            !text.contains("PRIVATE KEY") && !text.to_lowercase().contains("api-key"),
            "{path}: secret material in a checked-in overlay"
        );

        let resolution = app::Resolution {
            build_class: "fixture".to_string(),
            mode,
            reasons: Vec::new(),
        };
        let effective = app::effective_buckconfig(
            &resolution,
            &text,
            Some(&endpoints),
            Some("/run/secrets/cache-client.pem"),
            Some("/run/secrets/cache-server-ca.pem"),
        )
        .expect("materialize endpoint DATA")
        .expect("warm config");
        let effective = app::parse_buckconfig(&effective);
        let effective_re = &effective["buck2_re_client"];
        for key in ["engine_address", "cas_address", "action_cache_address"] {
            assert_eq!(
                effective_re[key],
                binding.endpoint.re_address(),
                "{path}: {key}"
            );
        }
        assert_eq!(
            effective_re["instance_name"],
            endpoints.instance_name(),
            "{path}"
        );
        assert!(
            effective_re
                .values()
                .all(|value| !value.contains("__CACHE_")),
            "{path}: effective config retained a materialization token"
        );
    }
}

#[test]
fn endpoint_data_matches_the_nativelink_services_and_instances() {
    let root = repo_root();
    let endpoints = app::load_endpoint_profile(&root).expect("load endpoint profile");
    let manifest = std::fs::read_to_string(root.join(NATIVELINK_MANIFEST_PATH))
        .expect("read NativeLink manifest");
    let documents = serde_yaml::Deserializer::from_str(&manifest)
        .map(|document| YamlValue::deserialize(document).expect("parse NativeLink YAML document"))
        .collect::<Vec<_>>();
    validate_unique_kubernetes_identities(&documents)
        .expect("NativeLink manifest resource identities are unique");

    let config_map = documents
        .iter()
        .find(|document| {
            document.get("kind").and_then(YamlValue::as_str) == Some("ConfigMap")
                && document
                    .get("metadata")
                    .and_then(|metadata| metadata.get("name"))
                    .and_then(YamlValue::as_str)
                    == Some("nativelink-cas-config")
        })
        .expect("NativeLink config ConfigMap");
    let deployment = documents
        .iter()
        .find(|document| {
            document.get("kind").and_then(YamlValue::as_str) == Some("Deployment")
                && document
                    .get("metadata")
                    .and_then(|metadata| metadata.get("name"))
                    .and_then(YamlValue::as_str)
                    == Some("nativelink-cas")
        })
        .expect("NativeLink Deployment");
    validate_deployment_topology(deployment)
        .expect("NativeLink Deployment topology is singleton Recreate");
    let external_secret = documents
        .iter()
        .find(|document| {
            document.get("kind").and_then(YamlValue::as_str) == Some("ExternalSecret")
                && document
                    .get("metadata")
                    .and_then(|metadata| metadata.get("name"))
                    .and_then(YamlValue::as_str)
                    == Some("nativelink-cas-tls")
        })
        .expect("NativeLink TLS ExternalSecret");
    let pvc = documents
        .iter()
        .find(|document| {
            document.get("kind").and_then(YamlValue::as_str) == Some("PersistentVolumeClaim")
                && document
                    .get("metadata")
                    .and_then(|metadata| metadata.get("name"))
                    .and_then(YamlValue::as_str)
                    == Some("nativelink-cas-data")
        })
        .expect("NativeLink data PVC");
    validate_deployment_config_binding(config_map, deployment)
        .expect("Deployment consumes the validated NativeLink ConfigMap");
    let config_text = config_map
        .get("data")
        .and_then(|data| data.get("cas.json"))
        .and_then(YamlValue::as_str)
        .expect("NativeLink cas.json");
    validate_deployment_config_digest(config_text, deployment)
        .expect("NativeLink pod template rolls when cas.json changes");
    let config: Value = serde_json::from_str(config_text).expect("parse NativeLink cas.json");
    validate_deployment_data_binding(&config, pvc, deployment)
        .expect("NativeLink slow stores bind the durable data PVC");
    let servers = config["servers"].as_array().expect("NativeLink servers");
    validate_server_names(servers).expect("NativeLink server names are unique and exact");
    let ops = servers
        .iter()
        .find(|server| server["name"].as_str() == Some("ops"))
        .expect("NativeLink ops server");
    validate_ops_health_binding(ops, deployment)
        .expect("NativeLink ops listener matches its container port and probes");
    let ops_services = ops["services"]
        .as_object()
        .expect("NativeLink ops services")
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        ops_services,
        BTreeSet::from(["admin", "health"]),
        "plaintext ops listener must never expose cache services"
    );
    let ingress_policy = documents
        .iter()
        .find(|document| {
            document.get("kind").and_then(YamlValue::as_str) == Some("NetworkPolicy")
                && document
                    .get("metadata")
                    .and_then(|metadata| metadata.get("name"))
                    .and_then(YamlValue::as_str)
                    == Some("nativelink-cas-ingress")
        })
        .expect("NativeLink ingress NetworkPolicy");
    validate_workload_selectors(deployment, ingress_policy)
        .expect("Deployment and ingress policy select the NativeLink workload");
    let mut role_ports = Vec::new();
    let mut service_documents = Vec::new();

    for (role, endpoint, read_only) in [
        ("writer", endpoints.writer(), false),
        ("reader", endpoints.reader(), true),
    ] {
        let service_name = format!("nativelink-cas-{role}");
        let service = documents
            .iter()
            .find(|document| {
                document.get("kind").and_then(YamlValue::as_str) == Some("Service")
                    && document
                        .get("metadata")
                        .and_then(|metadata| metadata.get("name"))
                        .and_then(YamlValue::as_str)
                        == Some(service_name.as_str())
            })
            .unwrap_or_else(|| panic!("NativeLink {role} Service"));
        service_documents.push((role, service));
        validate_service_exposure(service, role).unwrap_or_else(|finding| panic!("{finding}"));
        validate_service_selector(service, deployment, role)
            .unwrap_or_else(|finding| panic!("{finding}"));
        let namespace = service["metadata"]["namespace"]
            .as_str()
            .expect("Service namespace");
        let port = validated_service_port(service, role).expect("validated Service port");
        role_ports.push((role, port));
        assert_eq!(
            endpoint.socket_address(),
            format!("{service_name}.{namespace}.svc.cluster.local:{port}")
        );

        let server = servers
            .iter()
            .find(|server| server["name"].as_str() == Some(role))
            .unwrap_or_else(|| panic!("NativeLink {role} server"));
        validate_listener_tls(server, role).unwrap_or_else(|finding| panic!("{finding}"));
        validate_cache_store_bindings(server, role).unwrap_or_else(|finding| panic!("{finding}"));
        let cache_services = server["services"]
            .as_object()
            .unwrap_or_else(|| panic!("{role} services"))
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            cache_services,
            BTreeSet::from(["ac", "bytestream", "capabilities", "cas"]),
            "{role} must expose only the validated cache service set"
        );
        assert_eq!(
            server["listener"]["http"]["socket_address"]
                .as_str()
                .expect("listener socket"),
            format!("0.0.0.0:{port}")
        );
        for service_kind in ["cas", "ac", "capabilities", "bytestream"] {
            let instances = server["services"][service_kind]
                .as_array()
                .unwrap_or_else(|| panic!("{role} {service_kind} instances"));
            assert!(!instances.is_empty(), "{role} {service_kind} is empty");
            assert!(
                instances.iter().all(|instance| {
                    instance["instance_name"].as_str() == Some(endpoints.instance_name())
                }),
                "{role} {service_kind} instance drifted from endpoint DATA"
            );
        }
        assert!(
            server["services"]["ac"]
                .as_array()
                .expect("AC instances")
                .iter()
                .all(|instance| instance["read_only"].as_bool() == Some(read_only)),
            "{role} AC posture drifted"
        );
    }
    let service_namespace =
        common_service_namespace(&service_documents).expect("common cache Service namespace");
    let runner_values = std::fs::read_to_string(root.join(RUNNER_VALUES_PATH))
        .expect("read runner scale-set values");
    let runner_values: YamlValue =
        serde_yaml::from_str(&runner_values).expect("parse runner scale-set values");
    validate_runner_role_labels(&runner_values, &["writer", "reader"])
        .expect("runner template carries both cache role labels");
    let cluster_store = std::fs::read_to_string(root.join(EXTERNAL_SECRET_STORE_PATH))
        .expect("read OpenBao ClusterSecretStore");
    let cluster_stores = serde_yaml::Deserializer::from_str(&cluster_store)
        .map(|document| {
            YamlValue::deserialize(document).expect("parse OpenBao ClusterSecretStore document")
        })
        .collect::<Vec<_>>();
    let cluster_store = cluster_stores
        .iter()
        .find(|document| {
            document["kind"].as_str() == Some("ClusterSecretStore")
                && document["metadata"]["name"].as_str() == Some("openbao-oya")
        })
        .expect("live openbao-oya ClusterSecretStore");
    validate_external_secret_store(external_secret, cluster_store)
        .expect("NativeLink ExternalSecret references the live OpenBao store");
    validate_deployment_runtime_binding(
        external_secret,
        deployment,
        &service_namespace,
        &role_ports,
    )
    .expect("Deployment exposes cache ports and consumes the TLS Secret");

    let runner_policies = std::fs::read_to_string(root.join(RUNNER_NETWORK_POLICY_PATH))
        .expect("read runner NetworkPolicies");
    let runner_policies = serde_yaml::Deserializer::from_str(&runner_policies)
        .map(|document| YamlValue::deserialize(document).expect("parse runner NetworkPolicy"))
        .collect::<Vec<_>>();
    let runner_egress_policy = runner_policies
        .iter()
        .find(|document| {
            document.get("kind").and_then(YamlValue::as_str) == Some("NetworkPolicy")
                && document
                    .get("metadata")
                    .and_then(|metadata| metadata.get("name"))
                    .and_then(YamlValue::as_str)
                    == Some("ci-runners-egress-allowlist")
        })
        .expect("runner egress NetworkPolicy");
    validate_endpoint_network_policy_ports(
        ingress_policy,
        runner_egress_policy,
        &service_namespace,
        &role_ports,
    )
    .expect("NetworkPolicy namespace and ports match endpoint DATA");

    let runbook = std::fs::read_to_string(root.join(EXTERNAL_SECRETS_RUNBOOK_PATH))
        .expect("read external-secrets runbook");
    let expected_sans = [
        endpoints.writer().socket_address(),
        endpoints.reader().socket_address(),
    ]
    .map(|address| {
        let (host, _) = address
            .rsplit_once(':')
            .expect("validated endpoint socket address");
        format!("DNS:{host}")
    })
    .into_iter()
    .collect::<HashSet<_>>();
    validate_server_san_preflight(&runbook, &expected_sans)
        .expect("server certificate SAN preflight matches endpoint DATA");
}

#[test]
fn endpoint_conformance_rejects_target_port_server_san_and_network_policy_drift() {
    let service: YamlValue =
        serde_yaml::from_str("spec:\n  ports:\n    - port: 50051\n      targetPort: 50052\n")
            .expect("parse Service fixture");
    assert!(
        validated_service_port(&service, "writer")
            .unwrap_err()
            .contains("disagrees")
    );
    let udp_service: YamlValue = serde_yaml::from_str(
        "spec:\n  ports:\n    - protocol: UDP\n      port: 50051\n      targetPort: 50051\n",
    )
    .expect("parse UDP Service fixture");
    assert!(
        validated_service_port(&udp_service, "writer")
            .unwrap_err()
            .contains("is not TCP")
    );

    let expected = [
        "DNS:writer.example.test".to_string(),
        "DNS:reader.example.test".to_string(),
    ]
    .into_iter()
    .collect::<HashSet<_>>();
    let stale_runbook = "cat >\"$tmp/server-sans.expected\" <<'EOF'\n\
                         DNS:writer.example.test\n\
                         DNS:stale.example.test\n\
                         EOF\n";
    assert!(
        validate_server_san_preflight(stale_runbook, &expected)
            .unwrap_err()
            .contains("disagrees")
    );

    let mut ingress_policy: YamlValue = serde_yaml::from_str(
        r#"
metadata:
  namespace: oya-ci
spec:
  policyTypes: [Ingress]
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: arc-runners
          podSelector:
            matchLabels:
              oya.io/nativelink-cas-writer: "true"
      ports: [{ protocol: TCP, port: 50051 }]
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: arc-runners
          podSelector:
            matchLabels:
              oya.io/nativelink-cas-reader: "true"
      ports: [{ protocol: TCP, port: 50052 }]
"#,
    )
    .expect("parse ingress policy fixture");
    let mut runner_egress_policy: YamlValue = serde_yaml::from_str(
        r#"
metadata:
  namespace: arc-runners
spec:
  podSelector:
    matchExpressions:
      - key: oya.io/ci-cell
        operator: In
        values: [general, live-postgres, future-console]
  policyTypes: [Egress]
  egress:
    - to:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: oya-ci
      ports:
        - { protocol: TCP, port: 50051 }
        - { protocol: TCP, port: 50052 }
        - { protocol: TCP, port: 50053 }
"#,
    )
    .expect("parse runner egress policy fixture");
    let role_ports = [("writer", 50051), ("reader", 50052)];
    validate_endpoint_network_policy_ports(
        &ingress_policy,
        &runner_egress_policy,
        "oya-ci",
        &role_ports,
    )
    .expect("matching NetworkPolicy fixture with an additive future egress port");

    ingress_policy["spec"]["policyTypes"][0] =
        serde_yaml::to_value("Egress").expect("serialize stale ingress direction");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("must be exactly `Ingress`")
    );
    ingress_policy["spec"]["policyTypes"][0] =
        serde_yaml::to_value("Ingress").expect("serialize restored ingress direction");
    runner_egress_policy["spec"]["policyTypes"][0] =
        serde_yaml::to_value("Ingress").expect("serialize stale egress direction");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("must be exactly `Egress`")
    );
    runner_egress_policy["spec"]["policyTypes"][0] =
        serde_yaml::to_value("Egress").expect("serialize restored egress direction");

    runner_egress_policy["spec"]["podSelector"]["matchExpressions"][0]["key"] =
        serde_yaml::to_value("oya.io/ci-cel").expect("serialize stale runner selector");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("oya.io/ci-cell")
    );
    runner_egress_policy["spec"]["podSelector"]["matchExpressions"][0]["key"] =
        serde_yaml::to_value("oya.io/ci-cell").expect("serialize restored runner selector");
    ingress_policy["spec"]["ingress"][0]["from"][0]["podSelector"]["matchLabels"]["oya.io/nativelink-cas-writer"] =
        serde_yaml::to_value("false").expect("serialize non-role peer");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("exposed to a non-role peer")
    );
    ingress_policy["spec"]["ingress"][0]["from"][0]["podSelector"]["matchLabels"]["oya.io/nativelink-cas-writer"] =
        serde_yaml::to_value("true").expect("serialize restored role peer");
    ingress_policy["spec"]["ingress"][0]["from"][0]["podSelector"]["matchExpressions"] =
        serde_yaml::from_str("[{ key: oya.io/ci-cell, operator: In, values: [nonexistent] }]")
            .expect("serialize stale role peer expression");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("must use only the exclusive role label")
    );
    ingress_policy["spec"]["ingress"][0]["from"][0]["podSelector"]
        .as_mapping_mut()
        .expect("role podSelector mapping")
        .remove(YamlValue::String("matchExpressions".to_string()));
    ingress_policy["spec"]["ingress"][0]["ports"][0]["endPort"] =
        serde_yaml::to_value(50052_u64).expect("serialize widened ingress range");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("must not widen")
    );
    ingress_policy["spec"]["ingress"][0]["ports"][0]
        .as_mapping_mut()
        .expect("ingress port mapping")
        .remove(YamlValue::String("endPort".to_string()));

    ingress_policy["spec"]["ingress"][0]["ports"][0]["port"] =
        serde_yaml::to_value(50050_u64).expect("serialize stale ingress port");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("writer ingress ports")
    );
    ingress_policy["spec"]["ingress"][0]["ports"][0]["port"] =
        serde_yaml::to_value(50051_u64).expect("serialize restored ingress port");
    ingress_policy["spec"]["ingress"][0]["ports"][0]["protocol"] =
        serde_yaml::to_value("UDP").expect("serialize stale ingress protocol");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("is not TCP")
    );
    ingress_policy["spec"]["ingress"][0]["ports"][0]["protocol"] =
        serde_yaml::to_value("TCP").expect("serialize restored ingress protocol");
    ingress_policy["spec"]["ingress"][0]["from"][0]["namespaceSelector"] = YamlValue::Null;
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("has no namespaceSelector")
    );
    ingress_policy["spec"]["ingress"][0]["from"][0]["namespaceSelector"] =
        serde_yaml::from_str("{}").expect("serialize all-namespaces selector");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("has no namespaceSelector.matchLabels")
    );
    ingress_policy["spec"]["ingress"][0]["from"][0]["namespaceSelector"] =
        serde_yaml::from_str("{ matchLabels: { kubernetes.io/metadata.name: arc-runners } }")
            .expect("restore exact runner namespace selector");
    ingress_policy["metadata"]["namespace"] =
        serde_yaml::to_value("other").expect("serialize stale ingress namespace");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("disagrees with Service namespace")
    );
    ingress_policy["metadata"]["namespace"] =
        serde_yaml::to_value("oya-ci").expect("serialize restored ingress namespace");
    runner_egress_policy["spec"]["egress"][0]["to"][0]["namespaceSelector"]["matchLabels"]["kubernetes.io/metadata.name"] =
        serde_yaml::to_value("other").expect("serialize stale egress namespace");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("no rule for namespace")
    );
    runner_egress_policy["spec"]["egress"][0]["to"][0]["namespaceSelector"]["matchLabels"]["kubernetes.io/metadata.name"] =
        serde_yaml::to_value("oya-ci").expect("serialize restored egress namespace");
    runner_egress_policy["spec"]["egress"][0]["to"]
        .as_sequence_mut()
        .expect("runner egress destinations")
        .push(
            serde_yaml::from_str("{ namespaceSelector: {} }")
                .expect("serialize all-namespaces egress destination"),
        );
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("exactly one destination")
    );
    runner_egress_policy["spec"]["egress"][0]["to"]
        .as_sequence_mut()
        .expect("runner egress destinations")
        .pop();
    runner_egress_policy["spec"]["egress"]
        .as_sequence_mut()
        .expect("runner egress rules")
        .push(
            serde_yaml::from_str(
                r#"
to:
  - namespaceSelector: {}
ports:
  - { protocol: TCP, port: 50051 }
"#,
            )
            .expect("serialize broad cache-port egress rule"),
        );
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("destination outside `oya-ci`")
    );
    runner_egress_policy["spec"]["egress"]
        .as_sequence_mut()
        .expect("runner egress rules")
        .pop();
    runner_egress_policy["spec"]["egress"]
        .as_sequence_mut()
        .expect("runner egress rules")
        .push(
            serde_yaml::from_str(
                r#"
to: []
ports:
  - { protocol: TCP, port: 50051 }
"#,
            )
            .expect("serialize empty-destination cache-port egress rule"),
        );
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("no destination peers")
    );
    runner_egress_policy["spec"]["egress"]
        .as_sequence_mut()
        .expect("runner egress rules")
        .pop();
    runner_egress_policy["spec"]["egress"][0]["ports"][0]["protocol"] =
        serde_yaml::to_value("UDP").expect("serialize stale egress protocol");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("is not TCP")
    );
    runner_egress_policy["spec"]["egress"][0]["ports"][0]["protocol"] =
        serde_yaml::to_value("TCP").expect("serialize restored egress protocol");
    runner_egress_policy["spec"]["egress"][0]["ports"][1]["port"] =
        serde_yaml::to_value(50054_u64).expect("serialize stale egress port");
    assert!(
        validate_endpoint_network_policy_ports(
            &ingress_policy,
            &runner_egress_policy,
            "oya-ci",
            &role_ports,
        )
        .unwrap_err()
        .contains("runner egress ports")
    );
}

#[test]
fn endpoint_conformance_rejects_namespace_selector_config_and_tls_drift() {
    let mut writer_service: YamlValue = serde_yaml::from_str(
        r#"
metadata: { namespace: oya-ci }
spec:
  selector: { app: nativelink-cas }
  ports: [{ port: 50051, targetPort: 50051 }]
"#,
    )
    .expect("parse writer Service fixture");
    let mut reader_service: YamlValue = serde_yaml::from_str(
        r#"
metadata: { namespace: oya-ci }
spec:
  selector: { app: nativelink-cas }
  ports: [{ port: 50052, targetPort: 50052 }]
"#,
    )
    .expect("parse reader Service fixture");
    let mut deployment: YamlValue = serde_yaml::from_str(
        r#"
metadata:
  name: nativelink-cas
  namespace: oya-ci
spec:
  replicas: 1
  strategy: { type: Recreate }
  selector:
    matchLabels: { app: nativelink-cas }
  template:
    metadata:
      labels: { app: nativelink-cas, additive: allowed }
      annotations: { oya.io/config-sha256: fixture }
    spec:
      containers:
        - name: nativelink
          args: [/etc/nativelink/cas.json]
          ports:
            - { containerPort: 50051 }
            - { containerPort: 50052 }
            - { name: ops, containerPort: 50061 }
          readinessProbe:
            httpGet: { path: /status, port: ops }
          livenessProbe:
            httpGet: { path: /status, port: ops }
          volumeMounts:
            - { name: config, mountPath: /etc/nativelink, readOnly: true }
            - { name: tls, mountPath: /tls, readOnly: true }
            - { name: data, mountPath: /data }
      volumes:
        - name: config
          configMap: { name: nativelink-cas-config }
        - name: tls
          secret: { secretName: nativelink-cas-tls }
        - name: data
          persistentVolumeClaim: { claimName: nativelink-cas-data }
"#,
    )
    .expect("parse Deployment fixture");
    let config_map: YamlValue = serde_yaml::from_str(
        r#"
metadata:
  name: nativelink-cas-config
  namespace: oya-ci
data:
  cas.json: "{}"
"#,
    )
    .expect("parse ConfigMap fixture");
    let fixture_config_text = config_map["data"]["cas.json"]
        .as_str()
        .expect("fixture cas.json");
    deployment["spec"]["template"]["metadata"]["annotations"]["oya.io/config-sha256"] =
        serde_yaml::to_value(format!(
            "{:x}",
            Sha256::digest(fixture_config_text.as_bytes())
        ))
        .expect("serialize fixture config digest");
    let mut external_secret: YamlValue = serde_yaml::from_str(
        r#"
metadata:
  namespace: oya-ci
spec:
  secretStoreRef: { name: openbao-oya, kind: ClusterSecretStore }
  target: { name: nativelink-cas-tls, creationPolicy: Owner }
  data:
    - secretKey: tls.crt
      remoteRef: { key: oya/ci/nativelink-cas-tls, property: server-cert }
    - secretKey: tls.key
      remoteRef: { key: oya/ci/nativelink-cas-tls, property: server-key }
    - secretKey: ca-writer.crt
      remoteRef: { key: oya/ci/nativelink-cas-tls, property: writer-client-ca }
    - secretKey: ca-reader.crt
      remoteRef: { key: oya/ci/nativelink-cas-tls, property: reader-client-ca }
"#,
    )
    .expect("parse ExternalSecret fixture");
    let mut ingress_target: YamlValue = serde_yaml::from_str(
        r#"
spec:
  podSelector:
    matchLabels: { app: nativelink-cas }
"#,
    )
    .expect("parse ingress target fixture");
    let mut runner_values: YamlValue = serde_yaml::from_str(
        r#"
template:
  metadata:
    labels:
      oya.io/ci-cell: general
      oya.io/nativelink-cas-writer: "true"
      oya.io/nativelink-cas-reader: "true"
"#,
    )
    .expect("parse runner values fixture");
    let cluster_store: YamlValue = serde_yaml::from_str(
        r#"
kind: ClusterSecretStore
metadata: { name: openbao-oya }
"#,
    )
    .expect("parse ClusterSecretStore fixture");
    let pvc: YamlValue = serde_yaml::from_str(
        r#"
metadata:
  name: nativelink-cas-data
  namespace: oya-ci
spec:
  accessModes: [ReadWriteOnce]
"#,
    )
    .expect("parse PVC fixture");
    let slow_store_config = json!({
        "stores": [
            {
                "name": "CAS_MAIN_STORE",
                "verify": {
                    "backend": {
                        "fast_slow": {
                            "slow": {
                                "filesystem": {
                                    "content_path": "/data/cas-content",
                                    "temp_path": "/data/cas-tmp"
                                }
                            }
                        }
                    }
                }
            },
            {
                "name": "AC_MAIN_STORE",
                "fast_slow": {
                    "slow": {
                        "filesystem": {
                            "content_path": "/data/ac-content",
                            "temp_path": "/data/ac-tmp"
                        }
                    }
                }
            }
        ]
    });
    let mut resource_documents = vec![
        serde_yaml::from_str(
            "kind: Service\nmetadata: { name: nativelink-cas-writer, namespace: oya-ci }\n",
        )
        .expect("parse unique writer Service identity"),
        serde_yaml::from_str(
            "kind: Service\nmetadata: { name: nativelink-cas-reader, namespace: oya-ci }\n",
        )
        .expect("parse unique reader Service identity"),
    ];

    assert_eq!(
        common_service_namespace(&[("writer", &writer_service), ("reader", &reader_service),])
            .expect("common Service namespace"),
        "oya-ci"
    );
    validate_service_selector(&writer_service, &deployment, "writer")
        .expect("writer selector matches pod labels");
    validate_service_selector(&reader_service, &deployment, "reader")
        .expect("reader selector matches pod labels");
    validate_service_exposure(&writer_service, "writer")
        .expect("writer Service is cluster-internal");
    validate_service_exposure(&reader_service, "reader")
        .expect("reader Service is cluster-internal");
    validate_workload_selectors(&deployment, &ingress_target)
        .expect("Deployment and ingress target selectors agree");
    validate_runner_role_labels(&runner_values, &["writer", "reader"])
        .expect("runner values carry both cache roles");
    validate_deployment_topology(&deployment).expect("singleton Recreate Deployment fixture");
    validate_external_secret_store(&external_secret, &cluster_store)
        .expect("ExternalSecret references fixture store");
    validate_deployment_config_binding(&config_map, &deployment)
        .expect("Deployment consumes ConfigMap fixture");
    validate_deployment_config_digest(fixture_config_text, &deployment)
        .expect("Deployment carries fixture ConfigMap digest");
    let role_ports = [("writer", 50051), ("reader", 50052)];
    validate_deployment_runtime_binding(&external_secret, &deployment, "oya-ci", &role_ports)
        .expect("Deployment exposes cache ports and mounts TLS fixture");
    validate_deployment_data_binding(&slow_store_config, &pvc, &deployment)
        .expect("Deployment mounts the durable data PVC fixture");
    validate_unique_kubernetes_identities(&resource_documents)
        .expect("resource identities are unique");
    resource_documents.push(resource_documents[0].clone());
    assert!(
        validate_unique_kubernetes_identities(&resource_documents)
            .unwrap_err()
            .contains("duplicate Kubernetes identity")
    );

    deployment["spec"]["template"]["metadata"]["annotations"]["oya.io/config-sha256"] =
        serde_yaml::to_value("stale").expect("serialize stale config digest");
    assert!(
        validate_deployment_config_digest(fixture_config_text, &deployment)
            .unwrap_err()
            .contains("disagrees")
    );
    deployment["spec"]["template"]["metadata"]["annotations"]["oya.io/config-sha256"] =
        serde_yaml::to_value(format!(
            "{:x}",
            Sha256::digest(fixture_config_text.as_bytes())
        ))
        .expect("serialize restored config digest");

    deployment["spec"]["template"]["spec"]["volumes"][2]["persistentVolumeClaim"]["claimName"] =
        serde_yaml::to_value("missing-claim").expect("serialize stale PVC claim");
    assert!(
        validate_deployment_data_binding(&slow_store_config, &pvc, &deployment)
            .unwrap_err()
            .contains("nativelink-cas-data")
    );
    deployment["spec"]["template"]["spec"]["volumes"][2]["persistentVolumeClaim"]["claimName"] =
        serde_yaml::to_value("nativelink-cas-data").expect("serialize restored PVC claim");
    deployment["spec"]["template"]["spec"]["containers"][0]["volumeMounts"][2]["mountPath"] =
        serde_yaml::to_value("/stale-data").expect("serialize stale data mount");
    assert!(
        validate_deployment_data_binding(&slow_store_config, &pvc, &deployment)
            .unwrap_err()
            .contains("mounted writable at `/data`")
    );
    deployment["spec"]["template"]["spec"]["containers"][0]["volumeMounts"][2]["mountPath"] =
        serde_yaml::to_value("/data").expect("serialize restored data mount");

    writer_service["spec"]["type"] =
        serde_yaml::to_value("LoadBalancer").expect("serialize exposed Service type");
    assert!(
        validate_service_exposure(&writer_service, "writer")
            .unwrap_err()
            .contains("cluster-internal")
    );
    writer_service["spec"]
        .as_mapping_mut()
        .expect("writer Service spec")
        .remove(YamlValue::String("type".to_string()));
    writer_service["spec"]["externalIPs"] =
        serde_yaml::from_str("[203.0.113.10]").expect("serialize external Service IP");
    assert!(
        validate_service_exposure(&writer_service, "writer")
            .unwrap_err()
            .contains("externalIPs")
    );
    writer_service["spec"]
        .as_mapping_mut()
        .expect("writer Service spec")
        .remove(YamlValue::String("externalIPs".to_string()));

    deployment["spec"]["replicas"] =
        serde_yaml::to_value(0_u64).expect("serialize stale replica count");
    assert!(
        validate_deployment_topology(&deployment)
            .unwrap_err()
            .contains("exactly one replica")
    );
    deployment["spec"]["replicas"] =
        serde_yaml::to_value(1_u64).expect("serialize restored replica count");
    deployment["spec"]["strategy"]["type"] =
        serde_yaml::to_value("RollingUpdate").expect("serialize stale Deployment strategy");
    assert!(
        validate_deployment_topology(&deployment)
            .unwrap_err()
            .contains("Recreate")
    );
    deployment["spec"]["strategy"]["type"] =
        serde_yaml::to_value("Recreate").expect("serialize restored Deployment strategy");
    external_secret["spec"]["secretStoreRef"]["name"] =
        serde_yaml::to_value("stale-store").expect("serialize stale SecretStore reference");
    assert!(
        validate_external_secret_store(&external_secret, &cluster_store)
            .unwrap_err()
            .contains("openbao-oya")
    );
    external_secret["spec"]["secretStoreRef"]["name"] =
        serde_yaml::to_value("openbao-oya").expect("serialize restored SecretStore reference");

    runner_values["template"]["metadata"]["labels"]["oya.io/nativelink-cas-writer"] =
        serde_yaml::to_value("false").expect("serialize stale runner writer label");
    assert!(
        validate_runner_role_labels(&runner_values, &["writer", "reader"])
            .unwrap_err()
            .contains("nativelink-cas-writer")
    );

    reader_service["metadata"]["namespace"] =
        serde_yaml::to_value("other").expect("serialize stale Service namespace");
    assert!(
        common_service_namespace(&[("writer", &writer_service), ("reader", &reader_service),])
            .unwrap_err()
            .contains("disagrees")
    );
    reader_service["metadata"]["namespace"] =
        serde_yaml::to_value("oya-ci").expect("serialize restored Service namespace");

    writer_service["spec"]["selector"]["app"] =
        serde_yaml::to_value("other").expect("serialize stale Service selector");
    assert!(
        validate_service_selector(&writer_service, &deployment, "writer")
            .unwrap_err()
            .contains("disagrees")
    );
    writer_service["spec"]["selector"]["app"] =
        serde_yaml::to_value("nativelink-cas").expect("serialize restored Service selector");

    deployment["spec"]["selector"]["matchLabels"]["tier"] =
        serde_yaml::to_value("cas").expect("serialize stale Deployment selector");
    assert!(
        validate_workload_selectors(&deployment, &ingress_target)
            .unwrap_err()
            .contains("does not match pod labels")
    );
    deployment["spec"]["selector"]["matchLabels"]
        .as_mapping_mut()
        .expect("Deployment selector mapping")
        .remove(YamlValue::String("tier".to_string()));
    deployment["spec"]["selector"]["matchExpressions"] =
        serde_yaml::from_str("[{ key: app, operator: In, values: [other-workload] }]")
            .expect("serialize stale Deployment selector expression");
    assert!(
        validate_workload_selectors(&deployment, &ingress_target)
            .unwrap_err()
            .contains("only exact matchLabels")
    );
    deployment["spec"]["selector"]
        .as_mapping_mut()
        .expect("Deployment selector mapping")
        .remove(YamlValue::String("matchExpressions".to_string()));
    ingress_target["spec"]["podSelector"]["matchLabels"]["app"] =
        serde_yaml::to_value("stale").expect("serialize stale ingress target");
    assert!(
        validate_workload_selectors(&deployment, &ingress_target)
            .unwrap_err()
            .contains("ingress target selector")
    );
    ingress_target["spec"]["podSelector"]["matchLabels"]["app"] =
        serde_yaml::to_value("nativelink-cas").expect("serialize restored ingress target");

    deployment["spec"]["template"]["spec"]["volumes"][0]["configMap"]["name"] =
        serde_yaml::to_value("stale-config").expect("serialize stale ConfigMap reference");
    assert!(
        validate_deployment_config_binding(&config_map, &deployment)
            .unwrap_err()
            .contains("does not reference")
    );
    deployment["spec"]["template"]["spec"]["volumes"][0]["configMap"]["name"] =
        serde_yaml::to_value("nativelink-cas-config")
            .expect("serialize restored ConfigMap reference");
    deployment["spec"]["template"]["spec"]["volumes"][0]["configMap"]["items"] =
        serde_yaml::from_str("[{ key: cas.json, path: stale.json }]")
            .expect("serialize stale ConfigMap path mapping");
    assert!(
        validate_deployment_config_binding(&config_map, &deployment)
            .unwrap_err()
            .contains("must not remap")
    );
    deployment["spec"]["template"]["spec"]["volumes"][0]["configMap"]
        .as_mapping_mut()
        .expect("ConfigMap volume mapping")
        .remove(YamlValue::String("items".to_string()));
    deployment["spec"]["template"]["spec"]["containers"][0]["volumeMounts"][0]["readOnly"] =
        serde_yaml::to_value(false).expect("serialize writable config mount");
    assert!(
        validate_deployment_config_binding(&config_map, &deployment)
            .unwrap_err()
            .contains("mounted read-only")
    );
    deployment["spec"]["template"]["spec"]["containers"][0]["volumeMounts"][0]["readOnly"] =
        serde_yaml::to_value(true).expect("serialize restored config mount");

    external_secret["spec"]["data"][3]["secretKey"] =
        serde_yaml::to_value("stale-reader-ca").expect("serialize stale ExternalSecret key");
    assert!(
        validate_deployment_runtime_binding(&external_secret, &deployment, "oya-ci", &role_ports)
            .unwrap_err()
            .contains("exactly one mapping")
    );
    external_secret["spec"]["data"][3]["secretKey"] =
        serde_yaml::to_value("ca-reader.crt").expect("serialize restored ExternalSecret key");
    let duplicate_mapping = external_secret["spec"]["data"][0].clone();
    external_secret["spec"]["data"]
        .as_sequence_mut()
        .expect("ExternalSecret data mappings")
        .push(duplicate_mapping);
    assert!(
        validate_deployment_runtime_binding(&external_secret, &deployment, "oya-ci", &role_ports)
            .unwrap_err()
            .contains("exactly one mapping")
    );
    external_secret["spec"]["data"]
        .as_sequence_mut()
        .expect("ExternalSecret data mappings")
        .pop();
    external_secret["spec"]["target"]["creationPolicy"] =
        serde_yaml::to_value("Merge").expect("serialize stale Secret creation policy");
    assert!(
        validate_deployment_runtime_binding(&external_secret, &deployment, "oya-ci", &role_ports)
            .unwrap_err()
            .contains("creationPolicy must be `Owner`")
    );
    external_secret["spec"]["target"]["creationPolicy"] =
        serde_yaml::to_value("Owner").expect("serialize restored Secret creation policy");
    external_secret["spec"]["data"][2]["remoteRef"]["property"] =
        serde_yaml::to_value("reader-client-ca").expect("serialize swapped writer CA property");
    assert!(
        validate_deployment_runtime_binding(&external_secret, &deployment, "oya-ci", &role_ports)
            .unwrap_err()
            .contains("writer-client-ca")
    );
    external_secret["spec"]["data"][2]["remoteRef"]["property"] =
        serde_yaml::to_value("writer-client-ca").expect("serialize restored writer CA property");
    external_secret["metadata"]["namespace"] =
        serde_yaml::to_value("other").expect("serialize stale ExternalSecret namespace");
    assert!(
        validate_deployment_runtime_binding(&external_secret, &deployment, "oya-ci", &role_ports)
            .unwrap_err()
            .contains("disagrees with Deployment namespace")
    );
    external_secret["metadata"]["namespace"] =
        serde_yaml::to_value("oya-ci").expect("serialize restored ExternalSecret namespace");
    deployment["metadata"]["namespace"] =
        serde_yaml::to_value("other").expect("serialize stale Deployment namespace");
    external_secret["metadata"]["namespace"] =
        serde_yaml::to_value("other").expect("serialize matching stale ExternalSecret namespace");
    assert!(
        validate_deployment_runtime_binding(&external_secret, &deployment, "oya-ci", &role_ports)
            .unwrap_err()
            .contains("disagrees with Service namespace")
    );
    deployment["metadata"]["namespace"] =
        serde_yaml::to_value("oya-ci").expect("serialize restored Deployment namespace");
    external_secret["metadata"]["namespace"] =
        serde_yaml::to_value("oya-ci").expect("serialize restored ExternalSecret namespace");

    deployment["spec"]["template"]["spec"]["volumes"][1]["secret"]["secretName"] =
        serde_yaml::to_value("stale-tls").expect("serialize stale TLS Secret");
    assert!(
        validate_deployment_runtime_binding(&external_secret, &deployment, "oya-ci", &role_ports)
            .unwrap_err()
            .contains("does not reference")
    );
    deployment["spec"]["template"]["spec"]["volumes"][1]["secret"]["secretName"] =
        serde_yaml::to_value("nativelink-cas-tls").expect("serialize restored TLS Secret");
    deployment["spec"]["template"]["spec"]["containers"][0]["volumeMounts"][1]["mountPath"] =
        serde_yaml::to_value("/tls-v2").expect("serialize stale TLS mount");
    assert!(
        validate_deployment_runtime_binding(&external_secret, &deployment, "oya-ci", &role_ports)
            .unwrap_err()
            .contains("mounted read-only at `/tls`")
    );
    deployment["spec"]["template"]["spec"]["containers"][0]["volumeMounts"][1]["mountPath"] =
        serde_yaml::to_value("/tls").expect("serialize restored TLS mount");
    deployment["spec"]["template"]["spec"]["containers"][0]["ports"][1]["containerPort"] =
        serde_yaml::to_value(50054_u64).expect("serialize stale container port");
    assert!(
        validate_deployment_runtime_binding(&external_secret, &deployment, "oya-ci", &role_ports)
            .unwrap_err()
            .contains("is not declared")
    );

    let valid_writer = json!({
        "listener": {
            "http": {
                "tls": {
                    "cert_file": "/tls/tls.crt",
                    "key_file": "/tls/tls.key",
                    "client_ca_file": "/tls/ca-writer.crt"
                }
            }
        },
        "services": {
            "cas": [{ "cas_store": "CAS_MAIN_STORE" }],
            "bytestream": [{ "cas_store": "CAS_MAIN_STORE" }],
            "ac": [{ "ac_store": "AC_MAIN_STORE" }]
        }
    });
    let mut valid_ops = json!({
        "listener": {
            "http": { "socket_address": "0.0.0.0:50061" }
        }
    });
    validate_ops_health_binding(&valid_ops, &deployment)
        .expect("ops listener matches fixture probes");
    valid_ops["listener"]["http"]["socket_address"] = Value::String("0.0.0.0:50062".to_string());
    assert!(
        validate_ops_health_binding(&valid_ops, &deployment)
            .unwrap_err()
            .contains("50061")
    );
    let mut server_names = vec![
        json!({ "name": "writer" }),
        json!({ "name": "reader" }),
        json!({ "name": "ops" }),
    ];
    validate_server_names(&server_names).expect("unique server names");
    server_names.push(json!({ "name": "writer" }));
    assert!(
        validate_server_names(&server_names)
            .unwrap_err()
            .contains("exactly one each")
    );
    validate_listener_tls(&valid_writer, "writer").expect("writer listener TLS");
    validate_cache_store_bindings(&valid_writer, "writer")
        .expect("writer service stores are bound");
    let mut swapped_store = valid_writer.clone();
    swapped_store["services"]["cas"][0]["cas_store"] = Value::String("AC_MAIN_STORE".to_string());
    assert!(
        validate_cache_store_bindings(&swapped_store, "writer")
            .unwrap_err()
            .contains("CAS_MAIN_STORE")
    );
    swapped_store = valid_writer.clone();
    swapped_store["services"]["ac"][0]["ac_store"] = Value::String("CAS_MAIN_STORE".to_string());
    assert!(
        validate_cache_store_bindings(&swapped_store, "writer")
            .unwrap_err()
            .contains("AC_MAIN_STORE")
    );
    let mut swapped_ca = valid_writer.clone();
    swapped_ca["listener"]["http"]["tls"]["client_ca_file"] =
        Value::String("/tls/ca-reader.crt".to_string());
    assert!(
        validate_listener_tls(&swapped_ca, "writer")
            .unwrap_err()
            .contains("client_ca_file")
    );
    let mut missing_tls = valid_writer;
    missing_tls["listener"]["http"]["tls"] = Value::Null;
    assert!(
        validate_listener_tls(&missing_tls, "writer")
            .unwrap_err()
            .contains("listener TLS")
    );
}

#[test]
/// The SIBLING of root_buckconfig_stays_dark, which guards only `.buckconfig`.
///
/// `.buckconfig.local` is the ONLY mechanism that can wire the remote cache: buck2
/// resolves `[buck2_re_client]` into DaemonStartupConfig from project config files
/// ONLY, so `--config` / `--config-file` are inert for that section (measured). A
/// COMMITTED `.buckconfig.local` carrying warm-cache-rw content would therefore make
/// every build in the repo remote-cache-enabled with uploads on, bypassing the
/// resolver and the /specs/cache-warm-license.json kill-switch entirely — and would
/// poison the integrity canary, whose cold build depends on running with no overlay.
///
/// Deliberate asymmetry: `.buckconfig.d/` is NOT forbidden. Committed fragments there
/// are the FAIL-CLOSED way to ship real config, because a missing `--config-file`
/// path silently succeeds (BUILD SUCCEEDED, exit 0) while a committed fragment is
/// always read. This test bans the machine-local file, not the committed-fragment door.
fn buckconfig_local_is_ignored_and_untracked() {
    let root = repo_root();

    let gitignore = std::fs::read_to_string(root.join(".gitignore")).expect(
        "read .gitignore — it is the only thing keeping a warm-cache overlay uncommittable",
    );
    assert!(
        gitignore
            .lines()
            .any(|l| l.trim() == "/.buckconfig.local" || l.trim() == ".buckconfig.local"),
        "UNIGNORED CACHE OVERLAY: .gitignore must ignore .buckconfig.local. It is the only file \
         that can wire [buck2_re_client], so an unignored copy is one `git add -A` away from \
         enabling remote cache + uploads for every build in the repo, bypassing the resolver \
         and the warm-license kill-switch (ADR-0560 D6)"
    );

    let tracked = std::process::Command::new("git")
        .args(["ls-files", "--", ".buckconfig.local"])
        .current_dir(&root)
        .output()
        .expect("run git ls-files");
    assert!(
        String::from_utf8_lossy(&tracked.stdout).trim().is_empty(),
        "TRACKED CACHE OVERLAY: .buckconfig.local is committed. Remove it — its contents apply \
         to every buck2 invocation in this checkout, warm or cold, licensed or not"
    );
}

#[test]
fn root_buckconfig_stays_dark() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(".buckconfig")).expect("read root .buckconfig");
    let cfg = app::parse_buckconfig(&text);
    assert!(
        !cfg.contains_key("buck2_re_client"),
        "root .buckconfig grew a [buck2_re_client] section — cache wiring must stay opt-in \
         (ADR-0560 dark-wiring invariant)"
    );
    assert!(
        !cfg.contains_key("oya_cache"),
        "root .buckconfig grew an [oya_cache] section — cache wiring must stay opt-in"
    );
    assert_eq!(
        cfg["build"]["execution_platforms"], "prelude//platforms:default",
        "the default execution platform must stay the prelude default"
    );
}

#[test]
fn canary_workflow_is_scheduled_cold_and_wires_the_proof() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(CANARY_WORKFLOW_PATH)).unwrap_or_else(|e| {
        panic!(
            "read {CANARY_WORKFLOW_PATH}: {e} — the canary MUST ship \
                                    with the CAS wiring (ADR-0556 D2: no canary, no warm)"
        )
    });
    let schedule = std::fs::read_to_string(root.join(CANARY_SCHEDULE_WORKFLOW_PATH)).unwrap();
    assert!(
        schedule.contains("schedule:"),
        "canary must be cron-scheduled (ADR-0556 D4.3)"
    );
    assert!(
        !text.contains("actions/cache@") && !schedule.contains("actions/cache@"),
        "FROM-EMPTY VIOLATION: the canary workflow restores a cache — the proof is circular \
         (ADR-0556 D5 cold-must-stay)"
    );

    // THE WARM SIDE. Every assertion below this point in the original test covered the
    // COLD step only (--unstable-write-invocation-record + assert-cold), both of which
    // the cold build already satisfied — so the gate LOOKED like it guarded the canary
    // while never checking the half that could lie. canary_verdict compares
    // target->output-digest pairs, so a probe that fetched nothing and rebuilt locally
    // produces byte-identical digests, full overlap, zero divergence => GREEN, and that
    // GREEN licenses warm reads fleet-wide.
    assert!(
        text.contains("--isolation-dir canary-warm-probe")
            && text.contains("--unstable-write-invocation-record /tmp/canary-warm-record.json"),
        "WARM PROBE UNPROVEN: the probe build must write its OWN invocation record, or its \
         cache participation cannot be checked and a zero-fetch local rebuild emits GREEN \
         (ADR-0556 D2)"
    );
    assert!(
        text.contains("--warm /tmp/canary-warm-manifest.json")
            && text.contains("--warm-record /tmp/canary-warm-record.json"),
        "WARM MANIFEST ADMITTED WITHOUT PROOF: canary-verdict must receive --warm-record \
         alongside --warm so the probe's participation gates the comparison (ADR-0556 D2)"
    );
    assert!(
        text.contains("--unstable-write-invocation-record"),
        "canary must capture the structured invocation record"
    );
    assert!(
        text.contains("assert-cold"),
        "canary must mechanically prove zero cache participation (assert-cold)"
    );
    assert!(
        text.contains("integrity-canary"),
        "canary must run under the integrity-canary build class"
    );
    assert!(
        text.contains("canary-verdict"),
        "canary must emit the structured verdict artifact"
    );
}

#[test]
fn required_workflow_cache_hit_report_is_binding() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(REQUIRED_WORKFLOW_PATH)).unwrap_or_else(|e| {
        panic!(
            "read {REQUIRED_WORKFLOW_PATH}: {e} — the required CI workflow must ship the \
             cache-hit report guard"
        )
    });
    let telemetry_step = text
        .split("- name: Cache-hit telemetry + warm-mode guard (ADR-0560)")
        .nth(1)
        .and_then(|tail| {
            tail.split("- name: Upload cache-hit telemetry artifact")
                .next()
        })
        .expect("required workflow must contain the cache-hit telemetry guard step");
    assert!(
        telemetry_step.contains("--unstable-write-invocation-record")
            || text.contains(
                "--unstable-write-invocation-record /tmp/buck2-lane-invocation-record.json"
            ),
        "the buck2 lane must capture a structured invocation record before reporting cache health"
    );
    assert!(
        telemetry_step.contains(" report --record /tmp/buck2-lane-invocation-record.json")
            && telemetry_step.contains("--out /tmp/cache-hit-report.json"),
        "the cache-hit report must be generated from the structured invocation record"
    );
    assert!(
        telemetry_step.contains(" assert-warm --record /tmp/buck2-lane-invocation-record.json"),
        "warm/bypass cache participation must be asserted in the binding telemetry step"
    );
    assert!(
        !telemetry_step.contains("continue-on-error"),
        "the cache-hit telemetry guard must be binding; missing counters or 0% warm hits cannot pass"
    );

    // DELETED: three assertions that matched the `Upload cache-hit telemetry artifact` step's own
    // YAML literals (`name: cache-hit-report-buck2-lane`, `path: /tmp/cache-hit-report.json`,
    // `if-no-files-found: error`) and claimed they made the report "binding". They asserted
    // nothing. The upload step is `if: failure()`, so on a green lane it never runs; on a red
    // lane the job is already failing, so `if-no-files-found: error` cannot change any verdict
    // ever. `cache-hit-report-buck2-lane` also has ZERO consumers — it appears only in the
    // workflow that produces it and in this test — so the artifact going missing breaks nothing.
    // The step's own comment in oya-ci-required.yml says it outright: "`assert-warm` above is the
    // enforcing check; this upload never was." Those three asserts could only fail if somebody
    // edited the YAML, which converted "we have a gate for that" into false assurance.
    //
    // WHERE THE REAL ASSURANCE LIVES — do not re-add a YAML-literal check here:
    //   * that the report is PRODUCED and a cold/0%-hit warm lane goes RED: the binding
    //     `Cache-hit telemetry + warm-mode guard (ADR-0560)` step, which is `if: always()` and
    //     carries no `continue-on-error`. Its wiring is asserted above in THIS test; its
    //     behaviour is asserted directly against the kernel by
    //     `cache_hit_guard_behavior_covers_bypass_warm_and_malformed_records` below.
    //   * that a stale/missing invocation record cannot pass: `app::assert_warm_cache_participation`,
    //     exercised over bypass/warm/zero-hit/malformed records in that same test.
    // Artifact retention and upload success are runtime-only properties of a failure-path
    // diagnostic. A pure test cannot observe them, and nothing depends on them.
}

#[test]
fn workflows_use_the_local_config_controller_and_keep_the_cold_canary_absent() {
    let root = repo_root();
    let required = std::fs::read_to_string(root.join(REQUIRED_WORKFLOW_PATH)).unwrap();
    let required_words = required
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("\\ ", "");
    assert!(
        required_words.contains("-- run --build-class \"${CACHE_BUILD_CLASS}\" --mode-out /tmp/cache-mode -- buck2 test //ci/..."),
        "required CI must execute Buck2 as the controller child"
    );
    assert!(!required.contains("CACHE_MODE=bypass"));

    let canary = std::fs::read_to_string(root.join(CANARY_WORKFLOW_PATH)).unwrap();
    let cold = canary
        .split("- name: Cold from-empty build of the pinned target set")
        .nth(1)
        .and_then(|tail| tail.split("- name: Prove zero cache participation").next())
        .expect("cold canary step");
    assert!(!cold.contains("run --warm-probe"));
    assert!(!cold.contains(".buckconfig.local"));
    assert!(canary.contains(" -- run --workflow-mode"));
    assert!(canary.contains("--mode-out /tmp/canary-cache-mode -- buck2"));
    assert!(!canary.contains("--config-file infra/ci/buckconfig"));
    assert!(!canary.contains("--config \"buck2_re_client"));
}

#[test]
fn workflows_exchange_oidc_only_for_trusted_jobs_and_never_use_static_cert_secrets() {
    let root = repo_root();
    let controller =
        std::fs::read_to_string(root.join("ci/facade/build-cache-policy/src/main.rs")).unwrap();
    let bypass = controller
        .split("if resolution.mode == app::CacheMode::Bypass {")
        .nth(1)
        .and_then(|tail| {
            tail.split("let endpoints = app::load_endpoint_profile(root)?;")
                .next()
        })
        .expect("controlled_child bypass branch");
    let bypass_kill = bypass
        .find("kill_buck2(root, &isolation)?;")
        .expect("bypass daemon kill");
    let bypass_run = bypass
        .find("return run_child(root, &child_command)")
        .expect("bypass child run");
    assert!(
        bypass_kill < bypass_run,
        "a declared-cold child must kill its isolation before Buck2 starts"
    );
    for binding in [
        "ACTIONS_ID_TOKEN_REQUEST_URL",
        "/v1/auth/jwt/login",
        "/v1/{pki_mount}/issue/{pki_role}",
        "identity role, PKI mount, PKI role, and URI SAN do not match a trusted tuple",
        "CACHE_SERVER_CA_ENV",
        "write_private_file",
        ".connect_timeout(Duration::from_secs(10))",
        ".timeout(Duration::from_secs(30))",
        "oidc_authorization.set_sensitive(true)",
        "prove_identity_boundary",
        "rustls::Error::AlertReceived",
        "expected a typed peer alert before HTTP/2/gRPC",
        "Capabilities probe requires negotiated HTTP/2 and HTTP 200",
        "Capabilities response must contain exactly one grpc-status trailer",
        "assert_writer_seed_record",
    ] {
        assert!(
            controller.contains(binding),
            "missing identity exchange binding {binding}"
        );
    }
    let stop = controller.find("let stop = kill_buck2").unwrap();
    let remove = controller
        .find("let remove = app::remove_local_buckconfig")
        .unwrap();
    let combine = controller.find("match (stop, remove)").unwrap();
    assert!(
        stop < remove && remove < combine,
        "cache config removal must be attempted before cleanup errors propagate"
    );
    let required = std::fs::read_to_string(root.join(REQUIRED_WORKFLOW_PATH)).unwrap();
    let writer = required
        .split("  cache-writer-identity:")
        .nth(1)
        .and_then(|tail| tail.split("  gate-affected-target-set:").next())
        .expect("trusted writer identity job");
    assert!(writer.contains("github.event_name == 'push'") && writer.contains("refs/heads/dev"));
    assert!(writer.contains("vars.OYA_CAS_IDENTITY_PROOF_ENABLED == 'true'"));
    assert!(writer.contains("id-token: write"));
    assert!(writer.contains("uses: ./.github/workflows/cache-integrity-canary.yml"));
    assert!(writer.contains("writer_seed: true"));
    assert!(!writer.contains("run:"), "writer must add no inline shell");
    let fan_in = required
        .split("  oya-ci-required:")
        .nth(1)
        .expect("required fan-in");
    assert!(!fan_in.contains("needs.cache-writer-identity"));
    assert!(!fan_in.contains("- cache-writer-identity"));

    let untrusted = required
        .split("  buck2:")
        .nth(1)
        .and_then(|tail| tail.split("  cache-writer-identity:").next())
        .expect("untrusted buck2 job");
    assert!(untrusted.contains("CACHE_BUILD_CLASS: untrusted-author-presubmit"));
    assert!(!untrusted.contains("id-token: write"));
    assert!(!untrusted.contains("issue-identity"));

    let canary = std::fs::read_to_string(root.join(CANARY_WORKFLOW_PATH)).unwrap();
    let schedule = std::fs::read_to_string(root.join(CANARY_SCHEDULE_WORKFLOW_PATH)).unwrap();
    assert!(
        !canary.contains("id-token:"),
        "the reusable executor must inherit permissions from its caller so the cold call cannot elevate"
    );
    assert!(
        !schedule.contains("run:"),
        "the privilege-separating scheduler must only call the reviewed Rust-backed executor"
    );
    let cold = schedule
        .split("  cold:")
        .nth(1)
        .and_then(|tail| tail.split("  reader-identity:").next())
        .expect("contents-only cold job");
    assert!(cold.contains("permissions:") && cold.contains("contents: read"));
    assert!(!cold.contains("id-token:"));
    assert!(cold.contains("uses: ./.github/workflows/cache-integrity-canary.yml"));
    let reader = schedule
        .split("  reader-identity:")
        .nth(1)
        .expect("activation-gated reader job");
    assert!(reader.contains("vars.OYA_CAS_IDENTITY_PROOF_ENABLED == 'true'"));
    assert!(reader.contains("github.ref == 'refs/heads/dev'"));
    assert!(reader.contains("github.event_name == 'workflow_dispatch'"));
    assert!(reader.contains("needs.cold.result == 'success'"));
    assert!(reader.contains("needs.cold.outputs.warm_licensed == 'true'"));
    assert!(reader.contains("id-token: write"));
    assert!(reader.contains("actions: read"));
    assert!(reader.contains("reader_probe: true"));
    assert!(reader.contains("writer_run_id:"));
    assert!(canary.contains("workflow_call:"));
    assert!(!canary.contains("\n  schedule:"));
    assert!(!canary.contains("\n  workflow_dispatch:"));
    assert!(canary.contains("writer_seed:"));
    assert!(canary.contains("reader_probe:"));
    assert!(canary.contains("writer_run_id:"));
    assert!(
        canary.contains("--workflow-mode \"${{ inputs.writer_seed && 'writer' || 'reader' }}\"")
    );
    assert!(schedule.contains("vars.OYA_CAS_IDENTITY_PROOF_ENABLED == 'true'"));
    assert!(canary.contains("OYA_CACHE_TLS_SERVER_CA_CERT: /etc/nativelink/ca/ca.crt"));
    assert!(canary.contains("prelicense_probe:"));
    assert!(canary.contains("timeout-minutes: 120"));
    assert!(canary.contains("--prelicense-probe"));
    assert!(canary.contains("OYA_CACHE_TLS_CLIENT_CERT: /tmp/oya-cache-client.pem"));
    assert!(canary.contains("OYA_CACHE_TLS_CA_CERTS: /tmp/oya-cache-server-ca.pem"));
    assert!(!canary.contains("cas-identity-boundary-"));
    assert!(!canary.contains("${{ runner.temp }}"));
    assert!(!canary.contains("name: Exchange GitHub OIDC"));
    assert!(!canary.contains("name: Remove short-lived cache identity"));
    assert!(canary.contains("name: Download cold proof from the zero-OIDC invocation"));
    assert!(canary.contains("cache-integrity-cold"));
    assert!(canary.contains("name: cache-writer-${{ github.sha }}"));
    assert!(canary.contains("github-token: ${{ github.token }}"));
    assert!(canary.contains("repository: ${{ github.repository }}"));
    assert!(canary.contains("run-id: ${{ inputs.writer_run_id }}"));
    assert!(canary.contains("warm-proof --role"));
    assert!(canary.contains("/tmp/canary-writer-report.json"));
    assert!(canary.contains("/tmp/canary-writer-receipt.json"));
    assert!(canary.contains("--report-out \"${{ inputs.writer_seed && '/tmp/canary-writer-report.json' || '/tmp/canary-reader-report.json' }}\""));
    assert!(canary.contains("/tmp/canary-warm-record.json"));
    assert!(canary.contains("--writer-manifest"));
    assert!(canary.contains("--writer-run-id \"$WRITER_RUN_ID\""));
    assert!(canary.contains("/tmp/writer-proof/canary-writer-receipt.json"));
    let writer_upload = canary
        .split("      - name: Upload validated writer proof")
        .nth(1)
        .and_then(|tail| tail.split("      - name: Canary verdict").next())
        .expect("writer proof upload step");
    assert!(!writer_upload.contains("always()"));
    let reader_upload = canary
        .split("      - name: Upload canary artifacts")
        .nth(1)
        .expect("reader/canary artifact upload step");
    assert!(reader_upload.contains("/tmp/canary-warm-record.json"));
    assert!(reader_upload.contains("/tmp/canary-reader-report.json"));
    assert!(schedule.contains("writer_run_id:"));
    assert!(schedule.contains("default: \"\""));
    assert!(
        canary.contains("vars.OYA_CAS_IDENTITY_PROOF_ENABLED != 'true'"),
        "activation-off cold runs must execute the INACTIVE verdict and remain RED"
    );
    assert!(controller.contains("fixed_identity_options"));
    assert!(controller.contains("remove_identity_files"));
    assert!(controller.contains("github-cas-reader-integrity-canary"));
    assert!(controller.contains("github-cas-writer-dev-push"));
    for workflow in [&required, &canary, &schedule] {
        assert!(!workflow.contains("OYA_CACHE_WRITER_TLS_CLIENT_CERT_PATH"));
        assert!(!workflow.contains("OYA_CACHE_READER_TLS_CLIENT_CERT_PATH"));
        assert!(!workflow.contains("OYA_CACHE_TLS_CA_CERTS_PATH"));
    }
}

#[test]
fn live_postgres_coverage_remains_split_across_required_same_pod_jobs() {
    let root = repo_root();
    let required = std::fs::read_to_string(root.join(REQUIRED_WORKFLOW_PATH)).unwrap();
    assert_eq!(
        required.matches("  gate-live-postgres-adapters:").count(),
        1
    );
    assert_eq!(required.matches("  gate-live-postgres-facades:").count(), 1);
    assert!(required.contains("needs.gate-live-postgres-adapters"));
    assert!(required.contains("needs.gate-live-postgres-facades"));
    assert!(!required.contains("  gate-live-postgres:"));
    assert!(required.contains("buck2 test — durable adapters"));
    assert!(required.contains("buck2 test — durable facades"));
    assert!(required.contains("      - gate-live-postgres-adapters # #901:"));
    assert!(required.contains("      - gate-live-postgres-facades  # #901:"));
}

#[test]
fn required_workflow_never_archives_buck_out() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(REQUIRED_WORKFLOW_PATH))
        .unwrap_or_else(|e| panic!("read {REQUIRED_WORKFLOW_PATH}: {e}"));
    let violations = actions_cache_buck_out_violations(Some(&root), REQUIRED_WORKFLOW_PATH, &text);

    assert!(
        violations.is_empty(),
        "{}: {violations:?}",
        concat!(
            "UNSAFE RUNNER SNAPSHOT: the required workflow archives `buck-out`. Buck2's local ",
            "state and materialized outputs are runner-local and the archive can exhaust an ",
            "ephemeral runner during extraction before any binding test executes (ADR-0554 D10)"
        ),
    );
    assert!(
        !text.contains("runner-disk-reclaim-buck2.json"),
        "DEAD ARTIFACT: the retired owned-runner reclaim producer has no output to upload; remove its failure-only artifact path (ADR-0554 D10)"
    );
}

#[test]
fn buck_out_archive_guard_rejects_yaml_path_variants_and_renamed_steps() {
    for path_yaml in [
        "path: ./buck-out",
        "path: buck-out/v2/cache",
        "path: |\n              ~/.rustup\n              ./buck-out/v2/cache",
        "path:\n              - ~/.rustup\n              - buck-out/v2/cache",
        "path: ${{ github.workspace }}/buck-out",
        "path: .",
        "path: ${{ github.workspace }}/",
        "path: ${{ github.workspace }}/**",
        "path: ./**",
        "path: '**'",
        "path: '?uck-out'",
        "path: '[b]uck-out'",
        "path: 'buck-*'",
        "path: '!!buck-out'",
        "path: '! !buck-out'",
        "path: 'toolchain/../buck-out'",
        "path: \"${{ 'buck-out' }}\"",
        "path: \"${{ format('buck-{0}', 'out') }}\"",
        "path: '${{ github.workspace }}/${{ inputs.cache_path }}'",
        "path: 'safe/${{ inputs.cache_path }}'",
        "path: '${{ github.workspace }}/safe/${{ inputs.cache_path }}'",
        "path: '${{ github.workspace }}suffix'",
        "path: '!${{ inputs.cache_path }}'",
        "path: /home/runner/_work/oyatie/oyatie/buck-out",
        "path: /home/runner/_work/oyatie/oyatie",
        "path: /home/runner/_work/**",
        "path: ~/_work/oyatie/oyatie/buck-out",
        "path: ~/_work/**",
        "path: ~/**",
        "path: 'D:\\a\\oyatie\\oyatie\\buck-out'",
        "path: 'C:buck-out'",
        "path: '\\\\server\\share\\oyatie\\buck-out'",
    ] {
        let fixture = format!(
            "jobs:\n  renamed-job:\n    steps:\n      - name: Innocuous renamed step\n        uses: actions/cache/restore@pinned\n        with:\n          key: unrelated-key\n          {path_yaml}\n"
        );
        assert!(
            !actions_cache_buck_out_violations(None, "<fixture>", &fixture).is_empty(),
            "guard accepted forbidden YAML variant:\n{fixture}"
        );
    }

    let mixed_case = "jobs:\n  gate:\n    steps:\n      - uses: AcTiOnS/CaChE@pinned\n        with:\n          path: ./buck-out\n";
    assert!(
        !actions_cache_buck_out_violations(None, "<fixture>", mixed_case).is_empty(),
        "action repository casing must not bypass the guard"
    );

    for action in [
        "actions\\cache@pinned",
        "actions//cache@pinned",
        "actions/cache/@pinned",
        "actions\\cache\\restore@pinned",
        "actions//cache//save@pinned",
        "actions/cache/restore/@pinned",
        "actions/cache/save/@pinned",
    ] {
        let fixture = format!(
            "jobs:\n  gate:\n    steps:\n      - uses: {action}\n        with:\n          path: buck-out\n          key: fixture\n"
        );
        assert!(
            !actions_cache_buck_out_violations(None, "<fixture>", &fixture).is_empty(),
            "runner-equivalent external cache action reference bypassed the guard: {action:?}"
        );
    }

    let safe = "jobs:\n  gate:\n    steps:\n      - uses: actions/cache@pinned\n        with:\n          path: |\n            ~/.rustup/toolchains\n            ~/.rustup/update-hashes\n            toolchain-*\n            rustup-*\n            [rt]ustup-cache\n            tool chain-*\n            ${{ github.workspace }}/toolchain-*\n            !buck-out\n            # buck-out\n";
    assert!(
        actions_cache_buck_out_violations(None, "<fixture>", safe).is_empty(),
        "toolchain-only actions/cache must remain allowed"
    );
}

#[test]
fn buck_out_archive_guard_follows_local_composite_actions() {
    let fixture_root = std::env::temp_dir().join(format!(
        "oya-cache-composite-fixture-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed")
    ));
    let action_dir = fixture_root.join(".github/actions/cache-wrapper");
    let _ = std::fs::remove_dir_all(&fixture_root);
    std::fs::create_dir_all(&action_dir).expect("create local composite fixture");
    std::fs::write(
        action_dir.join("action.yml"),
        "name: cache wrapper\ndescription: fixture cache wrapper\ninputs:\n  cache_path:\n    description: cache path\n    required: true\nruns:\n  using: composite\n  steps:\n    - uses: ACTIONS/CACHE/SAVE@pinned\n      with:\n        path: '${{ inputs.cache_path }}'\n        key: fixture\n",
    )
    .expect("write local composite fixture");
    let workflow = "jobs:\n  gate:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.github/actions/cache-wrapper\n        with:\n          cache_path: buck-out\n";
    let violations = actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", workflow);
    assert!(
        !violations.is_empty(),
        "local composite action must not hide a forbidden checkout archive"
    );

    let benign_prefix_dir = fixture_root.join(".github/actions/cache");
    let at_named_dir = fixture_root.join(".github/actions/cache@wrapper");
    std::fs::create_dir_all(&benign_prefix_dir).expect("create benign prefix action fixture");
    std::fs::create_dir_all(&at_named_dir).expect("create at-named action fixture");
    std::fs::write(
        benign_prefix_dir.join("action.yml"),
        "name: benign prefix\ndescription: must not mask at-named sibling\nruns:\n  using: composite\n  steps:\n    - run: echo safe\n      shell: bash\n",
    )
    .expect("write benign prefix action fixture");
    std::fs::write(
        at_named_dir.join("action.yml"),
        "name: unsafe at-named wrapper\ndescription: runner resolves the at sign literally\nruns:\n  using: composite\n  steps:\n    - uses: actions/cache@pinned\n      with:\n        path: buck-out\n        key: fixture\n",
    )
    .expect("write at-named action fixture");
    let at_named_workflow = "jobs:\n  gate:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.github/actions/cache@wrapper\n";
    let at_named_violations =
        actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", at_named_workflow);
    assert!(
        !at_named_violations.is_empty(),
        "local action references containing `@` must resolve the literal runner path, not a benign truncated prefix"
    );

    let windows_at_named_workflow = "jobs:\n  gate:\n    runs-on: windows-latest\n    steps:\n      - uses: .\\.github\\actions\\cache@wrapper\n";
    let windows_at_named_violations = actions_cache_buck_out_violations(
        Some(&fixture_root),
        "<fixture>",
        windows_at_named_workflow,
    );
    assert!(
        !windows_at_named_violations.is_empty(),
        "Windows-form local action references must fail closed because their host interpretation is ambiguous"
    );

    let normalized_benign_dir = fixture_root.join(".github/actions/cache/wrapper");
    let literal_backslash_unsafe_dir = fixture_root.join(".github/actions/cache\\wrapper");
    std::fs::create_dir_all(&normalized_benign_dir)
        .expect("create normalized benign action fixture");
    std::fs::create_dir_all(&literal_backslash_unsafe_dir)
        .expect("create literal-backslash unsafe action fixture");
    std::fs::write(
        normalized_benign_dir.join("action.yml"),
        "name: normalized benign action\ndescription: must not mask the POSIX literal-backslash sibling\nruns:\n  using: composite\n  steps:\n    - run: echo safe\n      shell: bash\n",
    )
    .expect("write normalized benign action fixture");
    std::fs::write(
        literal_backslash_unsafe_dir.join("action.yml"),
        "name: literal-backslash unsafe action\ndescription: Linux runner preserves the interior backslash\nruns:\n  using: composite\n  steps:\n    - uses: actions/cache@pinned\n      with:\n        path: buck-out\n        key: fixture\n",
    )
    .expect("write literal-backslash unsafe action fixture");
    let cross_host_workflow = "jobs:\n  gate:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: ./.github/actions/cache\\wrapper\n";
    let cross_host_violations =
        actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", cross_host_workflow);
    std::fs::remove_dir_all(&fixture_root).expect("remove local composite fixture");
    assert!(
        !cross_host_violations.is_empty(),
        "host-sensitive interior backslashes must fail closed rather than inspect only the normalized benign action"
    );
}

#[test]
fn buck_out_archive_guard_follows_local_reusable_workflows() {
    let fixture_root = std::env::temp_dir().join(format!(
        "oya-cache-reusable-workflow-fixture-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed")
    ));
    let workflow_dir = fixture_root.join(".github/workflows");
    let _ = std::fs::remove_dir_all(&fixture_root);
    std::fs::create_dir_all(&workflow_dir).expect("create reusable workflow fixture");
    std::fs::write(
        workflow_dir.join("cache-wrapper.yml"),
        "name: cache wrapper\non:\n  workflow_call:\n    inputs:\n      cache_path:\n        required: true\n        type: string\njobs:\n  cache:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/cache@pinned\n        with:\n          path: '${{ inputs.cache_path }}'\n          key: fixture\n",
    )
    .expect("write reusable workflow fixture");
    let workflow = "name: caller\non: pull_request\njobs:\n  delegated-gate:\n    uses: ./.github/workflows/cache-wrapper.yml\n    with:\n      cache_path: buck-out\n";
    let violations = actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", workflow);
    assert!(
        !violations.is_empty(),
        "same-repository reusable workflow must not hide a forbidden checkout archive"
    );

    std::fs::write(
        workflow_dir.join("safe.yml"),
        "name: safe cache wrapper\non:\n  workflow_call:\njobs:\n  safe-cache:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/cache@pinned\n        with:\n          path: toolchain-*\n          key: fixture\n",
    )
    .expect("write safe reusable workflow fixture");
    let safe_workflow = "name: caller\non: pull_request\njobs:\n  delegated-gate:\n    uses: ./.github/workflows/safe.yml\n";
    assert!(
        actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", safe_workflow)
            .is_empty(),
        "safe same-repository reusable workflows must not produce false positives"
    );

    let external = "jobs:\n  delegated-gate:\n    uses: owner/repo/.github/workflows/cache.yml@0123456789abcdef\n";
    assert!(
        !actions_cache_buck_out_violations(Some(Path::new(".")), "<fixture>", external).is_empty(),
        "uninspected external reusable workflows must fail closed"
    );
    std::fs::remove_dir_all(&fixture_root).expect("remove reusable workflow fixture");
}

#[test]
fn buck_out_archive_guard_terminates_local_document_cycles() {
    let fixture_root = std::env::temp_dir().join(format!(
        "oya-cache-document-cycle-fixture-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed")
    ));
    let workflow_dir = fixture_root.join(".github/workflows");
    let _ = std::fs::remove_dir_all(&fixture_root);
    std::fs::create_dir_all(&workflow_dir).expect("create cycle fixture");
    std::fs::write(
        workflow_dir.join("cycle-a.yml"),
        "jobs:\n  delegated:\n    uses: ./.github/workflows/cycle-b.yml\n",
    )
    .expect("write first cycle fixture");
    std::fs::write(
        workflow_dir.join("cycle-b.yml"),
        "jobs:\n  delegated:\n    uses: ./.github/workflows/cycle-a.yml\n",
    )
    .expect("write second cycle fixture");
    let workflow = "jobs:\n  delegated:\n    uses: ./.github/workflows/cycle-a.yml\n";
    let violations = actions_cache_buck_out_violations(Some(&fixture_root), "<fixture>", workflow);
    std::fs::remove_dir_all(&fixture_root).expect("remove cycle fixture");
    assert!(
        violations.is_empty(),
        "cycle protection must terminate structural traversal: {violations:?}"
    );
}

#[test]
fn cache_hit_guard_behavior_covers_bypass_warm_and_malformed_records() {
    let bypass_zero = invocation_record_fixture(0.0, 0, 12, 0);
    assert!(
        app::assert_warm_cache_participation(&bypass_zero, "gate-fleet-shared-graph", "bypass")
            .is_ok(),
        "current bypass/cold posture must stay allowed even with zero cache hits"
    );

    let warm_hit = invocation_record_fixture(0.25, 3, 9, 0);
    assert!(
        app::assert_warm_cache_participation(&warm_hit, "gate-fleet-shared-graph", "warm-rw")
            .is_ok(),
        "warm mode with a positive hit rate and positive action-cache count must pass"
    );

    let warm_zero = invocation_record_fixture(0.0, 0, 12, 0);
    let findings =
        app::assert_warm_cache_participation(&warm_zero, "gate-fleet-shared-graph", "warm-rw")
            .unwrap_err();
    assert!(
        findings.iter().any(|f| f.contains("0% hit rate"))
            && findings
                .iter()
                .any(|f| f.contains("run_action_cache_count=0")),
        "warm mode with 0% hits must be RED: {findings:?}"
    );

    let malformed = json!({ "exit_result_name": "SUCCESS" });
    let findings =
        app::assert_warm_cache_participation(&malformed, "gate-fleet-shared-graph", "warm-rw")
            .unwrap_err();
    assert!(
        findings
            .iter()
            .any(|f| f.contains("record-shape violation")),
        "missing or renamed cache counters must be RED: {findings:?}"
    );
}

#[test]
fn bundled_canary_targets_stay_inside_the_binding_gate_cone() {
    let policy = app::canary_policy().expect("bundled canary policy");
    let targets = policy["pinned_targets"].as_array().unwrap();
    assert!(!targets.is_empty());
    for target in targets {
        let t = target.as_str().unwrap();
        assert!(
            t.starts_with("//"),
            "pinned target `{t}` must be a repo-anchored pattern"
        );

        // PATH LIVENESS — the assertion this test was missing, and the reason it never fired.
        // `!targets.is_empty()` above proves the ARRAY has entries; `starts_with("//")` proves each
        // is SHAPED like a pattern. Neither proves a pattern names anything that exists. This block
        // pinned `//cloud/cloud-ci/...` long after the gate-fleet move VACATED that tree, so the
        // pattern resolved to ZERO targets — and the canary that anchors the entire warm-cache/RE
        // trust chain (ADR-0556 D2 licensing, ADR-0612 D5 "no RE-covering canary, no RE") would
        // have built nothing and reported success having verified nothing.
        //
        // Checked as a PATH here, deliberately, not by shelling `buck2 targets`: this stays a pure
        // test, and a vacated root is exactly the reorg-move failure mode that got us. It does not
        // claim the pattern resolves to >=1 buck2 TARGET — the canary job's own from-empty build is
        // what proves that, and it cannot even start against a root that does not exist.
        // The FULL package prefix, not just the first segment. Checking only the first segment is
        // itself the bug this test exists to catch: `//cloud/cloud-ci/...` has root `cloud`, and
        // `cloud/` still exists as a legacy root, so a first-segment check passes while the
        // `cloud-ci` subtree it actually names is gone. Verified by restoring the vacated pattern
        // and watching a first-segment version of this assertion stay GREEN.
        let prefix = t
            .trim_start_matches('/')
            .split("/...")
            .next()
            .unwrap_or_default()
            .split(':')
            .next()
            .unwrap_or_default()
            .trim_end_matches('/');
        assert!(
            !prefix.is_empty(),
            "pinned target `{t}` has no resolvable package prefix"
        );
        let prefix_path = repo_root().join(prefix);
        assert!(
            prefix_path.is_dir(),
            "pinned canary target `{t}` names a package prefix that does not exist: {}. A move \
             vacated it and nothing noticed — the canary would build an EMPTY target set and pass. \
             Re-point the pattern at the tree the gates actually live in, or drop it.",
            prefix_path.display()
        );
    }
}
