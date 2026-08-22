---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
status: Active
entry_gate: |
  ADR-0117 + ADR-0120 + ADR-0121 + ADR-0131 accepted; /specs/per-microservice-flat-layout.json published; cargo workspace ready to accept the 62 new crates under microservices/cloud-k8s/src/crates/; bare-metal / OCI compute provisioned by cloud-iac is reachable; OpenBao + audit-chain µservices live (cloud-k8s consumes their substrate).
exit_gate: |
  All 15 IPs merged; cluster bootstrap end-to-end drill completes ≤ 30 min p99 against the on-prem KR primary cell; cargo nextest run --workspace exits 0; oya gate validate per-microservice-layout --microservice cloud-k8s exits 0; oya gate validate authority-cohesion exits 0; oya gate validate cis-k8s-benchmark exits 0; HG-CLOUD-K8S gate in /specs/hyperscaler-gates.json registers green; ClusterBootstrapped event end-to-end observed in audit-chain seal.
depends_on:
  - milestone: M01-foundation
    phase: prior phases per master-plan-sequencing
    reason: workspace + branch-protection + OpenBao + audit-chain authority must precede cluster authoring
owner_team: axis-cloud
related_adrs: [ADR-0117, ADR-0120, ADR-0121, ADR-0139, ADR-0131]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/hyperscaler-gates.json]
date: 2026-05-17
doc_status: published
---

# P01-onprem-k8s-substrate: Land the on-prem Kubernetes cluster substrate

## Purpose

