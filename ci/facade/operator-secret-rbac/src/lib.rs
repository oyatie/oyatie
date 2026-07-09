//! # cloud-ci-operator-secret-bootstrap (GH #980 + GH #988)
//!
//! Least-privilege secret RBAC, declarative join-token bootstrap, and OpenBao/ESO scope isolation.
//! The original SVID-delivery operator (PR #793) defect class was an operator that PRODUCED one
//! Secret but held namespace-wide scoped verbs over every Secret in its namespace. GH #988 expands
//! the same security gate instead of adding another `oya-*` / `cloud-*` debt surface: ESO
//! ClusterSecretStore use must stay policy-scoped by consumer namespace + OpenBao key prefix, and a
//! plaintext OpenBao listener must be protected by a concrete NetworkPolicy.
//!
//! ## Invariants, all DATA-driven by `operator-secret-bootstrap-policy.json`
//! 1. **Name-scoped secret writes.** Kubernetes RBAC unconditionally honors `resourceNames` for the
//!    single-object verbs `get/update/patch/delete` (the `scoped_secret_verbs`). `create` (and
//!    `deletecollection`) can never be `resourceNames`-scoped (the object name is unknown at
//!    authorization time); `list/watch` can be, but only when the client sends a matching
//!    `metadata.name` field selector. So any Role rule that grants a scoped verb on `secrets` MUST
//!    carry `resourceNames` bound to exactly the operator's `produced_secret_name`. `list/watch/create`
//!    may stay namespace-wide — that is the documented RBAC floor (narrowable once the operator
//!    binary guarantees a field selector), justified in the chart, not a gate concern.
//! 2. **Bootstrap-Secret provisioning.** The operator-internal bootstrap Secret (the join token,
//!    named by `join_token_values_ref`) must be either provisioned declaratively (an `ExternalSecret`
//!    / `SealedSecret` / `Secret` template referencing that values path) OR guarded by a fail-closed
//!    Helm preflight (`fail` referencing the join-token values group). An operator enabled with an
//!    unprovisioned, unvalidated bootstrap Secret fails closed.
//! 3. **ESO store scope.** Every committed `ExternalSecret` remote key that references a governed
//!    store in the policy scan roots must match a policy-declared `(store kind, store name, bound
//!    OpenBao role, consumer namespace, remoteRef.key prefix)` tuple. A store with an `oya/ci/`
//!    OpenBao role cannot project `cloud-k8s/csi/*`, and the cloud-k8s CSI store cannot be consumed
//!    outside `cloud-k8s-system`.
//! 4. **OpenBao transport isolation.** If the OpenBao listener is plaintext (`tls_disable = true`),
//!    the committed manifest must include a NetworkPolicy selecting the OpenBao workload and
//!    restricting ingress for the declared listener ports. Plain HTTP without a concrete isolation
//!    artifact is RED.
//!
//! ## Kernel contract
//! - [`collect_operators`] `(root, policy) -> observed` is the ONLY I/O: read-only `fs` reads of the
//!   governed RBAC templates, ExternalSecret manifests / values-backed template projections, and
//!   OpenBao manifest. No shell, no network, no VCS.
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without a
//!   filesystem.
//! - [`evaluate`] is the bare-code projection of [`evaluate_keyed`], the single source of the verdict.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `OSB-SECRET-RBAC-OVERBROAD`             — a secrets Role rule grants a scoped verb without any
//!   `resourceNames` (namespace-wide get/update/patch/delete).
//! - `OSB-SECRET-RBAC-RESOURCENAME-MISMATCH` — a scoped secrets rule carries `resourceNames`, but the
//!   set is not exactly `{produced_secret_name}` (scopes the wrong / extra Secret).
//! - `OSB-JOIN-TOKEN-UNPROVISIONED`          — the bootstrap Secret has neither a declarative
//!   provisioning template nor a fail-closed preflight in the chart.
//! - `OSB-ESO-STORE-UNDECLARED`              — an `ExternalSecret` references a store not declared in
//!   policy DATA, or policy DATA names a store absent from the committed ClusterSecretStore manifests.
//! - `OSB-ESO-STORE-ROLE-MISMATCH`           — a ClusterSecretStore's Kubernetes auth role differs
//!   from the role declared for that store's namespace/key-prefix contract.
//! - `OSB-ESO-REMOTE-KEY-OUT-OF-SCOPE`       — an `ExternalSecret` uses the wrong namespace or
//!   remoteRef.key prefix for its store role.
//! - `OSB-OPENBAO-TRANSPORT-UNISOLATED`      — plaintext OpenBao listener lacks restrictive
//!   NetworkPolicy coverage for the declared ports.
//! - `OSB-POLICY-GATE-ID-MISMATCH`           — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `OSB-POLICY-MALFORMED`                  — required policy lists are missing/malformed
//!   (fail-closed: the gate would have nothing to enforce).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-operator-secret-bootstrap";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 9] = [
    "OSB-SECRET-RBAC-OVERBROAD",
    "OSB-SECRET-RBAC-RESOURCENAME-MISMATCH",
    "OSB-JOIN-TOKEN-UNPROVISIONED",
    "OSB-ESO-STORE-UNDECLARED",
    "OSB-ESO-STORE-ROLE-MISMATCH",
    "OSB-ESO-REMOTE-KEY-OUT-OF-SCOPE",
    "OSB-OPENBAO-TRANSPORT-UNISOLATED",
    "OSB-POLICY-GATE-ID-MISMATCH",
    "OSB-POLICY-MALFORMED",
];

/// The sentinel key for policy-level (non-per-operator) findings.
const POLICY_KEY: &str = "<policy>";

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only, hermetic — no shell / network / VCS)
// ---------------------------------------------------------------------------

/// Errors collecting the observed view. Returned instead of panicking so the caller decides how to
/// surface them — an unreadable chart or a malformed RBAC template is a fail-closed error, never a
/// silently skipped operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    /// A read-only filesystem operation failed (a governed chart could not be scanned).
    Io(String),
    /// An RBAC template did not parse as YAML after Helm-action neutralization (fail-closed: an
    /// unparseable template could hide an overbroad rule).
    Parse(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::Io(message) => write!(f, "operator-secret-bootstrap io: {message}"),
            CollectError::Parse(message) => write!(f, "operator-secret-bootstrap parse: {message}"),
        }
    }
}

impl std::error::Error for CollectError {}

/// One secrets RBAC rule reduced to the fields this gate audits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretRule {
    /// The rule's `verbs`.
    pub verbs: Vec<String>,
    /// The rule's `resourceNames` (empty when the rule is namespace-wide).
    pub resource_names: Vec<String>,
}

/// The governed-operator descriptors declared in policy DATA.
fn operators(policy: &Value) -> Option<Vec<Value>> {
    policy
        .get("operators")
        .and_then(Value::as_array)
        .map(|list| list.to_vec())
}

