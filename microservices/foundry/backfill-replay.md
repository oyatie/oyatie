---
doc_class: BACKFILL-REPLAY
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: axis-foundry + ops-sre-reliability
related_adrs: [ADR-0136, ADR-0137]
---

# Backfill + Replay — foundry (consolidated)

## Scope

Cross-BC backfill + replay strategies. Per-BC plans preserved at
`bc-sources/<bc>/backfill-replay.md`.

## Backfill scenarios

| Scenario | Affected BCs | Method | Boundedness |
|---|---|---|---|
| Audit-chain re-seal after bridge outage | evidence | Replay sealed-pending packs through bridge in monotonic order | Sev cap SEV-2; ≤24h backlog |
| Eval golden-output regen after model upgrade | eval | Re-run capabilities against new model; mark new generation | Sev cap SEV-3; offline |
| Supervision event replay after bus outage | supervisor | Replay from PostgreSQL append-only log to event-bus subscribers | Bounded by supervisor-log retention (1y) |
| Capability registry-cache rebuild | runtime | Pull full descriptor list from supervisor; rebuild cache | Bounded by capability count (≤100k/pack) |
| Provider receipt reconciliation after billing-system outage | providers | Replay receipts from internal log against billing-system reconciliation API | Bounded by 30d receipt retention |
| Guardrail decision log replay (debugging false-positive surge) | guardrails | Re-evaluate ruleset against historical hash(prompt) records | Sev cap SEV-3; offline |
| Evidence pack rebuild (lost S3 blob) | evidence | Rebuild from invocation-recorder + supervision-events + guardrail-decisions + provider-receipts | Bounded by source-record retention (1y baseline) |

## Replay determinism (eval BC canonical)

Per `bc-sources/eval/backfill-replay.md` + `bc-sources/eval/PRD.md` §
"Acceptance Criteria":

- Eval replay through runtime sandbox pool deterministically reproduces a
  sealed invocation given:
  - Capability descriptor (versioned, content-addressed)
  - Provider response (mocked from sealed receipt) OR live-call mode with
    "best-effort parity" annotation
  - Session-state snapshot (from evidence pack)
  - Guardrail ruleset version (from supervision-event-log)
- AC-X6 from `microservices/foundry/PRD.md` enforces.

## Cross-BC replay ordering

When replaying across BCs (e.g., re-seal a period's evidence after bridge
outage), the replay order is fixed:

1. Provider receipts (providers BC) — they are the leaves; their hashes
   feed evidence packs.
2. Guardrail decisions (guardrails BC) — they reference invocation IDs
   that exist by the time provider receipts are replayed.
3. Capability invocations (runtime BC) — reference both above.
4. Supervision events (supervisor BC) — reference invocations.
5. Eval-runs (eval BC) — reference invocations + supervision events.
6. Evidence packs (evidence BC) — aggregate everything above.

This order is the partial-DAG over the cross-BC event types; violating it
produces dangling references that evidence-pack-builder refuses.

## Per-BC backfill-replay archives

- `bc-sources/runtime/backfill-replay.md`
- `bc-sources/supervisor/backfill-replay.md`
- `bc-sources/eval/backfill-replay.md`
- `bc-sources/evidence/backfill-replay.md`
- `bc-sources/guardrails/backfill-replay.md`
- `bc-sources/providers/backfill-replay.md`

## References

- ADR-0136 / ADR-0137: foundry topology.
- `microservices/foundry/PRD.md` — AC-X6 replay determinism.
