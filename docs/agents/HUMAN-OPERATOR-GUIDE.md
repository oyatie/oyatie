---
doc_class: HumanOperatorGuide
shape: protocol
status: Accepted
authority_tier: 2
length_cap: 120
date: 2026-05-12
purpose: |
  For human operators (Founder, council-architecture, ops-on-call). Used only when
  `ESCALATION-MATRIX.md` matches and an agent halts with
  `BLOCKED_ON_HUMAN_ORCHESTRATOR`. Names exact actions, audit row format, and
  hand-back signal.
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/agents/ESCALATION-MATRIX.md
  - docs/AGENTS.md
  - docs/RACI-OWNERSHIP.md
doc_status: published
---

# Human Operator Guide

> Read this only when an agent has halted with `BLOCKED_ON_HUMAN_ORCHESTRATOR` (per [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md)). Steady-state autonomy is the default; human intervention is a rare exception. Foundation: ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim).

## How halts surface to you


```
topic=cutover-orchestrator-actions
importance=critical
content="BLOCKED_ON_HUMAN_ORCHESTRATOR: <case-id>: <one-line>"
keywords=halt,<area>
```


## Per-case actions

### `HALT-01` — autonomy-ceiling uplift required

Trigger: IP wants T1→T2 (or higher) uplift, which requires Cedar policy + runtime gate.

Human action:
1. Open the IP file; confirm the requested tier and the capability record path.
2. If approved: author the Cedar policy diff per [`docs/standards/autonomy-ceiling.md`](../standards/autonomy-ceiling.md); attach to a fresh PR.
3. Emit hand-back:
   ```
     -c "UNBLOCK <case-id>: tier uplift T<n>→T<m> approved; Cedar policy at <PR-link>; agent <agent-id> may resume" \
     -i critical -k "unblock,<area>"
   ```
4. The agent re-enters [`AGENT-ENTRY-POINT.md`](AGENT-ENTRY-POINT.md) §Step 4 and re-claims.

### `HALT-02` — destructive operation on shared ref

Trigger: `git push --force` to `main`, hard-reset of merged history, regulated-field schema-class downgrade.

Human action:
1. Verify with Founder + council-architecture (RACI: this is escalation tier 1).
2. If approved: a council member runs the destructive op manually with full audit; emit:
   ```
     -c "<op> on <ref> approved by <Founder + council-arch quorum>; rationale: <one-line>" \
     -i critical -k "git,destructive,<context>"
   ```
3. Then emit unblock row as in HALT-01.
4. Add `MISTAKES-LEDGER.md` row if the halt was caused by a preventable upstream defect (mechanical prevention recommended).

### `HALT-03` — sanctioned-primitive infra error


Human action:
1. ops-sre-reliability on-call investigates the primitive's health endpoint.
4. Emit unblock row; the agent resumes from its last `context-oyatie` checkpoint.

## Hand-back signal (universal)

```
  -c "UNBLOCK <case-id>: <one-line on what changed>; agent <agent-id> may resume" \
  -i critical -k "unblock,<area>"
```

The agent polls `cutover-orchestrator-actions` filtered by its agent id. Upon seeing `UNBLOCK <case-id>`, it re-enters its prior step.

## What humans MUST NOT do

- Insert raw `git` operations without the `direct-tool-invocations` audit row.
- Silently resolve a halt without emitting the unblock row — the agent will not resume.

## RACI reference

| Halt case | Responsible | Accountable | Consulted | Informed |
|---|---|---|---|---|
| HALT-01 | council-architecture | Founder (if T3↔T4) | ops-security, council-privacy | All teams |
| HALT-02 | council-architecture | Founder | ops-security, axis lead | All teams |
| HALT-03 | ops-sre-reliability | council-architecture | axis-foundry | All teams |

Full RACI: [`docs/RACI-OWNERSHIP.md`](../RACI-OWNERSHIP.md).

## Cadence + drill

Quarterly halt drill (per `docs/INCIDENT-MANAGEMENT.md` schedule): inject a synthetic HALT-01/02/03; verify hand-back signal works end-to-end; verify agent resumes within 15 min of unblock row. Postmortem any drill missing that target.
