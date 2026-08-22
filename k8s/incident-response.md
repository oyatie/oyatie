---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy
deciders: ops-sre-reliability, ops-security, council-privacy, axis-cloud, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0121, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cloud-k8s/threat-model.md
  - microservices/cloud-k8s/dpia.md
  - microservices/cloud-k8s/compliance.md
  - microservices/cloud-k8s/failure-modes.md
  - microservices/cloud-k8s/multi-region.md
  - microservices/cloud-k8s/runbooks/
review_cadence: quarterly + after every Sev-1/Sev-2 incident
doc_status: published
---

# Incident Response Playbook (cloud-k8s µservice)

## Purpose

End-to-end incident-response procedure for cloud-k8s events. Covers severity classification, response roles, escalation paths, communication templates, postmortem cadence, and per-pack regulatory-notification timelines. Cross-referenced from `failure-modes.md` (severity) and `compliance.md` (regulatory frameworks).

## Severity Definitions

Per Bominal ADR-0028 (inherited) + `docs/standards/incident-severity.md`. **Control-plane down = Sev-1 always (per ADR-0121 §"Migration triggers": control-plane unavailability blocks every workload µservice from any cluster mutation).**

| Severity | Definition | Response time (page-to-ack) | Examples |
|---|---|---|---|
| **Sev-1** | **Control-plane down (always Sev-1)**; multi-tenant impact; security breach; regulatory-notification triggers; data breach | ≤ 5 min (24/7 on-call paged) | FM-01 kube-apiserver outage; FM-02 etcd quorum loss; FM-03 encryption key rotation failure; FM-08 ingress DDoS; FM-12 NetworkPolicy regression cross-tenant; FM-13 Cosign bypass; FM-14 api-proxy outage |
| **Sev-2** | Single-tenant or sub-tenant impact; operational degradation without data loss | ≤ 15 min (24/7 on-call paged) | FM-04 CP node partition; FM-05 worker node failure; FM-06 Cilium agent failure; FM-07 istiod outage; FM-09 Envoy TLS misconfig; FM-10 CSI failure; FM-11 kubeadm upgrade rollback |
| **Sev-3** | Localized impact; degraded but functional | ≤ 1h (business hours) | Single sidecar restart; single CSI PV provision delay |
| **Sev-4** | Cosmetic | next business day | Dashboard label typo; minor doc drift |

## Response Roles

| Role | Held by | Responsibility |
|---|---|---|
| **Incident Commander (IC)** | Rotating on-call lead (ops-sre-reliability primary; ops-security for security incidents) | Owns incident declaration → closure |
| **Operations Lead (OpsLead)** | ops-sre-reliability secondary | Executes runbook steps; DR failover if needed |
| **Communications Lead (CommsLead)** | gtm-customer-success or designated | Drafts + sends tenant + status page + regulatory notifications |
| **Subject-Matter Expert (SME)** | axis-cloud + relevant component owner (Cilium / Istio / CSI / etc.) | Diagnoses root cause; proposes mitigation |
| **Privacy Lead** | council-privacy chair | Activates for Sev-1 confidentiality; owns regulatory notification chain |
| **Executive Sponsor** | council-architecture chair (Sev-1 only) | Decision-rights for cross-org or external comms |
| **Scribe** | Any on-call team member | Captures timeline + decisions in `#inc-<id>` |

## Escalation Path

```text
Alert fires (Mimir/Alertmanager → OnCall)
    ↓
Primary on-call paged (ops-sre-reliability primary)
    ↓ (no ack in 5min)
Primary on-call re-paged
    ↓ (no ack in 10min total)
Secondary on-call paged
    ↓ (no ack in 15min)
Engineering manager (axis-cloud lead) paged + Slack alert
    ↓ (Sev-1 + no resolution in 30min)
Director (ops-sre-reliability + ops-security) engaged
    ↓ (Sev-1 + breach-suspect)
council-privacy chair + ExecSponsor engaged
    ↓ (confirmed breach)
Regulatory notification chain begins
    ↓ (GDPR-scope confirmed data subject impact)
72-hour clock starts (GDPR Art. 33)
```

