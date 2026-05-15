---
doc_status: published
---

# Checklist: Vertical Onboarding

> **When:** Onboarding a new vertical product (or an existing skeleton vertical promoting to draft / preview).
> **Owner:** Per-vertical team lead + `council-architecture` co-sign.
> **Validator:** `vertical-onboarding-evidence` lane + per-vertical PRD §11 satisfied.

---

## 0. Pre-flight (before authoring kicks off)

1. ☐ Council ratification of the vertical scope per [ADR axis admission protocol](../decisions/ADR-0012-axis-admission-protocol.md) (vertical = sub-axis under axis 3 / Vertical Industry Cloud)
2. ☐ Owning team formed; charter at `teams/vertical-<name>/CHARTER.md`
3. ☐ Per-vertical PRD authored from `products/_TEMPLATE.md` (all 13 sections)
4. ☐ Regulatory pack(s) identified per [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md) — primary regulator + per-region overlay
5. ☐ Vertical kernel flat-crates target reserved at `crates/oya-vertical-<name>-kernel-*`

## 1. Architecture

6. ☐ Bounded context defined per [DESIGN.md §4](../DESIGN.md) — kernel / domain / app / adapter / api / worker / runtime layers
7. ☐ ≥ 5 kernel entities authored with full Rust struct sketches; data_class + plane + subject_attributes
8. ☐ Aggregate boundaries declared
9. ☐ Persistence layout (sharding key + partition + replication + retention)
10. ☐ Event schemas (≥ 5) authored under `contracts/asyncapi/<vertical>-events.yaml`
11. ☐ Cross-axis contracts consumed (Tenant / Identity / Audit / Foundry / Cloud / Workspace / Search / Ads — minimum 6) per [DESIGN §10](../DESIGN.md)
12. ☐ Internal seams declared (what other axes depend on this vertical)

## 2. Data Use Boundary + Privacy

13. ☐ Per-vertical override per [PRIVACY-PROGRAM.md §2.2.3](../PRIVACY-PROGRAM.md):
    - Healthcare: PHI / PII-identifying / PII-quasi / Sensitive-PIPA-Art23 hard-deny for ads
    - Fintech: PCI / Financial-KR-신용정보 hard-deny
    - Education-K12: CHILDREN_UNDER_14 hard-deny
    - Public-sector: per-jurisdiction tighter
14. ☐ DPIA template completed per `templates/dpia-template.md`
15. ☐ Audit-chain emission contract: which capability invocations emit + which event topic
16. ☐ DSR cascade tested with synthetic record across vertical-kernel + cross-axis stores

## 3. Regulatory pack binding

17. ☐ Primary regulator binding per pack (e.g. KR MFDS for healthcare; KR FSC for fintech)
18. ☐ Per-region pack overlay applied (KR + JP + US + EU + IN + ... per where vertical launches)
19. ☐ Evidence-collection cadence declared per [COMPLIANCE-MATRIX.md §3.X](../COMPLIANCE-MATRIX.md)
20. ☐ Per-regulator notification SLA (PIPA 24h FSS / 72h PIPC / GDPR 72h / HIPAA 60d / etc.)
21. ☐ Per-vertical specific KR statutes (e.g. 의료법, 약사법 for healthcare; 자본시장법, 신용정보법 for fintech) bound

## 4. Foundry integration

22. ☐ Per-vertical Foundry capabilities authored (≥ 5 covering primary use cases) per `templates/capability-record-template.yaml`
23. ☐ Per-capability eval set (≥ 20 cases each)
24. ☐ Autonomy-tier cap per regulated capability (T1 for safety; T2 max for fintech regulated)
25. ☐ Per-vertical agent triage / draft / summarize / schedule / classify capabilities

## 5. Cloud + cell binding

26. ☐ Per-tenant cell-tier declared per [ADR-0009 cell architecture](../decisions/ADR-0009-cell-architecture-per-tenant-per-region.md)
27. ☐ Per-cell HSM partition allocated (KCMVP for KR per-cell; FIPS for global)
28. ☐ Per-region residency declared per regional pack

## 6. Workspace integration (where applicable)

29. ☐ Vertical-specific Mail / Doc / Drive / Calendar templates
30. ☐ Per-vertical DLP rules
31. ☐ Per-vertical retention policy
32. ☐ Per-vertical legal-hold pattern

## 7. Search integration (per consent)

33. ☐ Per-tenant private index for vertical content
34. ☐ Per-class allowlist for index ingestion
35. ☐ DSR-cascade purge from index tested

## 8. SLOs + runbooks

36. ☐ Per-vertical SLOs declared in [SLO-CATALOG.md §2.3](../SLO-CATALOG.md)
37. ☐ Per-vertical runbooks authored: `docs/runbooks/vertical-<name>/<runbook-id>.md` for each Sev-1/2 alert
38. ☐ On-call rotation defined per `RACI-OWNERSHIP.md`
39. ☐ Drill scheduled (within 90 days of preview)

## 9. GTM + customer success

40. ☐ Design partner identified
41. ☐ Pricing + packaging declared per [GTM-PLAN.md §4](../GTM-PLAN.md)
42. ☐ Per-vertical migration playbook from competitor stacks
43. ☐ Trust-portal entry for vertical

## 10. Wave-gate exit

44. ☐ ≥ 1 design-partner tenant running end-to-end
45. ☐ ≥ 6 cross-axis contracts exercised
46. ☐ Audit-chain emission complete
47. ☐ DSR cascade tested in production-equivalent
48. ☐ Per-regulator evidence pack regenerated
49. ☐ SLO baseline within budget
50. ☐ Council sign-off

## 11. After preview

- ☐ Per-quarter regulator-watch lane checks for vertical's regulators
- ☐ Per-quarter eval-set refresh per Foundry capabilities
- ☐ Per-quarter DR drill including vertical's data
- ☐ Per-tenant onboarding self-serve evaluation (target ≤ 5min by stable wave)
