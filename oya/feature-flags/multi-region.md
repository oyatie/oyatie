---
doc_class: MultiRegion
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0248
  - ADR-0251
  - ADR-0252
  - ADR-0284
companion_docs:
  - microservices/feature-flags/ARCHITECTURE.md
  - microservices/feature-flags/capacity-model.md
  - microservices/feature-flags/manifest.json
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Multi-Region Architecture — Feature Flags

## Cell topology (ADR-0248)

Feature-flags deploys to **Tier 2** (control-plane substrate) cells in every active region.

| Cell | Tier | Role | Sovereign constraint |
|---|---|---|---|
| `us-east-cell-1` | Tier 2 | US primary | None |
| `us-west-cell-1` | Tier 2 | US DR-pair | None |
| `eu-west-cell-1` | Tier 2 | EU primary | GDPR data-residency hard-stop |
| `eu-central-cell-1` | Tier 2 | EU DR-pair | GDPR data-residency hard-stop |
| `kr-cell-1` | Tier 2 | KR sovereign | KR-ISMS-P + KR-PIPA data-residency |
| `us-gov-cell-1` | Tier 2 | FedRAMP | FedRAMP-High boundary |
| `jp-cell-1` | Tier 2 | JP | Act on Protection of Personal Information |

DR-pair cells are active-passive; flag evaluation continues from DR-pair within ≤15s of primary cell failure (failover per ADR-0241 DR-pair failover runbook).

## Flag definition replication

Flag definitions are the authoritative source of truth in each cell. Replication model:

1. **Home cell** (determined by tenant's `home_cell` field): primary write destination for all flag mutations.
2. **Cross-region WAL streaming** (Patroni): changes propagate to all other cells asynchronously. Target lag: ≤5s p99.
3. **Kill-switch overrides**: use Kafka broadcast path (not Postgres WAL) to guarantee ≤1s propagation to all cells globally. Kill-switch values stored in a separate `killswitch_state` table with higher replication priority.

### Replication lag behavior

| Scenario | Behavior |
|---|---|
| Normal operation | Flag definition available in all cells ≤5s after write |
| Kill-switch activation | Propagated to all cells ≤1s via Kafka broadcast |
| Home-cell unavailable (write path) | Write fails; client retries against DR-pair cell (automatic failover by SDK) |
| DR-pair cell unavailable (read path) | Evaluation falls back to last-known-good (LKG) cache (30min TTL on disk) |
| Full regional partition | LKG cache serves evaluation; mutations blocked until partition heals |

## Sovereign-cell awareness

### EU (GDPR)

- EU tenant flag definitions MUST NOT be replicated outside `eu-*` cells.
- Enforced at the Patroni replication configuration: `eu-*` cells form an isolated replication group.
- EU evaluation requests routed to `eu-*` cells via Anycast DNS (per ADR-0253 HTTPS RR).
- Pack overlay: `gdpr-eu` pack forces consent-related flags off by default.

### KR (KR-PIPA, KR-ISMS-P)

- KR tenant data stays in `kr-cell-1`. No cross-border replication of flag definition or evaluation data.
- KR-FSS pack overlays apply financial flag restrictions.
- Backup: `kr-dr-cell-1` (in-country DR pair).

### FedRAMP

- FedRAMP tenant evaluation occurs only within `us-gov-cell-1` boundary.
- FIPS-140-3 cryptographic modules enforced on all cryptographic operations in FedRAMP cells.
- No commercial-cell replication of FedRAMP flag definitions.

## Failure modes across regions

### Scenario 1: Primary cell network partition

```
Primary cell isolated →
  SDK client retries with exponential backoff (3 attempts, 100ms → 200ms → 400ms) →
  SDK falls back to LKG cache (if ≤30min stale) →
  SDK returns default variant if cache expired
  
SLO impact: flag-state-propagation SLO degraded; alert fires
Recovery: automated failover to DR-pair cell in ≤15s (per ADR-0241)
```

### Scenario 2: Cross-region replication lag spike

```
Patroni WAL lag > 5s →
  Metric `oya_feature_flag_replication_lag_seconds` fires alert →
  On-call investigates: network congestion vs. write storm
  Mitigation: kill-switch path is NOT affected (uses Kafka, not WAL)
  Non-kill-switch flags: clients continue serving the stale value from LKG cache
```

### Scenario 3: Kill-switch during regional outage

```
SRE activates kill-switch →
  Kafka broadcast to all reachable cells: ≤1s
  Unreachable cells: Kafka message queued; delivered when cell reconnects
  LKG cache on unreachable cells: kill-switch value NOT in LKG (kill-switch uses in-memory override)
  
IMPORTANT: Kill-switch takes effect on all REACHABLE cells immediately.
           Unreachable cells deliver the kill-switch when Kafka connectivity restores.
           Design choice: availability of kill-switch on reachable cells > consistency with unreachable cells.
```

### Scenario 4: Byzantine cell (data corruption)

```
Cell emits inconsistent flag values →
  Cross-cell consistency probe (runs every 30s via synthetic evaluation) detects divergence →
  `FlagStateInconsistencyDetected` event emitted →
  Affected cell isolated from replication mesh →
  Cells resynced from WAL checkpoint
```

## Time coordination (ADR-0252)

- HLC used for all flag-state-changed event timestamps.
- Kill-switch activation uses TrueTime (opt-in) to ensure absolute ordering across cells.
- HLC drift tolerance: ≤100ms between cells (Patroni logical clock advancement).

## Backup and DR

- Daily Postgres snapshots per cell; retained 30 days.
- Point-in-time recovery (PITR): ≤1 minute RPO via WAL archiving.
- RTO: ≤15 minutes (DR-pair failover); ≤4 hours (full cell restoration from backup).
- Backup portability: JSON export per ADR-0276 for DSAR cascade.
- Cross-cell backup: each cell backs up independently (data-residency compliance); no cross-sovereign backup.

## manifest.json cell_eligibility

```json
"cell_eligibility": {
  "tier": 2,
  "cells": ["us-east-cell-1", "us-west-cell-1", "eu-west-cell-1", "eu-central-cell-1", "kr-cell-1", "us-gov-cell-1", "jp-cell-1"],
  "dr_pair_mapping": {
    "us-east-cell-1": "us-west-cell-1",
    "eu-west-cell-1": "eu-central-cell-1",
    "kr-cell-1": "kr-dr-cell-1"
  },
  "sovereign_cells": {
    "eu-west-cell-1": "gdpr-eu",
    "kr-cell-1": "kr-isms-p",
    "us-gov-cell-1": "fedramp-high"
  }
}
```
