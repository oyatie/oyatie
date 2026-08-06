---
doc_status: published
---

# Checklist: Legacy ADR Deletion (one-time, 2026-05-09)

> **When:** ONLY after `docs/decisions/` is complete (50 ADRs target) AND regression-check passes.
> **Owner:** `crew-adr-promotion` + `council-architecture` co-sign + Founder ratification.
> **Triggered by:** User directive 2026-05-09: "DELETE LEGACY ADR when you are done and sure that we have not regressed (in feature, function, depth, maturity) and have only expanded in positive manner."
> **Anti-pattern note:** Per prior policy ("Don't delete legacy ADRs — forensic value + git-blame integrity matter"), legacy ADR deletion is a deliberate departure from the original anti-pattern. The user explicitly overrode 2026-05-09. Git history retains every legacy ADR (so `git log --all decisions/` and `git show <sha>:decisions/ADR-####-*.md` continue to work — git itself is the forensic backstop).

---

## 1. Prerequisites (cannot delete until ALL satisfied)

1. ☐ All 50 new pack ADRs authored at `docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md` (or council-approved fewer if consolidation reduces count)
2. ☐ Per-new-ADR Status: Proposed → Accepted via council ratification
3. ☐ `docs/decisions/README.md` reflects final pack
4. ☐ `decisions/RETIRED.md` exists (already authored 2026-05-09)
5. ☐ `docs/ADR-CONSOLIDATION-PLAN.md` supersession map complete (every legacy ADR-#### → new pack ADR-#### mapping)
6. ☐ `docs/ADR-INDEX.md` regenerated as the new pack's index
7. ☐ `docs/ADR-INDEX-LEGACY.md` snapshot saved (forensic; per ADR-CONSOLIDATION-PLAN §8 open-question 2)

## 2. Regression check (per-legacy-ADR-substance verification)

For each of the 127 legacy ADRs:

8. ☐ Identify the substance (key decision + key constraints + key consequences)
9. ☐ Find the new pack ADR(s) that capture the substance
10. ☐ Verify no feature loss (every capability declared in legacy is present in new)
11. ☐ Verify no function loss (every described behavior is preserved)
12. ☐ Verify no depth loss (every implementation constraint is preserved or explicitly relaxed with reason)
13. ☐ Verify no maturity loss (status posture preserved or explicitly upgraded)
14. ☐ Record the mapping in `docs/ADR-CONSOLIDATION-PLAN.md` §3 supersession-map

Per-legacy-ADR mapping table (filled out as part of regression check):

```
| Legacy ADR | Substance summary | Captured in new pack ADR | Coverage verdict |
|---|---|---|---|
| ADR-0010 (Metrics consolidation) | per-platform observability lib | ADR-0042 observability stack | FULL |
| ADR-0011 (Isolation operating model) | per-pillar / per-tenant isolation | ADR-0009 cell architecture + ADR-0008 DUB | FULL |
| ADR-0016 (Clinical canonical record) | clinical-record authority + released-view | ADR-0033 vertical-industry-cloud-pack-architecture (vertical-healthcare slice) | PARTIAL — flag for council |
| ADR-0017 (Unified governance catalog) | catalog projection model | ADR-0011 cross-axis-contract-registry + ADR-0019 doc-catalog | FULL |
| ADR-0018 (Tenancy + RLS) | per-tenant RLS enforcement | ADR-0002 tenant-and-identity-kernel + ADR-0006 object-graph engine-enforced isolation | FULL |
| ADR-0019 (Runtime target metadata) | runtime-target schema | ADR-0028 cloud-provider-architecture + ADR-0009 cell architecture | FULL |
| ADR-0020 (Multi-runtime platform standard) | multi-runtime support | ADR-0028 cloud-provider-architecture + ADR-0029 workspace + ADR-0044 service mesh | FULL |
| ... (continue for all 127) | ... | ... | ... |
```

Coverage verdict values: **FULL** / **PARTIAL** / **DROPPED-WITH-REASON** / **EXPANDED** (new pack added beyond legacy).

15. ☐ Every row has FULL or EXPANDED, or PARTIAL/DROPPED with explicit founder + council justification
16. ☐ Council architecture sign-off on the regression mapping
17. ☐ Founder ratification

## 3. Reference sweep (consolidated docs no longer cite legacy ADRs)

18. ☐ `oya-governance-adr-citation` CI lane passes — zero `ADR-####` citations in active consolidated docs (where NNNN ≤ legacy max)
19. ☐ Forensic-allowed citations only in: `ADR-CONSOLIDATION-PLAN.md`, `CONTRADICTION-LEDGER.md`, `RETIRED.md`, `ADR-INDEX-LEGACY.md`
20. ☐ All per-product PRDs sweep — zero legacy ADR-#### refs
21. ☐ All team charters sweep — zero legacy ADR-#### refs
22. ☐ All standards docs sweep — zero legacy ADR-#### refs
23. ☐ All checklists sweep — zero legacy ADR-#### refs
24. ☐ All templates sweep — zero legacy ADR-#### refs
25. ☐ All machine-readable JSONs sweep — zero legacy ADR-#### refs (except `decisions.json` may carry the legacy `_metadata.legacy_adr_supersession_map`)
26. ☐ All regional packs sweep — zero legacy ADR-#### refs
27. ☐ All recon artifacts in `docs/raw/` left UNTOUCHED (raw artifacts may contain legacy refs)

## 4. Pre-deletion safety

28. ☐ `git log --all decisions/` confirms full history retained (git is the forensic backstop)
29. ☐ Tag the pre-deletion state: `git tag pre-legacy-adr-deletion`
30. ☐ Push the tag to origin (so remote retains the pre-deletion state)
31. ☐ Per-regulator notification check: KR FSC + KR PIPC + EU SA — confirm no audit cycle in progress that depends on per-ADR file presence (if so, defer deletion)
32. ☐ Council architecture council-meeting recorded approval

## 5. Execution (only after §1-§4 complete)

33. ☐ `git rm decisions/ADR-*.md` (the 127 legacy files at top of `decisions/`)
34. ☐ Retain: `docs/decisions/`, `decisions/RETIRED.md`
35. ☐ Retain: any explicitly-listed forensic-essential legacy ADRs (TBD — none expected; will surface during regression check)
36. ☐ `git commit` with body: "Per user directive 2026-05-09 + per regression-check at docs/checklists/legacy-adr-deletion.md, retire 127 legacy ADRs in favor of docs/decisions/. Pre-deletion state at git tag pre-legacy-adr-deletion."
37. ☐ Audit-chain emit `EVT-LEGACY-ADR-DELETED` per ADR-0003

## 6. Post-deletion

38. ☐ Trust portal updated with deletion notice
39. ☐ Per-tenant notification (if any tenant-facing ref to legacy ADR existed)
40. ☐ `docs/CHANGELOG.md` row added
41. ☐ `crew-adr-promotion` weekly digest publishes the deletion

## 7. Rollback (if regression discovered post-deletion)

If post-deletion any tenant / regulator / engineer surfaces a regression:
- `git revert` the deletion commit (restores the legacy files)
- Open ADR amendment in new pack to capture the missed substance
- Update regression-check checklist with the discovery
- Re-run from step §2 with corrections

## 8. Sources

- User directive 2026-05-09 (verbatim above)
- [`decisions/RETIRED.md`](../decisions/RETIRED.md)
- [`docs/decisions/README.md`](../../../docs/decisions/README.md)
- [`docs/ADR-CONSOLIDATION-PLAN.md`](../ADR-CONSOLIDATION-PLAN.md)
- All 127 legacy ADRs at `decisions/ADR-*.md`
- All ~50 new pack ADRs at `docs/decisions/ADR-*.md`
