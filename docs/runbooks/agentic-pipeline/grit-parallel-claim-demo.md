# Runbook: grit parallel-claim demo

> **Owner:** `council-architecture + axis-foundry`
> **Status:** Active
> **Severity supported:** Sev 4
> **Severity scope:** Sev 4
> **Last verified:** 2026-05-14 by Codex autopilot in local P01-IP-010 drill
> **Related:** [ADR-0052](../../decisions/ADR-0052-inventory-grit-cutover.md), [ADR-0053](../../decisions/ADR-0053-grit-icm-as-sanctioned-primitives.md), [ADR-0054](../../decisions/ADR-0054-grit-scaffold-claim-pattern.md), [M-CC-P01](../../plans/M-CC-01-cutover/INDEX.md)

---

## Trigger

Open this runbook when P01 needs proof that two agents can hold non-overlapping `grit` symbol locks in the same file, work in separate auto-created worktrees, and release through `grit done` without conflict.

This is the P8/A7 acceptance drill for the agentic-pipeline cutover. Session-mode orchestration remains deferred to P9; this drill intentionally uses session-less symbol claims.

---

## Pre-checks (5 minutes max)

- [ ] Pre-check 1: no active locks — verify with `grit status`; expected `No active locks.`
- [ ] Pre-check 2: both demo symbols are indexed — verify with `grit symbols` and the two symbol names below.
- [ ] Pre-check 3: run from `/Users/jasonlee/oyatie`; expected `AGENTS.md`, `.omc/`, `docs/`, and `crates/` are present.

If any pre-check fails, **STOP** and record the blocker in `icm` before retrying. Do not broaden to unrelated master-plan work.

---

## Demo symbols

| Agent | Symbol | Purpose |
|---|---|---|
| `codex-ip010-agent-a` | `crates/oya-cloud-billing-application/src/lib.rs::CloudBillingEventIngestAppStatus` | first symbol in the shared file |
| `codex-ip010-agent-b` | `crates/oya-cloud-billing-application/src/lib.rs::CloudBillingMeterUnitRecord` | second symbol in the same shared file |
| `codex-ip010-agent-c` | `crates/oya-cloud-billing-application/src/lib.rs::CloudBillingEventIngestAppStatus` | negative duplicate-claim probe |

Legacy P8 planning text named `crates/oya-cloud-billing-app/src/lib.rs`; the current filesystem path is `crates/oya-cloud-billing-application/src/lib.rs`, and the 2026-05-14 drill successfully claimed the current path.

---

## Procedure

1. ☐ Run the reproducible drill — `bash docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.sh`.
   Expected: agent A and agent B both receive `Granted` for distinct symbols in the same file.
   If differs: run `grit status`, release only locks owned by the demo agents, and record the blocker.

2. ☐ Confirm the duplicate-claim negative case.
   Expected: agent C is blocked on agent A's symbol while agent B's different symbol remains independently claimable.
   If differs: record the transcript and do not claim A7 green.

3. ☐ Confirm cleanup.
   Expected: final `grit status` returns `No active locks.`
   If differs: run `grit done --agent <demo-agent>` for only the demo agents and rerun `grit status`.

---

## Expected output excerpts

A successful 2026-05-14 local drill produced:

```text
agent-a: + Granted: crates/oya-cloud-billing-application/src/lib.rs::CloudBillingEventIngestAppStatus
agent-b: + Granted: crates/oya-cloud-billing-application/src/lib.rs::CloudBillingMeterUnitRecord
status-after-claims: 2/30434 symbols locked
agent-c: Error: Some symbols are blocked
watch.log: live `grit watch` capture for claim/done window
final-status: No active locks.
```

Transcript artifacts from the acceptance run are under `/evidence/agentic-pipeline/ip-010-parallel-claim-demo-transcript/`.

---

## Rollback

The script performs cleanup with:

```text
grit done --agent codex-ip010-agent-a
grit done --agent codex-ip010-agent-b
grit done --agent codex-ip010-agent-c
```

Only the demo agents are released. If a later IP regresses this runbook, revert that IP's merge and rerun this drill.

---

## Verification

- [ ] `grit status` shows no active locks before and after the drill.
- [ ] Agent A and agent B claims both exit 0 and are simultaneously visible in `grit status`.
- [ ] Agent C duplicate claim emits a blocked-symbol error while agent A owns that symbol.
- [ ] `watch.log` contains the live `grit watch` capture for the drill window.
- [ ] Transcript files exist under `/evidence/agentic-pipeline/ip-010-parallel-claim-demo-transcript/`.

---

## Post-incident updates

After any failed invocation, update:
- [ ] this runbook with the failure mode and recovery step;
- [ ] `docs/MISTAKES-LEDGER.md` if the failure exposes a repeatable prevention;
- [ ] the P01 evidence packet for the active IP.

---

## Audit-chain emission

For P01 drills, emit durable memory through `icm` with:
- runbook-id: `grit-parallel-claim-demo`
- invoker-id
- timestamp
- outcome (`resolved`, `blocked`, or `unresolved`)
- evidence path

---

## Sources scanned

- `.omc/plans/milestones/M-CC-cross-cutting/phases/P01-agentic-pipeline-cutover/IP-010-parallel-claim-demo.md`
- `.omc/plans/ralplan-oyatie-sst-consolidation.md` §P8/A7
- `.omc/scratch/pre-cutover-drafts-2026-05-12.md` §Draft 3
- `docs/plans/M-CC-01-cutover/open-questions-resolutions.md` §Q5
