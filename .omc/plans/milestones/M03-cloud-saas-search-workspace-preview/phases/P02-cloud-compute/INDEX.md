---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M03-P02
title: Cloud Compute (VM / K8s / Functions / Capacity / DC-Ops)
status: in-progress (IP-001 AWS/OCI deterministic VM adapter request-contracts green 2026-05-21; IP-002 K8s/Functions stable API entrypoints green 2026-05-21; IP-003 provider-neutral capacity kernel green 2026-05-21; live-provider smoke and self-hosted/colo adapter pending)
purpose: Bring managed VM/K8s/Functions + capacity + DC-Ops surfaces to W-Cloud-Preview readiness; provider-agnostic adapter pattern.
execution_variant: merge-into-existing-crates
decided_at: "2026-05-17"
decided_by: user-directive-option-2
execution_variant_note: "IP-003 symbols (ReservedCapacity, CommittedUseContract, SpotPool) merged into oya-cloud-capacity-kernel::committed_use; no new crate scaffolds, no new workspace deps. Mirrors M03-P01 pattern."
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
| IP-001 | Cloud Compute VM API (provider-agnostic; AWS/OCI/GCP/Azure/self-hosted/colo adapters) | aws-oci-request-contract-green; live-provider-smoke+selfhosted-colo-adapter pending | [`IP-001-vm-api-adapters.md`](IP-001-vm-api-adapters.md) |
| IP-002 | Cloud Compute K8s + Functions API | stable-api-entrypoints-green; app/transport/provider-adapter runtime not claimed | [`IP-002-k8s-functions-api.md`](IP-002-k8s-functions-api.md) |
| IP-003 | Capacity management (reserved / committed-use / spot) | provider-neutral-kernel-green; provider procurement/runtime adapters not claimed | [`IP-003-capacity-management.md`](IP-003-capacity-management.md) |

## Estimated parallelism
3 agents; each IP disjoint crate suffix.

## Symbols-touched
`crates/oya-cloud-compute-{vm,k8s,functions,capacity}-{api,app,adapter-{aws,oci,gcp,azure,selfhosted}}-*`, `crates/oya-cloud-dcops-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M03-P02 complete: cloud compute VM+K8s+Functions API stable; capacity mgmt operational" -i critical -k "M03,P02,cloud-compute,complete"
```