Two-channel corroboration: Sev-1/Sev-2 fires BOTH a Mimir metric (`cloud_k8s_incident_active{severity="N"}`) AND an OnCall page.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| **1. Detection** | Alert fires; metric + page both received | ≤ 60s alert-to-page p99 |
| **2. Acknowledgement** | Primary on-call ack; opens `#inc-<id>` Slack; pages IC | ≤ 5 min Sev-1 / ≤ 15 min Sev-2 |
| **3. Triage** | IC declares severity; assigns roles; starts timeline | ≤ 10 min |
| **4. Containment** | OpsLead executes immediate-mitigation from `failure-modes.md`; Privacy Lead engaged if suspect | varies; stabilize within RTO |
| **5. Diagnosis** | SME identifies root cause | varies |
| **6. Mitigation / Resolution** | Runbook procedures; service restored | per RTO in `failure-modes.md` |
| **7. Communication** | CommsLead: tenants (status page + email); regulatory if data-impactful | per "Regulatory Notifications" |
| **8. Closure** | IC declares resolved; steady state ≥ 30 min | – |
| **9. Postmortem** | Within 5 business days; published to ops + council + auditors | ≤ 5 business days |
| **10. Action items** | Postmortem remediation tracked + owned + scheduled | indefinite |

## Tenant Communications

### Status page

- Updated ≤ 5 min of Sev-1/Sev-2 declaration.
- Updated every 30 min during active incident.
- Final resolution ≤ 30 min of closure.
- Lives at `status.oyatie.dev` (cloud-iac µservice).

### Tenant operator email — Sev-1 (data-affecting)

```
Subject: [Sev-1 / cloud-k8s] Incident in <pack>: <one-line summary>

We are investigating an incident affecting <component> in <pack> that may impact
your tenant. Started at <ISO8601>. Current status: <Investigating | Mitigating |
Resolved>. ETA to resolution: <est>.

What you may experience: <impact>
What we're doing: <action>
What you should do: <if anything; usually nothing>

We will update you again within 30 minutes or upon resolution. If this impact
involves your tenant's data, we will follow with a separate breach-notification
email per your DPA within 72 hours.

For real-time updates: <status.oyatie.dev>
For questions: <support email>
Your tenant onboarding contact: <name>
```

### Tenant operator email — Sev-2 (operational, no data impact)

```
Subject: [Sev-2 / cloud-k8s] Degradation in <pack>: <one-line summary>

We are investigating a service degradation in <pack> affecting <component>.
Started at <ISO8601>. Current status: <Investigating | Mitigating | Resolved>.

What you may experience: <impact, e.g., elevated pod scheduling latency,
delayed PV provisioning>
What we're doing: <action>

This incident is not affecting your tenant data; we will update at resolution.
```

### Customer-facing message template

Provided at tenant onboarding portal; pre-localized per pack. Tenants retain editorial control.

## Regulatory Notifications

### GDPR Art. 33 (EU; 72h from awareness)

| Event | Notification |
|---|---|
| Confirmed personal-data breach affecting EU tenants | ≤ 72h: notify lead DPA via portal |
| High-risk to data subjects (Art. 34) | Also notify affected subjects without undue delay |
| Late notification | Justify the delay in the notification |

Template at `microservices/cloud-k8s/legal/notification-templates/gdpr-art-33.md`.

### HIPAA §164.404 / §164.406 / §164.408 (US OCR)

| Event | Notification |
|---|---|
| Breach of unsecured PHI affecting < 500 individuals | OCR ≤ 60d of end of calendar year |
| Breach affecting 500+ individuals | OCR ≤ 60d + media (§164.406) + individuals (§164.404) |
| Business Associate (oyatie) → covered-entity tenant | Per BAA window (typically 24h–7d) |

### KR PIPA Art. 34 (PIPC)

| Event | Notification |
|---|---|
| Breach affecting 1+ subjects | ≤ 72h notify affected |
| Breach affecting 1000+ OR sensitive data (Art. 23) OR resident registration numbers | ≤ 72h notify PIPC + website publish |

### APPI Art. 26-2 (JP PPC)