fn str_field<'a>(operator: &'a Value, key: &str) -> Option<&'a str> {
    operator.get(key).and_then(Value::as_str)
}

/// Collect the observed view the policy asks about. Read-only — NO shell, NO network, NO VCS.
///
/// For each governed operator:
/// 1. Parse the RBAC template's `secrets` Role rules (verbs + resourceNames) — the only structural
///    YAML parse, over the static rules block after neutralizing the Helm actions.
/// 2. Shallow-scan the chart templates dir for join-token provisioning evidence: a provisioning-kind
///    doc (`ExternalSecret`/`SealedSecret`/`Secret`) referencing the join-token values group, and a
///    fail-closed Helm preflight (`fail` referencing that group).
///
/// When the policy carries GH #988 sections, this also scans the declared ExternalSecret manifests
/// and the OpenBao manifest. The return shape is additive so the original GH #980 tests and callers
/// keep using the same `operators` field.
pub fn collect_operators(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let descriptors = operators(policy).unwrap_or_default();
    let mut out = Vec::new();
    for descriptor in &descriptors {
        let Some(name) = str_field(descriptor, "name") else {
            continue;
        };
        let rbac_rel = str_field(descriptor, "rbac_template").unwrap_or_default();
        let templates_rel = str_field(descriptor, "chart_templates_dir").unwrap_or_default();
        let produced = str_field(descriptor, "produced_secret_name").unwrap_or_default();
        let join_ref = str_field(descriptor, "join_token_values_ref").unwrap_or_default();

        let rbac_path = root.join(rbac_rel);
        let rbac_text = fs::read_to_string(&rbac_path)
            .map_err(|e| CollectError::Io(format!("read {}: {e}", rbac_path.display())))?;
        let rules = parse_secret_rules(&rbac_text)
            .map_err(|e| CollectError::Parse(format!("{}: {e}", rbac_path.display())))?;

        let (has_provisioning, has_preflight) =
            scan_join_token_evidence(&root.join(templates_rel), join_ref)?;

        out.push(json!({
            "name": name,
            "produced_secret_name": produced,
            "secret_rules": rules
                .iter()
                .map(|r| json!({ "verbs": r.verbs, "resource_names": r.resource_names }))
                .collect::<Vec<_>>(),
            "has_provisioning_template": has_provisioning,
            "has_failclosed_preflight": has_preflight,
        }));
    }

    let mut observed = json!({ "operators": out });
    if policy.get("external_secret_scopes").is_some() {
        observed["external_secrets"] = json!(collect_external_secret_refs(root, policy)?);
        observed["cluster_secret_stores"] = json!(collect_cluster_secret_stores(root, policy)?);
    }
    if policy.get("openbao_transport").is_some() {
        observed["openbao_transport"] =
            collect_openbao_transport(root, policy)?.unwrap_or(Value::Null);
    }
    Ok(observed)
}

fn collect_external_secret_refs(root: &Path, policy: &Value) -> Result<Vec<Value>, CollectError> {
    let governed_stores = governed_external_secret_store_names(policy);
    let mut manifest_paths = BTreeSet::new();
    for scope in policy
        .get("external_secret_scopes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        for path in json_str_seq(scope.get("manifest_paths")) {
            manifest_paths.insert(path);
        }
    }
    for scan_root in json_str_seq(policy.get("external_secret_scan_roots")) {
        collect_yaml_manifest_paths(root, &scan_root, &mut manifest_paths)?;
    }

    let mut refs = Vec::new();
    for rel in manifest_paths {
        let path = root.join(&rel);
        let text = fs::read_to_string(&path)
            .map_err(|e| CollectError::Io(format!("read {}: {e}", path.display())))?;
        for doc in parse_yaml_documents(&text, &rel)? {
            if doc.get("kind").and_then(serde_yaml::Value::as_str) != Some("ExternalSecret") {
                continue;
            }
            let store_name =
                yaml_str_at(&doc, &["spec", "secretStoreRef", "name"]).unwrap_or_default();
            if !governed_stores.contains(&store_name) {
                continue;
            }
            let name = yaml_str_at(&doc, &["metadata", "name"]).unwrap_or_default();
            let namespace = yaml_str_at(&doc, &["metadata", "namespace"]).unwrap_or_default();
            let store_kind = yaml_str_at(&doc, &["spec", "secretStoreRef", "kind"])
                .unwrap_or_else(|| "SecretStore".to_owned());
            let remote_keys = external_secret_remote_keys(&doc);
            refs.push(json!({
                "path": rel,
                "name": name,
                "namespace": namespace,
                "store_name": store_name,
                "store_kind": store_kind,
                "remote_keys": remote_keys,
            }));
        }
    }
    refs.extend(collect_value_template_external_secret_refs(root, policy)?);
    Ok(refs)
}

fn governed_external_secret_store_names(policy: &Value) -> BTreeSet<String> {
    policy
        .get("external_secret_scopes")
        .and_then(Value::as_array)
        .map(|scopes| {
            scopes
                .iter()
                .filter_map(|scope| scope.get("store_name").and_then(Value::as_str))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn collect_value_template_external_secret_refs(
    root: &Path,
    policy: &Value,
) -> Result<Vec<Value>, CollectError> {
    let mut refs = Vec::new();
    for descriptor in policy
        .get("external_secret_value_templates")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let values_rel = descriptor
            .get("values_path")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let values_path = root.join(values_rel);
        let values_text = fs::read_to_string(&values_path)
            .map_err(|e| CollectError::Io(format!("read {}: {e}", values_path.display())))?;
        let values: serde_yaml::Value = serde_yaml::from_str(&values_text)
            .map_err(|e| CollectError::Parse(format!("{values_rel}: yaml parse: {e}")))?;

        if let Some(enabled_path) = descriptor.get("enabled_path").and_then(Value::as_str)
            && yaml_bool_dotted(&values, enabled_path) == Some(false)
        {
            continue;
        }

        let remote_keys = json_str_seq(descriptor.get("remote_key_paths"))
            .iter()
            .filter_map(|path| yaml_str_dotted(&values, path))
            .collect::<Vec<_>>();

        refs.push(json!({
            "path": descriptor.get("path").and_then(Value::as_str).unwrap_or_default(),
            "name": descriptor
                .get("name_path")
                .and_then(Value::as_str)
                .and_then(|path| yaml_str_dotted(&values, path))
                .unwrap_or_default(),
            "namespace": descriptor
                .get("namespace_path")
                .and_then(Value::as_str)
                .and_then(|path| yaml_str_dotted(&values, path))
                .unwrap_or_default(),
            "store_name": descriptor
                .get("store_name_path")
                .and_then(Value::as_str)
                .and_then(|path| yaml_str_dotted(&values, path))
                .unwrap_or_default(),
            "store_kind": descriptor
                .get("store_kind_path")
                .and_then(Value::as_str)
                .and_then(|path| yaml_str_dotted(&values, path))
                .unwrap_or_else(|| "SecretStore".to_owned()),
            "remote_keys": remote_keys,
        }));
    }
    Ok(refs)
}

fn yaml_str_dotted(value: &serde_yaml::Value, dotted: &str) -> Option<String> {
    yaml_at_dotted(value, dotted).and_then(|value| value.as_str().map(str::to_owned))
}

fn yaml_bool_dotted(value: &serde_yaml::Value, dotted: &str) -> Option<bool> {
    yaml_at_dotted(value, dotted).and_then(serde_yaml::Value::as_bool)
}

fn yaml_at_dotted<'a>(value: &'a serde_yaml::Value, dotted: &str) -> Option<&'a serde_yaml::Value> {
    let mut cursor = value;
    for segment in dotted.split('.') {
        cursor = cursor.get(segment)?;
    }
    Some(cursor)
}

