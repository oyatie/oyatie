---
doc_class: ReferenceImplementation
title: Bootstrap a regulated workload cluster with `oya-cloud-k8s-cluster-bootstrap-sdk`
microservice: cloud-k8s
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-cloud
related_adrs: [ADR-0121, ADR-0131, ADR-0244, ADR-0254]
related_artifacts:
  - microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml
  - microservices/cloud-k8s/contracts/proto/cloud-k8s.proto
  - microservices/cloud-k8s/sdk-plan.md
  - microservices/cloud-k8s/IP-003-cluster-bootstrap-kernel.md
  - microservices/cloud-k8s/IP-CLUSTERAPI-001-clusterclass-templates.md
doc_status: published
---

# Reference implementation — Bootstrap a regulated cluster + NetworkPolicy + event stream via `oya-cloud-k8s-cluster-bootstrap-sdk`

Runnable Rust program that mints a Cluster API workload cluster from an oyatie `ClusterClass`, attaches the Cilium CNI +
Istio (1.22) mesh control plane, applies a default-deny `NetworkPolicy`, waits for the reviewer-agent admission verdict,
attests the SLSA build provenance for every container image, and tails the cluster event stream until the first
`PodDisruptionBudget` is observed healthy. Mirrors the displacement target stated in the competitor-parity matrix: a tenant
who already runs an EKS cluster, an AKS cluster, or a Rancher RKE2 cluster should be able to land a regulated equivalent
on Cloud Hypervisor + Kata pods in under twelve minutes.

## `Cargo.toml`

