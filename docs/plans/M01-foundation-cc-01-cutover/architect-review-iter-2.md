---
doc_status: published
---

# Architect Review — ralplan-oyatie-sst-consolidation (Iteration 2)

<!--
status: Accepted
date: 2026-05-12
related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055
-->

Captured 2026-05-12. Reviewer: oh-my-claudecode:architect. Verdict: **ITERATE**.

---

## Summary verdict


## Revision landing checklist

| Revision | Landed? (Y/N + cite) | Sound? (Y/N + reason) |
|---|---|---|
| V3 P10 scaffold-claim | Y — plan L165 names scaffold-claim per ADR-0054 for `governance-authoritative-tracked-kernel/` | Y — pattern consistent |
| V4 ADR bootstrap-window | **N — claim-only** | N — plan §3 L183 says "iter-2 edits to lift-source: add bootstrap-window clause to §Decision" but ADR draft mtime 20:02:09 < plan mtime 20:14:07; §Decision still reads "exactly three" with no bootstrap-window carve-out. Plan's Principle L14 carries the bootstrap-window text but the ADR lift-source does not — divergence is back |
| Arch #2 bootstrap clause | **N — claim-only** | N — same as V4 above. The bootstrap-window clause appears in plan §1 Principles but the canonical ADR lift-source was not edited. ADR-shape lane will flag this divergence at landing |
| Arch #3 P7 gate concrete | Y — plan L140-143 enumerates three gates: banned-primitives lane green + archive-orphan lane + per-row `archived_at` non-null | Y — all three are falsifiable; the `archived_at IS NULL` query is a clean SQL-shaped invariant |
| Arch #8 §3 ADR block deleted | Y — plan §3 L180-188 replaces inline ADR text with 3-line pointer naming canonical lift-sources for ADR-0052/0053/0054 | Y — pointer-only is the correct invariant; downstream V4/Arch#2/Arch#4/Arch#5 failures are now in the lift-source, not in two places |
| Critic #2 tooling-agent-read | Y — appears 11 times in plan body (P1 helper row L91, P2 L94-99, P4 L120, etc.); inventory ledger row reconciled | Y — name is consistent with existing `tooling-cli-dev-runtime` sibling crate (orchestrator existence findings confirm). NOTE: ADR draft still reads `agent-read` at L15, L31, L50, L55, L74 — same lift-source-not-edited problem, but plan-internal usage is clean |
| Critic #3 A6 enumeration | Y — plan L126 P5 deliverable produces `oyatie/docs/AGENT-INSTRUCTION-SOURCES.md`; scope is `**/*.md`, `**/*.json`, `**/.claude/**` for files containing the agent-instruction HTML fences | Y — concrete deliverable; executor knows which files to touch. Out-of-repo Stop hook handled via inventory-ledger reference row |
| Critic #4 A9 attestation | Y — plan L172 and L219 both state "SATISFIED" with Critic-attestation framing | Y — status footer makes attestation explicit |
| P3.5 cross-cite | Y — plan L108-115 frames P3.5 as cross-cite (not salvage), pins landing to `oyatie/docs/products/foundry/PHASE-00-SPEC.md`, deadline BEFORE P4 (not BEFORE P6), phantom-path correction for `oyatie/.omx/ultragoal/` included, inputs cite foundry-salvage doc | Y — framing is correct (KEEP-classified sources are not destruction-prevention candidates); landing matches foundry-salvage §H recommendation; Linus reshape line is present |
| Existence corrections | Y — `.codex/worktree_init.sh` dropped (plan L91); demo subdir `docs/runbooks/agentic-pipeline/` (plan L65, L152, L202); RACI row added at P0.5 (plan L83); A6 hook-path resolution option (a) chosen explicitly (plan L126) | Y — all four sub-items handled in the right phase |

## New principle violations

- **NEW PV1 (re-emerging V4)** — Plan §3 pointer asserts ADR draft was edited; file mtime contradicts. This is not a NEW violation of spec §Constraints; it is V4 from iter-1 *re-asserting itself* because the fix was claimed-not-executed. Severity HIGH because the ADR-shape fitness lane will deterministically flag the divergence at landing time.

No genuinely-new violations introduced. The plan body itself is sound; the gap is entirely in the un-edited lift-source.

## Verification matrix walk (one row per A1-A10)

| Criterion | iter-1 → iter-2 transition | iter-2 status | Notes |
|---|---|---|---|
| A1 | AMBER → GREEN | **GREEN** | P3 + P3.5 both green-able; cross-cite to foundry corpus added; scaffold-claim resolves the lane-crate FK problem |
| A2 | GREEN | **GREEN** | P1 inventory; ADR slot 0052 reserved; `archived_at` column added; phantom-path correction included |
| A3 | AMBER → GREEN | **GREEN** | P6 + P7; three concrete gates replace the unfalsifiable "two CI green runs"; archive-orphan lane explicit |
| A4 | AMBER → GREEN | **GREEN** (plan body) / **AMBER** (ADR draft still names `agent-read`) | Plan body landed; the ADR lift-source divergence will surface at P5 when ADR-0053 lands and references the helper by its old name |
| A5 | GREEN | **GREEN** | P4 scaffold + P5 lane activation; HTML-comment-fence convention is concrete; banned-primitives token list (per pre-cutover-drafts §Draft 6) is enumerable |
| A6 | AMBER → GREEN | **GREEN** | P5 produces `AGENT-INSTRUCTION-SOURCES.md`; option (a) chosen explicitly; out-of-repo Stop hook scoped via inventory-ledger reference row |
| A7 | AMBER → GREEN | **GREEN** | P8 symbols pinned, single-file/different-identifiers framing matches A7 criterion |
| A8 | AMBER → GREEN | **GREEN** | P10 scaffold-claim resolves the new-crate gap |
| A9 | RED → GREEN | **GREEN** | Explicit "SATISFIED — verified by Critic via direct file-path check" in plan §4 row and §5 footer |
| A10 | AMBER → GREEN | **GREEN** | Final-integration audit lists five reshape items: (a) special cases, (b) hierarchies, (c) ceremony, (d) P5 reshape, (e) P10 reshape. Empty=fail gate present |

Net: 10 GREEN-or-GREEN-pending-ADR-edit. The single AMBER (A4) is downstream of the ADR-draft-not-edited gap and resolves the moment that gap is fixed.

## Pre-mortem walk
- S2 (P7 deletion before re-wire): GREEN (Arch #3 three gates land; archive-orphan lane is the missing piece that catches deletion mistakes specifically)

## Architect verdict

**ITERATE**


4. **§Follow-ups** — unify the current 5 items with the prior plan §3 follow-ups (also 5 items, currently divergent per Critic #1). Single source-of-truth list, no duplicates.

No further plan-body edits needed. The plan is structurally sound; only the lift-source claim is unhonored.

## Architect signature

"Architect (oh-my-claudecode:architect) — 2026-05-12 iter-2"