| Event | Notification |
|---|---|
| Leakage of personal information affecting 1+ persons | ≤ 72h to PPC + affected individuals |

### LGPD Art. 48 (BR ANPD)

| Event | Notification |
|---|---|
| Security incident affecting personal data | ANPD + subjects ≤ 2 business days (per ANPD guidance) |

### DPDPA 2023 (IN DPB)

| Event | Notification |
|---|---|
| Personal-data breach | DPB ≤ 72h |

### PDPA (SG/AU/etc.)

Per-pack timelines at `regional-packs/<pack>/cloud-k8s-incident-overlay.md`. Universally 72h target.

### NIS2 (EU; cluster crosses Annex I when oyatie crosses tenant-count thresholds)

- Early warning: ≤ 24h.
- Incident notification: ≤ 72h.
- Final report: ≤ 1mo.

### DORA (EU financial services)

- Major ICT-related incident: ≤ 4h notification to competent authority.
- Initial report: ≤ 72h.
- Intermediate: ≤ 1 month.
- Final: ≤ 6 months.

### KR-FSS (financial-services KR)

- ≤ 24h notification for incidents affecting financial data integrity / availability.

### KR-CSAP

- Cluster-isolation breach (cross-tenant data exposure) reportable to CSAP authority within 24h of confirmation.

## Postmortem Procedure

Per `docs/templates/incident-postmortem-template.md`:

1. ≤ 5 business days of resolution, IC convenes postmortem.
2. Scribe's timeline = starting input.
3. Postmortem covers:
   - Summary (5 lines)
   - Timeline (chronological)
   - Impact (tenant + internal)
   - Root cause (5-whys; cite FM-ID)
   - Lessons learned
   - Action items (each owned + scheduled)
   - Runbook adequate? (yes/partial/no + improvement)
   - Trust-portal entry (for external publication if customer-facing)
4. Published to `evidence/postmortems/<year>/<incident-id>.md` (audit-chain-sealed).
5. Reviewed quarterly by council-architecture for systemic patterns.

**Blameless culture (Google SRE Workbook ch. 12)**: postmortems focus on systems + processes; postmortem document is privileged information for improvement.

## On-Call Rotation

| Tier | Rotation | Cadence |
|---|---|---|
| ops-sre-reliability primary | weekly (6 engineers) | follow-the-sun: KR / EU / US shifts |
| ops-sre-reliability secondary | weekly (same pool; offset 1 week) | – |
| axis-cloud SME | weekly (3 engineers) | KR + EU primary; US business-hours fallback |
| ops-security on-call | weekly (4 engineers); paged on Sev-1 confidentiality | 24/7 |
| council-privacy chair | named role; permanent | always-on-call for breach-suspect |
| Executive Sponsor | named role; permanent | Sev-1 only |

On-call compensation + handoff per `runbooks/oncall-rotation.md`.

## Verification

- `cargo run -p dev-cli -- gate validate incident-runbook-coverage --microservice cloud-k8s` — exit 0; every FM-ID has matching runbook.
- Quarterly DR failover drill validates the response chain end-to-end (per `multi-region.md`).
- Annual tabletop simulates Sev-1 regional outage; comms + regulatory chain rehearsed.

## References

- `microservices/cloud-k8s/failure-modes.md` (FM-IDs + severity).
- `microservices/cloud-k8s/compliance.md` §"Regulatory Notifications" (per-pack timelines).
- `microservices/cloud-k8s/multi-region.md` (DR failover).
- `microservices/cloud-k8s/runbooks/*` (per-scenario procedures).
- `microservices/cloud-k8s/dpia.md` (data-subject impact assessment).
- `microservices/cloud-k8s/threat-model.md` (security-incident threat IDs).
- `docs/standards/incident-severity.md` (cross-cutting severity).
- `docs/templates/incident-postmortem-template.md`.
- ADR-0028 (audit-chain); ADR-0121 (substrate).
- Google SRE Workbook ch. 12–14.
- GDPR Art. 33 + 34; KR PIPA Art. 34; HIPAA §164.404-408; APPI Art. 26-2; LGPD Art. 48; DPDPA 2023 §13; NIS2; DORA; KR-FSS.