```toml
[package]
name = "cloud-k8s-bootstrap-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
oya-cloud-k8s-cluster-bootstrap-sdk = "0.42.0"
oya-cloud-k8s-kubernetes-api-proxy-sdk = "0.42.0"
oya-trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tokio-stream = "0.1"
futures-util = "0.3"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::{Context, Result};
use futures_util::StreamExt;
use oya_cloud_k8s_cluster_bootstrap_sdk::{
    AdmissionDecision, ClusterApiTemplate, ClusterBootstrapClient, ClusterBootstrapConfig,
    ClusterBootstrapRequest, ClusterClass, CniChoice, ControlPlaneSize, K8sError, MeshChoice,
    NodePoolSpec, PackId, RuntimeChoice, ShuffleShardId, Tenant,
};
use oya_cloud_k8s_kubernetes_api_proxy_sdk::{
    EventClass, KubeProxyClient, KubeProxyConfig, NetworkPolicySpec, NodeReadyState,
    PodDisruptionBudgetSpec,
};
use oya_trace::TraceContext;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    // 1. — operator credentials bound to the regulated cell tier (Tier 1 = HIPAA-eligible per ADR-0254)
    let bootstrap_cfg = ClusterBootstrapConfig::builder()
        .endpoint("https://cloud-k8s.oyatie.local/v1".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.regulated.altavista-health")?)
        .pack(PackId::UsHipaa)
        .shuffle_shard(ShuffleShardId::derive_from_tenant())
        .service_account_credentials_path("/etc/oya/cloud-k8s/sa-creds.json")
        .request_timeout(Duration::from_secs(15))
        .bootstrap_deadline(Duration::from_secs(720)) // 12-minute parity target
        .build()?;

    let client = ClusterBootstrapClient::connect(bootstrap_cfg).await?;
    info!("connected to cloud-k8s control plane");

    // 2. Pick a ClusterClass — oyatie ships parity templates aligned to EKS / AKS / RKE2 sizing
    //    `regulated-3az-cloud-hypervisor-v1` ⇒ 3-AZ stretched control plane + Cloud Hypervisor + Kata pods
    let template = ClusterApiTemplate::parse("regulated-3az-cloud-hypervisor-v1")?;

    let req = ClusterBootstrapRequest::builder()
        .name("altavista-prod-east-1")
        .cluster_class(ClusterClass::from(template))
        .runtime(RuntimeChoice::KataOnCloudHypervisor)
        .cni(CniChoice::Cilium { ebpf_kube_proxy_replacement: true })
        .mesh(MeshChoice::Istio { version: "1.22.1".into(), ambient: true })
        .control_plane_size(ControlPlaneSize::Hardened {
            replicas: 5,
            etcd_disk_iops: 16_000,
            apiserver_audit_log_max_age_days: 90,
        })
        .node_pool(NodePoolSpec::builder()
            .name("regulated-workloads")
            .replicas(9)
            .machine_size("c8g.4xlarge-confidential")
            .availability_zones(["us-east-1a", "us-east-1b", "us-east-1c"])
            .surge_strategy_max_surge_percent(20)
            .image_signing_required(true)
            .build()?)
        .pod_security_admission_default("restricted")
        .image_signing_required(true) // ADR-0039 supply-chain
        .audit_chain_emission(true)
        .build()?;

    let bootstrap = client.bootstrap(req, trace.child()).await?;
    info!(
        cluster_id = %bootstrap.id(),
        cluster_class = %bootstrap.cluster_class(),
        capi_revision = %bootstrap.capi_revision(),
        "cluster bootstrap accepted"
    );

    // 3. Wait for the reviewer-agent admission verdict
    //    Five facets are evaluated: F1 architecture, F2 security, F3 SLO, F4 cost, F5 compliance (ADR-0254 §G)
    let mut backoff = Duration::from_millis(750);
    let verdict = loop {
        match client.admission_status(bootstrap.id()).await? {
            AdmissionDecision::Pending { facets_in_flight } => {
                info!(?facets_in_flight, "admission still pending");
                sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(15));
            }
            other => break other,
        }
    };

    match verdict {
        AdmissionDecision::Approve { facets_passed, evidence_root } => {
            info!(facets_passed, %evidence_root, "reviewer-agent APPROVED");
        }
        AdmissionDecision::Block { facet_failures } => {
            warn!(?facet_failures, "reviewer-agent BLOCKED; aborting");
            return Ok(());
        }
        AdmissionDecision::NeedsHuman { reason, escalation_link } => {
            warn!(%reason, %escalation_link, "admission escalated to human; aborting auto-flow");
            return Ok(());
        }
        AdmissionDecision::Pending { .. } => unreachable!(),
    }

    // 4. Wait for the control plane to settle (apiserver healthy + etcd quorum + first node Ready)
    client
        .await_control_plane_ready(bootstrap.id(), Duration::from_secs(420))
        .await
        .context("control-plane ready timeout")?;

    let node_ready = client
        .await_first_node(bootstrap.id(), Duration::from_secs(120))
        .await?;
    match node_ready {
        NodeReadyState::Ready { node_name, attestation_kind } => {
            info!(%node_name, %attestation_kind, "first node Ready with measured-boot attestation");
        }
        NodeReadyState::FailedAttestation { node_name, reason } => {
            anyhow::bail!("node {node_name} failed attestation: {reason}");
        }
    }

    // 5. Switch to the kubernetes-api-proxy SDK — every kubectl-equivalent call is Cedar-evaluated + audit-chained
    let proxy_cfg = KubeProxyConfig::builder()
        .endpoint(bootstrap.api_proxy_endpoint())
        .tenant(Tenant::parse("oyatie.b2b.regulated.altavista-health")?)
        .pack(PackId::UsHipaa)
        .request_timeout(Duration::from_secs(8))
        .build()?;
    let proxy = KubeProxyClient::connect(proxy_cfg).await?;

    // 6. Default-deny NetworkPolicy — required by HIPAA pack overlay; refusal evidence emitted otherwise
    let np = NetworkPolicySpec::builder()
        .namespace("regulated-workloads")
        .name("default-deny-all")
        .pod_selector_match_all()
        .policy_types(["Ingress", "Egress"])
        .build()?;
    let applied_np = proxy.apply_network_policy(np, trace.child()).await?;
    info!(
        np_uid = %applied_np.uid(),
        cedar_decision_id = %applied_np.cedar_decision_id(),
        audit_chain_event_id = %applied_np.audit_chain_event_id(),
        "default-deny NetworkPolicy applied"
    );

    // 7. PodDisruptionBudget — minAvailable 80% blocks regulators from accepting fewer than 4/5 control-plane replicas
    let pdb = PodDisruptionBudgetSpec::builder()
        .namespace("regulated-workloads")
        .name("regulated-workloads-min-80pct")
        .selector_match_labels([("app", "regulated-workloads")])
        .min_available_percent(80)
        .unhealthy_pod_eviction_policy("IfHealthyBudget")
        .build()?;
    let applied_pdb = proxy.apply_pod_disruption_budget(pdb, trace.child()).await?;
    info!(
        pdb_uid = %applied_pdb.uid(),
        "PodDisruptionBudget applied"
    );

    // 8. Tail the event stream until PDB observed healthy or timeout
    let mut stream = proxy
        .stream_cluster_events(bootstrap.id(), Duration::from_secs(180))
        .await?;

    while let Some(event) = stream.next().await {
        let event = event?;
        match event.class() {
            EventClass::PodDisruptionBudgetHealthy if event.subject() == applied_pdb.uid().as_str() => {
                info!(at = %event.observed_at(), "PDB healthy; bootstrap stable");
                break;
            }
            EventClass::PodSecurityAdmissionRefusal => {
                warn!(
                    pod = %event.subject(),
                    cedar_decision_id = %event.cedar_decision_id().unwrap_or_default(),
                    "Pod refused by Pod-Security-Admission (restricted profile)"
                );
            }
            EventClass::ImageSigningRefusal => {
                warn!(
                    image = %event.subject(),
                    expected_signer = %event.expected_signer().unwrap_or_default(),
                    "Image refused — not co-signed by the oyatie release key"
                );
            }
            other => {
                info!(class = ?other, subject = %event.subject(), "event observed");
            }
        }
    }

    info!(
        cluster_id = %bootstrap.id(),
        api_proxy = %bootstrap.api_proxy_endpoint(),
        "bootstrap complete — ready for tenant workloads"
    );

    Ok(())
}
```

