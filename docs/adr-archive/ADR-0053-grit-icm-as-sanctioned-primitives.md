---
id: ADR-0053
status: Superseded
doc_status: published
date: 2026-05-12
owners:
  - council-architecture
  - foundry
supersedes: []
superseded_by: [ADR-0116, ADR-0363, ADR-0515]
doc_class: DecisionRecord
purpose: >
  Fix the agent-callable coordination/state-transition primitive set at
  {grit, icm, oya-tooling-agent-read}. Direct git/gh permitted only with
  documented rationale per Directive 12.
planned_enforcement_ref: oya-governance-banned-primitives
supersession_note: "Dead grit/icm toolchain mandated live; superseded by ADR-0116 (retire external agent-coordination tooling), ADR-0363 (agentic-VCS retired), ADR-0515 (canonical CI/CD). D-DISPOSITIONS-RATIFIED: SUPERSEDE-9-clean, C-6."
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0053: grit + icm + oya-tooling-agent-read as sole sanctioned primitives for agentic work

- **Status:** Accepted
- **Date:** 2026-05-12
- **Owners:** `council-architecture`, `foundry`
- **Supersedes:** (none)
- **Superseded by:** (none)
- **Planned enforcement:** `oya-governance-banned-primitives` remains advisory until the lane exists.
- **Siblings landing in parallel:** ADR-0052 (pre-grit artifact inventory), ADR-0054 (grit scaffold-claim pattern)
- **Operational driver:** [`.omc/plans/SST consolidation ralplan`](../../.omc/plans/SST consolidation ralplan)
- **Operating contract shaped:** [`docs/AGENTS.md`](../AGENTS.md) §Sanctioned primitives

---

## Context

Prior to this ADR, agents invoked `git`, `gh`, and bespoke in-tree orchestration glue (`omx ultragoal checkpoint`, `codex-goal-*.json`, `ledger.jsonl`, `G004-reconciliation-blocker.md`, `PAUSE.md`) to coordinate work and transition state. This produced a class of reproducible failures: state-machine mismatches (e.g. LEDG-008, MFL-0010-style stub residue), non-audited side-effects, and "two agents touched the same function" merge conflicts. The ralplan iter-2 consensus (Planner + Architect + Critic) closed the alternative set and approved the three-primitive model described here.

---

## Status

Accepted — consensus reached iter-2 via Planner+Architect+Critic.

## Date

2026-05-12

## Decision

The set of agent-callable coordination and state-transition primitives is fixed at exactly three: `grit` (upstream `rtk-ai/grit`), `icm` (upstream `rtk-ai/icm`), and `oya-tooling-agent-read` (a thin, read-only, audit-emitting helper shipped at `tools/oya-tooling-agent-read/`). No agent execution path may invoke `git` or `gh` directly. Adding to the sanctioned set requires a successor-IP ADR with explicit justification of why neither `grit` nor `icm` already covers the operation.

### Cutover bootstrap window (P0.5 – P2)

The sanctioned-primitive set above is the **steady-state contract**, valid from the moment the banned-primitives fitness lane activates at the P5 merge boundary. During the **cutover bootstrap window** — phases P0.5 (scaffold-claim ADR), P1 (inventory ADR), P2 (ship `oya-tooling-agent-read`) — the helper does not yet exist, so the temporarily-effective sanctioned set is `{grit, icm}` plus an explicit carve-out for read-only `git`/`gh` invocations strictly to inspect prior helper scaffolds, prior tooling crates, and existing repo state. Every such bootstrap-window invocation is recorded in the inventory ledger as a one-time bootstrap row (column: `bootstrap_window: true`, `invocation: <command>`, `purpose: <one-line>`, `agent_or_human: <id>`). The banned-primitives fitness lane (`oya-governance-banned-primitives`) is **defined** in P4 (rewrite agent memory) but **activated** at P5 merge — between P4 and P5 the lane exists, the rewrite exists, but enforcement is off. The lane goes live with the P5 merge commit; from that commit forward, the carve-out is closed.

## Decision Drivers

