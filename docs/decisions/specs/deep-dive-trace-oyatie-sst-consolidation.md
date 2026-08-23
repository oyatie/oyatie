---
doc_status: published
---
# Deep Dive Trace: oyatie-sst-consolidation

<!--
status: Accepted
date: 2026-05-12
related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055
-->

Captured 2026-05-12. Source: `/deep-dive` Phase 3 synthesis with mid-trace direction-shift disclosures from the user folded in.

## Observed Result

The user owns two related repositories — `/Users/jasonlee/bominal/` (parent monorepo with extensive planning corpus) and `/Users/jasonlee/oyatie/` (child product/codebase). They want a single source of truth (SoT) for `oyatie` before applying a major direction shift, then to hand off to `/ralplan` + grit `claim → work → done`. Four doc paths were investigated: `bominal/agents/ultragoal/`, `bominal/docs/`, `oyatie/.omx/`, `oyatie/docs/`.

Mid-trace, the user disclosed the direction shift in five increments:

1. *"Major shift in direction of the project so discuss with me in detail. I want you to come up with agentic development implementation plan."*
2. *"Because of grit, we no longer have to over engineer the agentic pipeline of foundry (merge conflict no longer possible as long as agents follow grit protocol)."*
3. *"We can implement what works from https://github.com/rtk-ai/grit and https://github.com/rtk-ai/icm to our pipeline."*
4. *"agents should not use git at all. or gh. everything is through grit pipeline."*
5. *"Make sure all the files, directories, and scripts are accounted for."*

The direction shift is therefore not a product re-scoping; it is an **agentic-pipeline simplification + sanctioned-primitive lockdown**: grit becomes the only agent-side coordination/state-transition primitive, icm becomes the only cross-session knowledge primitive, and direct `git`/`gh` invocation by agents is banned.

## Ranked Hypotheses

| Rank | Hypothesis | Confidence | Evidence Strength | Why it leads |
|---|---|---|---|---|
| 1 | **SoT-ownership / orchestration** (Lane 2) is the structural problem | High | Strong | All 4 corpora declare authority but the bominal-parent → oyatie-child boundary is implicit; canonical-home symlink rule is declared in `ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md §0.2` but unenforced; orchestration glue (ledger.jsonl, codex-goal-*.json, omx ultragoal checkpoint) duplicates what grit's primitives already provide |
| 2 | **Contradiction / stale-premise** (Lane 3) is real but tracked | High | Moderate | Corpus has working machinery (CONTRADICTION-LEDGER 77 entries, MISTAKES-LEDGER, MFL fitness lanes); known contradictions are owned with SLAs; newly-discovered gaps (Workspace SPEC undercount, datacenter anti-scope editorial gap) are mechanical, not pathological |
| 3 | **Authoritative-content drift** (Lane 1) is minor — definitions agree | Medium | Weak-Moderate | All 4 corpora agree on "one cohesive flat-catalog EaaS (SaaS, Workspace, Vertical, Foundry, Cloud, Search, Ads + Analytics)"; divergences are sequencing/timeline (Foundry-second-vs-immediately-after-Foundation, in-house model timeline), not identity |

## Evidence Summary by Hypothesis

### Hypothesis 1 — SoT-ownership / orchestration
- Two authoritative-chain declarations co-exist with no precedence: `bominal/docs/source-of-truth.md` (portfolio router for the whole monorepo) and `oyatie/docs/CONSTITUTION.md` (project frame). Both declare identical chains `docs/ > catalog records > Redirect-class > drafts` but neither cites the other.
- Bominal `docs/consolidated/PRD.md` is the published portfolio-level PRD (97KB, 7 axes, brand "Oyatie", `oyatie.com` domain). Oyatie `docs/PRD.md` exists (Lane 1 quoted `:17-19`) and uses identical scope language — but neither cross-cites.
- Orchestration glue duplicates grit primitives. `bominal/agents/ultragoal/` carries 14+ artifacts that grit + icm subsume:
  - `goals.json` + 9× `codex-goal-*.json` → `grit claim --agent X --intent "…"` + `icm store -t goals-oyatie`
  - `ledger.jsonl` → `grit watch` event stream (room.sock) + grit lock state
  - `omx ultragoal checkpoint` / `complete-goals` → `grit done --agent X` + `icm store -t context-oyatie`
  - `G004-reconciliation-blocker.md` → does not exist under grit (no objective-state to mismatch)
  - `PAUSE.md` → not a grit verb; agent halts via `release` or TTL expiry
  - `.codex/worktree_init.sh` → `grit worktree` primitive
