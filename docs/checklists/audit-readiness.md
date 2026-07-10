---
doc_status: published
---

# Checklist: Audit Readiness

> **When:** Per audit cycle (annual + on-demand). Required before any external audit (SOC 2 / ISO 27001 / KR-ISMS-P / CSAP / HIPAA / PCI-DSS / per regulator).
> **Owner:** `ops-compliance` lead.
> **Validator:** `compliance-evidence-recency` + `compliance-matrix-coverage`.

---

## 1. Per-regulator pack confirmation

For each regulator in scope:

1. ☐ Regulator binding confirmed per [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md) §3.X
2. ☐ Per-regulator pack (regional pack overlay) up-to-date per [`regional-packs/`](../regional-packs/)
3. ☐ Per-regulator notification SLA verified
4. ☐ Per-regulator evidence pack regenerable on demand (≤ 4h target)
5. ☐ Per-regulator reporting cadence current

## 2. Control-evidence cadence

Per [COMPLIANCE-MATRIX.md §4](../COMPLIANCE-MATRIX.md) continuous control monitoring (per ADR-0024 + Issue #954):

6. ☐ Per-control evidence regenerated within declared cadence (continuous / daily / weekly / monthly / quarterly / annual)
7. ☐ Per-tenant evidence-pack regenerable
8. ☐ Per-vertical evidence-pack regenerable
9. ☐ Per-pack evidence-pack regenerable
10. ☐ No stale evidence (older than its cadence)

## 3. Audit chain integrity

Per ADR-0003:

11. ☐ Audit-chain integrity check passed daily for the audit window
12. ☐ Chain-replay drill passed in last 90 days
13. ☐ Per-tenant audit shard accessible
14. ☐ No emission gaps for regulated capability invocations

## 4. DSR + consent

Per ADR-0008 + ADR-0038:

15. ☐ DSR queue current (no over-SLA items)
16. ☐ DSR cascade tested in last 90d
17. ☐ Per-tenant consent-receipt audit
18. ☐ Per-class consent withdrawal cascade verified

## 5. Trust portal

Per [DOCUMENTATION.md §3](../DOCUMENTATION.md):

19. ☐ Trust portal mirror current
20. ☐ Per-attestation summary published (SOC 2 / ISO / ISMS-P / HIPAA per applicable)
21. ☐ Auditor self-serve evidence regen working
22. ☐ Per-incident postmortem published per [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)

## 6. Per-vertical regulatory binding

For verticals in audit scope:

23. ☐ Per-vertical PRD §11 open questions closed (or council-deferred with reason)
24. ☐ Per-vertical DPIA current per [`templates/dpia-template.md`](../templates/dpia-template.md)
25. ☐ Per-vertical override per [PRIVACY-PROGRAM §2.2.3](../PRIVACY-PROGRAM.md) verified

## 7. Per-pack regulator-watch

Per regional pack:

26. ☐ Regulator-change watch lane checked in last 14 days
27. ☐ No outstanding regulator advisory unaddressed
28. ☐ Per-pack residency contract honored

## 8. Vendor + dep ledger

Per [VENDOR-PARTNER-LEDGER.md](../VENDOR-PARTNER-LEDGER.md) + ADR-0013:

29. ☐ Per-vendor contract within renewal window
30. ☐ Per-vendor risk assessment within annual cycle
31. ☐ Per-dep license-tier verified (no AGPL/GPL/SSPL/BUSL drift in product code)
32. ☐ SBOM current per release

## 9. Threat model + security

33. ☐ Per-service threat model refreshed within quarter per [`templates/threat-model-template.md`](../templates/threat-model-template.md)
34. ☐ Quarterly external pen-test current (Sev 1 + 2 PCI-DSS Req 11)
35. ☐ Annual on-site QSA assessment (PCI Level 1 SP)
36. ☐ Annual KISA security assessment (KR)
37. ☐ Per-RUSTSEC + cargo-audit + Trivy 4-layer findings closed (or accepted-with-rationale)

## 10. Foundry capability eval

Per ADR-0024:

38. ☐ Per-capability eval-set passed within last release window
39. ☐ Adversarial / red-team eval current
40. ☐ Per-region linguistic eval current

## 11. Final pre-audit pack

41. ☐ Audit pack assembled by `oya admin compliance regenerate <regulator>`
42. ☐ Per-control mapping complete
43. ☐ Per-evidence link working
44. ☐ Trust-portal entry accessible to auditor
45. ☐ Council sign-off for audit submission

## 12. Sources

[COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md), [security-program.json](../security-program/security-program.json), [PRIVACY-PROGRAM.md](../PRIVACY-PROGRAM.md), [`standards/fintech-compliance.md`](../standards/fintech-compliance.md), per-vertical PRDs, per-pack PACK.md, ADR-0003/0008/0013/0024/0038/0039.
