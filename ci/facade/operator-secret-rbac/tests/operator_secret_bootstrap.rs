// GH #980 cloud-ci-operator-secret-bootstrap: self-test over TODAY's real chart plus a hermetic RED
// fixture. It asserts:
//   * the LIVE worktree is GREEN — the real collector parses the cloud-iam SVID-operator chart and
//     finds its secrets RBAC name-scoped (get/update/patch bound to resourceNames=[the produced
//     Secret]) and its join-token Secret provisioned (an ExternalSecret) + guarded by a fail-closed
//     preflight. This is the post-fix state; before the fix it would be RED.
//   * a RED fixture (a synthetic chart with namespace-wide secrets get/update/patch and NO
//     provisioning template / NO preflight) makes the gate FAIL on BOTH the overbroad-RBAC and the
//     unprovisioned-join-token codes — proving it genuinely catches the #980 defect, not an
//     always-pass stub.
//   * the committed policy gate_id matches the crate contract.
// The fixtures drive the REAL collector (the only I/O) end-to-end, so the collector's hermetic fs
// scan + Helm-action neutralization + the pure evaluator are all exercised.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ci_operator_secret_rbac::{GATE_ID, Verdict, collect_operators, evaluate, evaluate_keyed};
use serde_json::{Value, json};

/// Walk up from the test's working directory to the repo root (the dir holding the canonical
/// `specs/root-hub-pointers.json`). Verbatim from the sibling gates.
fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn gate_dir(root: &Path) -> PathBuf {
    root.join("ci/facade/operator-secret-rbac")
}