fn collect_yaml_manifest_paths(
    root: &Path,
    rel: &str,
    out: &mut BTreeSet<String>,
) -> Result<(), CollectError> {
    let path = root.join(rel);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|e| CollectError::Io(format!("metadata {}: {e}", path.display())))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            out.insert(rel.to_owned());
        }
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }

    let mut entries = fs::read_dir(&path)
        .map_err(|e| CollectError::Io(format!("read dir {}: {e}", path.display())))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CollectError::Io(format!("read dir entry {}: {e}", path.display())))?;
    entries.sort();
    for entry in entries {
        let child_rel = entry
            .strip_prefix(root)
            .map_err(|e| CollectError::Io(format!("strip prefix {}: {e}", entry.display())))?
            .to_string_lossy()
            .into_owned();
        collect_yaml_manifest_paths(root, &child_rel, out)?;
    }
    Ok(())
}

fn collect_cluster_secret_stores(root: &Path, policy: &Value) -> Result<Vec<Value>, CollectError> {
    let mut manifest_paths = BTreeSet::new();
    for scope in policy
        .get("external_secret_scopes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        for path in json_str_seq(scope.get("store_manifest_paths")) {
            manifest_paths.insert(path);
        }
    }

    let mut stores = Vec::new();
    for rel in manifest_paths {
        let path = root.join(&rel);
        let text = fs::read_to_string(&path)
            .map_err(|e| CollectError::Io(format!("read {}: {e}", path.display())))?;
        for doc in parse_yaml_documents(&text, &rel)? {
            if doc.get("kind").and_then(serde_yaml::Value::as_str) != Some("ClusterSecretStore") {
                continue;
            }
            stores.push(json!({
                "path": rel,
                "name": yaml_str_at(&doc, &["metadata", "name"]).unwrap_or_default(),
                "store_kind": "ClusterSecretStore",
                "openbao_role": yaml_str_at(&doc, &["spec", "provider", "vault", "auth", "kubernetes", "role"]).unwrap_or_default(),
            }));
        }
    }
    Ok(stores)
}

fn collect_openbao_transport(root: &Path, policy: &Value) -> Result<Option<Value>, CollectError> {
    let Some(transport) = policy.get("openbao_transport") else {
        return Ok(None);
    };
    let manifest_path = transport
        .get("manifest_path")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let path = root.join(manifest_path);
    let text = fs::read_to_string(&path)
        .map_err(|e| CollectError::Io(format!("read {}: {e}", path.display())))?;
    let workload_namespace = transport
        .get("workload_namespace")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let selector = transport.get("workload_selector").unwrap_or(&Value::Null);
    let required_ports = json_u16_set(transport.get("allowed_ingress_ports"));

    let mut config_seen = false;
    let mut tls_disabled = false;
    let mut isolated_ports = BTreeSet::new();
    let mut has_broad_ingress_allow = false;

    for doc in parse_yaml_documents(&text, manifest_path)? {
        match doc.get("kind").and_then(serde_yaml::Value::as_str) {
            Some("ConfigMap")
                if yaml_str_at(&doc, &["metadata", "name"]).as_deref()
                    == Some("openbao-config") =>
            {
                config_seen = true;
                let hcl = yaml_str_at(&doc, &["data", "openbao.hcl"]).unwrap_or_default();
                if openbao_hcl_disables_tls(&hcl) {
                    tls_disabled = true;
                }
            }
            Some("NetworkPolicy")
                if network_policy_selects_workload(&doc, workload_namespace, selector) =>
            {
                let Some(ports) = restrictive_ingress_ports(&doc, &required_ports) else {
                    has_broad_ingress_allow = true;
                    continue;
                };
                isolated_ports.extend(ports);
            }
            _ => {}
        }
    }

    Ok(Some(json!({
        "config_seen": config_seen,
        "tls_disabled": tls_disabled,
        "isolated_ports": isolated_ports.into_iter().collect::<Vec<_>>(),
        "has_broad_ingress_allow": has_broad_ingress_allow,
    })))
}

fn parse_yaml_documents(text: &str, source: &str) -> Result<Vec<serde_yaml::Value>, CollectError> {
    let mut docs = Vec::new();
    for chunk in text.split("\n---") {
        let chunk = chunk.trim();
        if chunk.is_empty() {
            continue;
        }
        let value = serde_yaml::from_str(chunk)
            .map_err(|e| CollectError::Parse(format!("{source}: yaml parse: {e}")))?;
        docs.push(value);
    }
    Ok(docs)
}

fn yaml_str_at(value: &serde_yaml::Value, path: &[&str]) -> Option<String> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    cursor.as_str().map(str::to_owned)
}

fn external_secret_remote_keys(doc: &serde_yaml::Value) -> Vec<String> {
    let mut keys = BTreeSet::new();
    if let Some(items) = doc
        .get("spec")
        .and_then(|spec| spec.get("data"))
        .and_then(serde_yaml::Value::as_sequence)
    {
        for item in items {
            if let Some(key) = yaml_str_at(item, &["remoteRef", "key"]) {
                keys.insert(key);
            }
        }
    }
    if let Some(items) = doc
        .get("spec")
        .and_then(|spec| spec.get("dataFrom"))
        .and_then(serde_yaml::Value::as_sequence)
    {
        for item in items {
            if let Some(key) = yaml_str_at(item, &["extract", "key"]) {
                keys.insert(key);
            }
            if let Some(path) = yaml_str_at(item, &["find", "path"]) {
                keys.insert(path);
            }
        }
    }
    keys.into_iter().collect()
}

