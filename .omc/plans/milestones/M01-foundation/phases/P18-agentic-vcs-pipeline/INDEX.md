---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P18
title: End-to-end agentic VCS pipeline
status: scaffolded
purpose: Implement the 5-ADR design contract (ADR-0110/0111/0112/0113/0114) that makes the dev > staging > production branch pipeline actually agentic — changeset state machine, projected-state merge queue with fix-at-any-stage, webhook-driven Foundry agent invocation, async orchestrator with cost budgets, and per-cell canary observability gate with dual rollback paths.
---

# M01-P18 — End-to-end agentic VCS pipeline

## Context

M01-P17 (pipeline maturity glue, landed 2026-05-15 via PR #3)
shipped IP-004 (pr-review dispatcher), IP-005 (CI fix-loop), IP-006
(merge-queue), IP-009 (subagent runtime) as a SUBSTRATE. The
2026-05-16 follow-up (PR #4) added the dev > staging > production
branch topology + naive promotion workflows.

But the substrate isn't AGENTIC end-to-end — there's no canonical
changeset state, no event-driven invocation, no conflict-avoidance
pre-admit, no canary gate, no async orchestrator. PR #5 lands the
5-ADR design contract (ADR-0110..0114); P11 lands the executable
implementation.

## IPs

| ID | Title | Wave | Depends on |
|---|---|---|---|
| IP-001 | changeset-state kernel + event log + monotonicity lane | A | ADR-0110 |
| IP-002 | merge-queue projected-state + fix-at-any-stage | A | ADR-0111, IP-006 from P10 |
| IP-003 | webhook-receiver substrate + event router | A | ADR-0112 |
| IP-004 | vcs-orchestrator + `oya vcs done` async-by-default | A | ADR-0113, IP-001 |
| IP-005 | canary controller + per-cell cohort + thresholds | A | ADR-0114, IP-001 |
| IP-006 | wave-B integration (gate promotions on canary; webhook-route IP-004/005/006) | B | All wave-A IPs |
| IP-007 | wave-C operational fitness lanes | C | IP-006 |

IP-001 is the foundation; every other IP depends on it. After IP-001
lands, IP-002/003/005 can run in parallel (no kernel overlap). IP-004
depends on IP-001's changeset-state kernel. IP-006 + IP-007 are
sequential after wave-A converges.

## Exit criteria

- All 7 IPs scaffolded and Accepted by council-foundry-vcs +
  council-architecture.
- `cargo run -q -p oya-dev-cli -- gate validate` passes for every
  new fitness lane added in this phase.
- A real PR closes the loop: opened by `oya vcs done`, gated by
  IP-002 conflict-avoidance, IP-005 CI fix-loop, IP-004
  pr-review, IP-006 merge-queue, lands on dev, auto-promotes
  through staging (canary controller passes) and production.
- Mean-time-to-detect canary failure ≤15 minutes (per ADR-0114
  observation window).
- p99 webhook → agent invocation latency ≤5 s (per ADR-0112 SLO).