fn load_policy(root: &Path) -> Value {
    let path = gate_dir(root).join("operator-secret-bootstrap-policy.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn unique_tmp(tag: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("osb-{tag}-{nanos}-{n}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn committed_policy_gate_id_matches_contract() {
    let root = repo_root();
    let policy = load_policy(&root);
    assert_eq!(
        policy.get("gate_id").and_then(Value::as_str),
        Some(GATE_ID),
        "policy gate_id must match the crate GATE_ID"
    );
}

#[test]
fn live_chart_is_green_after_the_fix() {
    let root = repo_root();
    let policy = load_policy(&root);
    let observed =
        collect_operators(&root, &policy).unwrap_or_else(|e| panic!("collect on live tree: {e}"));
    let ops = observed
        .get("operators")
        .and_then(Value::as_array)
        .expect("observed operators");
    assert!(!ops.is_empty(), "policy must govern at least one operator");

    let report = evaluate(&policy, &observed);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "live cloud-iam SVID-operator chart must be least-privilege + join-token-provisioned: {:?}",
        report.violations
    );
}

#[test]
fn red_fixture_overbroad_and_unprovisioned_fails() {
    let dir = unique_tmp("red");
    let templates = dir.join("templates");
    fs::create_dir_all(&templates).unwrap();
    // Overbroad RBAC: namespace-wide get/update/patch with NO resourceNames (the #980 defect).
    fs::write(
        templates.join("rbac.yaml"),
        "{{- if .Values.op.enabled }}\napiVersion: rbac.authorization.k8s.io/v1\nkind: Role\nmetadata:\n  name: {{ .Values.op.name }}\nrules:\n  - apiGroups: [\"\"]\n    resources: [\"secrets\"]\n    verbs: [\"get\", \"list\", \"watch\", \"create\", \"update\", \"patch\"]\n{{- end }}",
    )
    .unwrap();
    // A Deployment consuming the join token, but NO provisioning template and NO preflight.
    fs::write(
        templates.join("deployment.yaml"),
        "{{- if .Values.op.enabled }}\napiVersion: apps/v1\nkind: Deployment\nspec:\n  template:\n    spec:\n      containers:\n        - env:\n            - name: OYA_SVID_OPERATOR_JOIN_TOKEN\n              valueFrom:\n                secretKeyRef:\n                  name: {{ .Values.svidOperator.joinToken.secretName }}\n{{- end }}",
    )
    .unwrap();

    let policy = json!({
        "gate_id": GATE_ID,
        "scoped_secret_verbs": ["get", "update", "patch", "delete"],
        "operators": [{
            "name": "fixture-op",
            "rbac_template": "templates/rbac.yaml",
            "chart_templates_dir": "templates",
            "produced_secret_name": "the-produced-secret",
            "join_token_values_ref": "svidOperator.joinToken.secretName"
        }]
    });

    let observed =
        collect_operators(&dir, &policy).unwrap_or_else(|e| panic!("collect on red fixture: {e}"));
    let codes: std::collections::BTreeSet<String> = evaluate_keyed(&policy, &observed)
        .into_iter()
        .map(|f| f.code)
        .collect();

    assert_eq!(
        evaluate(&policy, &observed).verdict,
        Verdict::Red,
        "the #980 defect (overbroad RBAC + unprovisioned join token) must be RED"
    );
    assert!(codes.contains("OSB-SECRET-RBAC-OVERBROAD"), "{codes:?}");
    assert!(codes.contains("OSB-JOIN-TOKEN-UNPROVISIONED"), "{codes:?}");

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn green_fixture_scoped_and_provisioned_passes() {
    let dir = unique_tmp("green");
    let templates = dir.join("templates");
    fs::create_dir_all(&templates).unwrap();
    fs::write(
        templates.join("rbac.yaml"),
        "apiVersion: rbac.authorization.k8s.io/v1\nkind: Role\nmetadata:\n  name: {{ .Values.op.name }}\nrules:\n  - apiGroups: [\"\"]\n    resources: [\"secrets\"]\n    resourceNames: [\"the-produced-secret\"]\n    verbs: [\"get\", \"update\", \"patch\"]\n  - apiGroups: [\"\"]\n    resources: [\"secrets\"]\n    verbs: [\"list\", \"watch\", \"create\"]",
    )
    .unwrap();
    fs::write(
        templates.join("join-token-externalsecret.yaml"),
        "apiVersion: external-secrets.io/v1\nkind: ExternalSecret\nmetadata:\n  name: {{ .Values.svidOperator.joinToken.secretName }}\nspec:\n  target:\n    name: {{ .Values.svidOperator.joinToken.secretName }}",
    )
    .unwrap();

    let policy = json!({
        "gate_id": GATE_ID,
        "scoped_secret_verbs": ["get", "update", "patch", "delete"],
        "operators": [{
            "name": "fixture-op",
            "rbac_template": "templates/rbac.yaml",
            "chart_templates_dir": "templates",
            "produced_secret_name": "the-produced-secret",
            "join_token_values_ref": "svidOperator.joinToken.secretName"
        }]
    });

    let observed = collect_operators(&dir, &policy)
        .unwrap_or_else(|e| panic!("collect on green fixture: {e}"));
    let report = evaluate(&policy, &observed);
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.violations);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn red_fixture_external_secret_scope_and_openbao_transport_fail() {
    let dir = unique_tmp("eso-red");
    let templates = dir.join("templates");
    fs::create_dir_all(&templates).unwrap();
    let scan = dir.join("scan");
    fs::create_dir_all(&scan).unwrap();

    fs::write(
        templates.join("rbac.yaml"),
        "apiVersion: rbac.authorization.k8s.io/v1\nkind: Role\nmetadata:\n  name: fixture\nrules:\n  - apiGroups: [\"\"]\n    resources: [\"secrets\"]\n    resourceNames: [\"the-produced-secret\"]\n    verbs: [\"get\", \"update\", \"patch\"]\n  - apiGroups: [\"\"]\n    resources: [\"secrets\"]\n    verbs: [\"list\", \"watch\", \"create\"]",
    )
    .unwrap();
    fs::write(
        templates.join("join-token-externalsecret.yaml"),
        "apiVersion: external-secrets.io/v1\nkind: ExternalSecret\nmetadata:\n  name: join-token\nspec:\n  target:\n    name: {{ .Values.svidOperator.joinToken.secretName }}",
    )
    .unwrap();
    fs::write(
        scan.join("bad-externalsecret.yaml"),
        "apiVersion: external-secrets.io/v1\nkind: ExternalSecret\nmetadata:\n  name: csi-creds\n  namespace: cloud-k8s-system\nspec:\n  secretStoreRef:\n    name: openbao-oya\n    kind: ClusterSecretStore\n  data:\n    - secretKey: endpoint\n      remoteRef:\n        key: cloud-k8s/csi/block-volume\n        property: endpoint\n",
    )
    .unwrap();
    fs::write(
        scan.join("listed-externalsecret.yaml"),
        "apiVersion: external-secrets.io/v1\nkind: ExternalSecret\nmetadata:\n  name: github-ci-token\n  namespace: oya-ci\nspec:\n  secretStoreRef:\n    name: openbao-oya\n    kind: ClusterSecretStore\n  data:\n    - secretKey: token\n      remoteRef:\n        key: oya/ci/github-ci-token\n        property: token\n",
    )
    .unwrap();
    fs::write(
        dir.join("clustersecretstore.yaml"),
        "apiVersion: external-secrets.io/v1\nkind: ClusterSecretStore\nmetadata:\n  name: openbao-oya\nspec:\n  provider:\n    vault:\n      auth:\n        kubernetes:\n          role: eso-oya-ci\n",
    )
    .unwrap();
    fs::write(
        dir.join("openbao.yaml"),
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: openbao-config\n  namespace: oya-kms\ndata:\n  openbao.hcl: |\n    listener \"tcp\" {\n      tls_disable = true\n    }\n",
    )
    .unwrap();

    let policy = json!({
        "gate_id": GATE_ID,
        "scoped_secret_verbs": ["get", "update", "patch", "delete"],
        "operators": [{
            "name": "fixture-op",
            "rbac_template": "templates/rbac.yaml",
            "chart_templates_dir": "templates",
            "produced_secret_name": "the-produced-secret",
            "join_token_values_ref": "svidOperator.joinToken.secretName"
        }],
        "external_secret_scan_roots": ["scan"],
        "external_secret_scopes": [{
            "store_name": "openbao-oya",
            "store_kind": "ClusterSecretStore",
            "allowed_namespaces": ["oya-ci"],
            "allowed_remote_key_prefixes": ["oya/ci/"],
            "manifest_paths": ["scan/listed-externalsecret.yaml"],
            "store_manifest_paths": ["clustersecretstore.yaml"],
            "openbao_role": "eso-oya-ci"
        }],
        "openbao_transport": {
            "manifest_path": "openbao.yaml",
            "workload_namespace": "oya-kms",
            "workload_selector": {"app.kubernetes.io/name": "openbao"},
            "allowed_ingress_ports": [8200, 8201]
        }
    });

    let observed = collect_operators(&dir, &policy)
        .unwrap_or_else(|e| panic!("collect red #988 fixture: {e}"));
    let codes: std::collections::BTreeSet<String> = evaluate_keyed(&policy, &observed)
        .into_iter()
        .map(|f| f.code)
        .collect();

    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
    assert!(
        codes.contains("OSB-ESO-REMOTE-KEY-OUT-OF-SCOPE"),
        "{codes:?}"
    );
    assert!(
        codes.contains("OSB-OPENBAO-TRANSPORT-UNISOLATED"),
        "{codes:?}"
    );

    fs::remove_dir_all(&dir).ok();
}
