---
doc_class: Runbook
title: Payout settlement mismatch
microservice: developer-sdk
severity: "Sev-1 (financial)"
status: Accepted
owner_team: axis-ecosystem + ops-sre-reliability
date: 2026-05-18
related_artifacts:
  - microservices/developer-sdk/PRD.md §payout
doc_status: published
---

# Runbook: Payout settlement mismatch

## Trigger

- Ledger total ≠ bank settlement total
- Bank ACH/SEPA/KFTC rejection rate > 0.5%

## Severity

Sev-1 (financial)

## Impact

Tenant or developer experience is degraded; per-µservice SLO budget consumed; risk of cascading impact to dependent µservices (audit-chain, finops-portal) unless contained within MTTR target.

## Pre-checks

1. Identify breached SLO: open the Grafana dashboard for the affected BC; mark the deploy boundary or the upstream-dependency outage.
2. Check the audit-chain integrity for any forensic gap.
3. Confirm tenant scope: single tenant vs. cross-tenant vs. pack-wide.
4. Verify the upstream µservice dependency status (tenancy / identity / governance / workflow-engine event bus / audit-chain / cloud-secrets).
5. Capture a snapshot of the current Prometheus metrics + last 100 audit-chain seal events.

### Recovery Path A — Ledger event ordering issue

1. Halt next settlement batch.
2. Replay ledger from last verified offset.
3. Verify byte-equal; resume.

### Recovery Path B — Bank protocol field mismatch

1. Compare emitted file vs. bank-side rejection reason.
2. Patch protocol adapter; re-emit.
3. File ADR if protocol spec changed.

## Escalation

- Sev-1: page on-call + council-security + axis-ecosystem lead within 5 min.
- Sev-2: page on-call within 15 min.
- Sev-3: tickled to slack channel #axis-ecosystem-ops within 1h.

## Post-incident

1. File a post-mortem doc under `microservices/developer-sdk/evidence/incident-reports/<YYYY-MM-DD>-<slug>.md`.
2. Update the relevant SLO target if budget overshoot ≥ 50%.
3. File a follow-up IP if root cause is a fixable invariant gap.
4. Update this runbook with any newly-discovered recovery path.