- Inspection of seven suspect `foundry-*-kernel` crates (`claim-ceiling`, `authority-cohesion`, `bypass`, `pr-traceability`, `pre-push`, `quality-lane`, `cohesion-fitness`) shows they are **product-quality fitness/policy kernels**, not agent-coordination kernels — they survive the grit pivot. The "over-engineered" surface is the orchestration glue layer in `bominal/agents/` and the codex/omx wrappers, not the in-tree foundry crates.

### Hypothesis 2 — Contradiction / stale-premise
- `CONTRADICTION-LEDGER.md` carries 77 contradictions across BLOCKER/HIGH/MED/LOW with explicit owners and resolution batches. `MISTAKES-LEDGER.md` carries 13 active mistakes each backed by a CI fitness lane.
- Known OPEN BLOCKER/HIGH entries: LEDG-008 (5-6 vs 7 axis history), LEDG-017 (lifestyle/consumer scope), LEDG-021 ("no ads, ever" vs ads microservice), LEDG-024 (Korea regulatory gaps).
- Newly-discovered (not in existing ledger):
  - `SPEC.md §4` enumerates only 10 of the 14 Workspace surfaces declared in `DESIGN.md §1` (forms, sites, tasks, notes, translate, recordings missing).
  - `PRD.md §1 line 31/39` says owning datacenters is in-scope at W-DataCenter-Operations (updated 2026-05-09), but `PRD.md §3.3` anti-scope table still says "Hardware / data-center construction (always — leased always)" — same document contradicts itself.
  - `bominal/agents/ultragoal/oyatie-product-delivery-implementation-plan.md §3.4` prescribes nested Cargo-layer dependencies; `oyatie/docs/DESIGN.md §3.0.5` + ADR-0015 prescribes flat-crates. Orthogonal but uncited.
  - `bominal/agents/ultragoal/oyatie-product-delivery-baseline.md` counts "6 axis PRDs"; current OYATIE-DOCS asserts 7 axes (Workspace added 2026-05-09). ULTRAGOAL baseline is stale on axis inventory.
- ULTRAGOAL planning artifacts are tightly versioned with 2026-05-12 timestamps; the 2026-05-09 OYATIE-DOCS reframing (Workspace as Axis 2, Builder-OS → Foundry, in-house model substrate added) has not propagated to all ULTRAGOAL artifacts.

### Hypothesis 3 — Authoritative-content drift
- Unified definition across OYATIE-ROOT, OYATIE-DOCS, OYATIE-OMX, and ULTRAGOAL: *"one cohesive ecosystem-as-a-service across SaaS, Workspace, Vertical, Foundry, Cloud, Search, and Ads + Analytics."*
- BOMINAL-DOCS does not define oyatie; it routes oyatie questions back to oyatie/docs via `source-of-truth.md`. Bominal-as-parent owns portfolio strategy (healthcare/corporate/fintech/hospitality arms) — not the oyatie product frame.
- Real divergences: ULTRAGOAL baseline says "20 product PRDs (6 axis + 14 vertical)"; OYATIE-DOCS PRD says "one product across all microservices" with 14 verticals *within* the Vertical axis. This is a taxonomy-granularity drift, not an identity drift.
- Sequencing drift: OYATIE-DOCS ROADMAP gates Foundry on Foundation completion; ULTRAGOAL brief sequences "Phase 00 Foundry self-hosting account-auth bundle" *immediately after* Foundation ADRs accept. Both interpretations land within DESIGN §3 "Foundry is second, not first." Not a fundamental conflict.

## Evidence Against / Missing Evidence

