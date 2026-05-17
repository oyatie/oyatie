---
doc_class: IncidentResponse
template_id: TPL-INCIDENT-RESPONSE
microservice: community
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-community + ops-sre + ops-security
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0126, ADR-0131]
related_artifacts:
  - microservices/community/threat-model.md
  - microservices/community/failure-modes.md
  - microservices/community/runbooks/
doc_status: published
---

# Incident Response: community µservice

## Severity Definitions

| Sev | Definition | First response | Page |
|---|---|---|---|
| P0 | Cross-tenant leak; mass-deletion; auth bypass; safe-harbor at risk | 5 min | Primary + secondary + security |
| P1 | Single-tenant outage; search down; moderation stalled; >1 % error rate | 15 min | Primary on-call |
| P2 | Degraded performance; non-critical feature failure; partial region | 30 min | Primary on-call |
| P3 | Cosmetic; backlog issue | Next business day | None |

## On-Call Rotation

- Primary on-call: axis-community (24/7).
- Secondary on-call: axis-community (24/7).
- Security on-call: ops-security (24/7, paged for P0 only).
- Region escalation: regional ops-sre.

## Response Flow

```text
Alert → PagerDuty / Grafana OnCall → Acknowledge (5 min P0)
  → Page secondary if no ack within window
  → Open incident channel #inc-community-<utc-date>-<seq>
  → Declare severity in channel
  → Engage runbook from runbooks/
  → Communicate to tenant_admins if P0/P1 within 30 min
  → Mitigate
  → Stabilise + verify SLO posture
  → Post-mortem within 5 business days (P0/P1)
  → ADR if structural change required
```

## Communication

| Channel | Audience | P0 | P1 | P2 |
|---|---|---|---|---|
| #inc-community-<date> | Internal | yes | yes | yes |
| Status page `status.oyatie.io` | Public | yes (30 min) | yes (1 h) | optional |
| Tenant_admin email | Affected tenants | yes (30 min) | yes (1 h) | no |
| Regulator notification | Per-pack | 24-72 h depending on regime | as needed | no |

## Per-Scenario Playbooks

See `runbooks/`:
- `spam-flood-throttle.md` — mass-spam abuse response
- `vote-anomaly.md` — vote manipulation response
- `moderation-queue-clear.md` — flag-storm response
- `kb-attachment-restore.md` — S3 outage / corruption response
- `search-rebuild.md` — search index rebuild
- `post-mass-deletion.md` — compromised admin / mass-deletion recovery
- `cross-tenant-bleed.md` — P0 cross-tenant leak (when authored separately)
- `mention-reconcile.md` — mention-resolution sync
- `dsr-cascade-resume.md` — DSR cascade resume on partial failure

## Cross-Tenant Leak (P0)

Specific runbook for the most severe failure mode:

1. **0–5 min**: page security + community lead; ack in incident channel.
2. **5–10 min**: disable affected gateway path; force ingress deny on suspected route.
3. **10–30 min**: extract Cedar deny / RLS log evidence; identify scope (which tenants, which data).
4. **30–60 min**: revert deploy if regression; rebuild affected indexes; verify policy compilation.
5. **1–4 h**: tenant_admin notification to affected tenants.
6. **4–24 h**: regulator pre-notification when in scope (KR PIPA 24 h, GDPR 72 h, HIPAA 60 d).
7. **24–72 h**: full forensics; root-cause; published transparency note.
8. **5 d**: post-mortem; ADR; reverification of all Cedar fragments.

## Audit-Chain Integration

Every incident emits:
- `IncidentDeclared` event (sev, scope, runbook).
- `IncidentMitigated` event (mitigation, residual risk).
- `IncidentResolved` event (RTO actual, RPO actual, action items).

Sealed within 1 s by audit-chain µservice.

## Drill Cadence

- Monthly: tabletop on one runbook.
- Quarterly: live drill on one P0 scenario (in staging).
- Annually: full chaos drill across multiple scenarios.

## Post-Mortem Template

- What happened (timeline)
- Impact (tenants, members, data)
- Root cause (5 Whys)
- Mitigation (what we did)
- Action items (preventive, with owners + dates)
- Customer-facing summary
- Lessons learned

## Tenant SLA + Credit

- Per `PRD.md`, community SLA: 99.95 % read / 99.9 % write monthly.
- Breach: tenant credit per contract.
- Per-tenant credit calculator in `governance` µservice.
