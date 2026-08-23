---
doc_status: archived
---

# ADR end-state policy — clean live source of truth

**Status:** binding process for disposition program (2026-08-06)  
**Not merge authority alone.** Land via protected PR + dual-critic + `presubmit`.

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
- Single `presubmit` admission (live 0515 lineage)  
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

## Amendment 2026-08-09 — gated Proposed apexes (ADR-0710)

**Scope of the rule above, stated because it was being read wider than it was written.**
"Forbidden terminal statuses: `Proposed`" and "Decide **now** — no parking as Proposed" were
authored for the E1–E2 disposition of ~458 **inherited** ADRs whose decisions had already been
made in practice and never recorded. They forbid **parking**: leaving a decision undecided with
no gate, no owed evidence, and no date. They were never a rule that a newly authored apex must
assert a posture *before* the evidence that would justify it is in — which is the opposite
failure, and a worse one for a security clause.

**What is now admitted, and only this.** A **gated Proposed apex** may live under
`docs/decisions/`. It must satisfy all four:

1. It is a **topic apex**, not a member to be folded into one.
2. Its `status: Proposed` is attached to a **named clause**, stated in the ADR's Status section,
   which is the reason it is not Accepted.
3. That clause names **the evidence it waits on** and what outcome decides it either way — an
   answerable question, not an intention to revisit.
4. It carries **no implement authority while Proposed**, and that is *enforced rather than
   promised*. `2026-08-06-live-resolution-rule.json` already ranks Proposed/Deprecated/Rejected
   as "not implement authority", and `governance/check/adr-citation-closure` now fails closed
   under `adr_citation_rejected_authority` when any of the three authority surfaces
   (`CLAUDE.md`, `AGENTS.md`, `docs/AGENTS.md`) cites an ADR in one of those three statuses.

**Anything not meeting all four is still forbidden**, and the E2 rules above apply to it
unchanged: decide now, no parking.

**The distinction this amendment records.** Tree location is a **discoverability** property;
frontmatter status is an **authority** property. Conflating them is what made the original rule
read as forbidding this case, and it is also what made the old gate safe only by coincidence —
liveness was a *directory* property, so an unenforced status under `docs/decisions/` was invisible
to every rule. Enforcing the two separately is what makes admitting this case safe: a gated
Proposed apex is findable at step 1 and is not law, and the **gate**, not this prose, holds the
second half.

**Instances.**

- ADR-0710 (Kubernetes admission substrate), gated on clause D-8's workload-boundary evidence.
- ADR-0712 (node kernel + pool matrix / F1(a) / MPV2-0053), gated on A1 4-surface ABI matrix
  evidence (Linux-primary / Asterinas-soak interim until Accept). The number between ADR-0710 and
  ADR-0712 is reserved for PR #1644 Swarm Delivery Law and is not an F1 instance here.
- ADR-0713 (Node Substrate Architecture / MPV2-0054), severable gated apex: Accept (a) owned
  runtime waits on state-machine/recovery DoD + kill-9/upgrade reconvergence tests (founder
  choice alone is insufficient); Accept (b) `os/`-retirement encode waits on D-3 preconditions
  including machine-config harvest before `config-v1alpha1` delete, fleet-basis pin replacement,
  boot-marker contract, and `os/` charter amendment.
- ADR-0714 (isolation-property tier names / F1(c) / MPV2-0055), gated on enforcement re-home
  evidence before rename (outcome-determining: re-home lands or rename encode stays forbidden).
- ADR-0715 (F1 Admission package / F1(d) / MPV2-0056), gated on the D-8 evidence packet **or**
  explicit Reject of ADR-0710 under the dated timebox when hosted topology self-fails D-8 —
  not a second ungated parking slot for the same topic, and not a fold into ADR-0710's body;
  it is the F1 Admission package work item that closes either way.

**Amendment 2026-08-10 (PR #1929 Round-4) — non-gated Proposed-apex list entries removed.**
Former F1 list rows that waited only on unscoped founder choice, or that duplicated ADR-0710's
D-8 gate without an outcome-determining close path, are removed or re-gated as above. Merging
owned-runtime + `os/`-retirement into ADR-0713 eliminates the contradictory dual-Accept hazard.