This phase ships the full ADR-0121 design — vanilla kubeadm + containerd + Istio + Envoy + CSI drivers + Cedar-policy-mediated kube-apiserver proxy — under the per-microservice flat layout of ADR-0131. It is delivered as one phase in M01-foundation because every other oyatie µservice depends on having a cluster to schedule on; observability needs it for the Grafana stack; cell needs it for tenant scheduling; the workload µservices need it to host their pods.

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (vanilla upstream Kubernetes; same code OKE worker nodes run; CNCF conformance by construction).
- Nothing scheduled-for-distinct-tracked-work (no shell-script "TODO" path; every cluster mutation is a Foundry capability with audit-chain emission).
- No silent regression (CIS Kubernetes Benchmark v1.9 lane is BLOCKER; etcd encryption mandatory).
- Per-microservice flat layout (this phase is itself a native author under ADR-0131).
- Sovereignty-grade isolation (per-pack cluster boundary; no cross-pack PV; cross-cluster mTLS only).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `cloud-k8s` | `cluster-bootstrap`, `node-lifecycle`, `network-policy`, `service-mesh-control-plane`, `ingress-controller`, `csi-storage-driver`, `kubernetes-api-proxy` | All under `microservices/cloud-k8s/` per ADR-0131 | 62 crates per PRD §"Bounded Contexts" layer mapping |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- `.github/branch-protection.yaml` — add `cloud-k8s-iac-smoke`, `check-cis-k8s-benchmark` to required_status_checks on `dev` and `staging`.
- `docs/standards/cloud-k8s-stack.md` (NEW) — cross-cutting on-prem Kubernetes stack standard; LTS version pins; admission-controller configuration; etcd encryption posture.
- `Cargo.toml` (workspace) — register the 62 new crates under `microservices/cloud-k8s/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-CLOUD-K8S gate per ADR-0123.
- `/specs/cloud-k8s-cluster-state.json` (NEW) — cluster state machine; bootstrap → joined → upgrading → reset transitions.

Naming justifications for the new crate families are in `microservices/cloud-k8s/PRD.md` §"Bounded Contexts".

### Out-of-scope

- Multi-cluster Istio federation across packs (scheduled-for-distinct-tracked-work to M03 per ADR-0117 §"Cross-region story"). Each pack runs an isolated cluster at M01 launch.
- Karpenter integration (scheduled-for-distinct-tracked-work to M02 per ADR-0198 — Karpenter is the canonical node autoscaler; Cluster Autoscaler is explicitly rejected; node-add is manual via Foundry capability at M01).
- Workload Identity Federation (SPIFFE-to-cloud-IAM-vendor automatic federation; scheduled-for-distinct-tracked-work to M03 when OCI / AWS / GCP packs activate).
- In-process Kubernetes-API proxy (HTTP-reverse-proxy at M01; in-process fork scheduled-for-distinct-tracked-work to M04).
- Tenant-facing kubectl SDK (only operator + agent access at M01; tenant kubectl exposure is a successor-IP ADR).

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-layer-a-iac-kubeadm-containerd-istio-envoy.md`](IP-001-layer-a-iac-kubeadm-containerd-istio-envoy.md) | Layer-A IaC: Helm charts (`istio-base`, `istiod`, `envoy-gateway`, `cni-cilium`) + OpenTofu modules (`kubeadm-cluster`, `containerd-config`) + Kustomize base + pack-kr overlay; CSI drivers per backend (block-volume, object, file) deployed via Helm | pending | axis-cloud | — |
| [`IP-002-onprem-k8s-stack-standard.md`](IP-002-onprem-k8s-stack-standard.md) | `docs/standards/cloud-k8s-stack.md` cross-cutting standard: LTS version pins (containerd 2.3.0, runc 1.4.0, CNI plugins 1.6.0, k8s 1.35, Istio 1.29.2); admission-controller config; etcd encryption posture | pending | axis-cloud + ops-security | — |
| [`IP-003-cluster-bootstrap-kernel.md`](IP-003-cluster-bootstrap-kernel.md) | `cloud-k8s-cluster-bootstrap-kernel`: port traits (KubeadmCommander, EtcdSnapshotter, ControlPlaneInspector), entities (Cluster, ControlPlaneNode, KubeadmConfig, EtcdSnapshot, BootstrapEvidence) | pending | axis-cloud | IP-002 |
| [`IP-004-cluster-bootstrap-domain.md`](IP-004-cluster-bootstrap-domain.md) | `cloud-k8s-cluster-bootstrap-domain`: kubeadm-version compatibility arithmetic; etcd-snapshot integrity computation; upgrade-window math | pending | axis-cloud | IP-003 |
| [`IP-005-cluster-bootstrap-usecase.md`](IP-005-cluster-bootstrap-usecase.md) | `cloud-k8s-cluster-bootstrap-usecase` (per ADR-0106): orchestrators for cluster.create / control-plane.upgrade / etcd.backup / etcd.restore via ports | pending | axis-cloud | IP-004 |
| [`IP-006-cluster-bootstrap-adapter-kubeadm.md`](IP-006-cluster-bootstrap-adapter-kubeadm.md) | `cloud-k8s-cluster-bootstrap-adapter-kubeadm` (backend-qualified per ADR-0105 Amendment 3): shells out to kubeadm CLI; reads /etc/kubernetes/; etcd snapshot integration | pending | axis-cloud | IP-003 |
| [`IP-007-node-lifecycle-kernel-usecase.md`](IP-007-node-lifecycle-kernel-usecase.md) | `cloud-k8s-node-lifecycle-{kernel,domain,usecase}`: NodeRegistry + NodeDrainer ports; cordon/drain math; PDB-aware eviction planning | pending | axis-cloud | IP-003 |
| [`IP-008-network-policy-kernel-usecase.md`](IP-008-network-policy-kernel-usecase.md) | `cloud-k8s-network-policy-{kernel,domain,usecase}`: Cedar → NetworkPolicy + Istio AuthorizationPolicy emission | pending | axis-cloud + ops-security | IP-003 |
| [`IP-009-service-mesh-control-plane-istio.md`](IP-009-service-mesh-control-plane-istio.md) | `cloud-k8s-service-mesh-control-plane-{kernel,usecase,adapter-istio}`: IstioCommander port; istioctl wrap; IstioOperator CR; canary control-plane upgrade primitive | pending | axis-cloud | IP-001 |
| [`IP-010-ingress-controller-envoy.md`](IP-010-ingress-controller-envoy.md) | `cloud-k8s-ingress-controller-{kernel,usecase,adapter-envoy}`: Gateway / VirtualService / DestinationRule emission; TLS termination; SNI route | pending | axis-cloud | IP-009 |
| [`IP-011-csi-storage-driver-per-backend.md`](IP-011-csi-storage-driver-per-backend.md) | `cloud-k8s-csi-storage-driver-{kernel,usecase,adapter-block,adapter-object,adapter-file}`: CSI provisioner per-backend; QoS class enforcement; VolumeSnapshot integration | pending | axis-cloud | IP-003 |
| [`IP-012-kubernetes-api-proxy.md`](IP-012-kubernetes-api-proxy.md) | `cloud-k8s-kubernetes-api-proxy-{kernel,usecase,adapter,rest,worker,sdk,app}`: HTTP reverse-proxy mediating kube-apiserver; Cedar policy decision; audit-chain emit per call | pending | axis-cloud + ops-security | IP-003 |
| [`IP-013-cluster-bootstrap-rest-worker-sdk-app.md`](IP-013-cluster-bootstrap-rest-worker-sdk-app.md) | `cloud-k8s-cluster-bootstrap-{rest,worker,sdk,app}`: REST surface, bootstrap-watcher worker, Rust SDK, composition root | pending | axis-cloud | IP-005, IP-006 |
| [`IP-014-branch-protection-and-hyperscaler-gate.md`](IP-014-branch-protection-and-hyperscaler-gate.md) | Add `cloud-k8s-iac-smoke`, `check-cis-k8s-benchmark` to `.github/branch-protection.yaml`; register HG-CLOUD-K8S in `/specs/hyperscaler-gates.json` | pending | axis-foundry + axis-cloud | IP-001 .. IP-013 |
| [`IP-015-observability-slo-and-authority-cohesion.md`](IP-015-observability-slo-and-authority-cohesion.md) | Author `microservices/cloud-k8s/slos/*.openslo.yaml` (cluster-bootstrap availability, node-join latency, NetworkPolicy propagation, api-proxy decision latency); register HG-CLOUD-K8S in authority-cohesion | pending | axis-cloud + axis-observability | IP-014 |