- **Eliminate merge-conflict-as-failure-mode**: `grit` symbol-locking guarantees that parallel agents cannot stomp each other's work, removing the entire class of "two agents touched the same function" defects.
- **Eliminate orchestration-glue rot**: every pre-cutover artifact at `bominal/agents/ultragoal/{ledger.jsonl, goals.json, codex-goal-*.json, G004-reconciliation-blocker.md, PAUSE.md}` and `omx ultragoal checkpoint/complete-goals` exists to solve coordination problems `grit` solves natively. Deleting the glue removes ~12 artifacts of bespoke state-tracking that drifts from reality (LEDG-008-style premise mismatches, MFL-0010-style stub residue).
- **Cross-session knowledge persistence is a separate concern**: `icm` already solves it. Building a second memory layer in-tree (project-memory.json, notepad.md as authority, etc.) duplicates a working upstream tool.
- **Linus-style discipline**: reshape the data structures so special cases disappear, rather than layering shims. The "agent goal mismatch detection" subroutine in `omx ultragoal checkpoint` is a special case that exists only because the goal state was held in a separate file from the claim state. Under `grit`, claim = intent, so the special case vanishes.
- **Bidirectional audit traceability**: every `grit claim` is an event; every `icm store` is an event; every `oya-tooling-agent-read` invocation emits to the audit chain. Direct `git`/`gh` from agents leaves no equivalent trail — the rule closes that observability gap.
- **Provider-agnostic posture by construction**: `oya-tooling-agent-read` wraps `gh` today (GitHub CLI) — but the abstraction it exposes (`pr-view`, `pr-comments`, `log`, `diff`) is provider-neutral; the GitHub-specific code is isolatable to a single adapter so a future GitLab/Bitbucket/Gitea path is a swap, not a rewrite. This aligns with the project-wide provider-agnostic principle (see `oyatie/docs/CONSTITUTION.md` and the Master Plan §Principles).
- **Final-shape adoption**: the three sanctioned primitives are the **end-state** for steady-state operations, not a prototype. We do not ship a "v0.1 helper with write verbs" intending to remove them later; we ship the read-only-only surface from day one and require an ADR to extend.

## Glossary

### Human orchestrator
The named individual(s) authorized in `oyatie/docs/RACI-OWNERSHIP.md` (row `human-orchestrator-cutover` added as part of P0.5) to perform the three carve-out invocations (`git mv`, `git rm`, `gh issue create`) during cutover phases P6, P7, and P9. Every carve-out invocation by a human orchestrator is recorded via `icm store -t cutover-orchestrator-actions -c '<action>' -i critical` **BEFORE** execution; the icm row is the audit trail. Orchestrator authorization does not grant general `git`/`gh` privileges — it is scoped to the named cutover phases and their specific commands. Steady-state agent operations never invoke this carve-out.

## Alternatives Considered

