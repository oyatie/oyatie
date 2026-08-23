---
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
doc_status: published
---


## Goal


**Layer 1 — Product-content SoT.** `oyatie/docs/` (CONSTITUTION → PRD → DESIGN → SPEC → ROADMAP → ADRs) is the canonical product authority. `bominal/docs/consolidated/PRD.md` is acknowledged as the portfolio-parent document via a bidirectional citation. The 2026-05-09 reframing (Workspace as Axis 2, Builder-OS → Foundry, in-house model substrate) is propagated to all stale ULTRAGOAL artifacts. The four OPEN ledger entries (LEDG-008, LEDG-017, LEDG-021, LEDG-024) remain open on their existing resolution batches — the direction shift is orthogonal to them.


## Constraints

4. **Clean Architecture dependency direction is preserved.** `kernel ← domain ← app ← {api, worker, adapter} ← runtime` per the flat-crates ADR-0015 target. Any new crate introduced by the cutover names itself `oyatie-<context>-<role>[-<capability>]` and respects the dependency direction.
7. **Linus-style discipline.** Delete bureaucracy that hides bad data structures. Eliminate special cases by reshaping the data, not by adding shims. Flat structure > deep hierarchy when the deep one is ceremony. "Good taste" means the simplest representation that handles all cases without branching. No half-finished implementations.
8. **bominal-to-oyatie boundary is explicit.** `oyatie/docs/PRD.md` cites `bominal/docs/consolidated/PRD.md` as portfolio parent; `bominal` references `oyatie` as the canonical implementation home for the seven-axis product. Cross-cite enforcement lands as a new fitness lane: `governance-portfolio-citation`.
9. **All four OPEN ledger entries stay open.** Direction shift does not force-close LEDG-008, LEDG-017, LEDG-021, LEDG-024. They continue on their existing resolution-batch ownership.

## Non-Goals

- Re-scoping the seven-axis EaaS product frame. The product definition survives the direction shift unchanged.
- Re-decomposing the foundry kernels. The 7 suspect fitness/policy kernels (`claim-ceiling`, `authority-cohesion`, `bypass`, `pr-traceability`, `pre-push`, `quality-lane`, `cohesion-fitness`) are not coordination kernels; they govern product-quality and survive.
- Reversing ADR-0025 (Builder-OS → Foundry consolidation) or any other 2026-05-09 reframing decision.
- Rewriting `~/.claude/CLAUDE.md` (user-machine config). The agentic-pipeline rules land in `oyatie/CLAUDE.md` and `oyatie/AGENTS.md` only, unless the user explicitly broadens the rule.

## Acceptance Criteria

The spec is complete when all of the following hold; each criterion has a typed verification path.

1. **A1 — Bidirectional PRD citation.** `oyatie/docs/PRD.md` cites `bominal/docs/consolidated/PRD.md` as portfolio parent; bominal cites oyatie as canonical implementation home. *Test*: new fitness lane `governance-portfolio-citation` passes on both sides.
4. **A4 — `agent-read` helper shipped.** A thin read-only CLI exposing `agent-read log <N>`, `agent-read diff <ref1> <ref2>`, `agent-read pr-view <num>`, `agent-read pr-comments <num>`. Read-only by construction; emits an audit-chain event per invocation. *Test*: invocation count appears in audit-chain query; mutation attempts (anything not in the read set) fail closed.
8. **A8 — All authoritative artifacts repo-tracked.** A repo-walk audit confirms that every file referenced as authoritative in `docs/AGENTS.md` is tracked. `.gitignored` paths that house authoritative state are either committed or demoted to non-authoritative. *Test*: `governance-authoritative-tracked` lane.
10. **A10 — Linus-style audit.** The cutover PR body section "good-taste audit" enumerates: (a) the special cases eliminated by reshaping data (e.g., `G004-reconciliation-blocker.md` no longer needs to exist), (b) the deep hierarchies flattened, (c) the ceremony deleted. Empty section is a fail. *Test*: PR template gate.

## Assumptions Exposed

3. **`agent-read` is implementable as a thin wrapper.** No exotic functionality — it shells out to read-only `git`/`gh` invocations and emits audit events.
4. **Bominal-to-oyatie boundary is parent-child, not peer.** Bominal owns portfolio; oyatie owns product. Confirmed by the PRD-SoT decision.
5. **The 2026-05-09 reframing is intended to stick.** Workspace as Axis 2, Builder-OS folded into Foundry, in-house model substrate added. The cutover does not revisit these.
6. **The seven `foundry-*-kernel` fitness/policy crates are correctly scoped.** Their continued existence is assumed; this spec does not propose merging or splitting them.
9. **Parallel-claim demo will use ≥2 agents on non-overlapping symbols** — this is the canonical demonstration that the new pipeline preserves the parallelism the user explicitly requested.

