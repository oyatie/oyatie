---
doc_class: Runbook
title: Age-gate bypass incident
microservice: shorts
severity: "Sev-1 (child-protection breach; COPPA/GDPR Art. 8/KR 청소년 보호법 clock)"
status: Accepted
owner_team: axis-shorts + council-privacy + ops-legal + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/shorts/failure-modes.md (FM-14)
  - microservices/shorts/threat-model.md (T-I-09)
  - microservices/shorts/dpia.md (R-07)
  - microservices/shorts/decisions/ADR-SHORTS-0006-minor-protection-and-age-gate.md
doc_status: published
---

# Runbook: Age-gate bypass incident (FM-14)

## Trigger

- `oya_shorts_minor_protection_bypass_attempt_total` > 0.
- Tenant report: minor account accessed age-restricted content.
- External report: child-protection authority contact (KR 청소년 보호법 / COPPA / UK Ofcom / AU eSafety / EU DSA Coordinator).
- Pen-test finding: age-gate bypassed via API misuse.
- ops-legal page: regulatory escalation.

## Severity

Sev-1 default. Triggers:
- GDPR Art. 8 (child consent) clock — 72h Art. 33 + best-effort Art. 34.
- KR PIPA Art. 34 — 24h to PIPC + 72h to subjects.
- COPPA 15 USC §6501 — best-effort + FTC notification.
- KR 청소년 보호법 — KCC notification per applicable article.
- UK Online Safety Act 2023 — Ofcom communication.
- CA AB-2273 + UT SMRA — per-state notification.
- EU DSA Art. 28 + EU AVMSD Art. 28b — DSA Coordinator + AVMSD coordinator engagement.

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Sev-1 declared; war-room with council-privacy + ops-legal + axis-shorts + ops-security | ≤ 5 min |
| 2 | Identify scope: which minor accounts, which protections bypassed, which content surfaced | ≤ 10 min |
| 3 | Cordon affected accounts: force-revert to chronological-only + algorithmic-opt-out + DM-restricted | ≤ 10 min |
| 4 | If specific bypass path identified: hotfix to refuse the path; emit Sev-1 metric for each refused attempt | ≤ 30 min |
| 5 | If age-classification mis-labelled content surfaced to minors: cordon those videos pending re-classification | ≤ 20 min |
| 6 | Audit-chain query: list every minor-touch event in affected window | ≤ 30 min |
| 7 | council-privacy: GDPR Art. 8 + COPPA + KR 청소년 보호법 + CA AB-2273 + UT SMRA evaluation | ≤ 1h |

## Per-Pack Regulatory Notification

| Pack | Authority | Deadline | Owner |
|---|---|---|---|
| pack-kr | PIPC + KCC | 24h to PIPC; 72h to subjects | council-privacy + ops-legal |
| pack-eu | Lead DPA (DPC IE for VLOP-track) + DSA Coordinator | 72h GDPR Art. 33 | council-privacy + ops-legal |
| pack-us | COPPA: FTC; State: CA Attorney General + UT Division of Consumer Protection | per-jurisdiction | council-privacy + ops-legal |
| pack-uk | UK Ofcom + UK ICO | per Ofcom direction | ops-legal |
| pack-au | AU eSafety Commissioner | per Commissioner direction | ops-legal |
| pack-in | DPDPA Board | per board direction | council-privacy + ops-legal |
| pack-br | ANPD | per LGPD Art. 48 | council-privacy + ops-legal |
| Other packs | per pack-overlay | per pack-overlay | council-privacy + ops-legal |

## Parent Notification (via tenant-of-tenant DPA)

For confirmed bypass affecting minor accounts:
1. Identify parental-link records for affected minors.
2. Per-affected-parent notification through tenant DPA channel.
3. Notification includes: what content was accessed, when, mitigation steps, parent's options (account deletion, controls strengthen).
4. Per ops-legal: notification language reviewed for jurisdictional requirements (COPPA requires "verifiable consent" language; KR 청소년 보호법 requires Korean-language notice).

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Cedar policy misconfig (forbid-minor-content fragment incomplete) | bypass attempts in audit log | review Cedar; pen-test pack-affected; LEAN coverage lane re-run |
| Age-classification mis-labelled content | content tagged general_audience but actually mature | content-moderation classifier re-run on affected videos; re-classify |
| Age-attestation table read by unauthorised actor | `age_verification_reader` entitlement misuse | review entitlement grants; cloud-secrets audit |
| Parental-link table read by unauthorised actor | `parental_link_reader` entitlement misuse | review entitlement grants |
| API parameter manipulation (client-side spoofing of age claim) | gateway logs show age tampering | server-side authority (Cedar + age-gate BC); never trust client |
| Minor account with parental-consent fraud | parent and minor are same person | KYC tightening per pack threshold; ops-legal review |

## Recovery Verification

- `oya_shorts_minor_protection_bypass_attempt_total` rate = 0 for ≥ 7d.
- Cedar fragment for minor protection: 100% coverage on age-classification scenarios.
- Affected minor accounts: all reverted to default chronological-only + algorithmic-opt-out + DM-restricted.
- All confirmed bypass paths: hotfixed + integration test added.
- Regulatory notifications filed per deadlines.
- Parent notifications sent.

## Postmortem (mandatory; ≤ 5 business days)

Required sections:
1. Bypass path technical detail.
2. Scope (minor accounts affected, content accessed, duration).
3. Regulatory notification status per pack.
4. Parental notification status.
5. Root cause (policy misconfig / classifier mis-label / entitlement misuse / API spoofing).
6. Cedar policy revision.
7. Pen-test add to annual cadence covering this bypass.
8. Action items: integration test add; entitlement review cadence; classifier golden-set add.

## References

- `microservices/shorts/failure-modes.md` FM-14.
- `microservices/shorts/threat-model.md` T-I-09.
- `microservices/shorts/dpia.md` R-07.
- `microservices/shorts/decisions/ADR-SHORTS-0006`.
- GDPR Art. 8 + 25 + 33 + 34.
- KR PIPA Art. 8 + 34.
- KR 청소년 보호법.
- COPPA 15 USC §6501.
- CA AB-2273.
- UT Social Media Regulation Act.
- UK Online Safety Act 2023.
- AU Online Safety Act 2021.
- LGPD Arts. 14, 48.
- EU DSA Art. 28.
- EU AVMSD Art. 28b.
