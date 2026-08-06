# ADR end-state policy — clean live source of truth

**Status:** binding process for disposition program (2026-08-06)  
**Not merge authority alone.** Land via protected PR + dual-critic + `oya-ci-required`.

## Target end state

| In tree under `docs/decisions/` | Not in tree (git history only) |
|--------------------------------|--------------------------------|
| **Accepted** only — live law, consolidated | **Superseded** (after content fold into successor) |
| | **Rejected** |
| | Former **Proposed** after decision |

**Forbidden terminal statuses:** `Proposed`, `MISSING`, `Deprecated` (map Deprecated→Superseded or Rejected), bare `Amended` (fold into parent Accepted + `amended_by`).

## Decision rules for every current Proposed

Decide **now** — no parking as Proposed:

| Decision | When |
|----------|------|
| **Superseded** | Later Accepted already replaces it (`supersedes:` or clear successor) |
| **Accepted** | Aligns with north star + hyperscaler practice, not contradicted by higher live ADR, and either (a) required by live chain / gates / code, or (b) still-true design that is current law |
| **Rejected** | Obsolete, false path, contradicts live admission/layout/security, never-landed dead design, or activation-gated idea better folded into a living Accepted ADR as non-binding appendix then deleted |

Activation-gated work (warm CAS, RE, etc.): **do not leave Proposed**. Either:
- **Reject** and capture residual requirements in the living CAS/CI Accepted ADR as *future activation criteria* (not authority), or  
- **Accept** only the *policy boundary* that keeps activation fail-closed (no `remote_enabled` flip).

## Live resolution (unchanged)

Never treat `status: Accepted` as sufficient. Resolve supersession + `amended_by` before apply. After cleanup, live set should make resolution almost identity (few amends, no supersession chains in-tree).

## Consolidation rules

1. **Topic apex:** one Accepted ADR per topic when bodies overlap (CI admission, monorepo layout, faces/de-commit, capability boundaries, k8s port engine, CAS/cache policy, agent harness retirement, etc.).
2. **Renumber** into a dense live series only after content merge; emit `registry/adr-redirect.v1.json` mapping every historical number → live ADR id (or `rejected` / `historical-only`).
3. **Citation rewrite:** mechanical rewrite of `ADR-NNNN` in repo to live ids using redirect map (gates must stay green).
4. **Preserve:** any still-true normative text from Superseded must be copied into the superseding/live ADR *before* delete.
5. **Delete** Superseded + Rejected files from the tree after fold + redirect map entry. Git history is the archive.

## North star / hyperscaler bar (Accept filter)

- Owned Rust stack; capability-first monorepo  
- Single `oya-ci-required` admission (live 0515 lineage)  
- No dual-authority CI / no re-Prow / no external harness brand as law  
- Zero-trust, cell isolation, telemetry-first, constant-work  
- Full-depth product depth — not MVP cosplay  
- No warm CAS / RE as silent Accept without explicit activation gate language  

## Waves

| Wave | Work |
|------|------|
| **E0** | Policy + redirect registry schema + this doc |
| **E1** | MISSING status → Accept / Reject / Superseded |
| **E2** | All Proposed → Accept / Reject / Superseded (parallel by decade) |
| **E3** | Content fold Superseded → successors; reverse-links |
| **E4** | Delete Superseded + Rejected from tree; registry complete |
| **E5** | Consolidate Accepted topics; renumber; citation rewrite; dual-critic |

## Hard stops

- No mass-Accept of activation ADRs that enable RE/warm without gates  
- No delete of Superseded before fold receipt  
- No renumber without redirect map + citation rewrite in same PR train  
- No hand-edit `*.generated.json`  

## Progress (2026-08-06)

| Step | State |
|------|--------|
| No `Proposed` remaining | **DONE** (Accepted / Superseded / Rejected only) |
| Redirect map | `adr-redirect.v1.json` (partial live notes) |
| Delete Superseded/Rejected from tree | **BLOCKED** until ADR-0624 census epoch transition (P2 binds `docs/decisions` corpus). Plan: E4a move-to-`_historical/` with selector update, or E4b epoch advance then delete. |
| Consolidate Accepted | **QUEUED** — see `e5-consolidation-clusters.json` |
| Renumber | **QUEUED** after consolidation; citation rewrite + redirect map required |

## E4 safe path (choose one)

1. **Epoch advance (preferred long-term):** new census epoch receipt for Accepted-only corpus; then `git rm` Superseded+Rejected.  
2. **Quarantine dir:** move non-live ADRs under `docs/decisions/_historical/` and exclude from agent/default globs **and** update census selector in same PR train.

Do **not** blind-delete while P2 active.

## E3–E5 completion (2026-08-06 autonomous)

| Wave | Result |
|------|--------|
| **E3** | Folded Superseded gists into successors/apex (`e3-fold-log.json`) |
| **E4** | Archived **448** historical ADRs to `docs/adr-archive/` (outside P3 census direct children) |
| **E5** | **10 live apex ADRs** ADR-0700…0709 in `docs/decisions/`; members superseded then archived; path citations rewritten (~716 files) |
| **Redirect** | `adr-redirect.v1.json` maps old numbers → live apex + archive path |

**Live tree:** only Accepted apex files + README + `_disposition/`.