fn json_u16_set(value: Option<&Value>) -> BTreeSet<u16> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.as_u64()
                        .and_then(|n| u16::try_from(n).ok())
                        .or_else(|| item.as_str().and_then(|s| s.parse::<u16>().ok()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn openbao_hcl_disables_tls(hcl: &str) -> bool {
    hcl.lines().any(|line| {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        trimmed.starts_with("tls_disable") && trimmed.contains("true")
    })
}

fn network_policy_selects_workload(
    doc: &serde_yaml::Value,
    workload_namespace: &str,
    selector: &Value,
) -> bool {
    if yaml_str_at(doc, &["metadata", "namespace"]).as_deref() != Some(workload_namespace) {
        return false;
    }
    let match_labels = doc
        .get("spec")
        .and_then(|spec| spec.get("podSelector"))
        .and_then(|pod_selector| pod_selector.get("matchLabels"));
    selector
        .as_object()
        .map(|expected| {
            expected.iter().all(|(key, value)| {
                match_labels
                    .and_then(|labels| labels.get(key.as_str()))
                    .and_then(serde_yaml::Value::as_str)
                    == value.as_str()
            })
        })
        .unwrap_or(false)
}

fn restrictive_ingress_ports(
    doc: &serde_yaml::Value,
    required_ports: &BTreeSet<u16>,
) -> Option<BTreeSet<u16>> {
    let ingress = doc
        .get("spec")
        .and_then(|spec| spec.get("ingress"))
        .and_then(serde_yaml::Value::as_sequence)?;
    let mut covered = BTreeSet::new();
    for rule in ingress {
        let from = rule.get("from").and_then(serde_yaml::Value::as_sequence)?;
        if from.is_empty() || from.iter().any(|peer| !network_peer_is_restrictive(peer)) {
            return None;
        }
        let ports = rule.get("ports").and_then(serde_yaml::Value::as_sequence)?;
        if ports.is_empty() {
            return None;
        }
        for port in ports {
            let number = port.get("port").and_then(yaml_port_number)?;
            if !required_ports.contains(&number) {
                return None;
            }
            covered.insert(number);
        }
    }
    Some(covered)
}

fn yaml_port_number(value: &serde_yaml::Value) -> Option<u16> {
    value
        .as_i64()
        .and_then(|n| u16::try_from(n).ok())
        .or_else(|| value.as_str().and_then(|s| s.parse::<u16>().ok()))
}

fn network_peer_is_restrictive(peer: &serde_yaml::Value) -> bool {
    selector_has_constraints(peer.get("namespaceSelector"))
        || selector_has_constraints(peer.get("podSelector"))
        || ip_block_is_restrictive(peer.get("ipBlock"))
}

fn selector_has_constraints(selector: Option<&serde_yaml::Value>) -> bool {
    selector
        .and_then(serde_yaml::Value::as_mapping)
        .map(|mapping| {
            mapping
                .get(serde_yaml::Value::String("matchLabels".to_owned()))
                .and_then(serde_yaml::Value::as_mapping)
                .is_some_and(|labels| !labels.is_empty())
                || mapping
                    .get(serde_yaml::Value::String("matchExpressions".to_owned()))
                    .and_then(serde_yaml::Value::as_sequence)
                    .is_some_and(|expressions| match_expressions_are_restrictive(expressions))
        })
        .unwrap_or(false)
}

fn match_expressions_are_restrictive(expressions: &[serde_yaml::Value]) -> bool {
    !expressions.is_empty()
        && expressions.iter().all(|expression| {
            yaml_str_at(expression, &["key"]).is_some()
                && yaml_str_at(expression, &["operator"]).as_deref() == Some("In")
                && expression
                    .get("values")
                    .and_then(serde_yaml::Value::as_sequence)
                    .is_some_and(|values| !values.is_empty())
        })
}

fn ip_block_is_restrictive(ip_block: Option<&serde_yaml::Value>) -> bool {
    let Some(cidr) = ip_block
        .and_then(|block| block.get("cidr"))
        .and_then(serde_yaml::Value::as_str)
    else {
        return false;
    };
    cidr != "0.0.0.0/0" && cidr != "::/0"
}

/// The values-path "group" a provisioning template / preflight is expected to reference: the
/// join-token values ref minus its last `.segment` (`svidOperator.joinToken.secretName` ->
/// `svidOperator.joinToken`). Both the consumer (`...secretName`) and the preflight (`...joinToken`)
/// reference this group, so one token matches both.
fn join_token_group(join_ref: &str) -> &str {
    join_ref
        .rsplit_once('.')
        .map(|(head, _)| head)
        .unwrap_or(join_ref)
}

/// Shallow read-only scan of a chart templates dir for join-token provisioning evidence. Returns
/// `(has_provisioning_template, has_failclosed_preflight)`. A missing dir yields `(false, false)`
/// (fail-closed: no evidence). Only the top level is scanned (Helm templates are flat).
fn scan_join_token_evidence(dir: &Path, join_ref: &str) -> Result<(bool, bool), CollectError> {
    let group = join_token_group(join_ref);
    let mut has_provisioning = false;
    let mut has_preflight = false;
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((false, false)),
        Err(e) => return Err(CollectError::Io(format!("read dir {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("read entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let is_file = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?
            .is_file();
        if !is_file {
            continue;
        }
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(CollectError::Io(format!("read {}: {e}", path.display()))),
        };
        if scan_text_provisions_join_token(&text, group) {
            has_provisioning = true;
        }
        if scan_text_has_failclosed_preflight(&text, group) {
            has_preflight = true;
        }
    }
    Ok((has_provisioning, has_preflight))
}

/// Whether a template's text declares a provisioning-kind doc that references the join-token group.
/// Pure helper, exposed for tests.
pub fn scan_text_provisions_join_token(text: &str, group: &str) -> bool {
    if !text.contains(group) {
        return false;
    }
    text.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "kind: ExternalSecret"
            || trimmed == "kind: SealedSecret"
            || trimmed == "kind: Secret"
    })
}

/// Whether a template's text carries a fail-closed Helm preflight referencing the join-token group
/// (a single line containing both `fail` and the group token). Pure helper, exposed for tests.
pub fn scan_text_has_failclosed_preflight(text: &str, group: &str) -> bool {
    text.lines()
        .any(|line| line.contains("fail") && line.contains(group))
}

/// Parse the `secrets` Role rules out of a Helm-templated RBAC manifest. The rules block is static
/// YAML; the Helm actions live only in metadata/guards, so we neutralize them (drop standalone
/// `{{ ... }}` directive lines, replace inline `{{ ... }}` value expressions with a scalar
/// placeholder) and then parse each `---`-separated doc as YAML. FAIL-CLOSED: a non-empty doc that
/// does not parse is an `Err`, never a silently skipped (and potentially overbroad) rule. Pure
/// helper, exposed for tests.
pub fn parse_secret_rules(rbac_text: &str) -> Result<Vec<SecretRule>, String> {
    let neutralized = neutralize_helm(rbac_text);
    let mut rules = Vec::new();
    for chunk in neutralized.split("\n---") {
        let chunk = chunk.trim();
        if chunk.is_empty() {
            continue;
        }
        let value: serde_yaml::Value =
            serde_yaml::from_str(chunk).map_err(|e| format!("yaml parse: {e}"))?;
        let kind = value
            .get("kind")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or("");
        if kind != "Role" && kind != "ClusterRole" {
            continue;
        }
        let Some(rule_seq) = value.get("rules").and_then(serde_yaml::Value::as_sequence) else {
            continue;
        };
        for rule in rule_seq {
            let resources = yaml_str_seq(rule.get("resources"));
            if !resources.iter().any(|r| r == "secrets") {
                continue;
            }
            rules.push(SecretRule {
                verbs: yaml_str_seq(rule.get("verbs")),
                resource_names: yaml_str_seq(rule.get("resourceNames")),
            });
        }
    }
    Ok(rules)
}

fn yaml_str_seq(value: Option<&serde_yaml::Value>) -> Vec<String> {
    value
        .and_then(serde_yaml::Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// Neutralize Helm template syntax so the static YAML structure parses: drop standalone directive
/// lines (a trimmed line that both starts with `{{` and ends with `}}`, e.g. `{{- if .. }}`,
/// `{{- end }}`, `{{- /* .. */ -}}`) and replace each inline `{{ ... }}` value expression with a
/// scalar placeholder. Pure helper, exposed for tests.
pub fn neutralize_helm(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
            continue;
        }
        out.push_str(&replace_inline_actions(line));
        out.push('\n');
    }
    out
}

/// Replace each single-line `{{ ... }}` expression in `line` with a scalar placeholder. A line whose
/// `{{` has no closing `}}` (a multi-line action) is collapsed to a placeholder from the `{{` on.
fn replace_inline_actions(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("}}") {
            Some(end) => {
                out.push_str("helmvalue");
                rest = &after[end + 2..];
            }
            None => {
                out.push_str("helmvalue");
                return out;
            }
        }
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------------------
// Pure evaluation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect::<BTreeSet<_>>();
        Self {
            verdict: if violations.is_empty() {
                Verdict::Green
            } else {
                Verdict::Red
            },
            violations,
        }
    }
}

