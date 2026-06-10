# 10 — STATUS-vs-EDGE drift across all ADRs

**Lane:** Mechanical scan of every ADR in `/Users/jasonlee/Developer/source/docs/decisions/` for STATUS-vs-EDGE drift.
**Scope scanned:** `ADR-*.md` at the top level of `decisions/`.
**Date:** 2026-06-06 (read-only; the machine-readable index was NOT trusted — every claim below is from the actual ADR files).

## What "drift" means here

- **Case A (forward drift):** a non-empty `superseded_by:` (or `amended_by:`, or a body "**Superseded by ADR-xxxx**" declaration) **while** `status:` is still `accepted` / `Accepted` / `Proposed` / `deprecated` — i.e. NOT `Superseded`. This is the ADR-0015 pattern.
- **Case B (inverse drift):** `status: Superseded` **while** `superseded_by:` is empty / missing.
- **Body-vs-frontmatter drift:** the YAML frontmatter and the in-body `> **Status:**` / `> **Superseded-by:**` block disagree.

## Coverage / scope honesty (NO SILENT CAPS)

- **347** `ADR-*.md` files exist at the top of `decisions/`. (`ls ADR-*.md | wc -l` = 347; total dir entries 351 = 347 ADR + `INDEX.md` + `README.md` + `specs/` + `templates/`.)
- **321** ADR files have YAML frontmatter; **26** have NO leading `---` frontmatter (listed below). All 26 were scanned via their body `**Status:**` / `**Superseded by:**` lines and prose — **none drift** (all show empty/`none`/`N/A` supersession edges; ADR-0130 = body Status Accepted + Supersedes N/A; ADR-0221's "superseded by" prose refers to evidence files, not its own status).
- Status-value distribution (frontmatter files): `accepted` 170, `proposed` 126, `superseded` 21, `deprecated` 1, `amended` 1, `accepted (amendment)` 1, `proposed (conditional…)` 1.
- The parser handles BOTH inline `superseded_by: [ADR-xxxx]` AND YAML block-list form (`superseded_by:` then `  - ADR-xxxx` on following lines). The block-list form matters: **ADR-0358 would have been missed** by an inline-only scan.
- **Case B = 0** in frontmatter: all 21 `status: Superseded` files carry a non-empty `superseded_by:` (verified: 24 files have non-empty `superseded_by`; 21 are status Superseded, 3 are not → those 3 are Case A).
- 26 no-frontmatter files (all clean): ADR-0130, 0146, 0149, 0150, 0151, 0152, 0153, 0154, 0155, 0156, 0173, 0200, 0201, 0202, 0203, 0211, 0212, 0214, 0215, 0216, 0217, 0218, 0219, 0220, 0221, 0239.

## DRIFT FINDINGS

### Case A — edge says superseded/amended, but status is NOT Superseded (5 frontmatter + 1 deprecated body)

#### 1. ADR-0015 — `status: accepted` + `superseded_by: [ADR-0131]` (the named exemplar)
`ADR-0015-architectural-flattening-target.md`
```
3	status: accepted
5	superseded_by: [ADR-0131]
13	> **Superseded-by:** [ADR-0131](ADR-0131-per-microservice-flat-layout.md) (partial — only the docs-vs-crates top-level split; BC and layer rules remain in force)
```
DRIFT: frontmatter `status: accepted` (L3) while `superseded_by: [ADR-0131]` is non-empty (L5) and body repeats it (L13). Note L6 `supersession_note` explicitly argues "so status stays accepted" — i.e. drift is intentional/partial-supersession, but it IS a status-vs-edge mismatch by the literal rule.

#### 2. ADR-0316 — `status: Proposed` + `superseded_by: [ADR-0329]`
`ADR-0316-capability-tier-over-product-fragmentation.md`
```
3	status: Proposed
28	superseded_by: [ADR-0329]
29	supersession_note: "ADR-0329 (Accepted) retires the capability-tier doctrine; the cross-microservice retirement migration is scheduled for Wave 15J, so status remains Proposed until the migration lands."
```
DRIFT: frontmatter `status: Proposed` (L3) while `superseded_by: [ADR-0329]` is non-empty (L28). A Proposed ADR that is already declared superseded — note ADR-0329 is "Accepted" per its own supersession_note. (No body `> **Status:**` block in this file.)

#### 3. ADR-0358 — `status: Proposed` + `superseded_by:` block-list `[ADR-0392, ADR-0408]`
`ADR-0358-ideal-production-roadmap-strangler-bazel-oya-overlay.md`
```
3	status: Proposed
9	superseded_by:
10	  - ADR-0392
11	  - ADR-0408
13	amendment_note: "2026-05-29 (founder decision): §2 toolchain build-graph + CI engine reversed from Bazel rules_rust to Buck2. Superseded-by ADR-0392 (Buck2 canonical build graph) + ADR-0408 (Buck2-driven CI/CD). ONLY §2's build-graph/CI engine is reversed; …remain in force."
```
DRIFT: frontmatter `status: Proposed` (L3) while `superseded_by:` lists ADR-0392 + ADR-0408 (L9–L11). **Block-list form** — only caught by a YAML-list-aware scan.

#### 4. ADR-0363 — `status: Accepted` + `amended_by: [ADR-0510, ADR-0513]`
`ADR-0363-retire-agentic-vcs-foundry-to-intelligence-forgejo-substrate.md`
```
3	status: Accepted
9	superseded_by: []
10	amended_by: [ADR-0510, ADR-0513]
```
DRIFT (amended_by branch): `status: Accepted` (L3) with a non-empty `amended_by: [ADR-0510, ADR-0513]` (L10). `superseded_by` is empty (L9), so this is an amendment edge, not a supersession edge — flagged per the rule (`amended_by` non-empty while status not Superseded). Whether "Accepted + amended_by" is acceptable policy is a judgment call, but it is a status-vs-edge signal worth surfacing.

#### 5. ADR-0482 — `status: Accepted` + `amended_by: [kubers-anchor-2026-05-28]`
`ADR-0482-bespoke-substrate-roadmap.md`
```
3	status: Accepted
11	superseded_by: []
13	amended_by: [kubers-anchor-2026-05-28]
```
DRIFT (amended_by branch): `status: Accepted` (L3) with non-empty `amended_by: [kubers-anchor-2026-05-28]` (L13). The amender token is `kubers-anchor-2026-05-28` (NOT an `ADR-xxxx` id) — a dangling/non-ADR amendment reference, doubly suspect.

#### 6. ADR-0054 — `status: deprecated` + body "Superseded by ADR-0116" + `Superseded-by: ADR-0116`
`ADR-0054-grit-scaffold-claim-pattern.md`
```
3	status: deprecated
9	> **Superseded by ADR-0116 (2026-05-16)** — external agent-coordination tooling (grit, rtk, icm, vox) is retired; the Foundry pipeline (M01-P18) is the canonical workflow. …
11	> **Status:** Deprecated 2026-05-16
13	> **Superseded-by:** ADR-0116
```
DRIFT: frontmatter `status: deprecated` (L3) and body `> **Status:** Deprecated` (L11), but the body ALSO declares `> **Superseded by ADR-0116**` (L9) and `> **Superseded-by:** ADR-0116` (L13). The supersession edge is real (points at ADR-0116) yet status is `deprecated`, not `Superseded`. (No frontmatter `superseded_by:` key — the edge lives only in the body.)

### Body-vs-frontmatter drift (frontmatter is correct, body is stale)

#### 7. ADR-0052 — frontmatter `status: Superseded` + `superseded_by: [ADR-0118]`, but BODY says `Status: Accepted` and `Superseded-by: —`
`ADR-0052-inventory-grit-cutover.md`
```
4	status: Superseded
11	superseded_by: [ADR-0118]
29	> **Status:** Accepted
32	> **Supersedes:** — **Superseded-by:** —
```
DRIFT: the FRONTMATTER is internally consistent (Superseded + edge to ADR-0118), but the in-body block (L29 `Status: Accepted`, L32 `Superseded-by: —`) contradicts it. A reader trusting the body sees an Accepted, never-superseded ADR; a reader trusting frontmatter sees Superseded→ADR-0118. Inverse-flavored (body shows empty supersede edge while it IS superseded).

## Borderline / noted, NOT counted as STATUS-vs-EDGE drift

- **ADR-0147** (`ADR-0147-container-sandboxing-runtime-ladder.md`): `status: Amended` (L3) with `superseded_by: []` (L9) and NO `amended_by:` edge. This is a **self-amendment** documented in-body (L20 purpose "Amended 2026-05-18…", L29 "Amended — 2026-05-18 (original Accepted 2026-05-18)", L175 "## Amendment 2026-05-18"). Status `Amended` with no outbound `amended_by` edge is internally consistent (the amendment is the same ADR), so NOT a status-vs-edge contradiction — flagged here only for completeness because the status value is non-standard.
- **ADR-0120 / ADR-0121**: both `status: Superseded` + `superseded_by: [ADR-0375]` + body `> **Status:** Superseded by [ADR-0375]` — fully CONSISTENT (no drift). Listed because the body "Superseded by …" phrasing pattern-matches the drift regex; manual check confirms frontmatter agrees.
- **ADR-0377** (`status: Proposed (conditional: Accepted only after ADR-0377-D2 and ADR-0377-D3 …)`) and **ADR-0380** (`status: Accepted (amendment)`): non-standard status strings, both with empty supersede/amend edges → no status-vs-edge drift.

## Summary table

| ADR | status (frontmatter) | edge | line(s) | drift type |
|-----|----------------------|------|---------|------------|
| ADR-0015 | accepted | superseded_by:[ADR-0131] | L3 / L5,L13 | A (superseded_by, fm+body) |
| ADR-0316 | Proposed | superseded_by:[ADR-0329] | L3 / L28 | A (superseded_by, fm) |
| ADR-0358 | Proposed | superseded_by:[ADR-0392,ADR-0408] | L3 / L9–11 | A (superseded_by block-list, fm) |
| ADR-0363 | Accepted | amended_by:[ADR-0510,ADR-0513] | L3 / L10 | A (amended_by, fm) |
| ADR-0482 | Accepted | amended_by:[kubers-anchor-2026-05-28] | L3 / L13 | A (amended_by non-ADR token, fm) |
| ADR-0054 | deprecated | Superseded-by:ADR-0116 (body) | L3 / L9,L13 | A (body supersede edge, status=deprecated) |
| ADR-0052 | Superseded | superseded_by:[ADR-0118] (fm) vs body Status:Accepted / Superseded-by:— | L4,L11 vs L29,L32 | body-vs-frontmatter |

**Total STATUS-vs-EDGE drifts: 7** (6 Case-A-style + 1 body-vs-frontmatter). Case B (status Superseded + empty edge) in frontmatter: **0**.
