---
purpose: Bring managed VM/K8s/Functions + capacity + DC-Ops surfaces to W-Cloud-Preview readiness; provider-agnostic adapter pattern.
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M03-P02
title: Cloud Compute (VM / K8s / Functions / Capacity / DC-Ops)
status: partial (in-flight)
purpose: Bring managed VM/K8s/Functions + capacity + DC-Ops surfaces to W-Cloud-Preview readiness; provider-agnostic adapter pattern.
---

# M03-P02 — Cloud Compute

## Purpose
Per [`../../../../../docs/ROADMAP.md`](../../../../../docs/ROADMAP.md) §2.3. Compute is the highest-billable axis surface; provider-agnostic interfaces protect against lock-in.

## Acceptance
- `cloud.compute.vm.create`, `cloud.compute.k8s.cluster.create`, `cloud.compute.functions.invoke` SPEC §7 rows green at `stable`.
- Capacity management: reserved, committed-use, spot/preemptible (per [`../../../../../.omx/notepad.md`](../../../../../.omx/notepad.md) 2026-05-11 capacity checkpoint).
- DC-Ops kernel (per notepad cloud-dcops-kernel checkpoint).

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Cloud Compute VM API (provider-agnostic; AWS/OCI/GCP/Azure adapters) | partial | [`IP-001-vm-api-adapters.md`](IP-001-vm-api-adapters.md) |
| IP-002 | Cloud Compute K8s + Functions API | partial | [`IP-002-k8s-functions-api.md`](IP-002-k8s-functions-api.md) |
| IP-003 | Capacity management (reserved / committed-use / spot) | partial | [`IP-003-capacity-management.md`](IP-003-capacity-management.md) |

## Estimated parallelism
3 agents; each IP disjoint crate suffix.

## Symbols-touched
`crates/oya-cloud-compute-{vm,k8s,functions,capacity}-{api,app,adapter-{aws,oci,gcp,azure}}-*`, `crates/oya-cloud-dcops-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M03-P02 complete: cloud compute VM+K8s+Functions API stable; capacity mgmt operational" -i critical -k "M03,P02,cloud-compute,complete"
```
