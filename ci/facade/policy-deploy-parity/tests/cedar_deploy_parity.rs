// GH #16 / ADR-0608 cloud-ci-cedar-deploy-parity: self-test over TODAY's real tree plus hermetic
// RED/GREEN fixtures. It asserts:
//   * the committed policy gate_id matches the crate contract.
//   * the LIVE worktree is GREEN against the documented baseline — the real collector walks the tree,
//     finds every deployed Cedar ConfigMap, and the known-blanket set is grandfathered + accurate (no
//     CDP-STALE-BASELINE), so the gate is born-blocking-against-regressions and mergeable.
//   * the gate genuinely DETECTS the blanket: with the baseline emptied, the SAME live scan goes RED
//     with CDP-UNCONSTRAINED-PERMIT — proving the grandfather is the only thing keeping it green
//     (not an always-pass stub) and that the blanket-disarm follow-up has real work to shrink.
//   * RED/GREEN fixtures cover a fresh non-baselined blanket, a constrained permit present in authored
//     policy, and an authored forbid missing from deployed policy — the real collector + pure evaluator
//     end to end.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ci_policy_deploy_parity::{GATE_ID, Verdict, collect, evaluate, evaluate_keyed};
use serde_json::Value;

/// Walk up to the repo root (the dir holding `specs/root-hub-pointers.json`). Verbatim from the
/// sibling gates.
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
    root.join("ci/facade/policy-deploy-parity")
}

fn load_policy(root: &Path) -> Value {
    let path = gate_dir(root).join("cedar-deploy-parity-policy.json");
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
    let dir = std::env::temp_dir().join(format!("cdp-{tag}-{nanos}-{n}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn codes(findings: BTreeSet<ci_policy_deploy_parity::Finding>) -> BTreeSet<String> {
    findings.into_iter().map(|f| f.code).collect()
}

const BLANKET_CONFIGMAP: &str = r#"{{- if .Values.cedar.enabled }}
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ .Values.cedar.policyConfigMapName | quote }}
data:
  policies.cedar: |
    permit(
      principal,
      action,
      resource
    ) when {
      resource.microservice == "{{ .Values.microservice.id }}" &&
      principal.tenant_class == "{{ .Values.microservice.tenantClass }}"
    };
{{- end }}
"#;
const GH_987_CLOUD_PATHS: [&str; 14] = [
    "cloud/cell-lifecycle/iac/k8s/helm/templates/cedar.yaml",
    "cloud/cell-rebalancer/iac/k8s/helm/templates/cedar.yaml",
    "billing/tax/iac/k8s/helm/templates/cedar.yaml",
    "billing/iac/k8s/helm/templates/cedar.yaml",
    "cloud/cloud-data/iac/k8s/helm/templates/cedar.yaml",
    "cloud/cloud-iac/iac/k8s/helm/templates/cedar.yaml",
    "cloud/cloud-iam/iac/k8s/helm/templates/cedar.yaml",
    "cloud/cloud-k8s/iac/k8s/helm/templates/cedar.yaml",
    "cloud/cloud-kms/iac/k8s/helm/templates/cedar.yaml",
    "cloud/cloud-network-dns/iac/k8s/helm/templates/cedar.yaml",
    "cloud/cloud-network/iac/k8s/helm/templates/cedar.yaml",
    "cloud/cloud-secrets/iac/k8s/helm/templates/cedar.yaml",
    "cloud/cloud-storage/iac/k8s/helm/templates/cedar.yaml",
    "cloud/tenancy/iac/k8s/helm/templates/cedar.yaml",
];

const AUTHZ_004_DEAD_CONFIGMAP_PATHS: [&str; 1] =
    ["oya/analytics/iac/k8s/helm/templates/cedar.yaml"];

#[test]
fn committed_policy_gate_id_matches_contract() {
    let policy = load_policy(&repo_root());
    assert_eq!(
        policy.get("gate_id").and_then(Value::as_str),
        Some(GATE_ID),
        "policy gate_id must match the crate GATE_ID"
    );
}

