---
doc_class: IncidentResponsePlan
template_id: TPL-INCIDENT-RESPONSE
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: ops-sre-reliability + axis-intelligence + council-privacy
related_adrs: [ADR-0255, ADR-0263]
related_artifacts:
  - microservices/intelligence/failure-modes.md
  - microservices/intelligence/runbooks/
doc_status: published
---

# Incident Response — intelligence µservice

## Purpose

Define the incident-response process specific to the intelligence µservice, including
severity classification, escalation chains, regulator-notification SLAs per pack, and
post-incident review obligations.

## Severity classification

Same as `failure-modes.md` Severity table. The IC declares severity at incident open and
updates on new information.

| Severity | Examples | Response time | Comms cadence |
|---|---|---|---|
| Sev-1 | EU AI Act Annex III leak (FM-09); audit forgery (FM-11); regulator-notifiable breach | ≤ 15 min IC engaged | every 30 min |
| Sev-2 | Provider-outage cascade (FM-01..03); refusal false-positive cascade (FM-08) | ≤ 30 min IC engaged | every 2 h |
| Sev-3 | Single-provider hiccup (FM-04 short window); credential rotation issue (FM-06) | ≤ 2 h | once at start + once at resolve |
| Sev-4 | Self-healing (FM-18, FM-19) | best-effort | once at resolve |

## Escalation chain

```text
Detection (Mimir alert / audit-tap signature mismatch / tenant report)
   ↓
PagerDuty page → primary on-call (axis-intelligence)
   ↓ (if no ack in 5 min)
secondary on-call (ops-sre-reliability)
   ↓ (if Sev-1)
council-privacy (for data + AI Act)  +  ops-security (for breach)  +  ops-legal (for regulator notif)
   ↓ (if Sev-1 sustained > 1h)
ExecSponsor (intelligence)  +  CISO  +  CPO
```

## Sev-1 response template

| Stage | Action | Time |
|---|---|---|
| Open | IC declared; `#inc-<id>` channel open; severity logged | ≤ 5 min |
| Triage | Identify FM ID; consult `failure-modes.md`; mobilise runbook | ≤ 15 min |
| Containment | Halt affected dispatch (e.g., refuse all dispatch for the Annex III category) | ≤ 30 min |
| Forensics | Export audit-tap records for affected window; engage forensic chain | ≤ 2 h |
| Eradication | Root-cause fix landed via emergency PR (multispectrum-review compressed); deploy | ≤ 12 h |
| Recovery | Re-enable dispatch; verify SLOs return to green | ≤ 24 h |
| Communications | Status-page + tenant comms + regulator notif per pack | per pack SLA |
| Postmortem | Published within 5 business days; action items tracked | 5 BD |

## Regulator notification per pack

| Pack | Trigger | Notify | SLA | Form / template |
|---|---|---|---|---|
| pack-eu | Personal-data breach (GDPR Art. 33) | Member-state DPA via lead supervisory authority | 72 h | `legal/dpa-notification-template-eu.md` |
| pack-eu | Serious incident under EU AI Act Art. 73 | EU AI Office + national competent authority | 15 days (incident) / 10 days (incident with widespread infringement) / 2 days (irreversible harm) | `legal/ai-act-incident-template-eu.md` |
| pack-kr | KR PIPA Art. 34 personal-info infringement notification | PIPC | 72 h | `legal/dpa-notification-template-kr.md` |
| pack-us-healthcare | HIPAA breach (45 CFR §164.404-410) | HHS OCR + affected individuals + (if ≥ 500) media | 60 days | `legal/hipaa-breach-template.md` |
| pack-us | State breach-notification laws (50 states) | per-state attorneys general | varies (typically 30-60 days) | per-state |
| pack-jp | APPI Art. 26 (sensitive data leak ≥ 1k records) | PPC | promptly (Cabinet Order) | `legal/dpa-notification-template-jp.md` |
| pack-sg | PDPA Part VIA notifications | PDPC | 72 h (significant harm) / 30 days (≥ 500 individuals) | `legal/dpa-notification-template-sg.md` |
| pack-au | NDB scheme | OAIC + affected individuals | "as soon as practicable" | `legal/dpa-notification-template-au.md` |
| pack-in | DPDPA 2023 §8(6) | DPBI | as prescribed | `legal/dpa-notification-template-in.md` |
| pack-br | LGPD Art. 48 | ANPD + affected individuals | "reasonable time" | `legal/dpa-notification-template-br.md` |
| pack-ae | UAE PDPL Art. 26 | UAE Data Office | 72 h | `legal/dpa-notification-template-ae.md` |
| pack-ksa | KSA PDPL Art. 22 | SDAIA | 72 h | `legal/dpa-notification-template-ksa.md` |
| pack-cn | CN PIPL Art. 57 | CAC + affected individuals | promptly | `legal/dpa-notification-template-cn.md` |
| pack-uk | UK GDPR Art. 33 | ICO | 72 h | `legal/dpa-notification-template-uk.md` |
| pack-us-federal | FISMA + CISA + agency-specific | per-agency CIO + CISA | varies | `legal/cisa-incident-template.md` |

## Comms templates

- Status-page: `legal/status-page-templates/{outage,degradation,breach}.md`.
- Tenant comms: `legal/tenant-comms-templates/{sev-1,sev-2,resolution,postmortem-summary}.md`.

## Postmortem requirements

Every Sev-1 + Sev-2 requires a postmortem within 5 business days, published to
`evidence/postmortems/<year>/<incident-id>.md`. The postmortem must include:

1. Timeline (UTC).
2. Root cause (one-line + technical detail).
3. Contributing factors.
4. Why the gate / runbook did or did not catch it.
5. Action items (P0/P1/P2; with owners + dates).
6. Lessons learned (multispectrum-review-equivalent self-critique).
7. Regulator notification record (if applicable).
8. Status-page entry record.

## On-call rotation

- Primary: axis-intelligence (24/7 follow-the-sun across KR/EU/US).
- Secondary: ops-sre-reliability.
- Council-privacy and ops-security maintain their own on-call rotations engaged by IC for Sev-1.

## Drill cadence

| Drill | Frequency |
|---|---|
| Provider-outage failover (FM-01..04) | Quarterly |
| Sidecar credential-handle expired drill (FM-05) | Quarterly |
| Prompt-injection detected drill (FM-07) | Quarterly |
| Refusal false-positive cascade drill (FM-08) | Quarterly |
| Annex III leak forensic drill (FM-09) | Semi-annually |
| Audit-tap emission failure drill (FM-10) | Quarterly |
| DR failover drill (per pack with DR pair) | Annually |

## References

- ADR-0255, ADR-0263.
- `microservices/intelligence/failure-modes.md`.
- `microservices/intelligence/runbooks/`.
- `microservices/intelligence/threat-model.md`.
- Per-pack regulator templates in `legal/`.
