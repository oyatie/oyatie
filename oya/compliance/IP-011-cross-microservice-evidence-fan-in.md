---
microservice: compliance
ip: IP-011
title: Cross-µservice evidence fan-in (outbox subscriber consuming fleet events)
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [council-architecture]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0153, ADR-0209]
---

# IP-011 — Cross-µservice evidence fan-in

## Purpose

Compliance can't collect from each µservice via point-to-point integration (32+ µservices × N collectors = quadratic). Instead, every µservice's outbox (per ADR-0153) emits compliance-relevant events; the compliance µservice subscribes to the fleet-wide outbox topic.

## Acceptance criteria

1. Compliance subscriber consumes `oya.compliance.evidence.*` event topic.
2. Per-event handler maps the event to an `EvidenceArtifactKind` + collects.
3. Backpressure: bounded queue; circuit-break at depth > 10000; per-µservice rate-limit.
4. At-least-once delivery; idempotent emit (artifact_id deterministic from event payload hash).
5. Cross-cell fan-in (per ADR-0153 outbox replicator).
6. ≥ 6 integration tests: event-flow-end-to-end + idempotency + cross-cell-routing + backpressure + circuit-break + per-µservice-rate-limit.

## Event topic taxonomy

| Topic | Emitter | Maps to artifact |
|---|---|---|
| `oya.compliance.evidence.ci_build_complete` | ci-fix-loop / CI pipeline | ci-artifact-hash |
| `oya.compliance.evidence.deploy_promoted` | ADR-0181 promotion controller | deploy-receipt |
| `oya.compliance.evidence.access_review_complete` | tenancy + identity | access-review-snapshot |
| `oya.compliance.evidence.backup_drill_complete` | ops-sre dr drill | backup-restore-drill-receipt |
| `oya.compliance.evidence.vuln_scan_complete` | Trivy (per ADR-0181) | vuln-scan-report |
| `oya.compliance.evidence.dsar_request_closed` | this µservice | dsar-completion-record |
| `oya.compliance.evidence.phi_access` | healthcare-portal | minimum-necessary-access-log |

## Idempotency

`artifact_id = sha256(microservice || event_topic || event_id)`. Re-delivery of the same event yields the same artifact_id → ledger insert is no-op.

## Risk + mitigation

- **Risk:** event storm overwhelms compliance. **Mitigation:** bounded queue + circuit-break; per-µservice rate limit.
- **Risk:** missed events (subscriber down). **Mitigation:** outbox replicator (ADR-0153) replays from cursor.

## Acceptance evidence

`evidence/ip-011-cross-microservice-evidence-fan-in-acceptance.json`.

## Cross-references

- ADR-0145 — substrate.
- ADR-0153 — outbox pattern.
- ADR-0209 — substrate authority.
- IP-001 — collector bootstrap (consumer).
