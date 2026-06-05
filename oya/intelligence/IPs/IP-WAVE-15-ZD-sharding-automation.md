---
doc_class: ImplementationPlan
microservice: intelligence
source_vendor: Wave 15-ZD doctrine propagation
related_adrs: [ADR-0348, ADR-0513]
date: 2026-05-21
doc_status: superseded_active_guidance
superseded_by: ADR-0513
authority_posture: Buck2/Prow/Kubernetes-native oya-ci-required
---

# Intelligence sharding automation implementation plan — current authority overlay

## Intent

Intelligence keeps the valid sharding and cell-automation intent from the Wave 15-ZD plan while replacing superseded local verifier and bridge-substrate guidance with the current Buck2/Prow/Kubernetes-native path.

## Current stances

- Autosharding is control-plane driven; operators do not pick provider, runtime, evidence, or tenant placement manually.
- Auto-rebalance honors residency, compliance packs, PBAC/ABAC policy, provider data-use constraints, GPU/cost saturation, and blast-wall constraints before movement.
- Dynamic sharding uses declared hot-split and cold-merge thresholds, not hidden defaults.
- Every tenant, provider, runtime, evidence, cell, shard, replay, and rollback transition emits audit-chain evidence with pre-state and post-state.
- Buck2 owns build, test, check, benchmark, and coverage evidence; Prow consumes that evidence for oya-ci-required.
- GitHub is a temporary PR/publication adapter and shadow check surface until native SCM/CI/CD cutover.
- Deployment desired state is CUE/KRM first; compatibility packaging is generated only where an adapter requires it.

## Implementation requirements

| Requirement | Intelligence rule |
|---|---|
| Autosharding | Placement is computed by the Intelligence control plane from tenant class, provider policy, model/runtime heat, residency, GPU saturation, and policy permits. |
| Auto-rebalance | Candidate movement is denied unless residency, compliance, data-use, budget, and blast-wall checks pass before execution. |
| Hot-split | A split plan records source shard, destination shard, traffic-drain window, prompt/session safety, evidence continuity, and rollback inverse. |
| Cold-merge | A merge plan records source shards, target shard, compaction safety, provider routing safety, evidence continuity, and rollback inverse. |
| Policy | PBAC/ABAC decisions are evaluated before orchestration and attached to the operation ledger. |
| Evidence | Buck2 target output, Prow status, operation-ledger event, and audit-chain seal are linked from the PR or incident record. |
| Runtime safety | Default-deny network policy, workload identity, restricted pod privileges, immutable container file systems, dropped Linux capabilities, token automount disablement, sandboxed runtimes, and mTLS are required where applicable. |

## Verification backlog

- Buck2 unit and integration checks for placement, split, merge, rollback, replay safety, provider failover, and policy-denial paths.
- Prow oya-ci-required lane consuming the Buck2 evidence bundle.
- CUE/KRM desired-state validation for namespace, network, workload identity, runtime isolation, sandboxed model execution, and service-mesh mTLS posture.
- Multispectrum evidence for any production-like sharding, replay, provider failover, or model rollback drill.

## Acceptance

- No active Intelligence sharding plan cites a retired local CLI as authority.
- No active Intelligence sharding plan treats a temporary GitHub bridge or retired external CI/CD bridge as first-class authority.
- Product operation names and evidence links are sufficient for another lane to run without editing shared canonical docs.
