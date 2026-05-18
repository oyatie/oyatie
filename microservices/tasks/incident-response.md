---
doc_class: IncidentResponse
template_id: TPL-INCIDENT-RESPONSE
microservice: tasks
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
related_adrs: [ADR-0130, ADR-0140, ADR-TASKS-0006]
doc_status: published
---

# Incident Response — tasks µservice

## Purpose

Define incident classification, on-call response, regulator-notification timelines (GDPR Art. 33 / KR PIPA / HIPAA / NIS2 / EU AI Act Art. 73 / EEOC bias-incident / etc.), and post-incident review for tasks.

## Severity classification

| Sev | Definition | Examples |
|---|---|---|
| Sev-1 | Customer-impacting outage or data-loss / data-breach affecting > 1 tenant; or AI-bias incident | task-store unavailable for > 5 min; cross-tenant data leak (FM-13); tenant-DEK compromise; T2 auto-assign fairness incident (FM-07) |
| Sev-2 | Customer-impacting degradation or single-tenant data risk | search-index full-rebuild storm; bulk-edit throttle exhaustion; single-tenant DEK rotation failure |
| Sev-3 | Internal-only impact | worker pod restart loop without queue drain impact |
| Sev-4 | Non-urgent operational issue | minor SLO breach within error budget |

## On-call rotation

- Primary: axis-tasks (rotating 1-week shifts).
- Secondary: ops-sre-reliability (cross-µservice on-call).
- Tertiary: council-architecture (escalation for design-level issues).
- Security on-call (24/7): ops-security.
- Privacy on-call (24/5 + breach-trigger 24/7): council-privacy.
- **AI-incident on-call (24/5; bias / Art. 22 incidents): council-privacy + ops-security.**

Paging via Grafana OnCall.

## Incident lifecycle

```
Detection → Triage → Classify (Sev) → Mitigate (runbook) → Communicate → Resolve → Post-incident review
```

### Detection signals

- SLO burn-rate alerts (Mimir → Alertmanager → OnCall).
- Audit-chain emission failure alerts.
- Cross-tenant query anomaly alerts.
- Dependency-cycle invariant violation alerts.
- **AI fairness drift alerts (per `slos/auto-assign-fairness-correctness.openslo.yaml`)**.
- Tenant complaint via support channel.
- External security disclosure.
- EU AI Act notified-body finding (post-conformity).

### Triage (within 15 min of page)

| Action | Owner |
|---|---|
| Acknowledge page | primary on-call |
| Confirm scope (affected tenants / data classes / AI capabilities) | primary on-call |
| Classify Sev (1-4) | primary on-call |
| Page security + privacy + AI-incident if Sev-1 or AI-bias | primary on-call |
| Open incident channel | primary on-call |
| Update tenant-facing status page if Sev-1 or Sev-2 | gtm-customer-success |

### Mitigation

Per-runbook procedure (see `runbooks/`). Tasks-specific patterns:
- **Recurring storm:** scale recurrence-worker; rate-limit; runbook `recurring-task-materialisation-failure.md`.
- **Custom-field schema mid-migration:** rollback transactional; runbook `custom-field-schema-migration.md`.
- **Dependency cycle corruption:** manual edge identification + removal; runbook `dependency-cycle-corruption.md`.
- **Search-index rebuild failure:** activate degraded mode; runbook `search-index-rebuild.md`.
- **Bulk-edit throttle:** rate-limit + tenant comms; runbook `bulk-edit-throttle.md`.
- **Webhook fanout degraded:** circuit-breaker + tenant comms; runbook `webhook-fanout-degraded.md`.
- **AI assign classifier bias / rollback:** rollback to prior model; runbook `ai-assign-classifier-rollback.md`.

### Communication

