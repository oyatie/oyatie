---
doc_class: Runbook
title: Wasmtime sandbox escape suspected
microservice: plugin-app-store
severity: "Sev-1 (security)"
status: Accepted
owner_team: axis-ecosystem + ops-sre-reliability
date: 2026-05-18
related_artifacts:
  - ADR-0147
  - ADR-0200
doc_status: published
---

# Runbook: Wasmtime sandbox escape suspected

## Trigger

- Wasmtime isolation validator catches escape attempt during vetting
- Runtime syscall trace shows forbidden syscall from plugin process

## Severity

Sev-1 (security)

## Impact

Tenant or developer experience is degraded; per-µservice SLO budget consumed; risk of cascading impact to dependent µservices (audit-chain, finops-portal) unless contained within MTTR target.

## Pre-checks

1. Identify breached SLO: open the Grafana dashboard for the affected BC; mark the deploy boundary or the upstream-dependency outage.
2. Check the audit-chain integrity for any forensic gap.
3. Confirm tenant scope: single tenant vs. cross-tenant vs. pack-wide.
4. Verify the upstream µservice dependency status (tenancy / identity / governance / workflow-engine event bus / audit-chain / cloud-secrets).
5. Capture a snapshot of the current Prometheus metrics + last 100 audit-chain seal events.

### Recovery Path A — New Wasmtime CVE not yet patched

1. Quarantine affected plugin: kill-switch installation.
2. Update Wasmtime to patched version per ADR-0200.
3. Re-vet all plugins of the affected author.

### Recovery Path B — Plugin abuses a permitted but dangerous syscall

1. File security incident.
2. Revoke plugin.
3. Tighten seccomp profile.

## Escalation

- Sev-1: page on-call + council-security + axis-ecosystem lead within 5 min.
- Sev-2: page on-call within 15 min.
- Sev-3: tickled to slack channel #axis-ecosystem-ops within 1h.

## Post-incident

1. File a post-mortem doc under `microservices/plugin-app-store/evidence/incident-reports/<YYYY-MM-DD>-<slug>.md`.
2. Update the relevant SLO target if budget overshoot ≥ 50%.
3. File a follow-up IP if root cause is a fixable invariant gap.
4. Update this runbook with any newly-discovered recovery path.
