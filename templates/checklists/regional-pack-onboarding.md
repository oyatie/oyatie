---
doc_status: published
---

# Checklist: Regional Pack Onboarding

> **When:** New regional pack onboarding (e.g. JP / US / EU / IN / BR / KSA / UAE / AU / SG / etc.) OR existing pack version bump.
> **Owner:** `regional-packs` team + per-pack maintainer.
> **Validator:** `regional-pack-validator` lane + ADR-0010 seam-impl coverage.
> **Template:** [`regional-packs/_TEMPLATE.md`](../regional-packs/_TEMPLATE.md)

---

## 0. Pre-flight

1. ☐ Council ratification of new pack (per [ADR-0010 regional-pack architecture](../decisions/ADR-0010-regional-pack-architecture.md))
2. ☐ Pack id assigned (e.g. `oya-pack-jp`)
3. ☐ Per-pack maintainer hired or assigned
4. ☐ Per-pack catalog record reserved at `registry/catalog/regional-packs/oya-pack-<region>.yaml`
5. ☐ Pack scaffold via `oya pack new <region>` (Foundry skill `oya-regional-pack-author`)

## 1. Regulatory binding

6. ☐ Primary regulator(s) identified (e.g. JP APPI + ISMAP; US HIPAA + FedRAMP + per-state; EU GDPR + DORA + GAIA-X; IN DPDP + RBI + MeitY; BR LGPD + BACEN; KSA PDPL + SAMA; UAE TDRA + DIFC; AU APRA + IRAP; SG MAS + PDPA-SG)
7. ☐ Per-regulator control mapping authored
8. ☐ Per-regulator notification SLA declared
9. ☐ Per-regulator evidence cadence declared
10. ☐ Per-vertical regulator overlay (per-vertical compliance pack)

## 2. Compliance packs (per vertical that applies)

11. ☐ Healthcare regulator binding (per pack: JP PMDA / US FDA / EU EMA / IN CDSCO / BR ANVISA / KSA SFDA / AU TGA / SG HSA)
12. ☐ Fintech regulator binding per [`standards/fintech-compliance.md`](../standards/fintech-compliance.md)
13. ☐ Education regulator binding (children-protection statutes)
14. ☐ Public-sector binding (per-region procurement)
15. ☐ Per-vertical override per [PRIVACY-PROGRAM §2.2.3](../PRIVACY-PROGRAM.md)

## 3. i18n

16. ☐ Languages declared (primary + secondary)
17. ☐ Tokenizer / morphology impl (per `Tokenizer` trait) — pack-supplied
18. ☐ Date / time format
19. ☐ Address normalization impl (per `AddressValidator` trait)
20. ☐ Name conventions
21. ☐ RTL support if applicable
22. ☐ Sort collation per language

## 4. Currency + tax

23. ☐ ISO 4217 currency declared
24. ☐ Decimal precision + display
25. ☐ FX rate source
26. ☐ Tax-invoice format declared (per `TaxInvoiceFormatter` trait)
27. ☐ Tax-id format declared
28. ☐ Tax-engine adapter authored

## 5. Calendar

29. ☐ Per-region holiday list
30. ☐ Working days
31. ☐ Fiscal + school year + business-quarter

## 6. Identity providers

32. ☐ Per-IdP adapter (per `IdentityProvider` trait) for primary national IDs (Login.gov / マイナンバー / eIDAS / Aadhaar / gov.br / Absher / UAE-PASS / myGovID / Singpass / etc.)
33. ☐ Real-name verification adapter
34. ☐ KYC adapter (where applicable)

## 7. Payment rails

35. ☐ Per-rail adapter (per `PaymentRail` trait): national + commercial
36. ☐ Per-PG adapter
37. ☐ Per-region BNPL / wallet adapter

## 8. Address book

38. ☐ Address-validation impl per region
39. ☐ Postal-code format
40. ☐ Geocoding source

## 9. Ecosystem partners

41. ☐ Per-region ecosystem integration adapters (e.g. Yahoo!JP / LINE for JP; Google / Facebook globally; KakaoTalk / Naver / Toss for KR)
42. ☐ Per-region cloud-peer integrations

## 10. Content safety + ad policy

43. ☐ Per-region content moderation rules per `ContentSafetyRules` trait
44. ☐ Per-region ad-policy gate per `LocalAdPolicy` trait
45. ☐ Per-region children/medical/financial/political ad review workflows

## 11. Industry data models

46. ☐ Per-vertical per-locale extension impls (e.g. healthcare clinical coding; labor classifications; accounting standards)

## 12. Vendor partners

47. ☐ Per-region vendor ledger entries (cloud peer / payment / identity / SI partner)

## 13. Per-pack residency

48. ☐ Default residency class declared
49. ☐ Cross-border transfer constraints declared
50. ☐ Per-region DC location list

## 14. CI + validation

51. ☐ CI lane `oya-governance-regional-pack` passes
52. ☐ Per-seam impl coverage 100%
53. ☐ Per-pack test fixtures (per language / per regulator)
54. ☐ Per-pack semver declared
55. ☐ Cosign-signed per ADR-0039

## 15. Operational

56. ☐ Per-pack regulator-watch lane fires on regulatory feed updates
57. ☐ Per-pack on-call rotation declared
58. ☐ Per-pack runbook authored (e.g. `cross-axis/regional-pack-regulator-update.md`)

## 16. Sign-off + first tenant

59. ☐ Per-pack maintainer + `regional-packs` lead + council architecture sign-off
60. ☐ First tenant onboarded with new pack (design partner)
61. ☐ Per-tenant DPIA completed
62. ☐ Trust-portal per-pack entry live
63. ☐ Pack promoted from draft → preview

## 17. Sources

[ADR-0010 regional-pack architecture](../decisions/ADR-0010-regional-pack-architecture.md), [INTERNATIONALIZATION.md](../INTERNATIONALIZATION.md), [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md), [`regional-packs/_TEMPLATE.md`](../regional-packs/_TEMPLATE.md), [`regional-packs/oya-pack-kr/PACK.md`](../regional-packs/oya-pack-kr/PACK.md) (reference example).