### Against Hypothesis 1
- Both repos declare matching authority chains (validated by `governance-authority-cohesion` lane per `oyatie/docs/CONSTITUTION.md:114`). Authority is not in conflict, only un-cross-referenced.
- `RACI-OWNERSHIP.md` exists in oyatie and explicitly maps ownership — the boundary is not undefined, only weakly cross-cited.
- The "ownership leak" Lane 2 flagged (oyatie/docs/PRD.md not existing) was incorrect — Lane 1 verified the file exists and contains the canonical flat-catalog definition.

### Against Hypothesis 2
- Contradictions are tracked. Auto-emit machinery exists (`governance-cohesion` cross-microservice drift detector emits `EVT-CROSS-AXIS-CONTRADICTION-FOUND`). Resolution batches assign owners + waves.
- 4 of 6 newly-discovered contradictions are mechanical/known; 2 are honest gaps in documentation completeness, not logical contradictions.
- Q-NEW pattern explicitly marks unresolved questions; the corpus acknowledges what it doesn't know.

### Against Hypothesis 3
- No internal contradiction about identity. The 2026-05-09 consolidation (Workspace added, Builder-OS folded into Foundry, in-house model production added) is the single change event explaining most divergences. ULTRAGOAL artifacts pre-2026-05-09 are merely stale on the reframing, not conflicting.