#[test]
fn live_tree_is_green_against_documented_baseline() {
    let root = repo_root();
    let policy = load_policy(&root);
    let observed = collect(&root, &policy).unwrap_or_else(|e| panic!("collect on live tree: {e}"));
    let count = observed
        .get("configmaps")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    assert!(count > 0, "the scan must find deployed Cedar ConfigMaps");

    let findings = evaluate_keyed(&policy, &observed);
    let report = evaluate(&policy, &observed);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "live tree must be GREEN against the documented baseline (no NEW over-broad permit, no stale baseline): {:?}; keyed findings: {findings:#?}",
        report.violations
    );
}
#[test]
fn gh_987_cloud_templates_are_disarmed_from_blanket_baseline() {
    let root = repo_root();
    let policy = load_policy(&root);
    let baseline_paths = policy
        .get("baseline")
        .and_then(|baseline| baseline.get("paths"))
        .and_then(Value::as_array)
        .expect("baseline.paths array")
        .iter()
        .map(|path| path.as_str().expect("baseline path string"))
        .collect::<BTreeSet<_>>();

    let observed = collect(&root, &policy).unwrap_or_else(|e| panic!("collect on live tree: {e}"));
    let configmaps = observed
        .get("configmaps")
        .and_then(Value::as_array)
        .expect("collected configmaps array");

    for path in GH_987_CLOUD_PATHS {
        assert!(
            !baseline_paths.contains(path),
            "{path} must not remain in the blanket baseline after GH #987 disarm"
        );

        let configmap = configmaps
            .iter()
            .find(|configmap| configmap.get("path").and_then(Value::as_str) == Some(path))
            .unwrap_or_else(|| panic!("missing deployed Cedar ConfigMap for {path}"));
        assert_eq!(
            configmap.get("authored_found").and_then(Value::as_bool),
            Some(true),
            "{path} must resolve an authored action/resource-specific policy"
        );

        let permits = configmap
            .get("permits")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("missing permit scan for {path}"));
        assert!(
            !permits.is_empty(),
            "{path} must deploy explicit authored permits, not an empty allow surface"
        );
        for permit in permits {
            assert_eq!(
                permit.get("action_unconstrained").and_then(Value::as_bool),
                Some(false),
                "{path} has an action-agnostic permit after GH #987 disarm: {permit:?}"
            );
            assert_eq!(
                permit
                    .get("resource_scope_unconstrained")
                    .and_then(Value::as_bool),
                Some(false),
                "{path} has a resource/scope-agnostic permit after GH #987 disarm: {permit:?}"
            );
        }
        let forbids = configmap
            .get("forbids")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("missing forbid scan for {path}"));
        for forbid in forbids {
            assert_ne!(
                forbid.get("normalized").and_then(Value::as_str),
                Some("forbid ( principal, action, resource )"),
                "{path} deploys an executable unconditional default-deny forbid instead of relying on Cedar implicit deny: {forbid:?}"
            );
        }
    }
}

