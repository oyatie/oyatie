---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cell-substrate
deciders: axis-cell-substrate, council-architecture, ops-sre-reliability
related_adrs: [ADR-0130, ADR-0131]
related_artifacts:
  - microservices/cell/PRD.md
  - microservices/cell/capacity-model.md
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (cell µservice)

## Purpose

Specify how the cell substrate handles two scenarios:
1. **Backfill** — historical cell-assignment + lifecycle events need to be reconstructed from `registry/cell-assignment.jsonl` (the union-merged ledger) into the Postgres registry (e.g., after disaster recovery from backup; after Postgres rebuild).
2. **Replay** — a specific cell-state transition or migration event needs re-computation (e.g., after a bug fix in the lifecycle-manager state machine; after audit reconciliation).

## Backfill

### Contract

When the Postgres cell-registry needs to be rebuilt (full restore from `cell-assignment.jsonl` + recent Postgres backup):

1. Source of truth: append-only `registry/cell-assignment.jsonl` (every cell-assignment delta sealed Ed25519 per Bominal ADR-0028).
2. Replay from earliest record forward: each line is `(timestamp, tenant_id, cell_id, pack, scope, signature)`.
3. Backfill worker validates each signature against the audit-chain expected state.
4. Postgres rows inserted in `idempotent_upsert` mode (signed identity = canonical key).
5. Post-backfill: integrity verification compares Postgres state to ledger state hash.
6. If discrepancy: emit `BackfillDiscrepancyDetected` audit event; manual reconciliation required.

### Constraints

- Backfill does NOT modify historical events. The ledger is immutable.
- Backfill emits `BackfilledFromLedger` events with `backfill_run_id` label so observability can distinguish from live writes.
- Backfill is rate-limited: max 10k rows/sec to avoid Postgres saturation during recovery.
- Cost: backfill is per-recovery; bounded by ledger size (≤ a few GB even at 10k tenants × 5 migrations/tenant).
- Cross-pack ledger sections never cross-restore into wrong-pack Postgres shard.

### Verification

- Integration test: rebuild Postgres registry from a sample ledger; verify final state matches expected.
- Idempotency: re-running backfill on same ledger produces identical Postgres state.
- Signature integrity: each backfilled row's signature validates against audit-chain.

## Replay

### Contract

Replay re-executes a specific cell-state transition or tenant migration event:

- **Bug fix in state machine**: replay invalidates old transitions; emits new ones with corrected logic.
- **Audit reconciliation**: after a split-brain incident (FM-11), replay reconciles diverged writes against ledger.
- **Post-incident analysis**: replay against alternate state-machine versions to validate correctness.

### Procedure

1. Operator invokes: `oya cell replay --event-id <id> --reason "<rfc>"`.
2. CLI requires 2-person rule + ops-security approval (replay can shift historical "truth" and must be audit-bounded).
3. Replay engine re-executes the use case against current state machine + current Postgres state.
4. Emits `EventReplayed` event with `event_kind=replayed, prior_outcome=<original>, new_outcome=<replayed>, reason=<rfc>`.
5. Audit-chain seal: replay event is itself sealed, distinguishing from original.

### Constraints

- Replay does NOT mutate the original event record. Appends new audit-chain seal.
- Replay cannot extend retention beyond Postgres + ledger window (typically indefinite for cell substrate, but bounded by audit-chain SLO).
- Replay output never triggers retroactive cell-substrate actions ("yesterday's failed migration should have succeeded so let's redo it now"). Replays are advisory; not action-bearing.

### Verification

- Integration test: induce a synthetic state-machine bug; replay; verify replayed verdict differs from original.
- Audit-chain integrity: replay event sealed; original event remains sealed; chain reconstructable.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill on Postgres rebuild | per-disaster-recovery | ~$1.00 per 1M rows (10k tenants × 100 events / tenant ≈ $10) |
| Replay on bug-fix | per-fix | ~$0.10 (single event) |
| Replay on audit reconciliation | per-split-brain | varies (depends on diverged-row count) |

Cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers" — backfill/replay are budgeted as part of operational envelope.

## Limitations

- Backfill quality bounded by ledger completeness. If ledger entries are missing (e.g., pre-2026-05-17 historical data), backfill cannot reconstruct.
- Replay assumes determinism. If domain logic depends on wall-clock that changed between original and replay, results may differ. Replay output carries `replay_version + original_version` for transparency.

## References

- `microservices/cell/PRD.md`.
- `microservices/cell/capacity-model.md`.
- `microservices/cell/cost-budget.md`.
- `microservices/cell/contracts/asyncapi/cell-events.yaml`.
- Bominal ADR-0028 (audit-chain).
- Bominal ADR-0019 (runtime catalog + cell sharding).
