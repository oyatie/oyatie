---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy + axis-foundry
deciders: ops-sre-reliability, ops-security, council-privacy, axis-foundry, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/intelligence-providers/threat-model.md
  - microservices/intelligence-providers/dpia.md
  - microservices/intelligence-providers/compliance.md
  - microservices/intelligence-providers/failure-modes.md
  - microservices/intelligence-providers/multi-region.md
  - microservices/intelligence-providers/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (foundry-providers µservice)

## Purpose

End-to-end incident-response procedure for foundry-providers events. Covers severity classification, response roles, escalation paths, communication templates, postmortem cadence, and per-pack regulatory-notification timelines.

## Severity Definitions

| Severity | Definition | Response time (page-to-ack) | Examples |
|---|---|---|---|
| **Sev-1** | Confidentiality / integrity / availability impact affecting multiple tenants; regulatory-notification triggers; credential compromise; cross-pack data leakage | ≤ 5 min | FM-FP-03 credential leak; FM-FP-05 adapter substitution; FM-FP-08 cross-pack mis-route |
| **Sev-2** | Single-tenant or sub-tenant impact; degraded gate function; single-vendor outage with available alternates | ≤ 15 min | FM-FP-01 single-vendor outage; FM-FP-02 single-tenant rate-limit cascade; FM-FP-04 in-house regression (canary) |
| **Sev-3** | Localised impact; auto-recovery in progress | ≤ 1 h | FM-FP-09 single-AZ pod outage; FM-FP-11 Valkey Sentinel failover |
| **Sev-4** | Cosmetic; no operational impact | next business day | dashboard label typo; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander (IC)** | Rotating on-call lead (ops-sre-reliability primary; ops-security for credential / supply-chain events) | Owns incident from declaration to closure |
| **Operations Lead** | ops-sre-reliability secondary | Executes runbook steps; performs DR failover |
| **Communications Lead** | gtm-customer-success | Tenant + status page + regulatory notifications |
| **Subject-Matter Expert** | axis-foundry + relevant vendor specialist | Diagnoses + mitigation proposal |
| **Privacy Lead** | council-privacy chair | Sev-1 confidentiality events; regulatory notification chain |
| **Executive Sponsor** | council-architecture chair (Sev-1 only) | Cross-org / external comms approval |
| **Scribe** | Any on-call member | Timeline + decisions in `#inc-<id>` Slack channel |

## Escalation Path

```text
Alert fires (Mimir/Alertmanager → OnCall)
    ↓
Primary on-call paged (ops-sre-reliability primary)
    ↓ (if no ack in 5min)
Primary on-call re-paged
    ↓ (if no ack in 10min total)
Secondary on-call paged (ops-sre-reliability secondary)
    ↓ (if no ack in 15min total)
axis-foundry SME paged
    ↓ (if Sev-1 confidentiality event)
ops-security on-call paged
    ↓
PrivacyLead paged
    ↓ (if regulatory-notification clock starts)
ExecSponsor paged
```

## Communication Templates

### Internal incident channel header (`#inc-<id>`)

```
SEV: <Sev-1|2|3>
TITLE: <short>
IC: @<person>
OpsLead: @<person>
CommsLead: @<person>
SME: @<person>
PrivacyLead (Sev-1 only): @<person>
SCRIBE: @<person>
DECLARED: <UTC timestamp>
STATUS PAGE: <URL>
SLACK: #inc-<id>
```

### Status page entry (tenant-facing)

```
INVESTIGATING — <UTC>
We're seeing degraded behavior on <provider> calls for <pack>. Tenant
workloads using <alt-provider> are not affected; we have routed traffic
to alternates. ETA next update: 15min.

IDENTIFIED — <UTC>
Root cause: <one line>. Mitigation in progress.

MONITORING — <UTC>
Mitigation deployed; observing recovery.

RESOLVED — <UTC>
Service restored. Postmortem to follow within 5 business days.
```

### Per-pack regulatory-notification template

| Pack | Trigger | Authority | Window |
|---|---|---|---|
| pack-eu | GDPR Art. 33 personal-data breach | EU DPA(s) | 72h |
| pack-kr | KR PIPA breach | PIPC | 24h to PIPC; 72h to subjects |
| pack-us-healthcare | HIPAA breach (PHI ≥ 500 individuals) | HHS + media | 60 days |
| pack-jp | APPI breach | PPC (Japan) | timely (no fixed window) |
| pack-au | Privacy Act notifiable data breach | OAIC | 30 days |
| pack-in | DPDPA breach | Data Protection Board | timely |
| pack-br | LGPD breach | ANPD | reasonable timeframe |
| pack-ae | UAE PDPL breach | Office of the Data Protection Commissioner | timely |
| pack-ksa | PDPL breach | SDAIA | timely |

## Postmortem Cadence

| Severity | Postmortem? | Window |
|---|---|---|
| Sev-1 | Required | 5 business days |
| Sev-2 | Required | 10 business days |
| Sev-3 | Discretionary; required if recurring | 15 business days |
| Sev-4 | Not required | – |

Postmortem template at `docs/templates/postmortem.md`. Published to `evidence/postmortems/<year>/<incident-id>.md`. Reviewed at quarterly ops review.

## Per-Severity Procedure

### Sev-1

1. Page primary on-call → IC declared within 5 min of page.
2. `#inc-<id>` Slack channel created; header populated; PrivacyLead engaged.
3. Status page entry within 15 min of declaration.
4. Tenant notification within 30 min for affected tenants.
5. Regulatory-notification clock starts where applicable.
6. Mitigation per relevant runbook.
7. Verification once mitigation deployed; status page MONITORING.
8. RESOLVED status when SLI returns to baseline + sustained 30 min.
9. Postmortem in 5 business days.

### Sev-2

Same flow, less aggressive: status page within 30 min, tenant notification within 1 h, postmortem in 10 days.

### Sev-3 / Sev-4

Status page optional (Sev-3 if tenant-visible); no postmortem mandatory for Sev-4.

## Quarterly Drills

| Drill | Cadence | Owner |
|---|---|---|
| Credential rotation (per pack per vendor) | Quarterly | ops-security |
| Provider outage fail-over | Quarterly | axis-foundry + ops-sre-reliability |
| Region failover (per DR-pair pack) | Quarterly | ops-sre-reliability |
| In-house model rollback | Quarterly | axis-foundry |
| Adapter version pin under vendor breaking change | Annually | axis-foundry |
| Red-team T-01 / T-03 / T-05 | Annually | ops-security |

Evidence recorded at `evidence/runbook-drills/`.

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=incident-response --microservice foundry-providers` exits 0.
- Quarterly drill evidence committed.
- Per-Sev-1 postmortem published within window.

## References

- ADR-0028 — audit-chain.
- `microservices/intelligence-providers/threat-model.md`.
- `microservices/intelligence-providers/dpia.md`.
- `microservices/intelligence-providers/compliance.md` §"Breach notification".
- `docs/templates/postmortem.md`.
