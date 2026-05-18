---
doc_class: Runbook
template_id: TPL-RUNBOOK
title: Anonymity-Leak Incident Response (P0)
microservice: anonymous
severity: "Sev-0 / Sev-1 (the µservice's defining incident class)"
status: Accepted
owner_team: ops-security + axis-anonymous + general-counsel + council-privacy + executive-on-call
date: 2026-05-17
related_adrs: [ADR-ANON-0001, ADR-ANON-0002, ADR-ANON-0003]
related_artifacts:
  - microservices/anonymous/PRD.md I1, I2
  - microservices/anonymous/threat-model.md
  - microservices/anonymous/dpia.md
  - microservices/anonymous/incident-response.md
doc_status: published
---

# Runbook: Anonymity-Leak Incident Response (P0)

## Purpose

An anonymity leak — any event in which the platform structurally fails to honour invariant I1 (user_id ↔ post_id non-correlatability) or I2 (affinity-attestation reveals identity) — is the defining P0 incident class of this µservice. This runbook is invoked the moment such an event is detected, suspected, or alleged.

**A potential anonymity leak ALWAYS triggers Sev-1 immediately. Investigation upgrades to Sev-0 if confirmed.**

## Trigger

| Signal | Severity |
|---|---|
| Postgres `posts` table query returns `user_id` (column-presence) | Sev-0 |
| Audit-chain log contains a `user_id ↔ post_id` correlation outside legal-process workflow | Sev-0 |
| External researcher / journalist publishes claim that the platform de-anonymises users | Sev-1 (investigate immediately) |
| Internal employee discovers a code path that could correlate | Sev-1 (investigate, fix preventively) |
| Classifier verdict log leaks a user-identifying attribute | Sev-1 |
| Push notification payload contains a user-identifying attribute (real name, email, etc.) | Sev-1 |
| Affinity attestation flow leaks credential identity attributes to platform | Sev-1 |
| OIDC blinding proxy retains OIDC subject claim beyond ceremony | Sev-1 |
| Suspected compromise of blind-signature private key | Sev-0 |

## Severity definitions

- **Sev-0**: Confirmed structural anonymity leak; users de-anonymised at scale; regulatory notification mandatory; tenant notification mandatory; executive escalation.
- **Sev-1**: Suspected / partial / preventable anonymity leak; immediate investigation; fix-forward; regulatory notification may be required pending investigation.

## Immediate response (within 15 minutes of detection)

| Step | Action | Owner |
|---|---|---|
| 1 | Declare Sev-1 in `#inc-<id>`; page IC + executive-on-call + general-counsel | ops-security |
| 2 | Halt the suspected leaking code path: `cargo run -p oya-dev-cli -- anonymous halt-component --component <X> --reason "anonymity-leak-investigation"` | ops-security |
| 3 | Snapshot relevant Postgres + Redis + Meilisearch + audit-chain state for forensic analysis | ops-data |
| 4 | Begin investigation; collect evidence in `evidence/anonymity-leak/<incident-id>/` | IC |
| 5 | Notify Council Privacy chair + Council Architecture chair | IC |
| 6 | If a public claim has been made (researcher / journalist): coordinate with comms + general-counsel | exec-on-call |

## Investigation (next 4 hours)

| Step | Action | Owner |
|---|---|---|
| 1 | Identify the suspected leak vector (column-presence, log content, classifier verdict log, push payload, etc.) | axis-anonymous + ops-security |
| 2 | Confirm: is the leak (a) hypothetical (code path exists but no real leak), (b) sample-scale (single user affected), or (c) systemic (many users affected)? | axis-anonymous |
| 3 | If systemic: assess data-class of leaked attributes (PII_IDENTIFYING / BEHAVIORAL_USER_CONTENT / etc.) | council-privacy |
| 4 | Determine GDPR Art. 33 + KR PIPA Art. 34 + applicable regulatory notification clocks | general-counsel |
| 5 | Determine affected tenants + member counts | axis-anonymous |
| 6 | Prepare regulatory + tenant + user communication drafts | general-counsel + comms |

## Containment + remediation (next 24 hours)

| Step | Action | Owner |
|---|---|---|
| 1 | If structural code-path defect: deploy fix that closes the path; verify via test | axis-anonymous |
| 2 | If column-presence defect (Postgres `posts.user_id`): emergency schema migration + audit-chain seal | ops-data |
| 3 | If classifier log leak: rotate classifier log retention; audit-chain seal the rotation | axis-foundry-runtime |
| 4 | If push payload leak: roll back push template; audit-chain seal | axis-anonymous |
| 5 | If blind-signature key compromise: ceremony to rotate; per `blind-signature-key-ceremony.md` | ops-security |
| 6 | Verify the fix: re-run the I1 + I2 invariant tests; verify all return green | axis-anonymous |
| 7 | Audit-chain seal the remediation | – |

## Regulatory notification clocks

| Regulator | Clock | Trigger |
|---|---|---|
| GDPR (Art. 33) — supervisory authority | 72 hours from discovery | personal-data breach |
| GDPR (Art. 34) — affected individuals | "without undue delay" when high-risk | high-risk personal-data breach |
| KR PIPA (Art. 34) — PIPC | 24-72 hours (depending on category) | personal-data breach |
| UK ICO | 72 hours | personal-data breach |
| EU DSA — Commission (Art. 27) | quarterly transparency report (regular cadence); incident-specific cadence per Member State | systemic risk |
| US — state DPAs (e.g., CA AG Office) | per state law (often "without unreasonable delay") | per state law |
| US — affected consumers | per state law | per state law |
| HIPAA OCR (if PHI involved) | 60 days | per 45 CFR §164.404 |

**Default posture**: notify within the strictest applicable clock. If unsure, notify.

## Tenant + user notification

Within 24 hours of confirmed Sev-0:

1. Tenant operators receive a structured notification template (severity + leak vector + remediation + tenant-side action required).
2. Affected end-users (where identifiable through tenant directory) receive a notification through tenant.
3. Public status-page is updated.
4. If a public claim was made by a researcher / journalist, comms publishes a response.

## Post-mortem

Within 5 business days:

1. Root cause analysis written in `evidence/anonymity-leak/<incident-id>/postmortem.md`.
2. Council Privacy + Council Architecture review.
3. Action items: every action item maps to a specific code change, policy change, or CI lane (LEAN lane addition).
4. Where applicable, an ADR-ANON-* supersession is filed (e.g., if the cryptographic protocol selected in ADR-ANON-0001 proved inadequate, a superseding ADR is required).

## Cedar policy enforcement

This runbook does NOT bypass Cedar policy. Even during incident response, every action taken is gated by Cedar; emergency operations require ops-security + general-counsel dual-control per `policy/legal-process-disclosure.cedar` PERMIT 2 (re-purposed for emergency incident response).

## References

- ADR-ANON-0001 (cryptographic-blinding protocol)
- ADR-ANON-0002 (affinity-attestation verification)
- ADR-ANON-0003 (legal-process workflow)
- GDPR Arts. 33, 34
- KR PIPA Art. 34
- UK ICO breach-notification guidance
- HIPAA 45 CFR §164.404 (breach notification)
- Bominal ADR-0028 (audit-chain Merkle / Ed25519)
- `microservices/anonymous/incident-response.md` (cross-incident framework)
- `microservices/anonymous/threat-model.md`
