---
purpose: Oyatie — Doc Update Protocol
doc_status: published
---

# Oyatie — Doc Update Protocol

> **Status:** Draft v0.1 — 2026-05-09. The full step-by-step. [DOC-CATALOG.md §3](DOC-CATALOG.md) has the inline version; this doc is the canonical longer reference.
> **Owner:** `council-architecture`.

## The 5-stage protocol

```
1. Pre-flight  (read deps + check trigger + claim authorship)
2. Authoring   (draft the change)
3. Validation  (run all relevant CI/agent checks)
4. Review      (per change-class + per blast-radius)
5. Publish     (merge + audit-emit + dependent-doc cascade + trust-portal mirror if applicable)
```

## Stage 1: Pre-flight

1. Identify the trigger event from [DOC-CATALOG.md §1](DOC-CATALOG.md) (`EVT-*`).
2. Read the doc to be updated AND every doc in its `dependent_docs` column.
3. Read the team charter of `owner_team`. If you are not on that team, request a co-author.
4. If trigger is regulatory, read [COMPLIANCE-MATRIX.md](COMPLIANCE-MATRIX.md) for that regulator + relevant ADR.
5. Open the relevant `gh issue view` for any tracking issue.
6. Confirm `agent_authoring_allowed` for the doc — if NO and you're an agent, hand off to a human (per ADR-0050 governance + [DOC-CATALOG.md §2.4 modified](DOC-CATALOG.md): agents may DRAFT proposed changes; merge/acceptance requires the named owner).

## Stage 2: Authoring

7. Author the change. Use the canonical template from `templates/` if applicable.
8. Add a row to [`CHANGELOG.md`](CHANGELOG.md): `<doc.id> <iso-date> <author> <one-line summary>`.
9. Update the doc's "Sources scanned" footer with current timestamps.
10. If the change adds/removes/renames a doc, update [`README.md`](README.md), [`DOC-CATALOG.md`](DOC-CATALOG.md), AND [`machine-readable/catalog.json`](machine-readable/catalog.json).
11. If the change touches a [DESIGN §10](DESIGN.md) cross-axis contract row, update `machine-readable/contracts.json`.
12. If new ADR drafted, run ADR-INDEX regenerator.
13. If Foundry batch shape changed, regenerate `machine-readable/batches.json`.

## Stage 3: Validation

14. Run the `validation_check` per [DOC-CATALOG.md §2](DOC-CATALOG.md).
15. Run the dependent-docs cross-link check (`oya-governance-doc-catalog`).
16. Run [`oya-governance-glossary`](GLOSSARY.md) to sync any new terms.
17. For agent authoring: emit `EVT-DOC-UPDATED` audit-chain record per ADR-0003.

## Stage 4: Review

18. Open PR with `## Issue / Summary / Verification / Code Review` (four H2s per CLAUDE.md).
19. One author-distinct reviewer agent reviews and approves the exact PR head.
20. For Tier 1 docs, the reviewer applies the council-architecture lens; no human approval or reviewer quorum is required.
21. Per blast-radius class (see [DESIGN §3.0.5.3](DESIGN.md)), affected owners are notified for non-binding input.
22. Merge through the protected PR only after review threads resolve, `oya-ci-required` is green, no conflict exists, and branch protection is satisfied.

## Stage 5: Publish

23. Post-merge: emit `EVT-DOC-UPDATED` to audit chain.
24. If regulator-relevant, regenerate trust portal mirror (per [RUNBOOKS-INDEX.md](RUNBOOKS-INDEX.md) "trust portal publish" runbook).
25. If contract change, broadcast announcement to all consumer teams' charter inboxes.
26. If GLOSSARY term changed, run the `glossary-rename-cascade` Foundry capability.

## Anti-patterns

1. Skipping the CHANGELOG entry — erases audit history.
2. Editing a Tier 1 doc without reading dependents — drifts every dependent doc.
3. Letting an agent author a Tier 1 doc end-to-end — agents propose; humans approve.
4. Renaming a glossary term without the cascade — silent drift.
5. Multi-doc batch PR — one doc per PR for **bulk editorial rewrites**; bundling > 2 non-load-bearing docs is anti-pattern unless coordinated rename. **Exception (binding):** load-bearing doc updates MUST co-change with the code/policy wave that makes them true (see § Amendment: same-wave load-bearing co-change).
6. Bypassing the validator with `--no-verify` — never.

## Amendment: same-wave load-bearing co-change (`doc-update-protocol-overrule`)

Load-bearing doc updates MUST co-change with the code/policy wave that makes them true. Same-wave colocation **supersedes** one-doc-per-PR for load-bearing changes; one-doc-per-PR remains for bulk editorial rewrites.

- **achieves:** prose stays true with landed law; no lag drift.
- **origin:** one-doc-per-PR caused prose to lag landed law → drift.
- **rule:** load-bearing doc updates land in the same wave as the code/policy that makes them true; anti-pattern #5's one-doc-per-PR applies only to bulk editorial rewrites / non-load-bearing cascades.
- **ensure:** Done-Definition D2 + Claim `docs_touched`/`docs_action` packet; reviewer refuses load-bearing law without co-located docs.
- **overturn_when:** a recorded challenge shows same-wave colocation increases contradiction rate AND a replacement freshness gate is live.

## Sources
[DOC-CATALOG.md](DOC-CATALOG.md), CLAUDE.md, ADR-0050, ADR-0001, ADR-0040, ADR-0017, ADR-0711 Amendment D, `docs/AGENTS.md` § Doctrine survival (binding).
