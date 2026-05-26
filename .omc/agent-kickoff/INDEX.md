---
doc_class: AgentKickoffIndex
shape: index
status: pending approval
authority_tier: 1
length_cap: 80
purpose: |
  Index of files in `.omc/agent-kickoff/`. Single navigation point for the autonomous-
  agent-kickoff layer. Names purpose, reading order, and lift target per file.
lift_target: oyatie/docs/agents/INDEX.md
canonical_authority: docs/CONSTITUTION.md
related:
  - .omc/plans/MASTERPLAN.md
  - docs/AGENTS.md
  - docs/CONSTITUTION.md
---

# Agent Kickoff INDEX

> The kickoff layer is the bridge between [`docs/CONSTITUTION.md`](../../docs/CONSTITUTION.md) + [`docs/AGENTS.md`](../../docs/AGENTS.md) (the operating contract) and the four-tier plan tree at [`.omc/plans/MASTERPLAN.md`](../plans/MASTERPLAN.md). Read in order. Every file is `status: pending approval`; lift targets are under `docs/agents/`.

## Reading order

| # | File | Purpose | Lift target |
|---|---|---|---|
| 1 | [`AGENT-ENTRY-POINT.md`](AGENT-ENTRY-POINT.md) | Single page a fresh agent reads first. 7-step navigation contract. | `docs/agents/AGENT-ENTRY-POINT.md` |
| 2 | [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md) | Flowchart + per-branch rules for every common in-flight decision. | `docs/agents/AGENT-DECISION-TREE.md` |
| 3 | [`AGENT-TOOL-PROTOCOL.md`](AGENT-TOOL-PROTOCOL.md) | Canonical calling conventions for `{grit, icm, oya-tooling-agent-read}` + Directive-12 escape hatch. | `docs/agents/AGENT-TOOL-PROTOCOL.md` |
| 4 | [`AGENT-COMPLETION-PROTOCOL.md`](AGENT-COMPLETION-PROTOCOL.md) | C1–C11 the final mile to `grit done`. | `docs/agents/AGENT-COMPLETION-PROTOCOL.md` |
| 5 | [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md) | R1–R7 recoveries that keep the agent in the autonomous loop. | `docs/agents/AGENT-FAILURE-RECOVERY.md` |
| 6 | [`AGENT-ICM-TOPIC-CONVENTIONS.md`](AGENT-ICM-TOPIC-CONVENTIONS.md) | Canonical icm topic catalog (8 topics) + importance/keyword discipline. | `docs/agents/AGENT-ICM-TOPIC-CONVENTIONS.md` |
| 7 | [`CROSS-REFERENCE-INDEX.md`](CROSS-REFERENCE-INDEX.md) | Master doc index: path × class × purpose × owner × lifecycle × lane × cross-refs. | `docs/agents/CROSS-REFERENCE-INDEX.md` |
| 8 | [`AGENT-CHEAT-SHEET.md`](AGENT-CHEAT-SHEET.md) | 1-page printable: 10 commands + 5 lookups + 3 halts. | `docs/agents/AGENT-CHEAT-SHEET.md` |
| 9 | [`HUMAN-OPERATOR-GUIDE.md`](HUMAN-OPERATOR-GUIDE.md) | For humans when `ESCALATION-MATRIX.md` matches. | `docs/agents/HUMAN-OPERATOR-GUIDE.md` |
| 10 | [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md) | The exhaustive 3-row halt matrix. | `docs/agents/ESCALATION-MATRIX.md` |

## Minimum-viable workflow

A fresh agent in 7 steps:

1. Read `AGENT-ENTRY-POINT.md` end-to-end.
2. Descend MASTERPLAN → milestone INDEX → phase INDEX → IP.
3. `icm recall-context` for prior decisions; verify prerequisites.
4. `grit claim --agent --intent <symbols>` (scaffold-claim on FK).
5. Work in `.grit/worktrees/<id>/`; heartbeat hourly; `icm store` at named events.
6. Run acceptance commands; emit `EVT-*` audit row; walk Done-Definition D1–D18.
7. `grit done --agent <id>`. Pick next IP.

## Cross-references into the broader doc set

- Standards: [`.omc/standards/INDEX.md`](../standards/INDEX.md) (lifts to `docs/standards/`).
- Templates + checklists: [`/templates/INDEX.md`](../templates/INDEX.md) (lifts to `docs/templates/` and `docs/checklists/`).
- Fitness lanes: [`.omc/governance-lanes/`](../governance-lanes/) (parallel composer output).
- Plan hierarchy: [`.omc/plans/MASTERPLAN.md`](../plans/MASTERPLAN.md).
- Operating contract: [`docs/AGENTS.md`](../../docs/AGENTS.md).
- Constitutional frame: [`docs/CONSTITUTION.md`](../../docs/CONSTITUTION.md).

## Status

All files in this directory: **pending approval** (working drafts). On council-architecture + Founder sign-off, lift in batch to `docs/agents/`. Post-lift, [`docs/AGENTS.md`](../../docs/AGENTS.md) §Per-agent appendices gains a "Kickoff layer" pointer to `docs/agents/INDEX.md`, and [`docs/STANDARDS-AND-TEMPLATES.md`](../../docs/STANDARDS-AND-TEMPLATES.md) gains a row for the kickoff INDEX.
