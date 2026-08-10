---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-009-openbao-operator
status: pending
owner: axis-cloud-secrets + ops-sre
acceptance_lanes: [controller-conformance, kind-e2e]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: openbao-operator kernel + domain + usecase + api + adapter + app

## Intent

Ship the Kubernetes operator (kube-rs) that manages OpenBao cluster lifecycle: deploy via Helm, unseal via PKCS#11 HSM, drive rolling upgrades, monitor Raft health.

## ChangeSet boundary

Six new crates: kernel, domain, usecase, api, adapter, app. Single deployment binary.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-cloud-secrets-openbao-operator-kernel/` | create — `OpenBaoCluster`, `UnsealState`, `RaftPeer`, `UpgradePlan` |
| `…/oya-cloud-secrets-openbao-operator-domain/` | create — pure Raft-peer + unseal-quorum arithmetic |
| `…/oya-cloud-secrets-openbao-operator-usecase/` | create — Reconcile<OpenBaoCluster> |
| `…/oya-cloud-secrets-openbao-operator-api/` | create — CRD types |
| `…/oya-cloud-secrets-openbao-operator-adapter/` | create — kube-rs CRD watcher; Helm-template emission |
| `…/oya-cloud-secrets-openbao-operator-app/` | create — controller binary |
| 6× catalog yamls | create |

## CRD

```yaml
apiVersion: cloud-secrets.oyatie.dev/v1
kind: OpenBaoCluster
metadata:
  name: openbao
  namespace: cloud-secrets-kr
spec:
  pack: kr
  replicas: 5
  autoUnseal:
    type: pkcs11
    pkcs11SecretRef: openbao:secret/shared/cloud-secrets/hsm-pin
  storage:
    type: postgres
    postgresClusterRef: postgres-kr
status:
  raftLeader: openbao-2
  sealed: false
  raftPeers:
    - id: openbao-0
      state: voter
      reachable: true
```

## Acceptance Gates

```bash
cargo nextest run -p 'oya-cloud-secrets-openbao-operator-*'
# kind cluster e2e
kind create cluster --name cs-test
kubectl apply -f microservices/cloud-secrets/tests/e2e/cluster-fixtures/
cargo run -p oya-cloud-secrets-openbao-operator-app
# verify cluster reaches healthy within 5 min
```

## Test Plan

- Reconcile loop tests with mocked Kubernetes API.
- kind cluster e2e: deploy 5-node Raft + Postgres + HSM-emulator; reach Ready within 10 min.

## Halt Conditions

- Reconciler not idempotent — BLOCKER.

## Next IP

`IP-010-key-rotation-scheduler-worker.md`

## Wave 15-IP-substance A-G

### A. Problem
OpenBao must be a managed substrate, not a one-off install. Manual cluster lifecycle would leave unseal, Raft health, upgrades, namespace provisioning, and HSM bootstrap outside the same evidence system used by other Oyatie services.

### B. Approach
Build a kube-rs operator over the declared `openbao-operator` crate family. The operator reconciles OpenBao clusters, drives Helm/Kustomize output, performs HSM-backed unseal, watches Raft membership, and reports readiness to authority and SLO gates.

### C. Deliverables
- `oya-cloud-secrets-openbao-operator-{kernel,domain,usecase,api,adapter,app}`.
- CRD `OpenBaoCluster` and Kubernetes manifests under `iac/helm/openbao` and `iac/kustomize`.
- Catalog files for the operator crate family.
- Runbook evidence in `runbooks/openbao-restart.md` and `runbooks/namespace-controller-restart.md`.
- SLO linkage to `slos/hsm-availability.openslo.yaml` and `slos/secret-write-latency.openslo.yaml`.

### D. Ordered Implementation Steps
1. Define `OpenBaoCluster`, `UnsealState`, `RaftPeer`, and `UpgradePlan` in the kernel crate.
2. Implement pure domain rules for quorum, upgrade ordering, and unseal state transitions.
3. Add API/CRD schema generation for Kubernetes admission.
4. Implement kube-rs adapter reads/writes and Helm template emission.
5. Compose the controller app with idempotent reconcile and status conditions.
6. Run kind e2e with 5-node Raft, Postgres-HA, and HSM emulator.
7. Register operator health and SLO burn signals with observability.

### E. Acceptance
- `cargo nextest run -p 'oya-cloud-secrets-openbao-operator-*'`.
- kind cluster e2e reaches Ready within the stated window.
- Reconcile is idempotent across restarts and partial Kubernetes failures.
- HSM unseal cannot fall back to software keys in regulated packs.

### F. Evidence
Evidence anchors are `PRD.md` FR-07, `PHASE-01-OPENBAO-SECRETREFERENCE-SUBSTRATE.md`, `manifest.json`, `catalog/oya-cloud-secrets-openbao-operator-*.yaml`, `multi-region.md`, `policy/data-residency.md`, and `runbooks/openbao-restart.md`.

### G. Counterpart Comparison
HashiCorp Vault Enterprise has mature Raft and operational tooling; managed AWS/GCP/Azure secret stores hide lifecycle behind vendor control planes. Oyatie's parity gap is operational maturity, and this IP closes it by making OpenBao lifecycle explicit, Kubernetes-native, pack-resident, and auditable.

Grep-recognized counterpart anchor: GitHub Actions Secrets is relevant only for CI operator tests and chart promotion jobs that must source references without embedding raw credentials. The primary operator comparator remains Vault/OpenBao operational maturity.

## DR posture (per ADR-0343)

- Target source: `secrets/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`openbao_seal_unseal`, `postgres_wal_g`, `audit_chain_merkle_seal`].
- Surface evidence: `secrets/runbooks/hsm-key-rotation.md`, `secrets/runbooks/openbao-restart.md`, `secrets/manifest.json`, `secrets/IP-009-openbao-operator.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `secrets/manifest.json#paid_billing_components_emitted` is absent; this section is triggered by IP text and must be reconciled with the manifest billing model.
- Surface evidence: `secrets/manifest.json`, `secrets/IP-009-openbao-operator.md`.