#[test]
fn authz_004_dead_configmaps_are_deleted_from_blanket_baseline() {
    let root = repo_root();
    let policy = load_policy(&root);
    let baseline_paths = policy
        .get("baseline")
        .and_then(|baseline| baseline.get("paths"))
        .and_then(Value::as_array)
        .expect("baseline.paths array")
        .iter()
        .map(|path| path.as_str().expect("baseline path string"))
        .collect::<BTreeSet<_>>();

    let observed = collect(&root, &policy).unwrap_or_else(|e| panic!("collect on live tree: {e}"));
    let collected_paths = observed
        .get("configmaps")
        .and_then(Value::as_array)
        .expect("collected configmaps array")
        .iter()
        .filter_map(|configmap| configmap.get("path").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();

    for path in AUTHZ_004_DEAD_CONFIGMAP_PATHS {
        assert!(
            !baseline_paths.contains(path),
            "{path} must not remain in the blanket baseline after AUTHZ-004 deletion"
        );
        assert!(
            !collected_paths.contains(path),
            "{path} must be deleted rather than deployed as an unused blanket ConfigMap"
        );
    }
}

#[test]
fn gate_detects_the_blanket_when_baseline_is_emptied() {
    // Proves the green above is held ONLY by the documented grandfather, not by a stubbed evaluator:
    // emptying the baseline must turn the SAME live scan RED on the action-agnostic blanket.
    let root = repo_root();
    let mut policy = load_policy(&root);
    policy["baseline"] = serde_json::json!({ "paths": [] });
    let observed = collect(&root, &policy).unwrap_or_else(|e| panic!("collect on live tree: {e}"));

    let found = codes(evaluate_keyed(&policy, &observed));
    assert!(
        found.contains("CDP-UNCONSTRAINED-PERMIT"),
        "with no baseline the live blanket ConfigMaps must be detected: {found:?}"
    );
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
}

#[test]
fn red_fixture_fresh_blanket_configmap_is_red() {
    let dir = unique_tmp("red");
    let cm = dir.join("oya/fixture/iac/k8s/helm/templates/cedar.yaml");
    fs::create_dir_all(cm.parent().unwrap()).unwrap();
    fs::write(&cm, BLANKET_CONFIGMAP).unwrap();
    // Make repo-root discovery succeed for the collector's relative-path logic.
    fs::create_dir_all(dir.join("specs")).unwrap();
    fs::write(dir.join("specs/root-hub-pointers.json"), "{}").unwrap();

    let policy = serde_json::json!({
        "gate_id": GATE_ID,
        "deployed_suffix": "iac/k8s/helm/templates/cedar.yaml",
        "authored_subdirs": ["policy", "cedar"],
        "baseline": { "paths": [] }
    });
    let observed = collect(&dir, &policy).unwrap_or_else(|e| panic!("collect red fixture: {e}"));
    let found = codes(evaluate_keyed(&policy, &observed));
    assert_eq!(
        evaluate(&policy, &observed).verdict,
        Verdict::Red,
        "{found:?}"
    );
    assert!(found.contains("CDP-UNCONSTRAINED-PERMIT"), "{found:?}");

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn green_fixture_constrained_permit_in_authored_set_passes() {
    let dir = unique_tmp("green");
    // A deployed ConfigMap whose permit is action-constrained AND byte-equal (after normalization) to
    // the capability's authored policy — the parity target state.
    let permit = "permit ( principal, action == Action::\"doc.Read\", resource is Doc )\nwhen { principal.tenant_id == resource.tenant_id };";

    let cm = dir.join("oya/fixture/iac/k8s/helm/templates/cedar.yaml");
    fs::create_dir_all(cm.parent().unwrap()).unwrap();
    fs::write(
        &cm,
        format!(
            "apiVersion: v1\nkind: ConfigMap\ndata:\n  policies.cedar: |\n    {}\n",
            permit.replace('\n', "\n    ")
        ),
    )
    .unwrap();

    let authored = dir.join("oya/fixture/policy/doc.cedar");
    fs::create_dir_all(authored.parent().unwrap()).unwrap();
    fs::write(&authored, format!("// authored\n{permit}\n")).unwrap();

    let policy = serde_json::json!({
        "gate_id": GATE_ID,
        "deployed_suffix": "iac/k8s/helm/templates/cedar.yaml",
        "authored_subdirs": ["policy", "cedar"],
        "baseline": { "paths": [] }
    });
    let observed = collect(&dir, &policy).unwrap_or_else(|e| panic!("collect green fixture: {e}"));
    let report = evaluate(&policy, &observed);
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.violations);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn red_fixture_bare_resource_without_scope_predicate_is_red() {
    let dir = unique_tmp("resource");
    // Action is constrained and the permit is present in authored policy, but bare `resource` plus no
    // resource/scope predicate is still a production blanket over every resource of that action.
    let permit = "permit ( principal, action == Action::\"doc.Read\", resource );";

    let cm = dir.join("oya/fixture/iac/k8s/helm/templates/cedar.yaml");
    fs::create_dir_all(cm.parent().unwrap()).unwrap();
    fs::write(
        &cm,
        format!("apiVersion: v1\nkind: ConfigMap\ndata:\n  policies.cedar: |\n    {permit}\n"),
    )
    .unwrap();

    let authored = dir.join("oya/fixture/policy/doc.cedar");
    fs::create_dir_all(authored.parent().unwrap()).unwrap();
    fs::write(&authored, format!("// authored\n{permit}\n")).unwrap();

    let policy = serde_json::json!({
        "gate_id": GATE_ID,
        "deployed_suffix": "iac/k8s/helm/templates/cedar.yaml",
        "authored_subdirs": ["policy", "cedar"],
        "baseline": { "paths": [] }
    });
    let observed =
        collect(&dir, &policy).unwrap_or_else(|e| panic!("collect resource fixture: {e}"));
    let found = codes(evaluate_keyed(&policy, &observed));
    assert_eq!(
        evaluate(&policy, &observed).verdict,
        Verdict::Red,
        "{found:?}"
    );
    assert!(found.contains("CDP-UNCONSTRAINED-RESOURCE"), "{found:?}");
    assert!(
        !found.contains("CDP-UNCONSTRAINED-PERMIT"),
        "the resource/scope check must be independent from action broadness: {found:?}"
    );

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn red_fixture_missing_deployed_forbid_is_red() {
    let dir = unique_tmp("forbid");
    let permit = "permit ( principal, action == Action::\"doc.Read\", resource is Doc )\nwhen { principal.tenant_id == resource.tenant_id };";
    let forbid = "forbid ( principal, action == Action::\"doc.Read\", resource is Doc )\nwhen { resource.blocked };";

    let cm = dir.join("oya/fixture/iac/k8s/helm/templates/cedar.yaml");
    fs::create_dir_all(cm.parent().unwrap()).unwrap();
    fs::write(
        &cm,
        format!(
            "apiVersion: v1\nkind: ConfigMap\ndata:\n  policies.cedar: |\n    {}\n",
            permit.replace('\n', "\n    ")
        ),
    )
    .unwrap();

    let authored = dir.join("oya/fixture/policy/doc.cedar");
    fs::create_dir_all(authored.parent().unwrap()).unwrap();
    fs::write(&authored, format!("// authored\n{permit}\n{forbid}\n")).unwrap();

    let policy = serde_json::json!({
        "gate_id": GATE_ID,
        "deployed_suffix": "iac/k8s/helm/templates/cedar.yaml",
        "authored_subdirs": ["policy", "cedar"],
        "baseline": { "paths": [] }
    });
    let observed = collect(&dir, &policy).unwrap_or_else(|e| panic!("collect forbid fixture: {e}"));
    let found = codes(evaluate_keyed(&policy, &observed));
    assert_eq!(
        evaluate(&policy, &observed).verdict,
        Verdict::Red,
        "{found:?}"
    );
    assert!(found.contains("CDP-DEPLOYED-NOT-SUBSET"), "{found:?}");

    fs::remove_dir_all(&dir).ok();
}
