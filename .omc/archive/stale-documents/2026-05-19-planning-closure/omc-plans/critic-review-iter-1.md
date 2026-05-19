---
purpose: Auto-backfilled purpose for critic-review-iter-1.md
---

# Critic Evaluation — ralplan-oyatie-sst-consolidation (Iteration 1 → 2)

Captured 2026-05-12. Reviewer: oh-my-claudecode:critic. Verdict: **ITERATE**.

---

## Summary verdict
ITERATE. The Architect correctly identified 4 load-bearing violations and 8 revisions; Critic concurs on all four violations and adopts all 8 revision requests as MUST-LAND. However:

1. The orchestrator's new P3.5 directive is **partially based on a phantom path**: `/Users/jasonlee/oyatie/.omx/ultragoal/` does **not exist** (verified via `ls`). Inventory at line 473 already correctly notes "not found." The salvage concern is real but **only for `bominal/agents/ultragoal/`**, and inventory classifies all 9 foundry-relevant files there as `KEEP` (not archive/delete). So P3.5 reduces from "salvage at risk of information loss" to "**cross-cite the bominal foundry corpus into `oyatie/docs/products/foundry/PRD.md`**."

2. Plus four additional Critic-only findings on top of Architect's 4 violations + 8 revisions.

## Concurrence with Architect violations
- V1 (P2 chicken-and-egg): **CONFIRM**. Verified `tools/` does not exist; `grit symbols | grep Cargo.toml` returns 0.
- V2 (P3 chicken-and-egg): **CONFIRM**. Same FK problem for `oya-foundry-fitness-portfolio-citation-kernel`.
- V3 (P10 chicken-and-egg): **CONFIRM**. Same FK problem for `oya-foundry-fitness-authoritative-tracked-kernel`.
- V4 (ADR §Decision false during P1-P2): **CONFIRM**. `oya-agent-read` does not exist during P1-P2; ADR closed-set claim is literally false.

## Additional Critic findings

1. **(NEW) ADR-source-of-truth divergence is "stop the line", not cleanup.** Plan §3 ADR Block (lines 153-189) and `adr-draft-grit-icm-sanctioned-primitives.md` already DIVERGE in §Follow-ups (5 vs 5 different items). ADR-shape fitness lane will catch one or both as inconsistent. MUST resolve before iter-2 APPROVE — not optional.

2. **(NEW) Helper crate name divergence across three docs.** Plan §P2 says `tools/oya-agent-read/`; resolutions-doc says `tools/oya-tooling-agent-read/` (flat-crates conventional); inventory mentions `tools/oya-agent-read/`. Three docs, two names. **Adopt `tools/oya-tooling-agent-read/`** per spec §Constraints item 4 (Clean Architecture, `oya-<context>-<role>` form). Update all three docs.

3. **(NEW) Plan §P5 mentions Stop hook but never enumerates which hooks/skills survive vs rewrite.** Inventory says "A6 audit required" but no specific files. Iter-2 must reconcile: either inventory enumerates explicitly, or plan §P5 produces enumeration as output. Gap as written means executor doesn't know which files to touch.

4. **(NEW) A9 self-certification is fictive.** Plan declares its own existence as satisfaction. A9 is satisfied iff verified by Critic/Architect/executor — state explicitly: "A9 verified by Critic via direct file-path check; status = SATISFIED."

## Revision request triage

| Req | Source | Load-bearing? | Must land iter-2? |
|---|---|---|---|
| Arch #1 — Scaffold-claim ADR before P2 | Architect | YES | YES |
| Arch #2 — ADR bootstrap-window clause | Architect | YES | YES |
| Arch #3 — P7 gate concrete (lane + `archived_at` non-null) | Architect | YES | YES |
| Arch #4 — ADR §Consequences §Neutral: cutover is bootstrap | Architect | MEDIUM | YES |
| Arch #5 — Define "human orchestrator" + icm event per invocation | Architect | MEDIUM | YES |
| Arch #6 — P8 pin demo symbols to Draft 3 | Architect | YES | YES |
| Arch #7 — P5/P10 data-shape reshape statements for A10 | Architect | MEDIUM | YES |
| Arch #8 — ADR single source of truth (lift from pre-draft) | Architect (Critic escalated) | YES | YES |
| Critic #1 — Resolve ADR Follow-up divergence | Critic | YES | YES |
| Critic #2 — Use `tools/oya-tooling-agent-read/` everywhere | Critic | YES | YES |
| Critic #3 — Enumerate hook/skill files for A6 | Critic | YES | YES |
| Critic #4 — A9 explicit Critic-attest | Critic | LOW | YES |

