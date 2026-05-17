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
