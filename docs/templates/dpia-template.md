# DPIA: <Capability / Surface / Vertical> — <YYYY-MM-DD>

> **Owner:** `council-privacy` co-author + per-axis team
> **Triggered by:** <new capability / new vertical / new regional pack / regulator change / incident postmortem>
> **Regulatory anchor:** GDPR Art 35 / KR PIPA / HIPAA / per-pack
> **Status:** draft / in-review / approved / archived
> **Companion:** [PRIVACY-PROGRAM.md](../PRIVACY-PROGRAM.md), ADR-0008 (Data Use Boundary)

---

## 1. Subject of the assessment

What capability / surface / vertical / pack is under DPIA. One paragraph.

## 2. Purpose + lawful basis

| Purpose | Lawful basis | Subject class | Geography |
|---|---|---|---|
| (per [PRIVACY-PROGRAM §2.2.2](../PRIVACY-PROGRAM.md) purpose enumeration) | (consent / contract / legal-obligation / vital-interest / public-task / legitimate-interest) | (adult / minor / vulnerable / etc.) | (per regional pack) |

## 3. Data classes touched

Per ADR-0008 12-class taxonomy — list every class involved:

| Class | Source | Destination | Retention | Notes |
|---|---|---|---|---|
| (e.g. PII_QUASI_IDENTIFIER) | (where collected) | (per-axis store) | (per retention policy) | (per consent receipt) |

## 4. Subject + capacity attributes

Per ADR-0008 orthogonal `subject_class`:

- subject_class: <adult / minor / vulnerable / public>
- minor_status: (if applicable)
- jurisdiction: (per pack)
- residency: (per pack)
- lawful_basis: (per row 2)
- purpose: (per row 2)
- derivation_lineage: (per inference-boundary check)
- consent_receipt_id: (per consent surface)

## 5. Cross-axis flow map

Which axes does data move through? Per [DESIGN.md §10](../DESIGN.md) cross-axis contract surface:

```
[source axis] → [contract] → [destination axis] → [contract] → ...
```

Confirm every cross-axis flow has a Data Use Boundary class allowlist.

## 6. Risks

| # | Risk | Likelihood | Severity | Pre-mitigation score | Mitigation | Post-mitigation residual |
|---|---|---|---|---|---|---|
| 1 | (cross-tenant leak via cache) | low | catastrophic | high | per-tenant cache key + invalidation cascade | low |
| 2 | (DSR cascade misses derived feature) | med | high | high | inference-boundary lineage check per §2.2.5 | low |
| ... | | | | | | |

## 7. Tenant-class override

Per [PRIVACY-PROGRAM §2.2.3](../PRIVACY-PROGRAM.md):
- Healthcare tenant override applied? (PHI / PII / Sensitive-PIPA-Art23 hard-deny)
- Fintech tenant override applied? (PCI / Financial-KR-신용정보 hard-deny)
- Education-K12 override applied? (CHILDREN_UNDER_14 hard-deny)
- Public-sector override applied?
- Defense override applied? (anti-scope unless founder ratifies)

## 8. Audit-chain emission contract

| Operation | Event topic | Required fields |
|---|---|---|
| (per ADR-0003) | (per axis) | tenant_id, subject_attributes, data_classes_touched, consent_receipt_id |

## 9. DSR cascade integration

How does DSR cascade per [PRIVACY-PROGRAM §2.2.9](../PRIVACY-PROGRAM.md) reach this capability/surface/vertical?

- Per-store impact list
- Cascade SLA satisfied (30d preview / 14d stable / 7d GA)
- Proof-of-erasure record path

## 10. Foundry agent integration

Per ADR-0022 autonomy ceiling:
- Autonomy tier required: T1 / T2 / T3 / T4
- Per-tenant per-capability cap inherited
- Agent step audit-emission verified

## 11. Per-pack regulatory binding

Per [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md):
- Primary regulator: (e.g. KR PIPC + KISA / EU SA / US HHS)
- Per-jurisdiction overlay
- Notification SLA: (per regulator)
- Evidence cadence: (continuous / weekly / quarterly)

## 12. Council sign-off

- ☐ Privacy council lead
- ☐ Per-axis owning team lead
- ☐ Per-pack maintainer (where applicable)
- ☐ Founder (for catastrophic-class capabilities or new tenant class)

## 13. Open questions

(numbered list)

## 14. Sources scanned

- ADR-0008 (Data Use Boundary)
- [PRIVACY-PROGRAM.md](../PRIVACY-PROGRAM.md)
- [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md)
- KR PIPA / GDPR / HIPAA / PCI-DSS / per-pack statutes
- Per-tenant DPIA history
- Per-incident postmortems where applicable

*Sources footer regenerated whenever this DPIA is amended.*