fn scoped_secret_verbs(policy: &Value) -> Vec<String> {
    policy
        .get("scoped_secret_verbs")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn json_str_seq(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|seq| {
            seq.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Pure evaluator. `policy` is DATA (`operator-secret-bootstrap-policy.json`); `observed` is the view
/// shaped by [`collect_operators`]. RED iff any governed operator grants a scoped secret verb without
/// name-scoping to its produced Secret, scopes the wrong Secret, or consumes an unprovisioned
/// bootstrap Secret.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "OSB-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    let scoped = scoped_secret_verbs(policy);
    if scoped.is_empty() {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `scoped_secret_verbs` must be a non-empty array (the RBAC verbs that honor resourceNames); correct the policy before the gate can evaluate",
        ));
        return findings;
    }

    let Some(descriptors) = policy.get("operators").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `operators` must be an array of governed-operator descriptors; correct the policy",
        ));
        return findings;
    };
    if descriptors.is_empty() {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `operators` resolved to zero operators; the gate would have nothing to enforce — correct the policy",
        ));
        return findings;
    }

    let observed_ops = observed
        .get("operators")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for descriptor in descriptors {
        let Some(name) = descriptor.get("name").and_then(Value::as_str) else {
            findings.insert(Finding::new(
                "OSB-POLICY-MALFORMED",
                POLICY_KEY,
                "an operator descriptor is missing its `name`",
            ));
            continue;
        };
        let produced = descriptor
            .get("produced_secret_name")
            .and_then(Value::as_str)
            .unwrap_or_default();

        // Fail closed if the collector saw no observed view for a policy-declared operator.
        let Some(observed_op) = observed_ops
            .iter()
            .find(|op| op.get("name").and_then(Value::as_str) == Some(name))
        else {
            findings.insert(Finding::new(
                "OSB-JOIN-TOKEN-UNPROVISIONED",
                name,
                "no observed view for this operator (collector did not see its chart) — fail-closed",
            ));
            continue;
        };

        evaluate_secret_rules(observed_op, name, produced, &scoped, &mut findings);

        let provisioned = observed_op
            .get("has_provisioning_template")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let preflight = observed_op
            .get("has_failclosed_preflight")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !provisioned && !preflight {
            findings.insert(Finding::new(
                "OSB-JOIN-TOKEN-UNPROVISIONED",
                name,
                "the operator-internal bootstrap Secret has no declarative provisioning template (ExternalSecret/SealedSecret/Secret) and no fail-closed chart preflight",
            ));
        }
    }
    evaluate_external_secret_scopes(policy, observed, &mut findings);
    evaluate_openbao_transport(policy, observed, &mut findings);

    findings
}

fn evaluate_secret_rules(
    observed_op: &Value,
    name: &str,
    produced: &str,
    scoped: &[String],
    findings: &mut BTreeSet<Finding>,
) {
    let rules = observed_op
        .get("secret_rules")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for rule in &rules {
        let verbs = json_str_seq(rule.get("verbs"));
        let resource_names = json_str_seq(rule.get("resource_names"));
        let scoped_present: Vec<&String> = verbs.iter().filter(|v| scoped.contains(v)).collect();
        if scoped_present.is_empty() {
            // list/watch/create-only rule: the irreducible namespace-wide RBAC floor.
            continue;
        }
        let verb_key = scoped_present
            .iter()
            .map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(",");
        if resource_names.is_empty() {
            findings.insert(Finding::new(
                "OSB-SECRET-RBAC-OVERBROAD",
                &format!("{name}:{verb_key}"),
                format!(
                    "secrets rule grants scoped verb(s) [{verb_key}] with NO resourceNames — must be name-scoped to {produced}"
                ),
            ));
            continue;
        }
        let expected: BTreeSet<&str> = std::iter::once(produced).collect();
        let actual: BTreeSet<&str> = resource_names.iter().map(String::as_str).collect();
        if actual != expected {
            findings.insert(Finding::new(
                "OSB-SECRET-RBAC-RESOURCENAME-MISMATCH",
                &format!("{name}:{verb_key}"),
                format!(
                    "scoped secrets rule resourceNames {actual:?} is not exactly {{{produced}}}"
                ),
            ));
        }
    }
}

fn evaluate_external_secret_scopes(
    policy: &Value,
    observed: &Value,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(scopes) = policy
        .get("external_secret_scopes")
        .and_then(Value::as_array)
    else {
        return;
    };
    if scopes.is_empty() {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `external_secret_scopes` must be non-empty when present",
        ));
        return;
    }

    let Some(external_secrets) = observed.get("external_secrets").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            "observed view is missing `external_secrets`; run the collector for policies with `external_secret_scopes`",
        ));
        return;
    };
    if external_secrets.is_empty() {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            "policy declares ExternalSecret scopes but collector found zero ExternalSecret documents",
        ));
        return;
    }

    for scope in scopes {
        validate_external_secret_scope_policy(scope, findings);
    }
    evaluate_cluster_secret_store_roles(scopes, observed, findings);

    for secret in external_secrets {
        evaluate_external_secret_ref(secret, scopes, findings);
    }
}

