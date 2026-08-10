---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0297]
companion_docs: [microservices/social/policy/abuse-defence.cedar]
inbound_citations: [microservices/social/ARCHITECTURE.md]
---

# Runbook: Coordinated Inauthentic Behavior (CIB) response

## A. Trigger conditions

- Multi-cluster + cross-instance coordinated pattern (e.g., synchronized hashtag campaign across federated instances).
- External tip-off from research consortium (Stanford Internet Observatory, GNI partners, OpenAI red-team output).
- Internal classifier flags a coordinated narrative seeding pattern with confidence > 90.

## B. Pre-checks

1. Operator Cedar permit + senior trust-and-safety role.
2. Confirm multi-reviewer verdict + governance escalation.
3. Coordinate with legal counsel + comms.

## C. Procedure

1. Map clusters: each contributing sock-puppet cluster handled via `runbooks/sock-puppet-cluster-takedown.md`.
2. Preserve evidence end-to-end; chain-of-custody with TrueTime timestamps.
3. Identify amplifying authentic accounts (not part of CIB but spreading payload); de-rank but do not suspend.
4. Federation notification to peer instances participating in coordinated takedown.
5. External research disclosure: share evidence (with appropriate redaction) with research consortium.
6. Public transparency report: include in next DSA transparency report.
7. Postmortem within 7 days.

## D. Verification

CIB pattern no longer detectable on platform metrics.

## E. Rollback

N/A (per-cluster rollbacks possible).

## F. Post-incident

Update detection models + ADR amendments.

## G. References

- `runbooks/sock-puppet-cluster-takedown.md`
- `runbooks/dsa-transparency-report-generation.md`
- ADR-0297
