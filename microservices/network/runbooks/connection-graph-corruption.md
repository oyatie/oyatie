---
doc_class: Runbook
title: Connection-graph corruption + cross-tenant + cross-context + endorsement-chain integrity + ontology + minor-leak recovery
microservice: network
severity: "Sev-1 (integrity / privacy class)"
status: Accepted
owner_team: ops-security + axis-network + council-privacy
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/network/failure-modes.md (FM-05, FM-07, FM-10, FM-12, FM-13, FM-14, FM-16, FM-21)
  - microservices/network/policy/professional-context-isolation.md
  - microservices/network/backfill-replay.md
doc_status: published
---

# Runbook: Connection-graph corruption + paired integrity / privacy failure modes

## Trigger

- `network_connection_graph_drift_total` > 0 (FM-05).
- `network_cross_tenant_leak_detector_total` > 0 (FM-07).
- `network_professional_context_violation_total` > 0 (FM-10).
- `network_four_eyes_pairing_violation_total` > 0 (FM-12).
- `network_pack_residency_violation_total` > 0 (FM-13).
- `network_endorsement_chain_integrity_failure_total` > 0 or `network_endorsement_signature_verify_failure_rate` > 0.01% (FM-14).
- Unauthorised access to minor-protect records (FM-16).
- `network_mention_resolution_failure_rate` > 5% for ≥ 2 min (FM-21).

## Severity

All scenarios are Sev-1 except FM-21 (ontology degraded = Sev-3 graceful-degrade).

## Immediate Mitigation by Scenario

### FM-05 — Connection-graph corruption

| Step | Action | Time |
|---|---|---|
| 1 | Quarantine affected accounts (block writes; reads under JIT elevation only) | ≤ 5 min |
| 2 | Snapshot Postgres adjacency-list table for forensics | ≤ 10 min |
| 3 | Re-derive connection-graph from audit-chain authoritative replay (per `backfill-replay.md` §"connection-graph replay") | up to 30 min per affected account |
| 4 | Rebuild degree-of-separation cache per `backfill-replay.md` §"degree-cache rebuild" | ≤ 30 min |
| 5 | Cross-reference resulting edges against Postgres adjacency-list; flag and present discrepancies to ops-security | up to 30 min |
| 6 | Restore canonical edges from authoritative replay; tombstone discrepancies in audit-chain | ≤ 30 min |
| 7 | Re-emit `ConnectionEdgeReconciled` events to downstream consumers | ≤ 5 min |

### FM-07 — Cross-tenant leak (RLS misconfig)

| Step | Action | Time |
|---|---|---|
| 1 | Auto-rollback to last green Helm deployment via observability gate | ≤ 5 min |
| 2 | Isolate affected cluster; pause writes on all tenants | ≤ 5 min |
| 3 | Engage ops-security + council-privacy; declare breach-suspect | ≤ 5 min |
| 4 | Audit-chain replay scoped to affected window; enumerate every read crossing tenant boundary | up to 1h |
| 5 | Notify affected tenants per GDPR Art. 33 / KR PIPA Art. 34 / equivalent in pack | ≤ 72h GDPR clock |
| 6 | Re-enable writes only after RLS coverage lane green | when verified |

### FM-10 — Cross-context routing violation (Personal entity in `network`)

| Step | Action | Time |
|---|---|---|
| 1 | Block service write path immediately; reject any payload claiming Personal-tier context | ≤ 1 min |
| 2 | Quarantine affected entities; identify provenance (which API call introduced the violation) | ≤ 15 min |
| 3 | Engage council-privacy + ops-security; declare breach | ≤ 15 min |
| 4 | Audit-chain seal of attempt; trace upstream caller; if `social` µservice involvement: cross-engage axis-social | up to 1h |
| 5 | Roll back triggering code-path; verify LEAN lane `oya-check-professional-context-isolation` green | when patched |
| 6 | Notify affected tenants per pack regulatory clock | per IR |

### FM-12 — Four-eyes disclosure mis-pairing (insider attempt)

| Step | Action | Time |
|---|---|---|
| 1 | Cedar evaluator already denied at request time; verify audit-chain seal of denial | ≤ 5 min |
| 2 | Engage ops-security; treat as insider-malicious threat actor signal | ≤ 5 min |
| 3 | Inspect attempted principal pair; flag for HR / compliance review | ≤ 30 min |
| 4 | Audit-chain replay for the principal's prior disclosure history; flag anomalies | up to 1h |