fn validate_external_secret_scope_policy(scope: &Value, findings: &mut BTreeSet<Finding>) {
    let store = scope
        .get("store_name")
        .and_then(Value::as_str)
        .unwrap_or("");
    if store.is_empty()
        || json_str_seq(scope.get("allowed_namespaces")).is_empty()
        || json_str_seq(scope.get("allowed_remote_key_prefixes")).is_empty()
        || (json_str_seq(scope.get("manifest_paths")).is_empty()
            && json_str_seq(scope.get("value_template_paths")).is_empty())
        || json_str_seq(scope.get("store_manifest_paths")).is_empty()
        || scope
            .get("openbao_role")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
    {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            format!("external_secret_scopes entry for `{store}` must declare store_name, allowed_namespaces, allowed_remote_key_prefixes, static manifest_paths or value_template_paths, store_manifest_paths, and openbao_role"),
        ));
    }
}

fn evaluate_cluster_secret_store_roles(
    scopes: &[Value],
    observed: &Value,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(stores) = observed
        .get("cluster_secret_stores")
        .and_then(Value::as_array)
    else {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            "observed view is missing `cluster_secret_stores`; run the collector for policies with `external_secret_scopes`",
        ));
        return;
    };

    for scope in scopes {
        let store_name = scope
            .get("store_name")
            .and_then(Value::as_str)
            .unwrap_or("");
        let store_kind = scope
            .get("store_kind")
            .and_then(Value::as_str)
            .unwrap_or("SecretStore");
        let expected_role = scope
            .get("openbao_role")
            .and_then(Value::as_str)
            .unwrap_or("");
        let key = format!("{store_kind}/{store_name}");

        let Some(store) = stores.iter().find(|store| {
            store.get("name").and_then(Value::as_str) == Some(store_name)
                && store
                    .get("store_kind")
                    .and_then(Value::as_str)
                    .unwrap_or("SecretStore")
                    == store_kind
        }) else {
            findings.insert(Finding::new(
                "OSB-ESO-STORE-UNDECLARED",
                &key,
                "policy declares an ExternalSecret store scope, but the committed ClusterSecretStore manifest was not observed",
            ));
            continue;
        };

        let observed_role = store
            .get("openbao_role")
            .and_then(Value::as_str)
            .unwrap_or("");
        if observed_role != expected_role {
            findings.insert(Finding::new(
                "OSB-ESO-STORE-ROLE-MISMATCH",
                &key,
                format!(
                    "ClusterSecretStore `{store_name}` binds OpenBao role `{observed_role}`, expected `{expected_role}` from policy DATA"
                ),
            ));
        }
    }
}

fn evaluate_external_secret_ref(
    secret: &Value,
    scopes: &[Value],
    findings: &mut BTreeSet<Finding>,
) {
    let store_name = secret
        .get("store_name")
        .and_then(Value::as_str)
        .unwrap_or("");
    let store_kind = secret
        .get("store_kind")
        .and_then(Value::as_str)
        .unwrap_or("SecretStore");
    let namespace = secret
        .get("namespace")
        .and_then(Value::as_str)
        .unwrap_or("");
    let path = secret
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>");
    let name = secret
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("<unnamed>");
    let remote_keys = json_str_seq(secret.get("remote_keys"));

    let matching_store_scopes = scopes
        .iter()
        .filter(|scope| {
            scope.get("store_name").and_then(Value::as_str) == Some(store_name)
                && scope
                    .get("store_kind")
                    .and_then(Value::as_str)
                    .unwrap_or("SecretStore")
                    == store_kind
        })
        .collect::<Vec<_>>();

    if matching_store_scopes.is_empty() {
        findings.insert(Finding::new(
            "OSB-ESO-STORE-UNDECLARED",
            &format!("{path}:{namespace}/{name}"),
            format!("ExternalSecret references undeclared {store_kind} `{store_name}`; declare its namespace/key-prefix contract in policy DATA"),
        ));
        return;
    }

    if remote_keys.is_empty() {
        findings.insert(Finding::new(
            "OSB-ESO-REMOTE-KEY-OUT-OF-SCOPE",
            &format!("{path}:{namespace}/{name}"),
            "ExternalSecret has no remoteRef.key / dataFrom key; the gate cannot prove OpenBao scope",
        ));
        return;
    }

    for remote_key in remote_keys {
        let allowed = matching_store_scopes.iter().any(|scope| {
            json_str_seq(scope.get("allowed_namespaces"))
                .iter()
                .any(|allowed_namespace| allowed_namespace == namespace)
                && json_str_seq(scope.get("allowed_remote_key_prefixes"))
                    .iter()
                    .any(|prefix| remote_key.starts_with(prefix))
        });
        if !allowed {
            findings.insert(Finding::new(
                "OSB-ESO-REMOTE-KEY-OUT-OF-SCOPE",
                &format!("{path}:{namespace}/{name}:{remote_key}"),
                format!("ExternalSecret uses {store_kind} `{store_name}` for remote key `{remote_key}` in namespace `{namespace}`, outside the policy-declared namespace/prefix tuples"),
            ));
        }
    }
}

