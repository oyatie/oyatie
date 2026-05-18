---
microservice: compliance
doc: DPIA
status: Drafting
authority_tier: 2
owner: axis-compliance
co_owners: [axis-security]
date: 2026-05-18
related_adrs: [ADR-0209]
---

# Compliance — Data Protection Impact Assessment (GDPR Art. 35)

## Scope

The compliance µservice processes personal data on behalf of multiple controller tenants. This DPIA covers the µservice's own processing operations; per-tenant DPIAs remain the tenant's responsibility.

## Processing activities

1. **DSAR pipeline** — accepts subject requests, walks Ontology projection across µservices, produces export / executes deletion / applies rectification.
2. **HIPAA minimum-necessary logs** — receives PHI access events; stores subject pseudonym + accessor identity + purpose.
3. **Audit chain seal verification** — reads artifact metadata (may include subject pseudonyms in compliance-audit context).
4. **Manual evidence upload** — receives pen-test reports / BAA inventory; pen-test reports may reference subject pseudonyms.

## Personal data categories processed

| Category | Source | Retention |
|---|---|---|
| Subject pseudonym (random) | Ontology projection input | per IP-009 |
| Subject contact (for DSAR delivery) | DSAR request submission | 30 days post-completion |
| Accessor SPIFFE-ID | minimum-necessary log emit | 6 years (HIPAA) |
| Cedar policy decision | minimum-necessary log emit | 6 years (HIPAA) |
| Subject DOB / zip / gender (for k-anonymity validation) | DSAR export pipeline | not stored; transient during export |

## Lawful basis

- **DSAR** — Art. 6(1)(c) legal obligation (Art. 12 statutory).
- **HIPAA min-necessary logs** — Art. 6(1)(c) + Art. 6(1)(f) legitimate interests (compliance audit).
- **Evidence retention** — Art. 6(1)(c) (statutory retention per HIPAA / SOC 2).

## Risks identified + mitigation

1. **Cross-tenant data leak via DSAR** — see threat-model.md A2; kernel invariant + 5-layer guard.
2. **PII over-collection in pseudonym pipeline** — IP-008 scrubber + k-anonymity validator.
3. **Audit-log integrity loss** — cosign keyless OIDC + cold-tier re-seal (IP-005 + IP-006).
4. **Auditor over-access** — per-engagement Cedar role binding + engagement-end revoke (IP-007).
5. **Replay attack** — replay banner + original-seal-hex (IP-012).
6. **Anomaly detector false positives** — per-accessor baseline calibration (IP-013).

## Necessity + proportionality test

Each artifact-kind in `compliance.md` mapping is justified by an explicit AICPA Trust Services Criterion or statutory requirement. No personal data is processed beyond what compliance frameworks require.

## Data subject rights

- **Art. 15 access:** subject can request DSAR export.
- **Art. 16 rectification:** subject can request field update.
- **Art. 17 erasure:** subject can request deletion; non-subject-linked SOC 2 evidence retained.
- **Art. 20 portability:** export format is JSON-LD.
- **Art. 21 objection:** out-of-band (privacy@oyatie); routes to compliance officer.

## DPIA review cadence

- Annual review by axis-compliance + axis-security.
- Triggered re-review on any change to PRD goals + IP roster + retention table.

## References

- GDPR Art. 35 — Data Protection Impact Assessment.
- ADR-0209 — substrate authority.
- threat-model.md — security threat model.
- compliance.md — regulatory framework mapping.