Coverage check vs. ADR-0121 §"Required successor-IP" + ADR-0117 §"Compute" + ADR-0131 §"Per-microservice flat layout":
- Layer-A IaC (kubeadm + containerd + Istio + Envoy + Cilium CNI + CSI drivers) — IP-001 + IP-002.
- Kernel + Domain + Usecase + Adapter per BC — IP-003 .. IP-012.
- Wire-up + composition root — IP-013.
- CI lane registration — IP-014.
- SLO authoring + authority-cohesion — IP-015.

## Acceptance Gates

All gates must pass before `exit_gate` is declared.

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --microservice cloud-k8s        # layer ordering
oya gate validate lean-a2 --microservice cloud-k8s        # cross-product refusal
oya gate validate port-location --microservice cloud-k8s  # ports in kernel
oya gate validate layer-correctness --microservice cloud-k8s
oya gate validate per-microservice-layout --microservice cloud-k8s  # ADR-0131
oya gate validate statelessness --microservice cloud-k8s
oya gate validate shardability --microservice cloud-k8s
oya gate validate authority-cohesion                       # HG-CLOUD-K8S
oya gate validate hyperscaler-maturity-claims              # ADR-0123
oya gate validate cis-k8s-benchmark --microservice cloud-k8s # CIS K8s Bench v1.9
```

### Substrate gates introduced by this phase

```bash
oya gate validate cluster-bootstrap-conformance --pack pack-kr
oya gate validate cni-cilium-conformance
oya gate validate istio-strict-mtls-enforced
oya gate validate envoy-tls-13-only
oya gate validate kubernetes-api-proxy-only-path
oya gate validate cosign-admission-controller-enforced
oya gate validate etcd-encryption-at-rest-enforced
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Cluster bootstrap | `cargo nextest run -p cloud-k8s-cluster-bootstrap-usecase --test bootstrap_happy_path` | kubeadm init + control-plane Ready ≤ 30 min p99 |
| Node-join | `cargo nextest run -p cloud-k8s-node-lifecycle-usecase --test node_join_happy_path` | node Ready ≤ 5 min p99 |
| NetworkPolicy propagation | `cargo nextest run -p cloud-k8s-network-policy-usecase --test policy_propagation` | policy applied ≤ 30 s p99 |
| Istio mTLS strict enforcement | scripted e2e: cross-namespace cleartext refused | 100 % refused |
| Envoy SNI routing | scripted e2e: SNI-mismatched request refused | 100 % refused |
| CSI block-volume provision | `cargo nextest run -p cloud-k8s-csi-storage-driver-usecase --test block_provision` | PV bound ≤ 30 s p99 |
| API-proxy refuses direct 6443 | NetworkPolicy probe | 100 % refused at NodePort |
| Cosign-unsigned image refused | admission-webhook integration test | 100 % refused |
| etcd snapshot + restore | scripted e2e: snapshot → kill etcd → restore | data integrity verified |
| Kubeadm upgrade | scripted e2e: 1.35 → 1.36 minor-upgrade | ≤ 90 min cluster-wide |

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --microservice cloud-k8s
oya gate validate ontology-type-registry --microservice cloud-k8s
```

## Clean Architecture Compliance

Layer assignments and dependency direction:

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `*-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `*-domain` | `domain` | `kernel` | `usecase`, `adapter`, `rest`, `worker`, `app` |
| `*-usecase` | `usecase` | `domain`, `kernel` | `adapter`, `rest`, `worker`, `app` |
| `*-adapter` | `adapter` | `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `*-adapter-kubeadm` `*-adapter-istio` `*-adapter-envoy` `*-adapter-block` `*-adapter-object` `*-adapter-file` | `adapter` | same | same |
| `*-rest` | `rest` | `usecase`, `domain`, `kernel`, `api` | `adapter` directly (uses ports) |
| `*-worker` | `worker` | `usecase`, `domain`, `kernel`, `api` | `adapter` directly (uses ports) |
| `*-sdk` | `sdk` | `kernel`, `api` | adapters, worker, rest, app |
| `*-app` | `app` | (composition-root wiring only) | none — but only wiring |

Port traits live exclusively in `*-kernel` crates; implementations exclusively in `*-adapter` (or backend-qualified `*-adapter-<backend>`). Domain calls through ports; domain never imports adapter.

Cross-product integration check: this phase introduces NO direct imports between `cloud-k8s` and any other product µservice's crates. All cross-product data flow uses Workflow events (`ClusterBootstrapped`, `NodeJoined`, `NodeFailed`, `NodeDrained`, `NetworkPolicyApplied`, `IstioPolicyChanged`, `KubeadmUpgraded`) and Ontology reads/writes (`Cluster`, `Node`, `NetworkPolicy`, `IstioRevision`, `Gateway`, `StorageClass`).

## ChangeSet Contract per IP

Every IP in this phase emits a ChangeSet per ADR-0110. The minimum ChangeSet payload per IP at `microservices/cloud-k8s/evidence/multispectrum/<change_id>-<unix_ts>.json` on `oya vcs done`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "cloud-k8s",
  "milestone": "M01-foundation",
  "phase": "P01-onprem-k8s-substrate",
  "claim_paths": ["microservices/cloud-k8s/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["microservices/cloud-k8s/PRD.md§<section>", "ADR-0121§<section>"],
  "acceptance_lanes_green": ["cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny", "lean-a1", "lean-a2", "lean-a3", "lean-a4", "per-microservice-layout", "cis-k8s-benchmark"],
  "test_count": {"unit": <int>, "integration": <int>, "e2e": <int>},
  "coverage_pct": <float>,
  "multispectrum_review_facets": ["F1..F9", "A1..A7", "M1..M2"],
  "signature": "Ed25519:<sig>",
  "executed_at": "ISO8601"
}
```

