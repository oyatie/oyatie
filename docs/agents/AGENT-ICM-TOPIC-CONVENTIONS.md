---
doc_class: AgentIcmTopicConventions
shape: catalog
status: Accepted
authority_tier: 2
length_cap: 80
date: 2026-05-12
purpose: |
  Canonical icm topic names + importance/keyword conventions. Extends and supersedes the
  ICM section in repo-root `CLAUDE.md`. Every `icm store` MUST use one of these topics;
  unknown topics fail `icm health`.
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/agents/AGENT-ENTRY-POINT.md
  - docs/agents/AGENT-TOOL-PROTOCOL.md
  - CLAUDE.md
doc_status: published
---

# ICM Topic Conventions

> Canonical topics only. Unknown topics break `icm health` and lane `oya-governance-icm-topic-hygiene`. Importance ladder: `critical` > `high` > `medium` > `low`. Foundation: ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim).

## Canonical topics

| Topic | When to store | Importance floor | Required keywords |
|---|---|---|---|
| `decisions-oyatie` | Any architectural / non-trivial engineering decision an agent makes autonomously. | `high` | `<area>`, `<ADR-#### if any>` |
| `preferences` | User-stated preferences discovered mid-session (carry across sessions). | `critical` | `<preference-key>` |
| `context-oyatie` | Progress checkpoints; deferrals; significant work-completed summaries; ≥20-tool-call breakpoint. | `high` for completion; `medium` for checkpoints | `<MNN-PNN>`, `<IP-NNN>` |
| `errors-resolved` | Every resolved error (root cause + fix). | `high` | `<error-class>`, `<area>` |
| `scaffold-locks-oyatie` | Active scaffold-claim per ADR-0054. | `high` | `scaffold`, `<crate-path>` |
| `direct-tool-invocations` | BEFORE every Directive-12 `git`/`gh` raw call (rationale). | `high` | `git\|gh`, `<context>` |
| `cutover-orchestrator-actions` | Only for `BLOCKED_ON_HUMAN_ORCHESTRATOR` halts and orchestrator hand-back rows. | `critical` | `halt`, `<case-id>` |
| `goals-oyatie` | Long-horizon goals (rare; usually written by Founder / council-architecture). | `high` | `<wave>`, `<milestone>` |

## Importance discipline

- `critical`: halt rows, autonomy-ceiling decisions, user-preference discoveries. Page-worthy.
- `high`: normal decisions, errors-resolved, completed-work checkpoints, scaffold-locks, Directive-12 rationales.
- `medium`: in-flight progress at the ≥20-tool-call threshold; waits/defers.
- `low`: informational only; rarely used.

## Keyword discipline

Always include: (a) the area/crate/module (e.g. `oya-platform-tenant-kernel`, `foundry-eval-harness`); (b) the canonical IDs (`<MNN-PNN>`, `<IP-NNN>`, `<ADR-####>`, `<EVT-NNNN>`, `<MFL-NNNN>`, `<RM-NN>`) the row relates to. Lowercase, comma-separated, no spaces.

## What NOT to store

Trivial details, info already in CLAUDE.md, ephemeral state (build logs, `git status`), secrets, PII, full file contents, raw stack traces (summarize), copies of other ICM rows. Lane: `oya-governance-icm-payload-hygiene`.

## Authority

This file extends [`/Users/jasonlee/oyatie/CLAUDE.md`](../../CLAUDE.md) §"Persistent memory (ICM) — MANDATORY". On conflict, this file wins (it is closer to canonical lift target).