### Missing evidence (could not resolve in trace window)
- Whether an Architecture-Council ADR ratifies the 2026-05-09 consolidation (Lane 1's critical unknown) — `ADR-INDEX.md` not exhaustively scanned in this lane.
- Whether `oyatie/.omx/ultragoal/` is supposed to be a symlink to `bominal/agents/ultragoal/` per the §0.2 canonical-home rule, or an independent workspace.
- Full content of `bominal/docs/consolidated/PRD.md` — Lane 2 cited it as 97KB published portfolio PRD but read only metadata; relation to `oyatie/docs/PRD.md` not exhaustively reconciled.

## Per-Lane Critical Unknowns

- **Lane 1 (Authoritative-content)**: Whether the 2026-05-09 DESIGN consolidation (Workspace as Axis 2, Builder-OS → Foundry, in-house model production) has been formally ratified by the Architecture Council via an Accepted ADR, or remains aspirational direction pending Council review.
- **Lane 2 (SoT-ownership)**: Whether Oyatie operates as a sovereign project with its own canonical PRD/DESIGN/SPEC, or as an implementation of Bominal's portfolio strategy (consuming `bominal/docs/consolidated/PRD.md` as upstream).
- **Lane 3 (Contradiction / stale-premise)**: ~~What is the user's major direction shift?~~ **RESOLVED** mid-trace: the shift is agentic-pipeline simplification, not product re-scoping. Residual unknown: which OPEN ledger entries (LEDG-008/017/021/024) does the direction shift force-close vs leave open?

## Lane 3 Misplacement / SoT Ownership Scope

Per the deep-dive skill, every MOVE candidate must be classified by `ownership_scope` with cross-boundary flags. Lane 2 surfaced 7 cross-boundary candidates; this synthesis enriches them with the grit-pivot lens (what the candidate becomes under the simplification).

| Source | Candidate destination | ownership_scope (src → dst) | Boundary relationship | Default? | Warning |
|---|---|---|---|---|---|
| `bominal/agents/ultragoal/oyatie-product-delivery-implementation-plan.md` | `oyatie/docs/IMPLEMENTATION-PLAN.md` (with bidirectional cite to bominal upstream) | shared-config → project-scoped | cross-boundary | **no** | Source plan is parent-org planning corpus; copying without bidirectional cite breaks single-author authority. Prefer KEEP-IN-PLACE + add forward-ref from `oyatie/docs/README.md`. |
| `bominal/agents/ultragoal/latest-source-register.md` | `oyatie/docs/REGULATORY-SOURCING.md` | shared-config → project-scoped | cross-boundary | **no** | Regulatory sourcing is jurisdictional-pack input that may apply to other Bominal products too. Prefer COMPRESS into a thin oyatie pointer doc + KEEP-IN-PLACE upstream. |
| `oyatie/docs/raw/agentic-delivery-fabric-executable-prd.md` | promote to `oyatie/docs/AGENTIC-PIPELINE.md` OR move to `bominal/docs/consolidated/` | project-scoped (draft) → project-scoped (canonical) | same-scope | yes (promote in-place) | Working draft already cites `repo: bominal`. Under the grit pivot this doc becomes ground-zero for the new agentic-pipeline spec — promote in oyatie, do not move to bominal. |
| `oyatie/.omx/ultragoal/` | symlink to `bominal/agents/ultragoal/` per §0.2 canonical-home rule | personal-config (session state) → shared-config (canonical planning) | cross-boundary | **no** | Session-scoped working state. Under the grit pivot the right answer is DELETE: grit's worktree + lock + done lifecycle replaces `.omx/ultragoal/` session state. Do not symlink; retire. |
| `bominal/agents/ultragoal/{ledger.jsonl, goals.json, codex-goal-*.json, G004-reconciliation-blocker.md, PAUSE.md}` | archive then delete | shared-config (active orchestration) → archive | same-scope (within bominal) | yes (archive default; delete after grit cutover verified) | These are the orchestration-glue deletion targets. Archive under `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/` so history is preserved, then delete from active path. |
| `oyatie/.codex/worktree_init.sh` (if exists) and any equivalent codex/gemini per-agent init scripts | delete | personal-config (per-agent init) → removed | same-scope | yes | Replaced wholesale by `grit worktree` + `grit claim`. Inventory pass must enumerate every such script before deletion. |
| `oyatie/CLAUDE.md` RTK section + `~/.claude/CLAUDE.md` RTK section (as they apply to agent instructions) | rewrite to remove agent-side `rtk git`/`rtk gh` references | personal-config / shared-config → updated | **flagged for user decision** | RTK is the user's personal token-optimization layer. Banning agent-side `rtk git` is project-level; banning all `rtk git` in `~/.claude/CLAUDE.md` would extend the rule to every project the user owns. Default: edit oyatie/CLAUDE.md only; leave global RTK as-is unless user explicitly broadens the rule. |

**Cross-boundary rule audit**: every cross-boundary candidate above has `Default? = no` per the skill specification, with an explicit warning. The personal-config `~/.claude/CLAUDE.md` is flagged for user decision rather than auto-edited.

## Rebuttal Round

Leader: **Hypothesis 1 (SoT-ownership / orchestration)**. Strongest rebuttal: Hypothesis 2 (Contradiction / stale-premise).

- **Rebuttal**: "The corpus already has working machinery to detect and track contradictions (CONTRADICTION-LEDGER + MFL lanes). The structural-boundary problem Lane 2 flags is a symptom of in-flight 2026-05-09 reframing, not a permanent design defect. Therefore Hypothesis 2 is more load-bearing than Hypothesis 1."
- **Counter**: The direction shift the user disclosed is *agentic-pipeline simplification*, not a content reframe. Under that shift, the orchestration glue (Hypothesis 1 evidence) is the deletion target — `ledger.jsonl`, `codex-goal-*.json`, omx ultragoal checkpoint flow, `.codex/worktree_init.sh`. The contradiction ledger (Hypothesis 2 evidence) is *preserved* because contradictions are product-quality concerns, which survive the grit pivot. Therefore Hypothesis 1 leads under the disclosed direction shift; Hypothesis 2 is real but orthogonal.
- **Leader holds.** The deletion footprint of the direction shift maps precisely to Hypothesis 1's evidence set; Hypothesis 2's evidence set is unaffected by the shift.

## Convergence / Separation Notes

- **Lane 1 + Lane 2 disagree on one fact**: Lane 2 claimed `oyatie/docs/PRD.md` does not exist; Lane 1 quoted `oyatie/docs/PRD.md:17-19` verbatim. Direct citation wins — the file exists. Lane 2's "boundary opacity" thesis stands but the specific example is voided.
- **Lane 2 + Lane 3 converge** on the bominal→oyatie boundary as a fragility source. Lane 2 attributes it to weak cross-cite enforcement; Lane 3 attributes it to in-flight reframing leaving older ULTRAGOAL artifacts stale. Both diagnoses are correct and complementary.
- **All three lanes agree** that the 7-axis EaaS product frame survives any direction shift currently disclosed. The shift is about *how agents work the codebase*, not what the codebase produces.
- **Direction-shift convergence**: Lane 3 originally listed five possible direction-shift dimensions (sequencing, taxonomy, axis count, regional, compliance) and asked which applied. The user's actual shift fits NONE of those — it is a sixth dimension Lane 3 did not enumerate: **agentic-pipeline mechanism**. This is a genuine ontological gap in the Lane 3 framing; the trace synthesis adds it as the operative dimension.

## Most Likely Explanation

The "no single source of truth for oyatie" pain is **structurally an ownership/orchestration boundary problem (Hypothesis 1)** layered on a real-but-tracked contradiction backlog (Hypothesis 2). The major direction shift the user is about to apply does not touch product content — it absorbs the agentic-orchestration layer into upstream tools (`rtk-ai/grit` for coordination, `rtk-ai/icm` for cross-session knowledge). Under the shift, much of the duplicated authority surface (oyatie's `.omx/ultragoal/`, bominal's `agents/ultragoal/` orchestration glue, codex per-agent init scripts) becomes deletion target; the in-tree foundry fitness/policy kernels survive because they govern product quality, not agent coordination.

Net: the consolidation deliverable is **two layers**, not one:

1. **Product-content SoT layer** (Hypotheses 2 + 3): one canonical PRD/DESIGN/SPEC pair across the bominal-oyatie boundary, with the 2026-05-09 reframing propagated to ULTRAGOAL artifacts, and the OPEN ledger entries triaged against the direction shift to mark which close and which remain.
2. **Agentic-pipeline layer** (Hypothesis 1 under the direction shift): one canonical operating contract that names grit + icm as the only sanctioned primitives, bans agent-side git/gh, lists every deletion target with an archive→retire plan, and rewrites the oyatie `CLAUDE.md`/`AGENTS.md` to reflect the new rule.

## Critical Unknown

After the trace + direction-shift disclosure, one unknown dominates: **what is the agent-side read-side surface under the no-git/no-gh rule?** Specifically:

- `grit symbols` enumerates code symbols; can it answer "what files changed in the last claim" the way `git diff` does?
- `grit show-session` / `grit watch` exist but their read-path semantics for "give me the state of N concurrent agents right now" are not yet specified.
- For history operations agents currently use `git log` for (e.g., a debugger agent reading the last 20 commits to find a regression), grit has no equivalent — does the answer come from icm `recall -t commits-oyatie`, or does the agent invoke a *non-agent* helper that wraps `git log` and returns the result?
- For PR-equivalent operations agents currently use `gh pr create` for, `grit done` is the closest primitive — does `grit done` create a PR on a remote, or does it merge locally and leave PR creation to a separate non-agent flow?

These details are necessary inputs to the Phase 4 spec; without them, the no-git/no-gh rule cannot be operationalized.

## Recommended Discriminating Probe

Two probes that should run before Phase 4 spec crystallization:

1. **Read `grit done --help`, `grit watch --help`, `grit symbols --help`, `grit session --help`, `grit show-session --help` (if present), and `grit assign --help` in full to map the agent read-side surface.** Document which `git`/`gh` read-side calls are absorbed natively vs which require a non-agent helper wrapper.
2. **Ask the user the three Phase-4-gating questions in one batch**:
   - "Is `bominal/docs/consolidated/PRD.md` the Oyatie product north star (Oyatie consumes it as upstream), or is `oyatie/docs/PRD.md` the canonical and Bominal's is a legacy artifact?"
   - "For agent read-side surface where `grit` has no equivalent (e.g., `git log`-style commit-history queries), is the answer: (a) icm `recall -t commits-oyatie`, (b) a sanctioned non-agent helper, or (c) extend grit with the missing read primitive?"
   - "Of the OPEN ledger entries (LEDG-008 axis count, LEDG-017 consumer scope, LEDG-021 Connect-no-ads, LEDG-024 Korea regulatory gaps), which does this direction shift force-close, and which remain in flight?"

These two probes collapse the residual uncertainty enough to crystallize the Phase 4 spec with ambiguity ≤ 20%.