| Audience | Channel | Trigger |
|---|---|---|
| Engineering team | OnCall channel (Slack / Telegram) | Sev-1, Sev-2 |
| Customer-success | Internal Slack | Sev-1, Sev-2 |
| Affected tenants | Status page + per-tenant email | Sev-1 (always); Sev-2 (if customer-visible) |
| Public status page | status.oyatie.dev | Sev-1 |
| Council leadership | OnCall channel | Sev-1 |
| External regulator (DPA / PIPC / OCR / EU AI Act notified body) | Per-jurisdiction notification | Sev-1 with personal-data scope or AI-bias incident |
| Workers' council (employment-context) | Tenant-mediated | T2 auto-assign incident affecting employment-context decisions |

### Resolution

- All immediate-impact mitigations applied.
- Tenant-facing status: "resolved".
- Post-incident review scheduled within 5 business days.
- **AI-incident: model rollback + per-decision audit-chain replay for affected period.**

## Regulator notification timelines

### GDPR (Art. 33 + 34)

| Trigger | Timeline | Channel |
|---|---|---|
| Personal-data breach detected | 72h notification to supervisory authority | Per-DPA per-pack notification portal |
| High-risk breach affecting individuals | "without undue delay" notification to data subjects | Tenant DPA upstream-notification clause |
| **Automated-decision incident under Art. 22** | Without undue delay + notification to data subjects | Per-DPA |

### EU AI Act (Art. 73 serious-incident reporting)

| Trigger | Timeline | Channel |
|---|---|---|
| Serious incident in high-risk AI system (T2 auto-assign in employment) | **15 days to relevant market surveillance authority + notified body** | EU AI Act market-surveillance portal |
| Widespread infringement | Without undue delay | All affected MS authorities |

### KR PIPA (Art. 34 + 34-3)

| Trigger | Timeline | Channel |
|---|---|---|
| Personal info leak detected | 24h notification to affected users + within 72h to PIPC | PIPC portal |
| If ≥ 1000 users affected | Report to KISA | KISA portal |

### HIPAA (45 CFR Part 164 Subpart D)

| Trigger | Timeline | Channel |
|---|---|---|
| PHI breach < 500 individuals | HHS OCR within 60d of EOY + individuals within 60d | HHS OCR portal |
| PHI breach ≥ 500 individuals | HHS OCR + media within 60d | HHS OCR + media |
| Retention | 6 years | Compliance |

### NIS2 (2022/2555)

| Trigger | Timeline | Channel |
|---|---|---|
| Significant incident | 24h initial + 72h detailed + 1mo final | National CSIRT |

### APPI / PDPA / Privacy Act / DPDPA / LGPD / UAE PDPL / KSA PDPL

Per pack-overlay sections in `compliance.md`.

### US — EEOC + state-level AI laws

| Trigger | Timeline | Channel |
|---|---|---|
| Auto-assign disparate-impact incident | EEOC complaint window + state-AG notification (NY: 90d per Local Law 144 AEDT) | EEOC + state-AG portals |
| ADA accommodation-task failure incident | EEOC + DOJ ADA portal | per-incident |

## Specific incident playbooks

### Cross-tenant data leak (Sev-1 + GDPR / PIPA breach + EU AI Act if AI-mediated)

1. Acknowledge within 5 min.
2. Identify affected tenants from audit-chain query.
3. Block the leaky path (Cedar policy refusal at runtime).
4. Determine scope: number of affected data subjects.
5. Page council-privacy + ops-security.
6. Within 24h: notify affected tenants.
7. Within 72h: notify GDPR DPA per affected pack-eu tenant (Art. 33).
8. Within 72h: PIPC notification per affected pack-kr tenant.
9. Per-jurisdiction notification per affected pack.
10. **If AI-mediated**: notify EU AI Act notified body within 15 days (Art. 73).
11. Post-incident review within 5 business days.

### Tenant-DEK compromise (Sev-1)

Per calendar's pattern; identical workflow.

### Audit-chain emission gap (Sev-2)

Per calendar's pattern; identical workflow.

### AI auto-assign bias / fairness incident (Sev-1)

