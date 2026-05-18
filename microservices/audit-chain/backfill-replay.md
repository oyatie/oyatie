---
doc_class: ContractSpec
title: Backfill + Replay Contract (audit-chain)
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-audit-chain
deciders: axis-audit-chain, council-architecture, ops-sre-reliability, council-privacy
related_adrs: [ADR-0028, ADR-0003, ADR-0131]
related_artifacts:
  - microservices/audit-chain/PRD.md (FR-04 verification; FR-10 auditor export)
  - microservices/audit-chain/capacity-model.md
  - /specs/audit-chain-merkle-ed25519.json
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (audit-chain µservice)

## Purpose

Specify how audit-chain handles two scenarios:
1. **Historical seal verification** — verifying any past period's seal regardless of current key epoch.
2. **Backfill** — onboarding a new tenant whose retro-window historical events were captured by a different µservice but are now centralised into audit-chain.
3. **Replay** — re-running verification across the published chain for compliance evidence (e.g., quarterly auditor-ready frozen replay).

Per Bominal ADR-0028 §"No retroactive re-sign" — **the chain itself is immutable; backfill and replay are read-side reconstructions, not chain mutations**. This is the load-bearing invariant.

## Historical Seal Verification

### Contract

Given any past `(pack, tenant_partition, period_id, event_id)`, return:
- `verified: true | false`
- `signed_root`
- `merkle_proof`
- `signer_public_key` (resolved via KeyResolver for the period's epoch)
- `chain_link_back_to_genesis` (deterministic; ≤ ceil(log2(periods_since_genesis)) hops)

### KeyResolver semantics

The KeyResolver port maps `(pack, tenant_partition, period_id) → public_key`. The mapping is derived from the published GitHub-pinned + S3-mirrored `evidence/audit-chain-keys/<pack>/<epoch>.json`. Rotation events introduce new entries; retired keys remain in the resolver (verification still works post-rotation; old roots verify with their epoch's key).

Per `policy/seal-integrity.md` §"SI-13".

### Performance

- Verification of any past event: ≤ 200ms p99 (per `PRD.md` §"Performance").
- Chain walk to genesis: ≤ 1s p99 (Merkle-tree-shape; O(log periods)).
- Bulk verification of N events: parallelisable; ~5ms/event sustained throughput per verification-rest pod.

## Backfill

### Definition

Backfill is the import of historical events from a pre-audit-chain source (e.g., a tenant migrated from another platform; a historical pre-Bominal-cutover archive). Backfill events are **inserted into the chain as new emissions** with a `backfilled: true` envelope label and `original_emitted_at` field carrying the historical timestamp.

### Contract

1. The backfill operator (a 2-person-rule JIT-elevated principal) authors a backfill manifest:
```yaml
backfill_id: ULID
pack: <pack>
tenant_partition: <partition>
source: <source-system-identifier>
event_count: <N>
window_original_start: <ISO8601>
window_original_end: <ISO8601>
data_class_distribution: { ... }
reason: <RFC-shaped justification>
approver_principals: [<requester>, <approver>]
```
2. Backfill ingest writes events to the WAL with `backfilled: true` + `original_emitted_at` preserved + `backfilled_emitted_at = now()` (the chain timestamp); SealRecord ties this to the current epoch's key.
3. Backfill emissions are **NOT** retroactively merged into historical periods. They occupy current periods with a label distinguishing them from live events.
4. Every backfill emits a `BackfillExecuted` event sealed in the chain (so the backfill is itself audit-trailed).

### Constraints

- Backfill DOES NOT mutate the existing chain. Events are appended to current periods only.
- Pre-Bominal events that lack original Merkle proofs cannot be claimed as "sealed at their original time"; they are sealed at backfill time with a known later timestamp.
- A backfilled event's verification returns:
  - `sealed_at: <backfill-seal-time>`
  - `original_emitted_at: <historical-time>`
  - `is_backfilled: true`
  Verifiers can distinguish; auditors are informed in the bundle.

### Throughput

- Backfill ingest rate: capped at 50% of pack baseline emission capacity to avoid impacting live traffic; configurable per-backfill.
- A 10M-event backfill at XS-tier baseline: ~6h ingest + sealing.

### Approval

Per `policy/ci-scope.cedar` §"PERMIT 5" — backfill is one of the 2-person-rule operator actions. Approval recorded in OpenBao audit log + chain.

## Replay

### Definition

Replay re-runs verification across an arbitrary subset of past periods, producing a fresh evidence-bundle attesting to the integrity status as-of replay time. Used for:
- Quarterly auditor-ready evidence freeze.
- Post-incident integrity validation across a time window.
- Regulator inquiry response with frozen artifact.

### Contract

1. Replay-operator (auditor JIT token OR ops-compliance JIT-elevated principal) invokes:
   ```bash
   cargo run -p oya-dev-cli -- audit-chain replay \
     --pack <pack> \
     --tenant <tenant_id-or-all-tenant-partitions> \
     --start <ISO8601> \
     --end <ISO8601> \
     --output evidence/replay/<replay_id>.bundle
   ```
2. Replay reads all SealRecords + Merkle proofs for the (pack, tenant, time-range).
3. Independently verifies every period root against KeyResolver-resolved public key.
4. Produces a signed bundle:
   ```json
   {
     "replay_id": "ULID",
     "pack": "<pack>",
     "tenant_partition": "<partition>",
     "time_range": { "start": "...", "end": "..." },
     "periods_verified": N,
     "verification_pass_count": N,
     "verification_fail_count": 0,
     "fail_details": [],
     "replay_executed_at": "...",
     "replay_signed_by_key_fp": "<current-active-key-fingerprint>",
     "replay_signature": "<Ed25519 over the bundle metadata>"
   }
   ```
5. Bundle is itself sealed in the chain (replay-of-the-chain is audit-trailed).

### Constraints

- Replay does NOT mutate any past SealRecord.
- If replay detects a verification failure (verify returns false for any period), the bundle's `verification_fail_count > 0` AND a Sev-1 incident is automatically declared per `failure-modes.md` FM-10.
- Replay cost: proportional to (periods × events_per_period) verifications; ~5ms per verification; a 90d window at production-tier emission rate ≈ 8M verifications ≈ 11h sustained on one verification-rest pod; parallelised across the pod fleet typically ≤ 30 min.

### Frequency

- Quarterly automated replay: every pack, all tenants, last-90d window. Result published to `evidence/replays/<year>-q<n>/<pack>.bundle`.
- Ad-hoc replay: per auditor engagement or incident.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Single-event verification | per-request | $0.000001 |
| Replay (90d × all tenants × one pack at XS scale) | quarterly | $50 (compute) |
| Replay (90d × one tenant × all packs at M scale) | per-auditor-engagement | $200 (compute) |
| Backfill (10M events) | one-shot per migration | $10 (compute) + storage based on event size |

Costs surface in `cost-budget.md` §"Cost-Optimisation Levers".

## Limitations

- Backfill events are timestamped both at original-time and backfill-time; auditors must understand this distinction. The bundle metadata makes it explicit.
- Replay depends on KeyResolver having all historical key entries; if a key was retired and its public-key record lost (which shouldn't happen — keys are retained indefinitely), verification of that period fails.
- Cross-pack replay is not supported by a single command; each pack is independently replayed.

## References

- Bominal ADR-0028 §"No retroactive re-sign".
- Bominal ADR-0003 §"Idempotency" + §"Emission attribution".
- `microservices/audit-chain/PRD.md` FR-04 + FR-10.
- `microservices/audit-chain/capacity-model.md`.
- `microservices/audit-chain/cost-budget.md`.
- `microservices/audit-chain/policy/seal-integrity.md` §"SI-13" KeyResolver.
- `/specs/audit-chain-merkle-ed25519.json`.
- RFC 6962 Merkle-tree shape (verification semantics).
