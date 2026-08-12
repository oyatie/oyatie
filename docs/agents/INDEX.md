---
doc_class: AgentKickoffIndex
shape: index
status: Accepted
authority_tier: 1
length_cap: 80
date: 2026-05-12
purpose: |
  Index of files in `docs/agents/`. Single navigation point for the autonomous-
  agent-kickoff layer. Names purpose, reading order, and lift target per file.
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/MASTERPLAN.md
  - docs/AGENTS.md
  - docs/CONSTITUTION.md
doc_status: published
---

# Agent Kickoff INDEX

> The kickoff layer is the bridge between [`docs/CONSTITUTION.md`](../CONSTITUTION.md) + [`docs/AGENTS.md`](../AGENTS.md) (the operating contract) and the four-tier plan tree at [`docs/MASTERPLAN.md`](../MASTERPLAN.md). Read in order. Foundation: ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim).

## Reading order

| # | File | Purpose |
|---|---|---|
| 1 | [`AGENT-ENTRY-POINT.md`](AGENT-ENTRY-POINT.md) | Single page a fresh agent reads first. 7-step navigation contract. |
| 2 | [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md) | Flowchart + per-branch rules for every common in-flight decision. |
| 5 | [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md) | R1–R7 recoveries that keep the agent in the autonomous loop. |
| 7 | [`CROSS-REFERENCE-INDEX.md`](CROSS-REFERENCE-INDEX.md) | Master doc index: path × class × purpose × owner × lifecycle × lane × cross-refs. |
| 8 | [`AGENT-CHEAT-SHEET.md`](AGENT-CHEAT-SHEET.md) | 1-page printable: 10 commands + 5 lookups + 3 halts. |
| 9 | [`HUMAN-OPERATOR-GUIDE.md`](HUMAN-OPERATOR-GUIDE.md) | For humans when `ESCALATION-MATRIX.md` matches. |
| 10 | [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md) | The exhaustive 3-row halt matrix. |

## Minimum-viable workflow

A fresh agent in 7 steps:

1. Read `AGENT-ENTRY-POINT.md` end-to-end.
2. Descend MASTERPLAN → milestone INDEX → phase INDEX → IP.
6. Run acceptance commands; emit `EVT-*` audit row; walk Done-Definition D1–D18.

## Cross-references into the broader doc set

- Standards: [`docs/standards/INDEX.md`](../standards/INDEX.md).
- Templates + checklists: [`docs/templates/INDEX.md`](../templates/INDEX.md) (and `templates/checklists/`).
- Fitness lanes: [`docs/governance-lanes/`](../governance-lanes/) (parallel composer output).
- Plan hierarchy: [`docs/MASTERPLAN.md`](../MASTERPLAN.md).
- Operating contract: [`docs/AGENTS.md`](../AGENTS.md).
- Constitutional frame: [`docs/CONSTITUTION.md`](../CONSTITUTION.md).

## Status

All files in this directory: **Accepted** (lifted from `.omc/agent-kickoff/` on 2026-05-12). Foundation: ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim). [`docs/AGENTS.md`](../AGENTS.md) §Per-agent appendices gains a "Kickoff layer" pointer to `docs/agents/INDEX.md`, and [`docs/STANDARDS-AND-TEMPLATES.md`](../STANDARDS-AND-TEMPLATES.md) gains a row for the kickoff INDEX.
