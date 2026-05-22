---
doc_class: Incident-Response
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0180
  - ADR-0248
  - ADR-0263
companion_docs:
  - microservices/ops-dashboard-control-center/runbooks/admin-action-rollback.md
  - microservices/ops-dashboard-control-center/runbooks/dashboard-perf-degradation.md
  - microservices/ops-dashboard-control-center/runbooks/step-up-auth-bypass-attempt.md
  - microservices/ops-dashboard-control-center/failure-modes.md
planned_enforcement_ref: oya-governance-microservice-doc-suite
---

# Incident Response — ops-dashboard-control-center

## §1 Severity classification

| Severity | Criteria | Response time | On-call action |
|---|---|---|---|
| SEV0 | Control plane completely unavailable; operators cannot perform ANY admin action | 5 min | Page primary + secondary on-call; bridge within 5 min |
| SEV1 | Core mutation paths unavailable (deployment approve / rollback / incident declare); reads up | 15 min | Page primary on-call |
| SEV2 | Degraded performance (P99 > SLO budget); specific BC unavailable; step-up auth latency elevated | 30 min | Async notify + on-call monitors |
| SEV3 | Single panel degraded; non-critical BC unavailable; cosmetic UX issue | Next business day | Ticket filed |

## §2 Escalation chain

1. **Primary on-call** (ops-sre-reliability rotation): first responder; 5-min ack SLO.
2. **Secondary on-call**: escalate if no ack within 5 min.
3. **Engineering lead** (ops-sre-reliability): escalate SEV0/SEV1 if not resolved within 30 min.
4. **Platform VP**: escalate SEV0 if not resolved within 60 min.
5. **council-security**: escalate immediately on any suspected Cedar policy breach or audit chain tampering.

## §3 Runbook index

| Scenario | Runbook |
|---|---|
| Admin action needs rollback | `runbooks/admin-action-rollback.md` |
| Step-up auth bypass attempt detected | `runbooks/step-up-auth-bypass-attempt.md` |
| Tenant scope violation detected | `runbooks/tenant-scope-violation-detected.md` |
| On-call handoff failure | `runbooks/oncall-handoff-failure.md` |
| Dashboard performance degraded | `runbooks/dashboard-perf-degradation.md` |
| Admin MFA cascade (bulk step-up expiry) | `runbooks/admin-mfa-cascade.md` |
| Pack author quarantine triggered | `runbooks/pack-author-quarantine.md` |
| Forensic investigation handoff | `runbooks/forensic-investigation-handoff.md` |

## §4 Key metrics for triage

```promql
# Command availability burn rate
sum(rate(oya_ops_control_center_requests_total{status!~"5.."}[5m]))
  / sum(rate(oya_ops_control_center_requests_total[5m]))

# Cedar eval errors
sum(rate(oya_ops_control_center_cedar_eval_errors_total[5m]))

# Step-up auth failures
sum(rate(oya_ops_control_center_step_up_auth_failures_total[5m]))

# Tenant scope violations
sum(rate(oya_ops_control_center_tenant_scope_violations_total[5m]))

# Audit emission failures (CRITICAL — any non-zero is SEV1)
sum(rate(oya_ops_control_center_audit_emission_failures_total[5m]))
```

## §5 Communication templates

**SEV0 bridge opener:** "This is a SEV0 on ops-dashboard-control-center. Control plane is unavailable. Incident commander: [NAME]. Starting RCA. Status page update in 5 min."

**Stakeholder update (every 15 min on SEV0/SEV1):** "ops-dashboard SEV[N] update [T+Xmin]: [current state]. RCA in progress: [hypothesis]. ETA to resolution: [N min]. Next update in 15 min."

## §6 Post-incident

- Blameless post-mortem required for SEV0/SEV1 within 48h.
- Timeline from audit chain replay (always available — `runbooks/forensic-investigation-handoff.md`).
- Action items tracked in ADR-promotion-triage queue.
- SLO error budget impact documented in `slos/command-availability.openslo.yaml`.
