#![allow(clippy::expect_used, clippy::panic)]

use intelligence_worker_app::{WorkerKind, default_worker_ownership};

#[path = "xproxy_worker_ownership/agent_runtime.rs"]
mod agent_runtime;
#[path = "xproxy_worker_ownership/parity_drift.rs"]
mod parity_drift;
#[path = "xproxy_worker_ownership/route_auth.rs"]
mod route_auth;
#[path = "xproxy_worker_ownership/safety.rs"]
mod safety;

#[test]
fn worker_ownership_map_keeps_hot_path_and_control_plane_separate() {
    let map = default_worker_ownership();
    assert!(
        map.iter()
            .any(|worker| worker.name == "intelligence-app-gateway"
                && worker.kind == WorkerKind::GatewayDeployment)
    );
    assert!(map.iter().any(|worker| worker.name == "route-controller"));
    assert!(
        map.iter()
            .any(|worker| worker.name == "model-inventory-worker")
    );
    assert!(
        map.iter()
            .any(|worker| worker.name == "drift-parity-worker")
    );
    assert!(map.iter().any(|worker| worker.kind == WorkerKind::CronJob));
    assert!(
        map.iter()
            .all(|worker| !worker.hot_path || worker.name == "intelligence-app-gateway")
    );
    assert!(
        map.iter()
            .all(|worker| !worker.writes_raw_prompts_or_secrets)
    );
}

/// Read the committed worker manifest through its DECLARED binding.
///
/// This was a bare repo-relative path resolved against the process working directory. Buck runs
/// the action from the sandbox root so it resolved there, but `cargo test` runs from the package
/// directory and the read missed entirely. The Cargo merge path (ADR-0716) is the authority, so
/// the manifest is named rather than guessed: Buck binds it with `$(location ...)` and Cargo
/// with a non-forcing `relative = true` entry in `.cargo/config.toml`. An unbound or non-regular
/// resource fails closed instead of silently reading the wrong bytes.
fn manifest() -> String {
    const BINDING: &str = "OYATIE_INTELLIGENCE_K8S_MANIFEST";
    let path = std::env::var_os(BINDING)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| panic!("FAIL-CLOSED: declared manifest binding {BINDING} is unset"));
    let metadata = std::fs::symlink_metadata(&path)
        .unwrap_or_else(|err| panic!("FAIL-CLOSED: inspect {BINDING}={}: {err}", path.display()));
    assert!(
        !metadata.file_type().is_symlink() && metadata.is_file(),
        "FAIL-CLOSED: {BINDING}={} must be a regular non-symlink file",
        path.display()
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("FAIL-CLOSED: read {BINDING}={}: {err}", path.display()))
}

fn assert_manifest_contains(manifest: &str, needle: &str) {
    assert!(manifest.contains(needle), "manifest missing {needle}");
}

#[test]
fn k8s_manifest_declares_crds_workers_canaries_and_hardening() {
    let manifest = manifest();
    for crd in [
        "providerbackends.intelligence-app.oyatie.io",
        "modelroutes.intelligence-app.oyatie.io",
        "modelaliassets.intelligence-app.oyatie.io",
        "promptprofiles.intelligence-app.oyatie.io",
        "thinkingpolicies.intelligence-app.oyatie.io",
        "subscriptionseats.intelligence-app.oyatie.io",
        "wireprofiles.intelligence-app.oyatie.io",
        "toolcompatibilityprofiles.intelligence-app.oyatie.io",
        "gatewaycircuitbreakers.intelligence-app.oyatie.io",
        "capabilityparitybaselines.intelligence-app.oyatie.io",
        "agentruntimeprofiles.intelligence-app.oyatie.io",
        "agentmemorybindings.intelligence-app.oyatie.io",
        "agentskillbundles.intelligence-app.oyatie.io",
        "agentschedules.intelligence-app.oyatie.io",
        "agentdelegationpolicies.intelligence-app.oyatie.io",
        "agentworkspacebindings.intelligence-app.oyatie.io",
        "guardraildetectionprofiles.intelligence-app.oyatie.io",
        "evidenceretentionprofiles.intelligence-app.oyatie.io",
        "intransitredactionprofiles.intelligence-app.oyatie.io",
        "safetysignalpolicies.intelligence-app.oyatie.io",
        "manualreviewescalations.intelligence-app.oyatie.io",
    ] {
        assert_manifest_contains(&manifest, crd);
    }

    for deployment in [
        "name: intelligence-app-gateway",
        "name: route-controller",
        "name: model-inventory-worker",
        "name: credential-refresh-worker",
        "name: analytics-metering-worker",
        "name: circuit-breaker-worker",
        "name: ops-api",
        "name: agent-runtime-controller",
        "name: agent-scheduler-worker",
        "name: agent-delegation-worker",
        "name: safety-enforcement-controller",
        "name: guardrail-detection-worker",
        "name: evidence-retention-controller",
    ] {
        assert_manifest_contains(&manifest, deployment);
    }

    assert_manifest_contains(&manifest, "kind: CronJob");
    assert_manifest_contains(&manifest, "name: drift-parity-worker");
    assert_manifest_contains(&manifest, "kind: Job");
    assert_manifest_contains(&manifest, "name: compatibility-worker");
    assert_manifest_contains(&manifest, "kind: ServiceAccount");
    assert_manifest_contains(&manifest, "kind: Role");
    assert_manifest_contains(&manifest, "kind: RoleBinding");
    assert_manifest_contains(&manifest, "kind: NetworkPolicy");
    assert_manifest_contains(&manifest, "kind: PodDisruptionBudget");
    assert_manifest_contains(&manifest, "readOnlyRootFilesystem: true");
    assert_manifest_contains(&manifest, "allowPrivilegeEscalation: false");
    assert_manifest_contains(&manifest, "runAsNonRoot: true");
    assert_manifest_contains(&manifest, "path: /livez");
    assert_manifest_contains(&manifest, "path: /readyz");
    assert!(!manifest.contains("value: sk-"));
}
