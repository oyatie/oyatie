---
doc_class: IncidentResponsePlaybook
title: Incident Response Playbook (audit-chain)
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + council-privacy + axis-audit-chain
deciders: ops-sre-reliability, ops-security, council-privacy, axis-audit-chain, council-architecture
related_adrs: [ADR-0028, ADR-0003, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/audit-chain/threat-model.md
  - microservices/audit-chain/dpia.md
  - microservices/audit-chain/compliance.md
  - microservices/audit-chain/failure-modes.md
  - microservices/audit-chain/multi-region.md
  - microservices/audit-chain/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Playbook (audit-chain µservice)

## Purpose

End-to-end incident-response for audit-chain. **Audit tampering is always Sev-1** — chain integrity is the load-bearing posture for every µservice's compliance claim. Many failure modes that would be Sev-2 elsewhere are Sev-1 here.

## Severity Definitions

Per oyatie incident-severity standard (`docs/standards/incident-severity.md`) with audit-chain-specific overlay:

| Severity | Definition | Response time | Examples |
|---|---|---|---|
| **Sev-1** | Any chain-integrity event; cross-tenant leak; cross-pack misroute; HSM compromise suspect; data breach trigger | ≤ 5 min (24/7) | FM-02 HSM mismatched sig; FM-03 cross-channel divergence; FM-04 genesis mismatch; FM-09 cross-pack; FM-10 verify-failed spike; FM-12 mass-delete anomaly; FM-15 source impersonation |
| **Sev-2** | Operational degradation without integrity breach; emission OK, sealing degraded; single-tenant impact | ≤ 15 min (24/7) | FM-01 HSM outage (eventual-seal); FM-05 Postgres outage; FM-06 S3 outage; FM-07 worker crashloop; FM-08 emission overload; FM-11 DSR backlog; FM-13 key rotation overdue retire; FM-14 PII detected (non-PHI) |
| **Sev-3** | Localized; non-blocking | ≤ 1h | flaky tests; non-critical SLI breach |
| **Sev-4** | Cosmetic | next business day | doc drift |

## Response Roles

Same shape as `observability/incident-response.md` §"Response Roles" with audit-chain-specific addition:

| Role | Held by | Responsibility |
|---|---|---|
| Incident Commander | ops-sre-reliability primary OR ops-security (security incidents) | Coordinates; owns to closure |
| Operations Lead | ops-sre-reliability secondary | Runbook execution |
| Communications Lead | gtm-customer-success | Tenant + status-page + regulatory comms |
| Subject-Matter Expert | axis-audit-chain | Diagnoses chain-specific issues |
| Cryptography SME | axis-audit-chain + cloud-secrets | Activates for HSM / cryptography incidents |
| Privacy Lead | council-privacy | Activates for any breach-suspect (Sev-1 chain) |
| Executive Sponsor | council-architecture chair | Sev-1 only |
| Scribe | Rotating | Timeline + decisions in `#inc-<id>` |

## Escalation Path

```text
Alert fires (Mimir + OnCall + cross-channel-divergence-watcher)
    ↓
Primary on-call paged
    ↓ (no ack in 5min)
Re-paged
    ↓ (no ack in 10min)
Secondary on-call paged
    ↓ (no ack in 15min)
Engineering manager (axis-audit-chain lead) + Slack alert
    ↓ (Sev-1 / no resolution in 30min)
ops-security + ops-sre-reliability directors engaged
    ↓ (Sev-1 chain-integrity-suspect)
council-privacy chair + ExecSponsor + Cryptography SME engaged
    ↓ (confirmed breach)
Regulatory notification chain begins (see §"Regulatory Notifications")
    ↓ (GDPR-scope breach)
72h clock starts (GDPR Art. 33)
```

Three-channel corroboration: every Sev-1 alert fires Mimir metric + OnCall page + cross-channel root divergence watcher. If two channels are silent and one fires, treat as still-suspect until evidence clarifies.

## Incident Lifecycle

| Phase | Activity | Time bound |
|---|---|---|
| 1. Detection | Alert + page | ≤ 60s p99 |
| 2. Acknowledgement | Primary ack; open `#inc-<id>` | ≤ 5 min (Sev-1) |
| 3. Triage | IC declares severity; assigns roles | ≤ 10 min |
| 4. Containment | OpsLead executes immediate mitigation per `failure-modes.md`; Privacy + Crypto Leads engaged if Sev-1 | per RTO |
| 5. Diagnosis | SME identifies root cause; Cryptography SME for chain-integrity events | varies |
| 6. Mitigation / Resolution | Runbook procedures | per `failure-modes.md` RTO |
| 7. Communication | CommsLead notifies tenants + regulators per timelines | per §"Regulatory Notifications" |
| 8. Closure | IC declares resolved; ≥ 30min steady state | – |
| 9. Postmortem | Within 5 business days | ≤ 5 business days |
| 10. Action items | Tracked + owned + scheduled | indefinite |

## Tenant Communications

### Status page

- Updated within 5 min of Sev-1/2 declaration.
- Every 30 min during active.
- Final resolution within 30 min.
- Lives at `status.oyatie.dev`.

### Tenant operator email — Sev-1 (chain-integrity affecting)

```
Subject: [Sev-1 / audit-chain] Chain-integrity event in <pack>: <one-line summary>

We are investigating an incident affecting the audit-chain substrate in <pack>
that may impact the verifiability of recent audit records for your tenant.
Started at <ISO8601>. Current status: <Investigating | Mitigating | Resolved>.

What you may experience:
- Verification queries against recent events in <pack> may return inconclusive results.
- New audit events will continue to emit; sealing may be delayed.
- This DOES NOT affect existing records sealed before <pack-incident-window-start>.

What we're doing:
- Cryptography SME engaged; HSM partition <quarantined/verified>.
- Cross-channel root publication <under-investigation/restored>.
- ops-security investigating root cause.

What you should do:
- DEFER any time-sensitive audit-evidence exports until incident resolved.
- If you have a regulator inquiry pending within this window, contact <support email> for a hold-letter.

Next update: within 30 minutes or upon resolution, whichever is sooner.
If this incident is confirmed as a data integrity breach affecting your records,
we will follow with a breach-notification email per your DPA within 72 hours.

Status: <status.oyatie.dev link>
```

### Tenant operator email — Sev-2 (operational degradation; no integrity breach)

```
Subject: [Sev-2 / audit-chain] Degradation in <pack>: <one-line summary>

We are investigating a service degradation in audit-chain <component> in <pack>.
Started at <ISO8601>. Current status: <Investigating | Mitigating | Resolved>.

What you may experience:
- Audit events emitted during this window may show 'sealed: false' for longer
  than usual; sealing will catch up once <component> is restored.
- Forensic queries may return higher latency.

This incident is NOT affecting your tenant data integrity; we will update at resolution.

Status: <status.oyatie.dev link>
```

## Regulatory Notifications

### GDPR Art. 33 (EU SA, 72h clock from awareness)

| Event | Notification |
|---|---|
| Confirmed personal-data breach affecting EU-resident tenants | Within 72h: notify lead DPA |
| High risk to data subjects (Art. 34) | Also notify subjects without undue delay |
| Late | Justify delay |

Template — DPA notification (audit-chain specific addendum):
- Date/time of breach discovery: <ISO8601>
- Nature: <integrity-suspect / cross-tenant-leak / cross-pack-misroute / verification-failed-attack-pattern>
- Records affected: <est. number; pack(s); tenant(s)>
- Chain-integrity status: <intact / questionable / breached>
- Cryptographic evidence preserved: <YES — all S3 WORM intact; SealRecords preserved; HSM-side audit log retained>
- Measures: <HSM partition quarantined; cross-channel validator strengthened; rotation initiated; tenant-side verification SDK signed-build verified>
- DPO contact: council-privacy chair.

### HIPAA §164.404 / §164.406 / §164.408 (US OCR)

| Event | Notification |
|---|---|
| Breach of unsecured PHI affecting < 500 individuals | OCR notification ≤ 60d of end-of-calendar-year |
| Breach affecting 500+ individuals | OCR ≤ 60d + media (§164.406) + individual (§164.404) |
| Business Associate (oyatie) | Notify covered-entity tenant per BAA window (typically ≤ 24h to 7d for chain-integrity events) |

### KR PIPA Art. 34 (PIPC)

| Event | Notification |
|---|---|
| Breach affecting 1+ subject | Notify subjects within 72h |
| Breach affecting 1000+ subjects OR sensitive data (Art. 23) OR resident registration numbers | Notify PIPC within 72h + publish on website |

KR-specific: chain-integrity events affecting electronic-document records may trigger 전자문서법 obligations; engage KR-counsel.

### APPI Art. 26-2 (Japan PPC)

| Event | Notification |
|---|---|
| Leakage of personal information affecting 1+ persons | Notify PPC + subjects within reasonable time (target ≤ 72h) |

### LGPD Art. 48 (Brazil ANPD)

| Event | Notification |
|---|---|
| Security incident affecting personal data | Notify ANPD + subjects within "reasonable period" (target ≤ 2 business days) |

### DPDPA 2023 (India DPB)

| Event | Notification |
|---|---|
| Personal-data breach | Notify Data Protection Board within 72h of awareness |

### NIS2 (EU 2022/2555)

For oyatie above Annex I/II thresholds:
- Early warning ≤ 24h.
- Incident notification ≤ 72h.
- Final report ≤ 1 month.

### KR-FSS (financial-services KR tenants)

Notify FSS ≤ 24h for incidents affecting financial-record integrity.

### SAMA (Saudi Arabia financial-services)

Notify SAMA Cybersecurity Operations Center ≤ 4h for chain-integrity events affecting SAMA-regulated financial records.

## Postmortem Procedure

Per `docs/templates/incident-postmortem-template.md`:

1. Within 5 business days, IC convenes postmortem.
2. Scribe's timeline is the starting input.
3. Postmortem covers:
   - Summary (5 lines)
   - Timeline (chronological)
   - Impact (tenant-facing + internal + regulatory)
   - Root cause (5-whys; cite FM-ID)
   - **For Sev-1 chain-integrity events: cryptographic-forensic analysis** (what was tampered; what mitigation prevented full breach; what mitigation failed)
   - Lessons learned
   - Action items (each owned + scheduled)
   - Was the runbook adequate?
   - Trust-portal entry (for external publication if customer-facing)
4. Published to `evidence/postmortems/audit-chain/<year>/<incident-id>.md` (audit-chain-sealed; the postmortem-of-the-audit-chain is itself sealed).
5. Reviewed quarterly by council-architecture for systemic patterns.

Blameless culture per Google SRE Workbook ch. 12.

## On-Call Rotation

| Tier | Rotation | Cadence |
|---|---|---|
| ops-sre-reliability primary | weekly (6 engineers) | follow-the-sun KR / EU / US |
| ops-sre-reliability secondary | weekly (offset 1 week) | – |
| axis-audit-chain SME | weekly (3 engineers) | KR + EU primary; US business-hours |
| Cryptography SME | weekly (2 engineers from axis-audit-chain + 1 from cloud-secrets) | always-on-call for Sev-1 chain integrity |
| ops-security on-call | weekly (4 engineers) | 24/7 for Sev-1 |
| council-privacy chair | named role; permanent | always-on-call for breach-suspect |
| Executive Sponsor | named role; permanent | Sev-1 only |

On-call compensation + handoff per `runbooks/audit-chain-restart.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate incident-runbook-coverage --microservice audit-chain` — exit 0; every FM-ID has a matching runbook.
- Quarterly DR failover drill validates response chain.
- Annual tabletop simulates Sev-1 regional + chain-integrity-suspect scenario.

## References

- `microservices/audit-chain/failure-modes.md`.
- `microservices/audit-chain/compliance.md` §"Regulatory Notifications".
- `microservices/audit-chain/multi-region.md`.
- `microservices/audit-chain/runbooks/*`.
- `microservices/audit-chain/dpia.md`.
- `microservices/audit-chain/threat-model.md`.
- ADR-0028 + ADR-0003.
- Google SRE Workbook ch. 12–14.
- GDPR Arts. 33 + 34; HIPAA §164.404-408; KR PIPA Art. 34; APPI Art. 26-2; LGPD Art. 48; DPDPA 2023 §13; NIS2 2022/2555; SAMA Cybersecurity Framework.
