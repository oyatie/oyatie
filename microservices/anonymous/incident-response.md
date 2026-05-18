---
doc_class: IncidentResponse
template_id: TPL-INCIDENT-RESPONSE
microservice: anonymous
status: Accepted
date: 2026-05-17
owner_team: ops-security + axis-anonymous
related_adrs: [ADR-ANON-0003, ADR-ANON-0006]
review_cadence: quarterly + post-incident
doc_status: published
---

# Incident Response: anonymous µservice

## Severity ladder

| Severity | Definition | Trigger examples | Page |
|---|---|---|---|
| **P0** | Anonymity-leak class — platform's structural privacy promise is broken or at imminent risk of being broken | DB JOIN executed without legal-process Cedar; blind-signature key compromise; affinity-attestation key compromise; cross-tenant data exposure; analytics tracker SDK discovered in client; insider de-anonymization attempt detected | ops-security on-call + axis-anonymous on-call + council-privacy + legal counsel + CISO; PagerDuty Sev-1 + Slack #incident-anonymous |
| **P1** | Legal-process disclosure correctness — chain-of-custody at risk, transparency-report omission, NCMEC reporting missed SLA | Legal-process disclosure executed without dual-control; transparency-report misses cadence; NCMEC CyberTipline exceeds 48h SLA on confirmed CSAM | ops-security + legal + axis-anonymous; PagerDuty Sev-2 |
| **P2** | Service degradation impacting SLO | Feed-render p99 > 250ms sustained > 15min; post-create error rate > 1%; hard-delete propagation > 5s sustained > 15min | axis-anonymous on-call; PagerDuty Sev-3 |
| **P3** | Capacity / operational concerns | Storage quota nearing 80%; vote-counter Valkey nearing eviction threshold | axis-anonymous business-hours |

## P0 — Anonymity-leak incident

The defining incident class for this µservice. Mandatory drill quarterly.

### Detection signals

- Prometheus alert: `oya_anonymous_db_join_without_legal_process_total > 0` over 1 min
- Prometheus alert: `oya_anonymous_personal_tier_federation_attempt_total > 0` (refused at type level, but defence-in-depth metric)
- Audit-chain anomaly: `legal_process_disclosure_executed` count without corresponding `legal_process_disclosure_approved` paired event
- Third-party tracker detection in client bundle scan
- External report (responsible disclosure / bug bounty / user report)
- DB-access anomaly: `psql` direct connection from non-allowlisted source

### Immediate response (< 15 min)

1. Convene incident bridge (ops-security on-call + axis-anonymous on-call + council-privacy lead + legal counsel + CISO).
2. **Containment first**: if anomaly is currently in-flight (e.g., active DB direct-access session), revoke session at OpenBao + Postgres immediately.
3. **Preserve evidence**: snapshot Postgres + audit-chain + access logs immediately; chain-of-custody recorded.
4. Identify blast radius: how many records / users potentially affected?
5. Decide on tenant + user notification path (varies per pack — see Notification matrix below).

### Investigation (< 4h)

1. Reconstruct the actor + action chain via audit-chain + DB audit logs.
2. Determine whether disclosure exfiltration occurred (data egressed) or just access (data accessed but not egressed).
3. Document timeline + reproducibility.

### Notification matrix (per pack)

| Pack | Authority | Timeline | Format |
|---|---|---|---|
| pack-eu | Lead supervisory authority (DPA of LMA) | 72h (GDPR Art. 33) | per Art. 33(3) elements |
| pack-kr | PIPC (Personal Information Protection Commission) | 24h (KR PIPA Art. 29) | per Art. 29 elements |
| pack-uk | ICO | 72h (UK GDPR Art. 33) | per Art. 33 |
| pack-us | State AG (per state breach notification laws — varies) + affected users where required | varies (CA: 60 days max for users; some states 30 days) | per state |
| pack-au | OAIC (Privacy Act Notifiable Data Breaches scheme) | 30 days | per scheme |
| pack-jp | PPC | 3 working days (APPI Art. 26) | per Art. 26 |
| pack-sg | PDPC | 72h (PDPA §26B amended 2020) | per regulation |
| pack-in | DPB (DPDPA 2023) | 72h | per regulation |
| pack-br | ANPD | "reasonable time" (LGPD Art. 48) — operational target 72h | per regulation |
| pack-ae | UAE Data Office | per UAE PDPL — operational target 72h | per regulation |
| pack-ksa | SDAIA | per PDPL — operational target 72h | per regulation |

User notification per pack also required where individual rights affected; framing must NOT itself de-anonymize affected users.

### Remediation

1. Root-cause fix landed via expedited ADR-gated change.
2. Compensating control if root-cause complex (e.g., temporary DB GRANT lockdown).
3. Post-mortem published internally within 14 days; public version where required.

### Post-incident

1. Audit-chain forensic export to legal counsel.
2. SOC 2 evidence captured for CC7.4 incident management.
3. Update threat-model.md if new threat vector discovered.
4. Add LEAN lane if structural prevention possible.

## P1 — Legal-process disclosure incident

Triggered when:
- A legal-process disclosure was executed without dual-control approval recorded
- Transparency-report cadence missed
- NCMEC CyberTipline > 48h SLA on confirmed CSAM-suspect

### Response

1. Convene legal + ops-security + axis-anonymous.
2. Preserve evidence (audit-chain export + court-order documents).
3. Notify affected user(s) per 14-day window unless gag-ordered.
4. Adjust transparency-report numbers; if quarter-cutoff missed, supplement next report with note.
5. NCMEC: file immediately even if SLA already exceeded; document delay in audit-chain.

## P2 — Service degradation

Standard SLO-based response. See runbooks for specific paths.

## P3 — Capacity / operational

Business-hours response. Monitoring + capacity-plan adjustment.

## Communication

- Internal: #incident-anonymous Slack channel; bridge dial-in.
- External: status.oyatie.dev per-pack page.
- Customer: per-tenant TAM channel.
- Regulator: per Notification matrix above.

## Tabletop schedule

- Quarterly P0 anonymity-leak tabletop (rotating pack lead).
- Bi-annually legal-process disclosure tabletop (rotating pack jurisdiction).
- Annually blind-signature + affinity-attestation key compromise tabletop.

## Post-incident review template

(See `incident-template.md` referenced from oya-shared-incident-template; standard format applied.)
