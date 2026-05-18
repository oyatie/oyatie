---
doc_class: LegalRegister
title: Cross-Border Transfer Register
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance
doc_status: published
---

# Cross-Border Transfer Register (recordings µservice)

Per GDPR Art. 30 + ADR-RECORDINGS-0002 + `policy/data-residency.md`, this
register tracks every approved cross-border transfer of recording data.

## Default posture

Cross-pack transfer **forbidden by default**. Every entry below represents
a tenant-specific approved exception.

## Approved transfers

| Tenant | Source pack | Target pack | Mechanism | Approved by | Date | Re-review |
|---|---|---|---|---|---|---|
| _(none active at 2026-05-17)_ | | | | | | |

## Approval procedure

1. Tenant files transfer request via ops portal.
2. council-privacy + ops-compliance review per pack rules:
   - GDPR Arts. 44-50: SCC + DPA on file.
   - HIPAA: BAA + HIPAA-eligible source + target.
   - SEC 17a-4: target storage must support 17a-4 WORM.
3. Cedar policy + Helm overlay updated.
4. Audit-chain seal of the approval event.
5. Entry added to this register; annual re-review.

## Disallowed transfers (catalogued)

- pack-kr → any: KR PIPA Art. 17 + 통신비밀보호법 + KR PIPC posture forbids
  recording transfer offshore without explicit tenant + DPA approval.
- pack-ksa → any: PDPL data-localisation strictly enforced.
- pack-us-healthcare → non-HIPAA-eligible: HIPAA Safe Harbor §164.502.

## References

- GDPR Arts. 44-50.
- HIPAA 45 CFR §164.502.
- SEC Rule 17a-4(f).
- KR PIPA Art. 17 + 통신비밀보호법.
- ADR-RECORDINGS-0002.
- `policy/data-residency.md`.