### FM-13 — Cross-pack residency misroute

| Step | Action | Time |
|---|---|---|
| 1 | Halt writes for affected tenant in mis-routed cluster | ≤ 5 min |
| 2 | Identify root cause: bad Helm overlay, Cedar pack-router bug, or operator error | ≤ 30 min |
| 3 | Engage council-privacy; declare possible GDPR / PIPA / HIPAA breach | ≤ 30 min |
| 4 | Migrate data back to correct pack cluster (Postgres logical-replica seed + replay) | hours |
| 5 | Verify residency lane green; tenant reads enabled in correct pack only | when verified |

### FM-14 — Endorsement-chain integrity compromise

| Step | Action | Time |
|---|---|---|
| 1 | Quarantine affected endorsement-chain partition; mark affected endorsements as `integrity_under_verification` in user UI | ≤ 5 min |
| 2 | Engage ops-security + axis-audit-chain; treat as potential signing-key compromise | ≤ 5 min |
| 3 | KMS audit-log review for unauthorised key access on per-endorser Ed25519 keys | ≤ 30 min |
| 4 | Replay endorsement-chain per `backfill-replay.md` §"endorsement-chain replay"; verify Merkle root | up to 1h |
| 5 | If key compromise confirmed: revoke compromised keys; re-issue per-endorser keys; require re-endorsement | hours-days |
| 6 | Notify affected tenants + EU AI Act notified body if recruiter signal involved | per IR clocks |

### FM-16 — Minor-account leak / pivot

| Step | Action | Time |
|---|---|---|
| 1 | Revoke unauthorised access; rotate the relevant Cedar entitlement (`minor_protect_reader`) | ≤ 5 min |
| 2 | Audit-chain seal of the violation; engage council-privacy + ops-security | ≤ 5 min |
| 3 | Identify root cause (Cedar policy regression, code-path bypass, manual SQL access) | ≤ 30 min |
| 4 | Per-pack regulator notification per COPPA / GDPR Art. 8 / KR 청소년 보호법 / equivalent | per IR clocks |
| 5 | Post-mortem within 5 business days; council-privacy sign-off | ≤ 5d |

### FM-21 — Ontology degraded (mention-resolution failure)

| Step | Action | Time |
|---|---|---|
| 1 | Verify `ontology` µservice health: `kubectl -n ontology get pods` | ≤ 2 min |
| 2 | Cache last known ontology graph for Person / Company / Skill / Hashtag lookups | ≤ 5 min |
| 3 | Degrade mention-render to raw-text mode (no link, no notification fanout) | ≤ 5 min |
| 4 | Reconcile when ontology returns: re-resolve mentions in last hour of posts; emit scheduled-for-distinct-tracked-work notifications | ≤ 30 min |

## Recovery Verification

| Scenario | Verification |
|---|---|
| FM-05 | Connection-graph adjacency-list matches audit-chain replay; degree-cache matches adjacency BFS; no `drift_total` increment for 24h |
| FM-07 | RLS coverage lane green; cross-tenant detector counter at 0 for 24h; affected-tenant audit-replay completed |
| FM-10 | LEAN lane `oya-check-professional-context-isolation` green; no `professional_context_violation_total` increment for 24h |
| FM-12 | Cedar audit denies all observed four-eyes-mispair attempts; ops-security HR review closed |
| FM-13 | `pack_residency_violation_total` at 0; per-pack row-count audit clean |
| FM-14 | Endorsement-chain Merkle root matches authoritative replay; `endorsement_signature_verify_failure_rate` at 0 for 24h |
| FM-16 | `minor_protect` Cedar entitlement audit-passes; council-privacy sign-off |
| FM-21 | `mention_resolution_failure_rate` < 0.1% for 30 min |

## References

- `microservices/network/failure-modes.md` FM-05, FM-07, FM-10, FM-12, FM-13, FM-14, FM-16, FM-21.
- `microservices/network/policy/professional-context-isolation.md`.
- `microservices/network/backfill-replay.md`.
- `microservices/network/incident-response.md` §"Cross-Tenant or Cross-Context Leak = Sev-1".
- `microservices/network/incident-response.md` §"Endorsement-Chain Integrity Compromise = Sev-1".
- ADR-NET-0005 endorsement-chain integrity.
- GDPR Arts. 33, 34; KR PIPA Art. 34; HIPAA §164.412.
