---
doc_class: Playbook
shape: anchor
length_cap: 120
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Cross-axis contract changes; require all consumer axes to canary in lockstep.
planned_enforcement_ref:
  - oya-governance-canary-required
  - oya-governance-rollback-evidence
related_adrs: [ADR-0011, ADR-0012, ADR-0037, ADR-0053, ADR-0052, ADR-0054]
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
doc_status: published
---

# Playbook: Cross-Axis Contract Change

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Surface

Cross-axis contract registry ([ADR-0011](../../../docs/adr-archive/ADR-0011-cross-microservice-contract-registry.md)) + axis admission protocol ([ADR-0012](../../../docs/decisions/ADR-0709-general-live-apex.md)).

## 2. Default rail

**Canary, lockstep across all consumer axes.** Producer canary stage N gates on every consumer reaching stage N. A consumer regression in any axis halts the producer.

## 3. Pre-rollout requirements

Per [ADR-0037](../../../docs/decisions/ADR-0709-general-live-apex.md):

1. Contract version bump declared (semver per tier).
2. Backward-compatibility verified for all consumers OR a deprecation window opened (≥ 90 d for stable, ≥ 30 d for GA).
3. Consumer-axis owners signed off via PR review.
4. Dark-launch on producer side (write-side, per [`dark-launch-spec.md`](dark-launch-spec.md) §2).

## 4. Lockstep canary sequence

Producer axis P, consumer axes C1..Cn:

1. **Stage 0 (dark-launch).** P emits new contract shape in shadow; consumers read shadow + diff.
2. **Stage 1 (1%).** P serves new shape to 1% canary cohort. C1..Cn each canary their consuming-code on the same cohort. Burn-rate evaluated across producer + every consumer.
3. **Stage 2 (5%).** Same gating.
4. **Stage 3 (25%).** Same.
5. **Stage 4 (50%).** Same.
6. **Stage 5 (100%).** Same.

A consumer breach at any stage halts the producer. The producer cannot promote past the slowest consumer.

## 5. Deprecation choreography

Removing a contract field / version:

1. Mark field deprecated in registry (≥ 90 d before removal for stable).
2. Emit deprecation telemetry per [ADR-0037](../../../docs/decisions/ADR-0709-general-live-apex.md); track consumer usage decay.
3. Block removal if any consumer still calls the deprecated field with > 0 calls in last 7 d.
4. After zero-call window, remove via separate release.

`oya-governance-api-semver` (existing) gates this.

## 6. Consumer-axis canary independence

Each consumer canary is independent of others — a regression in C1 does not halt C2's promotion to the same stage. But the **producer** waits for all consumers.

Exception: if C1's regression is caused by the contract shape itself (not C1's consuming code), all consumers halt.

## 7. Cross-axis lockstep evidence

Per stage, emit a single D14 artefact aggregating:
- Producer canary stage + burn-rate.
- Per-consumer canary stage + burn-rate.
- Cohort intersection results.
- Lockstep verdict (promote / hold / abort).

Stored in `oya-intelligence-evidence-kernel`; verified by `oya-governance-rollback-evidence`.

## 8. Rollback

Cross-axis rollback is per-cell on the producer side, mirrored on every consumer. Mesh routes both producer + consumers back to prior version atomically. Per [`blue-green-spec.md`](blue-green-spec.md) §5 multi-mode.

## 9. SLO targets

Cross-axis contract SLO = the **tightest** SLO across producer + consumers. Producer must hit ≥ max(consumer SLO).

## 10. Hyperscaler equivalent

Google internal Stubby contract versioning; Amazon Smithy / AWS API Gateway versioning discipline; Microsoft Azure API Management revisions. We adopt the lockstep-canary across producer + consumers as the explicit ceremony.

## 11. Lift target

`oyatie/docs/playbooks/playbook-cross-axis-contract.md` on approval.