All 12 MUST-LAND iter-2. None deferrable.

## P3.5 refinement (per Critic verification)

**Reframed**: P3.5 — "Cross-cite ultragoal foundry corpus into canonical foundry PRD/SPEC" between P3 and P4.

- Source 1: `bominal/agents/ultragoal/` foundry files (mega-plan 97KB + substrate-master 97KB + implementation-plan 44KB + 6 more, all KEEP-classified).
- Source 2: `oyatie/.omx/ultragoal/` — **does not exist; phantom path** (correct that in inventory ADR).
- Foundry-salvage agent has already extracted normative claims to `.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md` (22KB).
- Landing target: `oyatie/docs/products/foundry/PHASE-00-SPEC.md` (new file) — recommendation from salvage agent.
- Deadline: BEFORE the inventory PR (P1) lands, not BEFORE the archive PR (P6). The 9 foundry files are KEEP; they're not deletion targets. Authority-cross-cite, not destruction-prevention.

## Open-questions handling
- Q3 carve-out scope: FLAGGED, DO NOT BLOCK. Plan proceeds under "humans orchestrating cutover may invoke git/gh". ADR landing requires explicit user confirmation.
- Q6 retention policy: FLAGGED, DO NOT BLOCK. Adopt 90 days (resolutions-doc recommendation); user may amend.

## ADR-source-of-truth resolution
**Lift-source: `adr-draft-grit-icm-sanctioned-primitives.md`.** Delete plan §3 ADR Block (lines 153-189); replace with 3-line pointer naming the canonical lift-sources (ADR-0052 ← inventory-draft; ADR-0053 ← adr-draft; ADR-0054 ← pre-cutover-drafts §Draft 2). Pre-draft also gets edits for bootstrap-window clause + human-orchestrator definition.

## Verification matrix walk
- A1: AMBER → GREEN once scaffold-claim adopted
- A2: GREEN (concrete; ADR slot 0052)
- A3: AMBER → GREEN once Arch #3 lands
- A4: AMBER → GREEN once scaffold-claim adopted + name reconciled
- A5: GREEN
- A6: AMBER → GREEN once Critic #3 enumeration lands
- A7: AMBER → GREEN once Arch #6 symbol pin lands
- A8: AMBER → GREEN once scaffold-claim adopted
- A9: RED → GREEN with Critic #4 explicit attestation
- A10: AMBER → GREEN once Arch #7 data-shape lines land

Net: 1 GREEN, 1 RED, 8 AMBER. All resolvable in iter-2.

## Pre-mortem walk
- S1 (P2 helper missing primitive): AMBER (escape-hatch hand-wavy; Planner add 30-min human-orchestrator-resume clarification)
- S2 (P7 deletion before re-wire): RED (mitigation cites the exact unfalsifiable gate Arch #3 calls out; fixes when Arch #3 lands)
- S3 (grit 0.3.0 widens to claim/done): GREEN

## Expanded test plan walk
- Unit: GREEN
- Integration: GREEN (minor: Q4 extension to archive-path tokens)
- E2E: AMBER → GREEN once Arch #6 symbol pin lands
- Observability: GREEN

## Critic verdict
**ITERATE**

Iter-2 with all 8 Architect revs + 4 Critic findings + P3.5 refinement + ADR source-of-truth resolution + the two flagged open-questions noted-not-blocked → should land at APPROVE on the next pass.

## Critic signature
"Critic (oh-my-claudecode:critic) — 2026-05-12"
