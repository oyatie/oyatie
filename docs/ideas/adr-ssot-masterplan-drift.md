# ADR-as-SSOT in the Masterplan + Continuous Drift Check

> Idea-refine output, 2026-05-31. Founder directives this session: "modern hyperscaler approach for
> our every adr as well as supersession / drift check" + "SSOT adr >> adr into masterplan" +
> (Q answers) catch all 4 drift classes; accelerate ADR->JSON registry; flag + auto-propose-fix;
> both scheduled + on-authoring cadence.

## Problem Statement

**How might we** keep every oyatie ADR continuously truthful — its lifecycle state, its supersession
links, AND its alignment with the actual code/specs/deployment — automatically, so the decision
record never silently drifts from reality as the autonomous fan-out builds out the platform?

The decision record is a **safety property of the closed-loop dogfood**: fan-out agents read ADRs as
ground truth, so a drifted ADR makes an agent build the wrong thing confidently. Today decisions
(`docs/decisions/*.md`) and the plan (`specs/masterplan.json`) are two disconnected SSOTs linked only
by loose `related_specs:` strings — that seam is where drift hides.

## Recommended Direction

**Fold the ADR corpus into the masterplan as one queryable decision+plan graph, and run a layered
drift-check (deterministic gate + LLM sweep) against that single graph — auto-fixing the mechanical
violations and flagging the semantic ones.**

Three layers, increasing power:

1. **SSOT projection (not a risky migration).** Markdown ADRs stay human-authorable, but a generated
   `/registry/adrs.json` (+ per-ADR content JSON) projects their structured frontmatter into the
   masterplan graph as **decision-nodes** linked to **plan-items**. The masterplan becomes the single
   queryable source; markdown becomes a rendered/authoring view. Aligns with `markdown-retirement-policy`
   PHASE-4 (already sanctioned). Full markdown->JSON migration is the *destination*, not the MVP.

2. **`governs:` code-anchors — the killer field.** A new frontmatter field naming the code/spec a
   decision controls turns ADR<->implementation drift from an LLM guess into a mechanical diff:
   `ADR-0336 governs depends('redis')==0`; `ADR-0515 governs oci BUCK base==static-debian12`.
   The lifecycle gate requires new ADRs to declare anchors; legacy ones acquire them lazily when touched.

3. **Layered drift detection** over the graph:
   - **Deterministic (Rust check crate, every PR):** L1 status-vocab FSM {Proposed|Accepted|Superseded|
     Rejected|Deprecated}; L2 terminal-status requires `superseded_by`; L3 reciprocity; L4 dangling refs;
     L5 hollow-superseded -> archive-candidate; **L6 `governs:` anchor satisfied by code** (where the
     assertion is grep/diff-checkable).
   - **Semantic (LLM, scheduled + on-authoring):** ADR<->ADR conflict (two live decisions clash, no
     supersede) + ADR<->code drift where the assertion needs reading. On a PR touching `docs/decisions/`,
     an agent proposes supersession links + flags conflicts inline. A weekly cron sweep posts a
     full-corpus drift report.

**Enforcement = flag + auto-propose-fix.** Mechanical violations (reciprocity, status flip, missing
links, status-vocab) get an auto-generated commit on the PR for human approval; semantic/drift findings
get a posted report. Hard-gate is deferred until the 347-ADR backlog is clean (else it blocks everything).

## Key Assumptions to Validate
- [ ] Projection (generated `/registry/adrs.json` into the masterplan) delivers the queryable-graph value
      without the full 347-file markdown->JSON migration. *Test:* build the projection, run 5 real drift
      queries (Valkey, base-image, http-stack, container-tooling, jenkins-retirement) against it.
- [ ] `governs:` anchors are cheap enough to author that decision-authors actually write them. *Test:*
      retrofit anchors on ~10 high-value ADRs; measure effort + how many real drifts L6 catches today.
- [ ] Auto-propose supersession (LLM, on-authoring) has a low enough false-positive rate to be trusted.
      *Test:* replay it over the ADRs we superseded this session; measure precision vs the human calls.
- [ ] One graph holding plan-items + decision-nodes stays legible (doesn't become a monolith). *Test:*
      keep them as two node types; confirm masterplan.json size + a sample plan-item->decisions query.

## MVP Scope
**In:**
- The L1-L5 deterministic lifecycle gate (the design already produced) as lane `adr-lifecycle`, run
  foreground, buck2-verified. Backfill the corpus audit's auto-fix set so it goes green.
- The decision-node SCHEMA: canonical status FSM + `supersedes`/`superseded_by` + `governs:` +
  `plan_items:` + `last_validated:` (frontmatter-as-schema; body stays prose).
- The SSOT **projection**: extend the Rust/Buck2 ADR index regenerator to emit `/registry/adrs.json` + masterplan links.
- L6 `governs:` drift on ~10 high-value ADRs (Valkey, base-image, http-stack, container-tooling, ...).
- Reuse the corpus-audit workflow as the **scheduled weekly sweep** (cron) + a thin **on-authoring**
  PR variant.

**Out (deferred):**
- Full 347-file markdown->JSON migration (project instead; migrate later).
- Hard-gate enforcement (flag + auto-propose first; flip to blocking once corpus is clean).
- `governs:` anchors on all 347 ADRs (lazy: new + touched only).

## Not Doing (and Why)
- **Not** making `specs/masterplan.json` a 347-entry blob — decisions are a separate node *type* in the
  graph, linked, not inlined. Avoids the monolith.
- **Not** hard-gating on day one — would block every PR until 347 legacy ADRs are backfilled. Flag +
  auto-propose buys the cleanup runway.
- **Not** LLM-judging what static rules can decide — semantic sweep only for genuine conflict/code-drift;
  everything mechanical stays in the cheap deterministic gate (DRY + cost).
- **Not** authoring `governs:` retroactively for the whole corpus — anchor rot would make the drift-check
  itself drift; lazy adoption keeps anchors live.

## Open Questions
- This decision is itself an ADR (ADR-as-SSOT-in-masterplan). Author it + fold it into the masterplan as
  the first decision-node — proving the loop on itself.
- Where does the prose body live post-projection — `body_markdown_archived_at` (git history per the
  retirement policy) or a rendered `/registry/adr-content/*.json`?
- Cron substrate for the weekly sweep: the oya-ci controller (once deployed) vs a standalone scheduled job.
