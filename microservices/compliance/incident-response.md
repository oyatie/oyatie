---
microservice: compliance
doc: IncidentResponse
status: Drafting
authority_tier: 2
owner: axis-compliance
co_owners: [axis-security, ops-sre-reliability]
date: 2026-05-18
related_adrs: [ADR-0209]
---

# Compliance — Incident Response

## Severity ladder

| Severity | Definition | Page | SLA |
|---|---|---|---|
| Sev-1 | Cross-tenant DSAR leak OR audit-chain seal verify failure OR engagement-end Cedar revoke failed | PagerDuty primary on-call (axis-security + axis-compliance) | 15-minute response; communicate within 1 hour |
| Sev-2 | DSAR backlog > 100 OR PHI access anomaly OR collector tier degraded | PagerDuty secondary | 1-hour response; communicate within 4 hours |
| Sev-3 | DSAR rate anomaly OR auditor portal latency burn | Slack #compliance-on-call | 4-hour response |
| Sev-4 | Single artifact emit fail | Slack notification | next business day |

## Sev-1 playbooks

### Cross-tenant DSAR leak

1. **Detect** — kernel invariant fired OR integration test caught in CI OR auditor flagged.
2. **Contain** — suspend DSAR API (`POST /api/v1/dsar/*` → 503).
3. **Triage** — identify scope: which DSAR responses, which subjects, which tenants.
4. **Notify** — within 72 hours of confirmation, notify affected subjects + tenants + data-protection authority (GDPR Art. 33).
5. **Investigate** — postmortem; root-cause via audit chain.
6. **Remediate** — fix invariant; add integration test; re-deploy.
7. **Re-open API** — staged rollout with extra logging.

### Audit-chain seal verify failure

1. **Detect** — `EVT-AUDIT-SEAL-VERIFY-FAILED` fires.
2. **Contain** — flag affected artifacts in auditor portal banner.
3. **Triage** — check seal chain continuity + cosign trust root + Rekor log.
4. **Investigate** — could be (i) OIDC issuer compromise, (ii) cosign key drift, (iii) Rekor log gap.
5. **Remediate** — depends on root cause.

## Communication templates

`runbooks/communication-templates/` includes draft text for:

- Subject notification (Art. 33).
- Tenant notification.
- Data-protection authority notification (per jurisdiction).
- Internal stakeholder notification.

## Postmortem requirements

Blameless postmortem within 5 business days. Posted to `evidence/postmortems/INC-<YYYY-MM-DD>-<slug>.md`. Includes:

- Timeline.
- Detection lag.
- Root cause.
- Remediation.
- Action items + owners.

## References

- ADR-0209 — substrate authority.
- GDPR Art. 33 — 72-hour breach notification.
- HIPAA Breach Notification Rule (45 CFR § 164.400-414).
