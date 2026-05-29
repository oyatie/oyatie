---
doc_id: finops-portal/backfill-plan
authored: 2026-05-18
status: ready
authority: ADR-0197 backup substrate + ADR-0199 FinOps canonical
classification: internal
---

# Backfill + replay plan — finops-portal

This document covers two related concerns:

1. **Backfill**: how to populate `finops-portal` with historical
   data when the µservice ships, OR when a new pack / region is
   added, OR after a data-loss event.
2. **Replay**: how to re-run a deterministic pipeline (e.g.
   re-emit a quarterly regulator envelope after a key rotation).

## Backfill scenarios

### Scenario A — Phase-2 cut-over from OpenCost-only

When `finops-portal` first ships (Phase 2 per ADR-0199), tenants
have historical OpenCost data going back 24 months. The cut-over
strategy:

1. Run the IP-014 FOCUS export pipeline retroactively against
   each completed month from the cut-over date − 24mo through
   cut-over date − 1mo.
2. For each month, produce a `TenantInvoice` per tenant via the
   IP-004 `finalize_invoice` usecase.
3. Mark these invoices `status=finalized_backfill` (a distinct
   status from real-time finalize) so audit-chain seals carry a
   `backfill=true` flag.
4. Quarterly emit for past quarters runs once per quarter at the
   cut-over moment, sealing the past evidence.

### Scenario B — New regulatory pack added

When a new pack (e.g. `us-financial`) is provisioned:

1. Pack overlay landed in repo via PR.
2. Tenants migrated to the pack run through a special initial
   finalize pass that re-applies cost-allocation policies under
   the new pack's defaults.
3. Audit-chain emits `TenantPackMigrated` for each tenant.
4. Quarterly emit picks up the new pack's tenants automatically
   the following quarter.

### Scenario C — Data-loss recovery from backup

Per ADR-0197 backup substrate (cloud-iac µservice owns it):

1. **Postgres**: pgbackrest restore to a point-in-time (PIT) per
   the RPO target (15 min default; 5 min for KR / US-healthcare /
   US-financial).
2. **SeaweedFS** (FOCUS exports + parquet copies): velero/restic
   restore from the daily snapshot.
3. **Audit-chain seals**: the audit-chain µservice is the
   source-of-truth for sealed events; finops-portal reconciles
   to the sealed view after restore.
4. After restore, the reconciler (`runbooks/credit-application-
   reconciliation.md`) re-emits any seals that were processed
   in-process but not yet sealed at the loss moment.

## Replay scenarios

### Replay 1 — Re-emit quarterly envelope after key rotation

If a quarterly Ed25519 key is rotated mid-quarter (rare; usually
quarterly cadence), past envelopes signed under the old key
remain valid (the public key is published on audit-chain for
verifiers). No replay needed unless the key was compromised
(`incident-playbook.md` §Key compromise).

### Replay 2 — Recompute invoices after a cost-allocation policy
amendment

If a cost-allocation policy is retroactively amended (rare; only
under regulator-mandated correction):

1. Run an explicit `replay_invoices` admin tool against the
   affected tenant + period.
2. Produce a **delta invoice** (positive or negative) and apply
   it as a credit-ledger entry.
3. **Never** mutate the original sealed invoice; the delta is the
   correction record.
4. Emit `TenantInvoiceCorrected` audit-chain event with reference
   to the original.

### Replay 3 — Recompute anomaly explanations

Because `explain()` is deterministic (per IP-011 §INV / unit
tests), a replay produces byte-identical output. No replay
needed unless the algorithm itself is changed; algorithm changes
require an ADR + a version bump per `no-silent-regression`.

## Tooling

A CLI tool ships with the app crate:

```sh
oya finops-portal backfill --from 2024-01 --to 2025-12 --tenant T --dry-run
oya finops-portal backfill --from 2024-01 --to 2025-12 --tenant T --commit
oya finops-portal replay-invoice --tenant T --period 2024-Q3 --reason "regulator-correction"
oya finops-portal reconcile-credit-seals --since 2026-04-01
```

Each tool:

- Requires the `ops-finops` group claim.
- Emits an audit-chain event before + after the action.
- Supports `--dry-run` to preview.

## Idempotency

All backfill + replay operations are idempotent:

- Per-period `(tenant, period)` finalize: dedup via the
  IP-004 `finalize_invoice` idempotency invariant.
- Per-id audit seal: dedup at the audit-chain via
  `seal_envelope_hash`.
- Credit-ledger append: dedup via `LedgerEntry.id` uniqueness
  invariant.

Re-running a backfill against the same window produces zero
net side effects on the platform state.

## Time budget

| Scenario                       | Estimated wall-clock        | Resource peak             |
|--------------------------------|------------------------------|----------------------------|
| 24 months × 1k tenants         | 4 h                          | 1 dedicated worker pod     |
| 24 months × 100k tenants       | 48 h (sharded)               | 8 worker pods              |
| Single-tenant PIT restore      | 30 min                       | postgres restore window    |
| Replay 1 quarter for 1 tenant  | 5 min                        | minimal                    |

## Verification

- Each backfill run prints a verification table:
  `tenants_processed`, `invoices_emitted`, `seals_landed`,
  `discrepancies`.
- A discrepancy > 0 stops the run and pages ops-finops.

## References

- ADR-0197 backup substrate.
- ADR-0199 FinOps canonical.
- ADR-0162 audit-log integrity.
- IP-004 + IP-014 + IP-015.
- `runbooks/credit-application-reconciliation.md`.
