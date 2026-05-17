---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy + axis-foundry-guardrails
deciders: ops-sre-reliability, ops-security, council-privacy, axis-foundry-guardrails, council-architecture
related_adrs: [ADR-0022, ADR-0028, ADR-0117, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/foundry-guardrails/threat-model.md
  - microservices/foundry-guardrails/dpia.md
  - microservices/foundry-guardrails/compliance.md
  - microservices/foundry-guardrails/failure-modes.md
  - microservices/foundry-guardrails/multi-region.md
  - microservices/foundry-guardrails/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (foundry-guardrails µservice)

## Purpose

End-to-end incident-response procedure for foundry-guardrails events. Covers severity classification, response roles, escalation paths, communication templates, postmortem cadence, per-pack regulatory-notification timelines.

**Special clause**: confirmed jailbreak success (FM-06) is ALWAYS Sev-1 regardless of single-tenant scope, because the safety floor itself failed; aggregate trust posture demands maximum response.

## Severity Definitions

Per `docs/standards/incident-severity.md` + this µservice's safety-bearing posture.

| Severity | Definition | Response time (page-to-ack) | Examples |
|---|---|---|---|
| **Sev-1** | Production-tier confidentiality / integrity / availability impact AND/OR safety-floor failure AND/OR regulatory trigger | ≤ 5 min (24/7) | FM-03 classifier outage cluster-wide; FM-05 default-deny drift; FM-06 jailbreak success (always); FM-12 cross-tenant rule leak; FM-13 pack misroute; FM-14 cluster-wide; FM-15 coupling failure |
| **Sev-2** | Single-tenant or sub-cluster impact; operational degradation; gate fail-closed | ≤ 15 min (24/7) | FM-01 classifier single AZ; FM-02 shadow regression; FM-04 cedar timeout isolated; FM-07 FP surge; FM-09 Postgres unavailable; FM-10 backup corruption; FM-14 isolated |
| **Sev-3** | Localized impact; degraded but functional | ≤ 1 h (business hours) | FM-08 tenant FP budget exhausted; FM-11 LLM-judge budget exceeded |
| **Sev-4** | Cosmetic | next business day | dashboard label typo |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander (IC)** | Rotating on-call lead (axis-foundry-guardrails for safety/policy; ops-sre-reliability for infra; ops-security for security) | Owns incident |
| **OpsLead** | ops-sre-reliability secondary | Executes runbook steps |
| **CommsLead** | gtm-customer-success or designated | Tenant + status-page + regulatory notifications |
| **SME** | axis-foundry-guardrails + relevant detector/model authors | Diagnoses |
| **PrivacyLead** | council-privacy chair | Sev-1 confidentiality + regulatory chain |
| **ExecSponsor** | council-architecture chair (Sev-1 only) | Cross-org decision-rights |
| **Scribe** | any on-call member | Timeline capture in `#inc-<id>` |

## Escalation Path

```text
Alert fires (Mimir → OnCall)
    ↓
Primary on-call paged (axis-foundry-guardrails primary)
    ↓ (if no ack in 5min)
Primary re-paged
    ↓ (if no ack in 10min)
Secondary on-call paged
    ↓ (if no ack in 15min)
Engineering manager (axis-foundry-guardrails lead) paged
    ↓ (if Sev-1 and no resolution in 30min)
Directors (ops-sre-reliability + ops-security + axis-foundry-guardrails) engaged
    ↓ (if Sev-1 and breach/safety-floor failure)
council-privacy chair + ExecSponsor engaged
    ↓ (if confirmed breach OR jailbreak-success-with-tenant-impact)
Regulatory notification chain (see below)
```

Two-channel corroboration: every Sev-1/Sev-2 fires Mimir metric AND OnCall page; both required.

## Incident Lifecycle

| Phase | Activity | Time |
|---|---|---|
| 1. Detection | Alert + page | ≤ 60s |
| 2. Acknowledgement | Ack + open `#inc-<id>`; page IC | ≤ 5 min (Sev-1) |
| 3. Triage | Declare severity; assign roles; start timeline | ≤ 10 min |
| 4. Containment | OpsLead executes mitigation per `failure-modes.md`; PrivacyLead engaged if suspect | per RTO |
| 5. Diagnosis | SME root-cause | varies |
| 6. Mitigation | Runbook procedures | per RTO |
| 7. Communication | CommsLead notifies tenants + status page + regulatory (per pack) | per timelines |
| 8. Closure | IC declares resolved after ≥ 30 min steady state | – |
| 9. Postmortem | Within 5 business days | – |
| 10. Action items | Tracked + owned + scheduled | indefinite |

## Tenant Communications

### Status page (public)
- Updated within 5 min of Sev-1/Sev-2 declaration.
- Updated every 30 min during active incident.
- Final resolution within 30 min of closure.
- Lives at `status.oyatie.dev`.

### Tenant operator email
- Sev-1: within 30 min of declaration.
- Sev-2: within 1 h.
- Templates at `legal/incident-comms-templates.md` (Slice D).

### Tenant-of-tenant (end-user) notification
- Triggered only if data-impact confirmed.
- Joint-controllership: oyatie notifies tenant operator within the regulatory timeline; tenant notifies its end-users per its own DPA obligations.

## Regulatory Notifications (per-pack timelines)

| Framework | Trigger | Timeline | Recipient |
|---|---|---|---|
| GDPR Art. 33 | Personal data breach affecting EU-resident data subjects | 72h from awareness | EU DPA (per tenant DPA) |
| GDPR Art. 34 | High-risk-to-rights breach | "without undue delay" | Data subjects (via tenant) |
| KR PIPA Art. 34 | Personal-info breach affecting KR-resident | 72h | PIPC + data subjects |
| HIPAA §164.404 | PHI breach affecting > 500 individuals | 60 days | HHS OCR + individuals + media (if > 500) |
| HIPAA §164.406 | PHI breach affecting < 500 | annual log | HHS OCR (annual) |
| LGPD Art. 48 | Personal data breach (BR-resident) | "reasonable timeframe" | ANPD + data subjects |
| DPDPA 2023 §10(2) | Personal data breach (IN-resident) | "as may be prescribed" (72h equivalent per draft rules) | Data Protection Board of India |
| APPI Art. 22-2 | Personal-info leakage > 1000 records OR sensitive | promptly | PPC |
| PDPA Part V (SG) | Significant breach | 72h | PDPC |
| UAE PDPL FDL 45/2021 | Personal-data breach | 72h | UAE Data Office |
| KSA PDPL | Personal-data breach | 72h | SDAIA |
| EU AI Act Art. 73 (post-market monitoring) | Serious incident in high-risk AI system | 15 days from awareness; 2 days if death/serious-injury/widespread | Market surveillance authority |
| NIS2 (2022/2555) | Significant incident (essential / important entity) | 24h initial + 72h detailed + 1mo final | Designated CSIRT |

## Jailbreak-Success Specific Protocol (FM-06; ALWAYS Sev-1)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage axis-foundry-guardrails IC + ops-security IC jointly | ≤ 5 min |
| 2 | Capture: prompt hash; provider output hash; affected tenant; affected capability; classifier model versions; ensemble verdict; cedar bundle SHA | ≤ 5 min |
| 3 | Freeze offending capability for affected tenant; OPTIONAL: freeze for all tenants if ensemble pattern suggests widespread risk | ≤ 5 min |
| 4 | Auto-allocate incident ID; auto-generate post-mortem template per `runbooks/jailbreak-escalation.md` | ≤ 2 min |
| 5 | Pin failing prompt to red-team fixture catalogue (`tests/jailbreak/golden_fixtures.rs`); will be re-tested on every classifier rollout | ≤ 1h |
| 6 | Determine tenant-impact + data-subject impact (was unsafe content delivered to an end-user?) — engage PrivacyLead | per investigation |
| 7 | If data-subject impact confirmed: begin breach-notification chain (per-pack timelines above) | per regulatory |
| 8 | Classifier retraining: data-team retrains affected models on the new fixture + adjacent perturbations; shadow→enforce rollout | days-to-weeks |
| 9 | Post-mortem within 5 business days; action items tracked | ≤ 5 BD |

## Cross-Tenant Rule-Leak Specific Protocol (FM-12; ALWAYS Sev-1)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-security IC | ≤ 5 min |
| 2 | Freeze affected REST endpoint(s) | ≤ 5 min |
| 3 | Revoke implicated SPIFFE / OIDC tokens | ≤ 10 min |
| 4 | Forensic trace: which tenants' rules were applied to which tenants' requests? Postgres mutation log + Cedar evaluation log replay | per investigation |
| 5 | If confirmed: breach-notification chain (per-pack timelines) | per regulatory |
| 6 | Cedar policy patch + Postgres RLS hardening; pen-test boundary | days |
| 7 | Post-mortem; lessons → LEAN lane upgrade if Cedar gap revealed | ≤ 5 BD |

## Pack-Misroute Specific Protocol (FM-13; ALWAYS Sev-1)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-security + council-privacy | ≤ 5 min |
| 2 | Freeze offending route; verify scope of misroute (how many tenants × how many requests × which packs?) | ≤ 15 min |
| 3 | If GDPR-scope: breach notification 72h chain begins | per Art. 33 |
| 4 | If KR-scope: PIPC 72h chain begins | per PIPA Art. 34 |
| 5 | Engineering fix to pack-routing logic (in foundry-runtime); LEAN lane gap closed | days |
| 6 | Post-mortem; transfer register updated | ≤ 5 BD |

## Postmortem Cadence

- Sev-1: postmortem published within **5 business days**.
- Sev-2: within **10 business days**.
- Sev-3: within **15 business days** (or aggregated quarterly).
- Sev-4: aggregated quarterly.

Template: `runbooks/postmortem-template.md` (Slice D); includes timeline + root-cause + mitigations + action-items + lessons.

## Quarterly Drill Cadence

- Q1: jailbreak red-team drill (FM-06 scenario simulation).
- Q2: DR failover drill (DR-pair pack).
- Q3: cross-tenant rule-leak drill (FM-12 simulation in non-prod).
- Q4: classifier-model rollback drill (FM-02 scenario).

## References

- ADR-0022, ADR-0028, ADR-0117, ADR-0130, ADR-0131, ADR-0140.
- `microservices/foundry-guardrails/threat-model.md`.
- `microservices/foundry-guardrails/dpia.md`.
- `microservices/foundry-guardrails/compliance.md`.
- `microservices/foundry-guardrails/failure-modes.md`.
- `microservices/foundry-guardrails/runbooks/`.
- `microservices/observability/incident-response.md` (sibling reference).
- GDPR (Reg 2016/679) Arts. 33-34.
- EU AI Act (Reg 2024/1689) Art. 73 (post-market monitoring + serious-incident reporting).
- HIPAA 45 CFR §§164.404-410.
- KR PIPA Art. 34.
- LGPD Art. 48.
- DPDPA 2023 §10.
- NIS2 (Directive 2022/2555).