fn evaluate_openbao_transport(policy: &Value, observed: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(transport_policy) = policy.get("openbao_transport") else {
        return;
    };
    let required_ports = json_u16_set(transport_policy.get("allowed_ingress_ports"));
    if required_ports.is_empty() {
        findings.insert(Finding::new(
            "OSB-POLICY-MALFORMED",
            POLICY_KEY,
            "openbao_transport.allowed_ingress_ports must be a non-empty array",
        ));
        return;
    }

    let Some(transport) = observed.get("openbao_transport") else {
        findings.insert(Finding::new(
            "OSB-OPENBAO-TRANSPORT-UNISOLATED",
            POLICY_KEY,
            "observed view is missing OpenBao transport data; run the collector for policies with openbao_transport",
        ));
        return;
    };

    if !transport
        .get("config_seen")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        findings.insert(Finding::new(
            "OSB-OPENBAO-TRANSPORT-UNISOLATED",
            POLICY_KEY,
            "OpenBao config map was not observed; cannot prove listener TLS posture",
        ));
        return;
    }

    let tls_disabled = transport
        .get("tls_disabled")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if !tls_disabled {
        return;
    }
    if transport
        .get("has_broad_ingress_allow")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        findings.insert(Finding::new(
            "OSB-OPENBAO-TRANSPORT-UNISOLATED",
            POLICY_KEY,
            "OpenBao plaintext listener has a NetworkPolicy ingress rule with no `from` restriction",
        ));
        return;
    }

    let isolated_ports = json_u16_set(transport.get("isolated_ports"));
    let missing_ports = required_ports
        .difference(&isolated_ports)
        .copied()
        .collect::<Vec<_>>();
    if !missing_ports.is_empty() {
        findings.insert(Finding::new(
            "OSB-OPENBAO-TRANSPORT-UNISOLATED",
            POLICY_KEY,
            format!("OpenBao plaintext listener is missing restrictive NetworkPolicy coverage for port(s) {missing_ports:?}"),
        ));
    }
}
/// Bare-code projection of [`evaluate_keyed`] — the single source of truth for the verdict.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "scoped_secret_verbs": ["get", "update", "patch", "delete"],
            "operators": [{
                "name": "op",
                "rbac_template": "rbac.yaml",
                "chart_templates_dir": "templates",
                "produced_secret_name": "the-secret",
                "join_token_values_ref": "svidOperator.joinToken.secretName"
            }]
        })
    }

    fn green_observed() -> Value {
        json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [
                { "verbs": ["get", "update", "patch"], "resource_names": ["the-secret"] },
                { "verbs": ["list", "watch", "create"], "resource_names": [] }
            ],
            "has_provisioning_template": true,
            "has_failclosed_preflight": true
        }]})
    }

    #[test]
    fn least_privilege_chart_is_green() {
        let report = evaluate(&policy(), &green_observed());
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.violations);
        assert!(evaluate_keyed(&policy(), &green_observed()).is_empty());
    }

    #[test]
    fn unscoped_get_update_patch_is_overbroad() {
        let observed = json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [
                { "verbs": ["get", "list", "watch", "create", "update", "patch"], "resource_names": [] }
            ],
            "has_provisioning_template": true,
            "has_failclosed_preflight": true
        }]});
        let codes: BTreeSet<String> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("OSB-SECRET-RBAC-OVERBROAD"), "{codes:?}");
    }

    #[test]
    fn listwatchcreate_only_rule_is_allowed_unscoped() {
        let observed = json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [ { "verbs": ["list", "watch", "create"], "resource_names": [] } ],
            "has_provisioning_template": true,
            "has_failclosed_preflight": false
        }]});
        assert!(
            evaluate_keyed(&policy(), &observed).is_empty(),
            "list/watch/create without resourceNames is the irreducible RBAC floor"
        );
    }

    #[test]
    fn scoping_the_wrong_secret_is_a_mismatch() {
        let observed = json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [ { "verbs": ["get", "update"], "resource_names": ["some-other-secret"] } ],
            "has_provisioning_template": true,
            "has_failclosed_preflight": true
        }]});
        let codes: BTreeSet<String> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(
            codes.contains("OSB-SECRET-RBAC-RESOURCENAME-MISMATCH"),
            "{codes:?}"
        );
    }

    #[test]
    fn extra_scoped_secret_is_a_mismatch() {
        let observed = json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [ { "verbs": ["patch"], "resource_names": ["the-secret", "extra"] } ],
            "has_provisioning_template": true,
            "has_failclosed_preflight": true
        }]});
        let codes: BTreeSet<String> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(
            codes.contains("OSB-SECRET-RBAC-RESOURCENAME-MISMATCH"),
            "{codes:?}"
        );
    }

    #[test]
    fn unprovisioned_join_token_fires() {
        let observed = json!({ "operators": [{
            "name": "op",
            "produced_secret_name": "the-secret",
            "secret_rules": [ { "verbs": ["get"], "resource_names": ["the-secret"] } ],
            "has_provisioning_template": false,
            "has_failclosed_preflight": false
        }]});
        let codes: BTreeSet<String> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("OSB-JOIN-TOKEN-UNPROVISIONED"), "{codes:?}");
    }

    #[test]
    fn either_provisioning_or_preflight_satisfies_bootstrap() {
        for (prov, pre) in [(true, false), (false, true)] {
            let observed = json!({ "operators": [{
                "name": "op",
                "produced_secret_name": "the-secret",
                "secret_rules": [ { "verbs": ["get"], "resource_names": ["the-secret"] } ],
                "has_provisioning_template": prov,
                "has_failclosed_preflight": pre
            }]});
            assert!(
                evaluate_keyed(&policy(), &observed).is_empty(),
                "provisioning OR preflight should satisfy the bootstrap contract ({prov},{pre})"
            );
        }
    }

    #[test]
    fn missing_observed_operator_fails_closed() {
        let observed = json!({ "operators": [] });
        let codes: BTreeSet<String> = evaluate_keyed(&policy(), &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("OSB-JOIN-TOKEN-UNPROVISIONED"), "{codes:?}");
    }

    #[test]
    fn wrong_gate_id_fails_closed() {
        let mut p = policy();
        p["gate_id"] = json!("nope");
        assert!(
            evaluate_keyed(&p, &green_observed())
                .iter()
                .any(|f| f.code == "OSB-POLICY-GATE-ID-MISMATCH")
        );
    }

    #[test]
    fn empty_operators_fails_closed() {
        let mut p = policy();
        p["operators"] = json!([]);
        assert_eq!(evaluate(&p, &green_observed()).verdict, Verdict::Red);
    }

    #[test]
    fn neutralize_drops_directives_and_inlines() {
        let src = "{{- if .Values.x }}\nname: {{ .Values.n | quote }}\nstatic: yes\n{{- end }}";
        let out = neutralize_helm(src);
        assert!(!out.contains("if .Values"));
        assert!(!out.contains("end"));
        assert!(out.contains("name: helmvalue"));
        assert!(out.contains("static: yes"));
    }

    #[test]
    fn parse_secret_rules_extracts_scoped_and_unscoped() {
        let rbac = "{{- if .Values.x }}\napiVersion: rbac.authorization.k8s.io/v1\nkind: Role\nmetadata:\n  name: {{ .Values.n | quote }}\nrules:\n  - apiGroups: [\"\"]\n    resources: [\"secrets\"]\n    resourceNames: [\"the-secret\"]\n    verbs: [\"get\", \"update\", \"patch\"]\n  - apiGroups: [\"\"]\n    resources: [\"secrets\"]\n    verbs: [\"list\", \"watch\", \"create\"]\n  - apiGroups: [\"\"]\n    resources: [\"events\"]\n    verbs: [\"create\"]\n{{- end }}";
        let rules = parse_secret_rules(rbac).unwrap();
        assert_eq!(
            rules.len(),
            2,
            "two secrets rules (events excluded): {rules:?}"
        );
        assert_eq!(rules[0].resource_names, vec!["the-secret".to_owned()]);
        assert!(rules[1].resource_names.is_empty());
    }

    #[test]
    fn provisioning_and_preflight_text_signals() {
        let group = "svidOperator.joinToken";
        let es = "kind: ExternalSecret\ntarget:\n  name: {{ .Values.svidOperator.joinToken.secretName }}";
        assert!(scan_text_provisions_join_token(es, group));
        assert!(!scan_text_has_failclosed_preflight(es, group));
        let pre = "{{- fail \"svidOperator.joinToken has no provisioning contract\" -}}";
        assert!(scan_text_has_failclosed_preflight(pre, group));
        let deployment =
            "kind: Deployment\nenv:\n  name: {{ .Values.svidOperator.joinToken.secretName }}";
        assert!(
            !scan_text_provisions_join_token(deployment, group),
            "consumer is not a provisioner"
        );
    }
    #[test]
    fn external_secret_scope_policy_accepts_declared_namespace_and_prefix() {
        let mut p = policy();
        p["external_secret_scopes"] = json!([{
            "store_name": "openbao-oya",
            "store_kind": "ClusterSecretStore",
            "allowed_namespaces": ["oya-ci"],
            "allowed_remote_key_prefixes": ["oya/ci/"],
            "manifest_paths": ["externalsecret.yaml"],
            "store_manifest_paths": ["clustersecretstore.yaml"],
            "openbao_role": "eso-oya-ci"
        }]);
        let mut observed = green_observed();
        observed["external_secrets"] = json!([{
            "path": "externalsecret.yaml",
            "name": "github-ci-token",
            "namespace": "oya-ci",
            "store_name": "openbao-oya",
            "store_kind": "ClusterSecretStore",
            "remote_keys": ["oya/ci/github-ci-token"]
        }]);
        observed["cluster_secret_stores"] = json!([{
            "path": "clustersecretstore.yaml",
            "name": "openbao-oya",
            "store_kind": "ClusterSecretStore",
            "openbao_role": "eso-oya-ci"
        }]);

        assert!(evaluate_keyed(&p, &observed).is_empty());
    }

    #[test]
    fn external_secret_scope_policy_rejects_wrong_prefix() {
        let mut p = policy();
        p["external_secret_scopes"] = json!([{
            "store_name": "openbao-oya",
            "store_kind": "ClusterSecretStore",
            "allowed_namespaces": ["oya-ci"],
            "allowed_remote_key_prefixes": ["oya/ci/"],
            "manifest_paths": ["externalsecret.yaml"],
            "store_manifest_paths": ["clustersecretstore.yaml"],
            "openbao_role": "eso-oya-ci"
        }]);
        let mut observed = green_observed();
        observed["external_secrets"] = json!([{
            "path": "externalsecret.yaml",
            "name": "csi-creds",
            "namespace": "cloud-k8s-system",
            "store_name": "openbao-oya",
            "store_kind": "ClusterSecretStore",
            "remote_keys": ["cloud-k8s/csi/block-volume"]
        }]);
        observed["cluster_secret_stores"] = json!([{
            "path": "clustersecretstore.yaml",
            "name": "openbao-oya",
            "store_kind": "ClusterSecretStore",
            "openbao_role": "eso-oya-ci"
        }]);

        let codes: BTreeSet<String> = evaluate_keyed(&p, &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(
            codes.contains("OSB-ESO-REMOTE-KEY-OUT-OF-SCOPE"),
            "{codes:?}"
        );
    }

    #[test]
    fn external_secret_scope_policy_rejects_store_role_mismatch() {
        let mut p = policy();
        p["external_secret_scopes"] = json!([{
            "store_name": "openbao-oya",
            "store_kind": "ClusterSecretStore",
            "allowed_namespaces": ["oya-ci"],
            "allowed_remote_key_prefixes": ["oya/ci/"],
            "manifest_paths": ["externalsecret.yaml"],
            "store_manifest_paths": ["clustersecretstore.yaml"],
            "openbao_role": "eso-oya-ci"
        }]);
        let mut observed = green_observed();
        observed["external_secrets"] = json!([{
            "path": "externalsecret.yaml",
            "name": "github-ci-token",
            "namespace": "oya-ci",
            "store_name": "openbao-oya",
            "store_kind": "ClusterSecretStore",
            "remote_keys": ["oya/ci/github-ci-token"]
        }]);
        observed["cluster_secret_stores"] = json!([{
            "path": "clustersecretstore.yaml",
            "name": "openbao-oya",
            "store_kind": "ClusterSecretStore",
            "openbao_role": "eso-cloud-k8s-csi"
        }]);

        let codes: BTreeSet<String> = evaluate_keyed(&p, &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(codes.contains("OSB-ESO-STORE-ROLE-MISMATCH"), "{codes:?}");
    }

    #[test]
    fn network_policy_rejects_broad_peer_and_omitted_ports() {
        let required = [8200_u16].into_iter().collect::<BTreeSet<_>>();
        let broad_peer: serde_yaml::Value = serde_yaml::from_str(
            "kind: NetworkPolicy\nspec:\n  ingress:\n    - from:\n        - namespaceSelector: {}\n      ports:\n        - port: 8200\n",
        )
        .unwrap();
        assert!(
            restrictive_ingress_ports(&broad_peer, &required).is_none(),
            "empty namespaceSelector is broad"
        );

        let omitted_ports: serde_yaml::Value = serde_yaml::from_str(
            "kind: NetworkPolicy\nspec:\n  ingress:\n    - from:\n        - namespaceSelector:\n            matchLabels:\n              kubernetes.io/metadata.name: external-secrets\n",
        )
        .unwrap();
        assert!(
            restrictive_ingress_ports(&omitted_ports, &required).is_none(),
            "omitted ports allow every port"
        );

        let exists_selector: serde_yaml::Value = serde_yaml::from_str(
            "kind: NetworkPolicy\nspec:\n  ingress:\n    - from:\n        - namespaceSelector:\n            matchExpressions:\n              - key: kubernetes.io/metadata.name\n                operator: Exists\n      ports:\n        - port: 8200\n",
        )
        .unwrap();
        assert!(
            restrictive_ingress_ports(&exists_selector, &required).is_none(),
            "Exists over the standard namespace-name label is broad"
        );
        let restrictive: serde_yaml::Value = serde_yaml::from_str(
            "kind: NetworkPolicy\nspec:\n  ingress:\n    - from:\n        - namespaceSelector:\n            matchLabels:\n              kubernetes.io/metadata.name: external-secrets\n      ports:\n        - port: 8200\n",
        )
        .unwrap();
        assert_eq!(
            restrictive_ingress_ports(&restrictive, &required),
            Some(required)
        );
    }
    #[test]
    fn plaintext_openbao_requires_restrictive_network_policy_ports() {
        let mut p = policy();
        p["openbao_transport"] = json!({
            "allowed_ingress_ports": [8200, 8201]
        });
        let mut observed = green_observed();
        observed["openbao_transport"] = json!({
            "config_seen": true,
            "tls_disabled": true,
            "isolated_ports": [8200, 8201],
            "has_broad_ingress_allow": false
        });
        assert!(evaluate_keyed(&p, &observed).is_empty());

        observed["openbao_transport"]["isolated_ports"] = json!([8200]);
        let codes: BTreeSet<String> = evaluate_keyed(&p, &observed)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert!(
            codes.contains("OSB-OPENBAO-TRANSPORT-UNISOLATED"),
            "{codes:?}"
        );
    }
}
