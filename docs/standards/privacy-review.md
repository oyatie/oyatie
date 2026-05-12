# Oyatie — Privacy Review Standard

> **Owner:** `council-privacy`.
> **Companion:** [PRIVACY-PROGRAM.md](../PRIVACY-PROGRAM.md), ADR-0008 (Data Use Boundary), [`templates/dpia-template.md`](../templates/dpia-template.md).

## 1. When privacy review is required

| Change class | Reviewer | Trigger |
|---|---|---|
| New data class introduced | `council-privacy` | ADR-0008 amendment required |
| Per-class allowlist change | `council-privacy` | per consent gate |
| Cross-axis data flow new contract | `council-privacy` co-sign | per DESIGN §10 + ADR-0011 |
| Per-tenant consent surface change | `council-privacy` | per ADR-0008 §2.2.2 purpose-permission matrix |
| New regulatory pack overlay | `council-privacy` + `regional-packs` | per ADR-0010 |
| New Foundry capability touching tenant data | `council-privacy` co-sign | per ADR-0021 + ADR-0008 |
| Per-vertical override change | `council-privacy` + per-vertical | per ADR-0034 |
| DSR cascade pipeline change | `council-privacy` | per ADR-0038 |
| Audit-chain emission pattern change | `council-privacy` co-sign | per ADR-0003 |
| Per-region residency change | `council-privacy` + `regional-packs` | per ADR-0049 |

## 2. Per-PR checklist

1. ☐ Data classes touched annotated (every field) per ADR-0008
2. ☐ Subject attributes verified (per ADR-0008 orthogonal)
3. ☐ Purpose-binding verified
4. ☐ Consent-receipt path verified
5. ☐ Per-tenant cell-isolation verified per ADR-0009
6. ☐ Per-vertical override applied per ADR-0034
7. ☐ DSR cascade integration tested per ADR-0038
8. ☐ Audit-chain emission verified per ADR-0003
9. ☐ Inference-boundary check passes (derived attrs inherit most-restrictive class)
10. ☐ Per-pack regulatory binding verified per ADR-0010

## 3. DPIA triggers

- New capability invokable on tenant data
- New vertical onboarded
- New regional pack onboarded
- New cross-axis data flow
- Per-regulator expectation (GDPR Art 35; PIPA equivalent; LGPD Art 38)
- Per-incident postmortem identifies gap

DPIA per [`templates/dpia-template.md`](../templates/dpia-template.md). Approved by `council-privacy`.

## 4. Per-tenant communication

- New data class collected → tenant admin notified
- Consent surface change → per-tenant re-consent flow
- DSR processing → per-DSR receipt + completion notification
- Cross-border transfer → per-tenant explicit consent + audit-emit

## 5. Bypass policy

- NEVER bypass privacy review for: new data class; cross-axis flow; cross-region flow; new tenant-class override; DSR pipeline change; consent surface change; per-vertical override
- Bypass acceptable: cosmetic doc; internal-tools UI; non-product analytics on aggregate-only data with k-anonymity verified

## 6. Sources
[PRIVACY-PROGRAM.md](../PRIVACY-PROGRAM.md), ADR-0003/0007/0008/0009/0010/0011/0021/0022/0034/0038/0049, KR-PIPA, GDPR, HIPAA, PCI-DSS, LGPD.