## Run it

```bash
cargo run --release
```

Expected output (trimmed):
```
INFO  connected to cloud-k8s control plane
INFO  cluster bootstrap accepted cluster_id=ck8s-altavista-prod-east-1-… cluster_class=regulated-3az-cloud-hypervisor-v1 capi-revision=1.7.4
INFO  admission still pending facets_in_flight=["F4-cost","F5-compliance"]
INFO  reviewer-agent APPROVED facets_passed=5 evidence_root=blake3-256:7d29…
INFO  first node Ready with measured-boot attestation node_name=ip-10-4-12-91 attestation_kind=tdx-attestation-v1
INFO  default-deny NetworkPolicy applied np_uid=… cedar_decision_id=… audit_chain_event_id=…
INFO  PodDisruptionBudget applied pdb_uid=…
INFO  PDB healthy; bootstrap stable
INFO  bootstrap complete — ready for tenant workloads
```

End-to-end median observed on staging:  10 m 42 s (control-plane ready), 11 m 17 s (first node Ready), 11 m 49 s (PDB
healthy). Beats the EKS-via-eksctl public reference of ~14 m and AKS-via-az-cli of ~16 m for an equivalent 3-AZ regulated
profile. Source: `microservices/cloud-k8s/competitor-parity-matrix.md` §3 bootstrap-latency rows.

## SDK correctness guarantees

1. `bootstrap(...)` is **deterministic** for a given `ClusterClass` + `NodePoolSpec` pair — the same request replays into
   the same audit-chain anchor. Replay is enforced by `cloud-k8s.audit_chain.replay` evidence checked by
   `oya gate validate audit-chain-replay`.
2. `admission_status(...)` cannot succeed without all five reviewer facets returning Approve or NeedsHuman; the SDK
   rejects forged `Approve` payloads whose Ed25519 signature does not chain back to the oyatie release key.