## Per-IP Test Coverage Threshold

Same shape as observability PHASE-01 (kernel 90 % / domain 95 % / usecase 90 % / adapter 85 % / rest 85 % / worker 85 % / sdk 90 % / app 60 % / IaC ≥ 1 helm-install + helm-test + 1 kind smoke per chart).

## branch-protection.yaml diff preview (IP-014)

```yaml
branches:
  dev:
    required_status_checks:
      # ADDED by this phase:
      - cloud-k8s-iac-smoke                # NEW
      - check-cis-k8s-benchmark            # NEW (Kubernetes hardening BLOCKER)
      - check-cosign-admission             # NEW (supply-chain BLOCKER)
      - check-etcd-encryption              # NEW (data-at-rest BLOCKER)
      - check-istio-strict-mtls            # NEW (mesh BLOCKER)
```

## Oya VCS Symbol Locks

Per ADR-0116, this phase uses `oya vcs` primitives exclusively. Grit and ICM are explicitly NOT used.

```bash
cargo run -p dev-cli -- vcs claim --agent <agent-id> --intent "<IP-NNN-slug>" --paths "microservices/cloud-k8s/src/crates/<crate>/**"
cargo run -p dev-cli -- vcs verify --agent <agent-id> --changeset <id>
cargo run -p dev-cli -- vcs done --agent <agent-id> --changeset <id>
cargo run -p dev-cli -- vcs promote --changeset <id>
```

Multispectrum evidence per docs/AGENTS.md §changeset: each IP emits `microservices/cloud-k8s/evidence/multispectrum/<change_id>-<unix_ts>.json` per `/specs/multispectrum-review.json` v2.4.0.

## References

- ADR-0121: On-prem Kubernetes stack (this phase's design authority).
- ADR-0117: Cloud-native infrastructure progression.
- ADR-0120: Rust-first on-prem tooling.
- ADR-0131: Per-microservice flat layout (this phase's location authority).
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0106: usecase rename.
- ADR-0123: Hyperscaler maturity claim gate (HG-CLOUD-K8S).
- ADR-0139: Agentic SLO-gated promotion (cluster substrate is precondition).
- `microservices/cloud-k8s/PRD.md`.
- Memory: `feedback_milestone_phase_hierarchy.md`, `feedback_naming_justification.md`, `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`.