## Technical Context

### Repo topology (after cutover)

```
oyatie/
  Cargo.toml                       # flat-crates workspace, 140+ crates, unchanged
  crates/                          # kernel ← domain ← app ← {api, worker, adapter} ← runtime
  docs/                            # canonical product authority (CONSTITUTION, PRD, DESIGN, SPEC, ADRs)
  contracts/                       # per-cross-axis contract files (OpenAPI/Proto/AsyncAPI)
  registry/      # catalog + capability records
  scripts/                         # build/lint/release helpers (humans + sanctioned CI)
  tools/agent-read/            # NEW: sanctioned read-only helper CLI
  .omc/                            # OMC plans + state (session-scoped; .gitignored for state subdirs)
  .omx/                            # working state ONLY; nothing authoritative
  CLAUDE.md, AGENTS.md             # Redirect-class files pointing to docs/AGENTS.md
  README.md
bominal/
  docs/consolidated/PRD.md         # portfolio parent PRD; cites oyatie as canonical impl home
  agents/ultragoal/                # planning corpus (active artifacts only post-cutover)
```

### Agent flow (canonical sequence — session-less mode)

```
   # symbols must be real indexed code symbols (file::Identifier);
```


No agent step calls `git` or `gh` directly.

### Inventory classification scheme

Each row in the inventory ledger uses one of:

- `KEEP` — survives unchanged
- `KEEP+ANNOTATE` — survives; needs cross-cite or metadata added
- `REPLACE-WITH-HELPER` — read-side function moved into `agent-read`
- `DELETE` — removed; not recoverable except via git history

## Ontology

| Entity | Stable definition | Where it lives |
|---|---|---|
| **Oyatie** | One cohesive ecosystem-as-a-service across seven axes (SaaS, Workspace, Vertical, Foundry, Cloud, Search, Ads + Analytics). Single product. | `oyatie/docs/PRD.md` (canonical) ← `bominal/docs/consolidated/PRD.md` (portfolio parent) |
| **Foundry** | Axis 4: AI agent runtime + engineering platform + control plane. Unified per ADR-0025 (2026-05-09). Multi-provider adapter (Claude/OpenAI/Gemini, plus future in-house). | `oyatie/docs/DESIGN.md §3` |

## Ontology Convergence

The trace surfaced one ontological gap: Lane 3 enumerated five direction-shift dimensions (sequencing, taxonomy, axis count, regional, compliance) but the user's actual shift was a sixth — **agentic-pipeline mechanism**. The spec adopts "agentic-pipeline mechanism" as a first-class dimension and names it explicitly.

Lane 1 and Lane 2 had one factual disagreement (whether `oyatie/docs/PRD.md` exists). Lane 1's direct citation wins; Lane 2's structural-boundary thesis stands but the example is voided. The ontology resolves: oyatie/docs/PRD.md is canonical, bominal/docs/consolidated/PRD.md is portfolio parent.

No remaining entity-stability issues. The 7-axis EaaS frame is stable across all corpora and survives the direction shift.

## Trace Findings

**Leader hypothesis**: SoT-ownership / orchestration (Hypothesis 1, confidence High). The "no single source of truth" pain is structurally an ownership/orchestration boundary problem layered on a real-but-tracked contradiction backlog. The major direction shift absorbs the agentic-orchestration layer into upstream tools; product content survives untouched.

**Per-lane critical unknowns resolved**:
- Lane 1 (was the 2026-05-09 reframing Council-ratified): deferred — spec assumes ratified per Constraint 5; ADR check is part of inventory pass.
- Lane 2 (is oyatie sovereign or downstream of bominal): RESOLVED — oyatie sovereign, bominal portfolio parent, bidirectional cite required.

**Evidence that shaped the interview**:
- Foundry kernel inspection showed the suspect `foundry-*-kernel` crates are fitness/policy kernels, not coordination — they survive. The deletion target is the orchestration glue layer, not the foundry crates.
- The published `bominal/docs/consolidated/PRD.md` and the existing `oyatie/docs/PRD.md` use identical seven-axis language but did not cross-cite. Bidirectional citation closes that gap without merging the two.

**Trace path**: `.omc/scratch/deep-dive-trace-oyatie-sst-consolidation.md`

## Interview Transcript

User signals captured across the session, in order:

2. "Major shift in direction of the project so discuss with me in detail. I want you to come up with agentic development implementation plan."
6. "Make sure all the files, directories, and scripts are accounted for."
8. "We can also work in parallel in clean architecture. structured and organized. How linus torvald would approach."

Final ambiguity ≤ 20%. Spec gated on `/ralplan --consensus --direct` next.