1. Acknowledge within 5 min.
2. Identify scope: per-pack + per-tenant + per-protected-class affected.
3. Auto-rollback to prior model version (per `runbooks/ai-assign-classifier-rollback.md`).
4. Page council-privacy + ops-security + axis-tasks.
5. Per-decision audit-chain replay for affected period; identify all auto-assignments potentially affected.
6. Within 24h: notify affected tenants + DPO.
7. Within 15d: notify EU AI Act notified body + market surveillance authority (Art. 73).
8. Within state-AG window (NY: 90d): notify per Local Law 144 AEDT.
9. Within 72h: GDPR Art. 22 automated-decision incident notification per affected pack-eu tenant.
10. Conduct bias-audit on rollback'd model + prior model; document root-cause.
11. Post-incident review within 5 business days; ADR-TASKS-0006 update if needed.

### Dependency-cycle corruption (Sev-2)

1. Acknowledge within 30 min.
2. Identify scope: per-tenant + per-project + affected edges.
3. Run cycle-detection scan; surface corrupt edges to tenant operator.
4. Manual triage: tenant operator removes corrupt edges.
5. Reactivate dependency-graph writes for affected project.
6. Post-incident review: was the importer pre-import cycle scan effective? Should we re-run cycle scan globally? Update LEAN check `oya-check-dependency-graph-cycle-prevention` if a new attack pattern was discovered.

### Bulk-edit throttle exhaustion (Sev-2)

1. Acknowledge within 30 min.
2. Identify scope: per-tenant + per-project.
3. Activate rate-limit at REST layer; surface "operation queued" to tenant.
4. Process bulk in 1000-task batches; emit progress events.
5. Post-incident review: should default second-confirmation threshold be lowered from 10k to 5k?

## Post-incident review (PIR)

Per Google SRE Workbook ch. 15. Conducted within 5 business days of resolution.

- Blameless.
- Outputs: root-cause analysis; corrective actions; LEAN-lane updates; runbook updates; threat-model re-review if applicable.
- **AI-incident PIRs include**: model rollback retrospective; fairness-audit findings; conformity-assessment re-trigger evaluation.
- Action items tracked in `evidence/incidents/<incident_id>.json`.

## Drills

| Drill | Cadence | Scope |
|---|---|---|
| Cross-tenant-leak simulation | Annually | red-team |
| DEK compromise simulation | Annually | ops-security |
| Region failover | Quarterly | per-pack |
| Recurring storm simulation | Bi-annually | axis-tasks |
| Importer payload-as-malware corpus replay | Quarterly | axis-tasks + ops-security |
| Audit-chain emission failure | Quarterly | observability + audit-chain |
| **AI auto-assign fairness drift simulation** | Quarterly | axis-tasks + council-privacy |
| **AI auto-assign rollback rehearsal** | Bi-annually | axis-tasks + foundry-runtime |
| Search-index full-rebuild (10M tasks; AC-09 target) | Quarterly | axis-tasks |
| Dependency-cycle corruption replay | Bi-annually | axis-tasks |

## References

- ADR-0130: SLO-gated promotion.
- ADR-0140: Cedar policy.
- ADR-TASKS-0006 (AI auto-assign EU AI Act bounds).
- `failure-modes.md`, `runbooks/*`, `compliance.md`, `multi-region.md`.
- GDPR Arts. 33 + 34; EDPB Guidelines 9/2022.
- KR PIPA Arts. 34 + 34-3.
- HIPAA 45 CFR Part 164 Subpart D.
- NIS2 (2022/2555).
- EU AI Act Regulation (EU) 2024/1689 Art. 73 (serious-incident reporting).
- APPI; PDPA; APP; DPDPA; LGPD; UAE PDPL; KSA PDPL.
- EEOC complaint procedures; NY Local Law 144 AEDT; CO AI Act HB23-1041; CA AB-331 (proposed).
- Google SRE Workbook ch. 11 + ch. 15.
- `microservices/calendar/incident-response.md` — sibling reference template.
