---
id: ADR-0368
status: Superseded
deciders: founder, council-architecture
date: 2026-05-26
owner: founder
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0363, ADR-0364, ADR-0365, ADR-0366, ADR-0367]
planning_impact: true
milestone: M-NORTH-STAR
depends_on: []
door: one-way
affected_surfaces:
  crates: [oya-dev-cli]
  microservices: [intelligence]
  specs: [/specs/masterplan.json]
deliverables:
  - id: ADR-0368-D1
    description: "Maximum-agent deployment: the fleet is kept at maximum safe concurrency at all times, fed by the masterplan (the generated collection of plans/tasks from ADRs)."
    exit_criteria: "an orchestrator deploys the max safe number of owner-agents against open masterplan deliverables; idle capacity is an alert."
    verified_by: "oya gate validate fleet-utilization"
  - id: ADR-0368-D2
    description: "Self-improving: every failure becomes a new gate (COE->gate flywheel, ADR-0366); the gate corpus monotonically grows; repeat failures are impossible."
    exit_criteria: "each COE links a new/updated gate; a regression that recurs without a new gate fails the flywheel check."
    verified_by: "oya gate validate coe-to-gate"
  - id: ADR-0368-D3
    description: "Self-healing: deterministic protocol violations are auto-repaired (ADR-0366 self-repair) without human/agent intervention."
    exit_criteria: "deterministic violations auto-repair end-to-end; only one-way/non-deterministic/unrepairable escalate."
    verified_by: "oya gate validate self-repair-coverage"
  - id: ADR-0368-D4
    description: "Self-governing: every step is a gate (ADR-0364/0365/0366/0367); nothing relies on agent or human discipline; the system enforces itself."
    exit_criteria: "no protocol step is bypassable; ungated mutation paths fail the meta-audit."
    verified_by: "oya gate validate aspirational-enforcement"
  - id: ADR-0368-D5
    description: "Human input is architecture-only: agents own everything except architectural decisions (ADRs). Human-touch on non-architecture merges is an exception that must be justified."
    exit_criteria: "merges carry no required human step except where door:one-way demands founder sign-off; human edits to generated/agent-owned artifacts are flagged."
    verified_by: "oya gate validate decision-door"
  - id: ADR-0368-D6
    description: "Even architecture is challenged: every ADR (including founder-authored) flows through best-practice-research + adversarial consensus review; an anti-pattern or a better hyperscaler alternative is surfaced and BLOCKS acceptance until reconciled."
    exit_criteria: "the adr-challenge gate fails an ADR with an unaddressed anti-pattern or a cited better-alternative; the founder's ADRs are not exempt."
    verified_by: "oya gate validate adr-challenge"
purpose: North-star charter. Oyatie is a self-improving, self-healing, self-governing agentic platform: a maximal fleet of agents executes a masterplan generated from ADRs; the only human input is architectural decisions — and even those are challenged against hyperscaler best-practice. This ADR is the apex; every other ADR serves it. (Becomes ADR-0000 in the re-foundation.)
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0368: Self-governing agentic platform — north-star charter

## Status
Accepted — 2026-05-26. The apex decision; becomes **ADR-0000** in the re-foundation (ADR-0364 §6).

## Context
Everything built this wave — generated masterplan (ADR-0364), automated decision lifecycle (ADR-0365),
self-enforcing pipeline (ADR-0366), trustless verification (ADR-0367) — serves one end-state that had
not yet been written down as the charter. Without it, the pieces are tactics without a thesis. The
distillation evidence (300 ADRs, 41% never-ratified, "spec-saturated, code-starved") is what the
charter exists to make structurally impossible.

## Decision
Oyatie is a **self-improving, self-healing, self-governing agentic platform.**

1. **Masterplan = work for agents.** The masterplan is the generated (ADR-0364) collection of
   plans/tasks. It is the fleet's work queue.
2. **Maximum agents, always.** Deploy the maximum *safe* number of agents at all times against open
   masterplan deliverables; idle capacity is a defect (D1). Safety = the ADR-0366 conflict-prevention +
   ADR-0367 trustless gateway, which is what makes high concurrency safe.
3. **Self-improving.** Every failure converts to a new gate (COE→gate flywheel, D2). The system gets
   stricter and more correct over time; the same mistake cannot recur.
4. **Self-healing.** Deterministic violations auto-repair (D3); the pipeline fixes itself.
5. **Self-governing.** Every step is a gate (D4); nothing relies on discipline — agent *or* human.
6. **Human input = architecture only.** Agents own all execution; the human authors architectural
   decisions (ADRs) and nothing else routinely (D5).
7. **Even architecture is challenged.** Every ADR — *including the founder's* — flows through
   best-practice-research + adversarial consensus review. An anti-pattern, or a better hyperscaler
   alternative, is surfaced and **blocks acceptance until reconciled** (D6). The human is the
   *architect*, not the unquestioned *authority*; the hyperscaler best-practice bar is the authority,
   and it reviews the human. This is the loop's closure: nothing escapes the bar.

## Rejected alternatives
- **Human-in-the-loop on every change** — rejected: doesn't scale to a max-concurrency fleet; the
  trustless gateway (ADR-0367) replaces the human as the merge authority.
- **Human architectural decisions are final/unquestioned** — rejected: that re-admits anti-patterns;
  D6 challenges even the founder against best-practice.
- **Optimize for features first** — rejected (ADR-0366): without the self-governing pipeline, agentic
  development drifts into thin scaffolds.

## Consequences
- Positive: a one-person + agent-fleet operation runs like a disciplined hyperscaler org; quality and
  correctness compound; the founder's leverage is concentrated where it's irreplaceable (architecture),
  and even that is quality-checked.
- Negative/cost: requires the full gate/automation backlog (D1–D6 + the ADR-0364–0367 deliverables) to
  be real; until then the charter is aspirational and the gates show as planned. Risk: the
  best-practice bar must be genuinely adversarial (Intelligence-reviewer + research), not a rubber
  stamp — the coe-to-gate flywheel catches misses.
- Neutral: this is doctrine; it directs, it doesn't itself ship code.

## Verification
Per-deliverable `verified_by`. The charter is *met* when the fleet runs at max safe concurrency on the
generated masterplan, failures become gates, violations self-repair, every step is gated, human input
is architecture-only, and the adr-challenge gate demonstrably pushes back on a founder ADR that carries
an anti-pattern.

## References
ADR-0364 (generated masterplan), ADR-0365 (automated lifecycle), ADR-0366 (self-enforcing pipeline),
ADR-0367 (trustless verification), ADR-0363 (substrate). Research: docs/ideas/hyperscaler-practices-to-adopt.md.