3. `apply_network_policy(...)` evaluates Cedar **before** the kube-apiserver round-trip. Cedar refusal short-circuits
   without producing an apiserver audit-log entry that could leak intent.
4. `apply_pod_disruption_budget(...)` rejects `minAvailable < 80` for any namespace whose tenant pack overlay declares a
   HIPAA / PCI / EU-AI-Act Annex-III rating; this is the ADR-0254 §H minimum-quorum invariant.
5. `stream_cluster_events(...)` is at-least-once with idempotent event IDs (`event.event_id()`); callers may dedupe
   safely by `event_id`.
6. Every mutating call carries a `TraceContext` child and emits a `cloud-k8s.audit.chain.appended` event before the
   response is acknowledged.

## Tests

```bash
cargo test --features hermetic
```

The `hermetic` feature uses `oya_cloud_k8s_cluster_bootstrap_sdk::testkit::Hermetic`, which spins up an in-process
mock-apiserver backed by `kine` (etcd-replacement) + a stubbed Cilium agent + a stubbed Istio control plane; tests finish
in ≤ 60 s and do not require a real Kubernetes cluster, real Cloud Hypervisor host, or real HSM.

## Error budget

- `K8sError::CapiTemplateRevisionPinExpired` — the `ClusterClass` was pinned to a CAPI revision that has since been
  retired. Fetch the latest revision with `client.list_cluster_class_revisions(...)` and resubmit.
- `K8sError::AdmissionFacetRetryable { facet }` — a transient infrastructure fault (e.g. a Cedar policy-snapshot fetch
  timed out). The SDK has already retried twice; surface to the operator and rerun after `cloud_k8s.slo.api_proxy_p99`
  recovers.
- `K8sError::NodeAttestationFailed { node_name, reason }` — the node's measured-boot quote did not chain to the platform
  root-of-trust. Quarantine the node, file `cloud_k8s.slo.attestation_failure`, and consult the runbook
  `microservices/cloud-k8s/runbooks/attestation-failure.md`.
- `K8sError::ImageSigningRefusal` — a container image is not co-signed by the oyatie release key. Resign via
  `oya supply-chain adr0039` or roll back to the last signed revision.

## Pack overlay behaviour

When `PackId::UsHipaa`, `PackId::EuGdpr`, or `PackId::EuAiActAnnexIii` is set:

- `pod_security_admission_default("restricted")` is forced; lower profiles are refused at the SDK boundary before any
  network call.
- `audit_chain_emission(true)` is forced.
- `image_signing_required(true)` is forced.
- `await_first_node(...)` requires `attestation_kind ∈ {tdx-attestation-v1, sev-snp-attestation-v1}`; bare `software-only`
  attestations are refused.
- `min_available_percent` floor is raised to 80; setter accepts but the server refuses anything below the floor and
  returns the refusal evidence under the audit-chain event id surfaced in the error variant.

## Migration parity callouts

- **From EKS / eksctl**: the `eks-cluster-config.yaml` → `ClusterBootstrapRequest` mapping is documented in
  `microservices/cloud-k8s/migration-playbooks/from-aws-eks.md` §3; node-group autoscaling translates to `NodePoolSpec`
  + `surge_strategy_max_surge_percent` directly.
- **From AKS / az aks**: the `--enable-addons azure-policy` flag maps to oyatie's `pack(PackId::*)` overlay; there is no
  per-cluster policy add-on — policy is tenant-pack overlay, not cluster scope.
- **From Rancher RKE2**: `ClusterClass::regulated-3az-cloud-hypervisor-v1` is the closest sizing parity to RKE2's
  `medium-hardened` profile; `cluster.yaml` `network.plugin: cilium` translates to `CniChoice::Cilium`.
- **From OpenShift OCP**: `oc adm policy` flows translate to `cedar_decision_id` enforcement evidence; SCC profiles are
  superseded by the pack overlay (no per-cluster SCC editor surface — refused by design).

See the migration playbooks under `microservices/cloud-k8s/migration-playbooks/` for vendor-by-vendor field-level mapping.
