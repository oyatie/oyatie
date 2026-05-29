---
doc_class: Runbook
title: PII leak incident (P0 — GDPR Art. 33 / HIPAA §164.404 / KR PIPA Art. 34)
microservice: forms
severity: "Sev-1 (P0)"
status: Accepted
owner_team: ops-sre-reliability + ops-security + council-privacy + council-legal-compliance + axis-forms
date: 2026-05-17
related_artifacts:
  - microservices/forms/threat-model.md
  - microservices/forms/dpia.md
  - microservices/forms/compliance.md
  - microservices/forms/incident-response.md
  - microservices/forms/failure-modes.md FM-06
doc_status: published
---

# Runbook: PII Leak Incident (P0)

## Purpose

A confirmed-or-suspected PII / PHI / Art. 9 special-category data exposure originating from Forms. This is the highest-severity runbook. Time-to-notify clocks start at first confirmed indicator.

## Trigger

ONE of:

1. **`oya_forms_export_pii_unredacted_total > 0`** — redaction bypass on export.
2. **`oya_forms_cross_tenant_pii_read_total > 0`** — Citus RLS / Cedar bypass.
3. **External report**: external party reports access to PII they should not have.
4. **Internal discovery**: engineer / auditor identifies leak vector during review.
5. **Confirmed XSS / embed exploit (cross-reference `embed-iframe-csp-incident.md` Path C)**.

## Severity

Sev-1 (P0). Privacy team always paged; legal-compliance always paged.

## Notification clocks (start at first confirmed indicator)

| Pack | Statute | Notify supervisory authority | Notify subjects | Notify tenant |
|---|---|---|---|---|
| pack-eu | GDPR Arts. 33-34 | ≤ 72h | "without undue delay" if high-risk | ASAP |
| pack-us-healthcare | HIPAA §164.404 | ≤ 60d (HHS) | ≤ 60d | ASAP |
| pack-kr | PIPA Art. 34 + PIPC notification | ≤ 72h (PIPC) | ≤ 72h | ASAP |
| pack-jp | APPI Art. 26 | "promptly" | "promptly" | ASAP |
| pack-au | Privacy Act Part IIIC | ≤ 30d (OAIC) | "as soon as practicable" | ASAP |
| pack-in | DPDPA 2023 §8(6) | ≤ ASAP (DPB) | ≤ ASAP | ASAP |
| pack-br | LGPD Art. 48 | "reasonable time" (ANPD) | "reasonable time" | ASAP |
| pack-ae | UAE PDPL | ≤ 72h | per DPA | ASAP |
| pack-ksa | KSA PDPL | ≤ 72h (NCA) | per regulation | ASAP |
| pack-us / pack-sg | state laws (CCPA / PDPA) + tenant DPA | per statute | per statute | ASAP |

## Phase 1 — STOP THE BLEED (within 15 min)

1. **Declare Sev-1.** Page: ops-sre + ops-security + council-privacy + council-legal-compliance + axis-forms.
2. **Identify scope**: which tenants, which subjects, which data classes?
3. **Quarantine**: lock affected forms (`cargo run -p oya-dev-cli -- forms publish-block --form <id> --reason p0`); revoke active export tokens (`cargo run -p oya-dev-cli -- forms export-token-revoke --tenant <id> --all`).
4. **Preserve evidence**: Postgres + audit-chain + logs snapshot before any restoration.
5. **Status page**: internal incident channel only at this stage; NO public statement yet.

## Phase 2 — FORENSICS (within 4h)

1. **Root cause**: how did PII escape?
   - Application bug (e.g., redaction missed a column)?
   - Configuration drift (e.g., Cedar policy misapplied)?
   - Insider (e.g., DBA read with DEK access — should be impossible per ADR-FORMS-0003)?
   - External exploit (e.g., XSS in renderer, embed exploit, SQL injection)?
2. **Audit-chain replay**: identify every PII row accessed during the leak window.
3. **Subject identification**: hash-resolve subject_hash → identifiers via DSR runner reverse-lookup (where statutorily permitted).
4. **Data class breakdown**: PII_IDENTIFYING vs SENSITIVE_GDPR_ART9 vs PHI vs FINANCIAL?
5. **Cross-product blast radius**: did leak propagate via sheets-bridge / workflow-trigger / webhook / bulk-distribute?

## Phase 3 — NOTIFY (per pack clock)

1. **Tenant notification**: phone + email per tenant DPA.
2. **Supervisory authority**: per pack table above. Use templates at `legal/incidents/breach-notification-templates/`.
3. **Subject notification**: per statute; use templates at `legal/incidents/subject-notification-templates/`.
4. **Public statement** (if required): coordinated with council-legal-compliance + gtm.
5. **Coordinated disclosure**: if external researcher reported, follow `legal/vulnerability-disclosure-policy.md`.

## Phase 4 — REMEDIATE (parallel to Phase 3)

1. **Patch the leak vector**: code change + Cedar policy change + redaction fix.
2. **Forced credential rotation** if leak via credential compromise.
3. **DEK rotation** if leak via DEK exposure (per ADR-FORMS-0003 quarterly cadence, accelerated).
4. **Affected response purge** if statutorily required (per DSR cascade).
5. **Audit-chain re-seal** of remediated state.

## Phase 5 — POSTMORTEM (within 5 business days)

1. Blameless postmortem template (`evidence/incidents/<id>.md`).
2. ADR for control gap.
3. Pen-test successor-IP scheduled.
4. Per-pack DPO sign-off.

## Phase 6 — REGULATORY FOLLOW-UP (per statute)

- GDPR Art. 33 documentation of breach + decisions.
- HIPAA OCR if PHI: per §164.404 / §164.408.
- PIPC report per Korean form.
- ANPD report per LGPD.
- DPB report per DPDPA.
- Subject-rights requests likely to spike; DSR runner pre-scaled.

## Invariants

- **NEVER hide a breach** — notification clocks start at first confirmed indicator, not "convenient time".
- **NEVER restore from a backup that includes the leak vector** — verify backup is from pre-vector point.
- **NEVER over-promise to subjects** — coordinate language with council-legal-compliance.
- **NEVER allow the leaking endpoint to remain live** during Phase 1-4 — quarantine first; restore last.

## References

- `compliance.md` §"1. GDPR" + §"2. KR PIPA" + §"3. HIPAA".
- `dpia.md`.
- `threat-model.md`.
- `incident-response.md` §"Sev-1 protocol".
- ADR-FORMS-0003 PII column encryption.
- GDPR Arts. 33-34; HIPAA §164.404 + §164.408; KR PIPA Art. 34; LGPD Art. 48; UAE PDPL; KSA PDPL; APPI Art. 26; DPDPA 2023 §8(6).
- ICO breach-notification guidance — `ico.org.uk/`.
- HHS OCR breach-reporting portal — `ocrportal.hhs.gov/ocr/breach/`.
- PIPC notification portal.
