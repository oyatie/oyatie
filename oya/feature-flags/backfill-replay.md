---
doc_class: BackfillReplay
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0028
  - ADR-0263
  - ADR-0276
companion_docs:
  - microservices/feature-flags/runbooks/audit-replay.md
  - microservices/feature-flags/compliance.md
  - microservices/feature-flags/multi-region.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Backfill and Replay — Feature Flags

## Backfill use cases

| Use case | Trigger | Scope |
|---|---|---|
| Audit event gap fill | Cell outage causes missed audit events; events emitted on reconnect | Sealed audit events for flagged time window |
| DSAR export (GDPR Art. 20) | Tenant data portability request | Per-tenant flag definitions + audit_required evaluations |
| Regulator audit pull | QSA (PCI), HIPAA compliance officer, FedRAMP auditor | Sealed audit event chain for specified time range |
| Experiment metric re-attribution | Bug in metric attribution logic; need to re-run attribution over historical events | Per-experiment metric events |
| Cross-cell consistency repair | Cell divergence detected; resync from WAL checkpoint | Flag definitions delta from checkpoint |
| Stale flag definitions after DR failover | DR-pair cell has definitions older than 5s at failover | WAL replay from WAL archive |

## Audit event replay

Per ADR-0028 and ADR-0263:

### Replay procedure

1. Identify gap window: `start_ts` + `end_ts` from incident report or compliance request.
2. Query ClickHouse cold tier for sealed events in window: `SELECT * FROM feature_flags_audit_events WHERE timestamp BETWEEN start_ts AND end_ts AND tenant_id = ?`.
3. Verify chain integrity: `oya-audit-chain-verifier --start $start_ts --end $end_ts --tenant $tenant_id`. Exit code 0 = chain intact; exit code 1 = gap detected.
4. If gap detected: replay from WAL archive. WAL archive retained 30 days; PITR to any point within 30 days.
5. Replay emits `AuditEventReplayed` event (distinguished from original by `replayed: true` flag); sealed with same key as original.
6. Chain re-verification after replay: same command; must return exit 0.

### DSAR export (portability)

Per ADR-0276:

```bash
# Triggered by tenancy µservice DSR cascade
oya feature-flags dsar-export \
  --tenant-id $TENANT_ID \
  --format json \
  --output /secure-export/$TENANT_ID-flags-export.json

# Output schema: flag-definitions-export-v1.json
# Contents: flag definitions + targeting rules (Cedar predicates as text) + experiment configs
# Excludes: other tenants' data; platform-internal fields
# Encrypted with tenant encryption-key BYOK (ADR-0251 §D-10) or platform DEK
```

Export includes:
- All flag definitions (current + archived within 2 years).
- Experiment configurations.
- Audit events for `audit_required: true` flags (for the requesting tenant).
- Does NOT include: other tenants' data, platform operation logs, Cedar fragment internals.

## Experiment metric re-attribution

When a bug is found in metric attribution logic:

1. Stop the affected experiment (via `runbooks/experiment-rollback.md`).
2. Identify affected time window.
3. Re-run attribution job: `oya feature-flags experiment reattribute --experiment-id $EXP_ID --start $start --end $end`.
4. Job replays metric events from ClickHouse cold tier; re-computes attribution using corrected logic.
5. New attribution results stored in `experiment_results_v<N>` table (versioned; original not overwritten).
6. Statistical significance re-computed on new attribution.
7. Audit event `ExperimentMetricReattributed` emitted.

## Cross-cell definition resync

After DR failover or Byzantine cell recovery:

```bash
# Identify divergent cell
oya feature-flags consistency-check --cell $CELL_ID

# If divergent, resync from WAL archive
oya feature-flags resync-from-wal \
  --cell $CELL_ID \
  --from-checkpoint $CHECKPOINT_LSN

# Verification
oya feature-flags consistency-check --cell $CELL_ID
# Must return: "CONSISTENT"
```

## Rollback path

Every state change in feature-flags has an explicit rollback:

| Operation | Rollback | Window |
|---|---|---|
| Flag mutation | `UndoFlagMutation` API or `FlagUpdate` with previous values | 15s undo window; unlimited via new mutation |
| Kill-switch engage | `DisengageKillSwitch` (SRE; step-up Class B) | No time limit |
| Experiment activation | `runbooks/experiment-rollback.md` | No time limit |
| Pack overlay activation | Requires pack-engine re-evaluation; cannot be manually reversed | N/A (pack-controlled) |
| Rollout stage advance | `RollbackRolloutStage` (automated on SLO breach or manual) | Per rollout plan |