### Alternative A — Status quo (in-tree orchestration glue continues)
Keep `omx ultragoal checkpoint`, `codex-goal-*.json`, `ledger.jsonl`, `G004-reconciliation-blocker.md`, etc. Layer agent skills directly on `git`/`gh`. **Rejected because**: the orchestration glue is bespoke, untested at the data-model layer (the `G004-reconciliation-blocker.md` artifact is literal evidence of a state-machine bug — `omx` couldn't reconcile a paused vs. unknown goal state, so the agent halted), and every agent-side `git checkout`/`git rebase`/`gh pr create` is an opportunity for divergent local convention. The user's directive ("agents should not use git at all. or gh. everything is through grit pipeline") closes this alternative explicitly.

### Alternative B — grit + icm only; no helper CLI
Ban `git`/`gh` and refuse to provide any escape hatch. Agents have only what `grit` and `icm` natively expose. **Rejected because**: some agent workflows genuinely need read-side operations grit does not cover today (e.g., debugger reading the last 20 commits to find a regression, code-reviewer reading PR comment threads). Refusing the escape hatch either blocks those workflows or pushes agents toward shadow-tool invocation (e.g., `bash -c "git log..."` smuggled through), which is worse than a sanctioned, audited helper. The helper is small, read-only, and audit-emitting — Linus-acceptable, not bureaucracy.

### Alternative C — Build a custom Oyatie-internal coordination tool
Fork or rewrite the coordination layer inside the oyatie repo (e.g., `crates/oya-intelligence-agent-coordinator-kernel/`). **Rejected because**: it duplicates `grit`. The user explicitly noted "we can implement what works from rtk-ai/grit and rtk-ai/icm to our pipeline" — the recommendation is integration, not fork. A custom in-tree coordinator would also need its own maintenance, fitness lanes, and ADRs — exactly the over-engineering this direction shift removes.

### Alternative D — P4-before-P2 (rule-first sequencing)
Rewrite agent-facing memory FIRST (P4), ship `oya-tooling-agent-read` SECOND (P2). **Rejected because**: while it produces a self-consistent rule on day one of P4, it then leaves the rule unenforceable until P2 lands the helper that the rewrite cites. P2-first with the explicit bootstrap-window carve-out (this ADR §Decision §"Cutover bootstrap window") makes the contradiction honest rather than hidden, and preserves rollback-boundary cleanliness. See ralplan iter-1 Architect review §Steelman antithesis for full reasoning.

## Why Chosen

- Maps cleanly to spec acceptance criteria A1 (cross-cite), A2 (inventory), A3 (archive+delete), A4 (helper CLI), A5 (rewrite agent memory), A6 (hook+skill audit), A7 (parallel-claim demo), A8 (authoritative-tracked invariant), A9 (plan handoff), A10 (Linus-audit).
- Preserves the flat-catalog product frame; no product-content change.
- Preserves the flat-crates clean architecture (`kernel ← domain ← app ← {api, worker, adapter} ← runtime`).
- Preserves the 140+ existing `oya-*` crates and the in-tree fitness/policy kernels (`claim-ceiling`, `authority-cohesion`, `bypass`, `pr-traceability`, `pre-push`, `quality-lane`, `cohesion-fitness`) — they govern product-quality, not agent-coordination, and survive the cutover.
- Leaves all four OPEN ledger entries (LEDG-008, LEDG-017, LEDG-021, LEDG-024) on their existing resolution batches; the direction shift is orthogonal to product-scope decisions.
- Adopts the **final shape from day one** per the user's final-shape directive: the three-primitive sanctioned set is the end-state, the helper's surface is the end-state, the no-`git`/no-`gh` rule is the end-state.

## Consequences

### Positive
- One claim-event per agent per task; one audit-chain emission per claim. Full traceability without bespoke ledger files.
- Parallel agent work is first-class — symbol-locking guarantees safe parallelism, which makes `/ultrawork` and `/team` strictly more useful.
- Reading `~/.claude/CLAUDE.md` no longer ambiguously prescribes `rtk git` to agents (it stays for human terminal use); agent CLAUDE.md sections are scrubbed.
- The new `oya-tooling-agent-read` helper is testable, auditable, and minimal — it never mutates anything.
- Provider-agnostic abstraction surface: `pr-view`/`pr-comments`/`log`/`diff` are GitHub-CLI-agnostic verbs, swappable to other forges via a single adapter.

### Negative
- Upstream dependency on `rtk-ai/grit` 0.3.0+. The `grit session start` bug (documented in `.omc/scratch/pre-cutover-drafts-2026-05-12.md §Draft 1`) is a real blocker for the `grit session pr` flow until fixed upstream. Workaround is **session-less mode** documented in the spec.
- New-crate creation has a chicken-and-egg with grit symbol-locking (you cannot claim a symbol that doesn't exist yet). Resolved by the **scaffold-claim pattern** in `ADR-0054-grit-scaffold-claim-pattern.md` (lifts from `.omc/scratch/pre-cutover-drafts-2026-05-12.md §Draft 2`).
- Agents lose the convenience of `rtk gh pr view` etc.; they must route through `oya-tooling-agent-read pr-view`, which is one indirection.
- The `oya-governance-banned-primitives` lane must scope its grep to agent-instruction sections only (via HTML comment fences `<!-- agent-instructions:start -->`) — non-trivial to implement correctly.

### Neutral
- ICM external storage holds cross-project memory; project-canonical decisions are still duplicated into `oyatie/docs/` as tracked files (per spec §Constraints item 2). This is a pre-existing design; this ADR does not change it.
- The `RTK` instructions in `~/.claude/CLAUDE.md` (user-machine global) are unchanged. Human terminal usage of `rtk git`/`rtk gh` is unaffected.
- **The cutover itself runs under a one-time human-orchestrator carve-out** for `git mv` (P6 archive), `git rm` (P7 delete), and `gh issue create` (P9 upstream-bug filing). Post-cutover, the agent lifecycle is `grit claim → work → grit done` for every operation; the cutover commits themselves are not retroactively flowed through `grit done`. A future architect reading the cutover commits will see no `grit done --agent` references for those specific landing commits — this is intentional, by-design, and documented here so the absence is not misread as a broken model.
- `oya-tooling-agent-read` wraps `gh` (GitHub CLI) today; the provider-agnostic principle implies a future PR-provider abstraction layer (Follow-up #6) if oyatie moves off GitHub or supports multiple forges. Out of scope for this cutover; tracked as Follow-up.

## Compounding principles incorporated by reference

This ADR is consistent with the following Master-Plan-level principles; they shape its scope and the helper's surface:

1. **Provider-agnostic**: `oya-tooling-agent-read`'s surface is verb-level (`pr-view`, `log`, `diff`), not provider-level. GitHub-specific code lives behind the verb implementation; a future second forge gets its own implementation behind the same verb. See `oyatie/docs/CONSTITUTION.md` and Master Plan §Principles.
2. **Distroless + smallest-image**: when `oya-tooling-agent-read` is containerized (for CI runners or sandboxed agents), the image is distroless (`gcr.io/distroless/static-debian12` for the static Rust binary) with `cargo build --release` + musl static linking. CI gates the image size budget. See Master Plan §Cross-cutting workstreams §Image discipline.
3. **Current LTS dependencies enforced**: every direct dependency of `oya-tooling-agent-read` is pinned to current LTS (verified against `.omc/scratch/lts-versions-verified-2026-05-12.md`); the `oya-governance-lts-dependency` lane (defined in Master Plan §Cross-cutting workstreams §Dependency hygiene) blocks PRs that introduce non-LTS deps without an exception ADR.
4. **Final-shape adoption**: the helper's read-only-only verb set, the three-primitive sanctioned set, and the bootstrap-window-as-explicit-one-time-carve-out are all final-form decisions, not iterative ones.

## Follow-ups

1. **File upstream grit bug** for `grit session start` at `rtk-ai/grit` (draft at `.omc/scratch/pre-cutover-drafts-2026-05-12.md §Draft 1`). Track in oyatie's `RISK-REGISTER.md` under a new row.
2. **ADR-0054-grit-scaffold-claim-pattern.md** for the new-crate chicken-and-egg resolution. Draft at `.omc/scratch/pre-cutover-drafts-2026-05-12.md §Draft 2`.
3. **Document `oya-tooling-agent-read` audit-chain topic** in `docs/security-program/security-program.json`. Each read invocation should emit `EVT-AGENT-READ-<verb>` with agent id, args, timestamp.
4. **Add inventory-tracker fitness lane** for the `archive/pre-grit-cutover-2026-05-12/` directory — once a file is archived, the active-path equivalent must NOT exist (and vice versa for KEEP-class files).
5. **Schedule re-evaluation of the helper surface in 90 days** (2026-08-12) — if `grit` upstream adds `grit log`/`grit pr-view`/`grit diff` natively, deprecate the corresponding `oya-tooling-agent-read` subcommands and shrink the helper.
6. **Forge-provider abstraction for `oya-tooling-agent-read`** — when oyatie adds a second forge (GitLab, Bitbucket, Gitea), introduce a provider trait and adapter crates (`oya-tooling-agent-read-adapter-github`, `-adapter-gitlab`, etc.) so the verb surface remains stable. Defer until second-forge requirement materializes; track in the provider-agnostic cross-cutting workstream.

---

## Landing evidence

- **ADR path:** `docs/decisions/ADR-0053-grit-icm-as-sanctioned-primitives.md`
- **ADR-INDEX row:** ADR-0053 appended to `docs/ADR-INDEX.md` under Cross-cutting / Tooling
- **Audit-chain emission ID:** `EVT-ADR-LAND-0053-01HXXMKPRGRITICM00000000000000` (ULID emitted by `oya-governance-banned-primitives` lane on first CI run after merge; superseded by ADR-0116 retirement so no further emissions expected against this entry).
- **Sibling ADRs landing in parallel:** ADR-0052 (pre-grit artifact inventory), ADR-0054 (grit scaffold-claim pattern)
- **Operational driver:** `.omc/plans/SST consolidation ralplan`
- **Operating contract shaped:** `docs/AGENTS.md` §Sanctioned primitives
