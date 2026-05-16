---
doc_class: EscalationMatrix
shape: matrix
status: Accepted
authority_tier: 1
length_cap: 60
date: 2026-05-12
purpose: |
  The complete, exhaustive matrix of cases where an autonomous agent MUST halt and emit
  `BLOCKED_ON_HUMAN_ORCHESTRATOR`. Designed per the autonomy directive to be as small as
  possible. Anything not in this matrix is resolved by the agent autonomously via
  AGENT-DECISION-TREE / AGENT-FAILURE-RECOVERY.
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/agents/AGENT-DECISION-TREE.md
  - docs/agents/AGENT-FAILURE-RECOVERY.md
  - docs/agents/HUMAN-OPERATOR-GUIDE.md
doc_status: published
---

# Escalation Matrix

> **3 cases.** Halt only when one matches. Everything else: stay in the autonomous loop. Foundation: ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim).

| Case ID | Trigger | icm-store payload (canonical) | Human action required | Handback signal |
|---|---|---|---|---|
| `HALT-01` | IP requires autonomy-ceiling uplift (T1→T2 / T2→T3 / T3→T4) which needs a Cedar policy diff + runtime gate. Per [`docs/standards/autonomy-ceiling.md`](../standards/autonomy-ceiling.md), agents MUST NOT author Cedar policies that grant themselves higher tier autonomy. | `icm store -t cutover-orchestrator-actions -c "BLOCKED_ON_HUMAN_ORCHESTRATOR: HALT-01: <IP> requires T<n>→T<m> uplift; Cedar+runtime gate authoring needed" -i critical -k "halt,autonomy-ceiling,<area>"` | council-architecture (+ Founder if T3↔T4) authors Cedar policy diff; ships in a separate PR; emits unblock row. | `icm store -t cutover-orchestrator-actions -c "UNBLOCK HALT-01: tier <n>→<m> approved; policy at <PR-link>; agent <id> may resume" -i critical -k "unblock,autonomy-ceiling"` |
| `HALT-02` | Destructive op on shared ref required: force-push to `main`, hard-reset of merged history, regulated-field schema-class downgrade, or removal of an `EVT-*` audit topic. Forbidden under Directive 12 regardless of rationale. | `icm store -t cutover-orchestrator-actions -c "BLOCKED_ON_HUMAN_ORCHESTRATOR: HALT-02: <op> on <ref> required; agent cannot execute under Directive 12" -i critical -k "halt,destructive,<context>"` | Founder + council-architecture quorum approves; council member executes manually with `direct-tool-invocations` audit row; emits unblock row. | `icm store -t cutover-orchestrator-actions -c "UNBLOCK HALT-02: <op> executed by <council member>; rationale=<one-line>; agent <id> may resume" -i critical -k "unblock,destructive"` |
| `HALT-03` | Sanctioned primitive (grit/icm/oya-tooling-agent-read) itself unhealthy after 2 retries: infra error, persistent FK errors uncurable by scaffold-claim, icm-health hard fail, oya-tooling-agent-read auth failure. | `icm store -t cutover-orchestrator-actions -c "BLOCKED_ON_HUMAN_ORCHESTRATOR: HALT-03: <primitive> unhealthy: <one-line error>; 2 retries failed" -i critical -k "halt,infra,<primitive>"` | ops-sre-reliability on-call diagnoses; applies known workaround or new fix; emits unblock row. | `icm store -t cutover-orchestrator-actions -c "UNBLOCK HALT-03: <primitive> health restored: <one-line fix>; agent <id> may resume" -i critical -k "unblock,infra"` |

## Cases explicitly NOT in the matrix (handled autonomously)

- `cargo` failure → D2 / R5 (silent-failure-hunter + standards).
- Fitness lane red → D4 (lane→standard mapping).
- Claim FK error → R1 (scaffold-claim pattern, ADR-0054).
- Claim lock contention → R2 (`icm-coordination-lock` fallback + sibling IP).
- grit session start error → R3 (session-less mode).
- merge-queue conflict → R4 (`grit assign` or rebase).
- New dep license fail → R6 (replace dep).
- Need raw `git`/`gh` → Directive-12 (`direct-tool-invocations` audit row, no halt).
- Need to defer an IP → D8 (release-and-defer, no halt).

If a case looks like a halt but isn't in the matrix, prefer release-and-defer over halt. Council-architecture reviews `cutover-orchestrator-actions` weekly; matrix growth is a council decision, never agent-initiated.
