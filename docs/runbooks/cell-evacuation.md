---
purpose: Oyatie Runbook — Tier 3 Cell Evacuation
doc_status: published
---

# Oyatie Runbook — Tier 3 Cell Evacuation

> **Status:** Active
> **Owner:** ops-sre-reliability + ops-dr-capacity
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during retired `./bin/oya verify` gate repair sweep)
> **Related ADRs:** ADR-0248 §D-4, ADR-0248 §D-7, ADR-0248 §D-12, ADR-0241, ADR-0009, ADR-0243

---

## §A Trigger Conditions

Initiate this runbook when **any** of the following are true for a Tier 3 data-plane cell:

- Cell hardware failure forecast (storage SMART failures, network-fabric degradation) with >4h lead time.
- Underlying cloud-provider maintenance window that requires node evacuation.
- Cell certification level downgrade (e.g., HSM partition decommission strips `hipaa-certified`).
- Security incident requires isolating the cell (see §§ cross-reference: `docs/runbooks/provider-credential-leak-response.md`).
- `auto-spawn` triggered a replacement cell and the old cell exceeds 70% capacity utilisation sustained ≥30 min (per ADR-0248 §D-10 auto-spawn procedure).
- Operator-initiated migration for regional-pack compliance re-alignment (ADR-0240).

**Do NOT use this runbook for Tier 2 control-plane cell failure** — that procedure differs in scope and is covered by `docs/runbooks/cell-failover-intra-region.md`.

---

## §B Pre-Checks

Complete all pre-checks before beginning evacuation. Estimated time: **10–20 min**.

1. **Identify cell scope.** Retrieve the cell record:
   ```
   oya git exec -- psql -c "SELECT cell_id, tier, certification_levels, tenant_count,
     home_tenant_count, dr_tenant_count, read_replica_tenant_count
     FROM cells WHERE cell_id = '<TARGET_CELL_ID>';"
   ```
   Confirm tier = `TIER_3`. Abort if tier = `TIER_2` (wrong runbook).

2. **List bound tenants.** Retrieve all tenants with `home_cell`, `dr_cell`, or `read_replica_cells` entry pointing to this cell:
   ```
   oya git exec -- psql -c "SELECT tenant_id, home_cell, dr_cell, array_length(read_replica_cells,1) as replica_count
     FROM tenants WHERE home_cell = '<TARGET_CELL_ID>'
        OR dr_cell = '<TARGET_CELL_ID>'
        OR '<TARGET_CELL_ID>' = ANY(read_replica_cells)
     ORDER BY home_cell DESC;"
   ```
   Record counts: `HOME_COUNT`, `DR_COUNT`, `REPLICA_COUNT`.

3. **Verify DR cell readiness for all home-tenants.** For each tenant whose `home_cell` equals the target cell:
   ```
   presubmit (retired CLI gate validate) cell-isolation-tolerance --cell <DR_CELL_ID> --tenant-count <HOME_COUNT>
   ```
   Gate must return `PASS`. If `FAIL`, provision capacity on DR cell or identify an alternate DR cell before proceeding.

4. **Confirm shuffle-sharding service is healthy:**
   ```
   presubmit (retired CLI gate validate) shuffle-sharding-parameters --cell-pool <PACK>
   ```

5. **Declare incident.** Open incident in `#incident-bridge`. Severity: SEV-2 (planned evacuation) or SEV-1 (forced evacuation <4h lead time). Assign incident commander. Notify `ops-compliance` if any tenant on the cell has regulated compliance packs installed.

6. **Check Cedar permit** for evacuation action:
   ```
   cedar-cli authorize \
     --principal "oyatie.ops-sre-reliability.<operator-id>" \
     --action "Cell::Action::InitiateCellEvacuation" \
     --resource "Cell::\"<TARGET_CELL_ID>\""
   ```
   Must return `PERMIT`. If `DENY`, escalate to `council-security` for emergency Cedar override.

---

## §C Procedure

### Step 1 — Halt new tenant placement on target cell (target: ≤2 min)

Mark the target cell `DRAINING` in the cell registry. This prevents the shuffle-sharding service from assigning the cell to newly onboarded tenants.

```
oya git exec -- psql -c "UPDATE cells SET status = 'DRAINING', drain_initiated_at = now()
  WHERE cell_id = '<TARGET_CELL_ID>';"
```

Emit audit event:
```
audit-emit CellEvacuationInitiated \
  --cell-id <TARGET_CELL_ID> \
  --operator oyatie.ops-sre-reliability.<operator-id> \
  --home-tenant-count <HOME_COUNT> \
  --reason "<REASON>"
```

### Step 2 — Migrate read-replica tenants (target: ≤15 min)

For tenants where the target cell is only a `read_replica_cell` (not home or dr), reassign the replica slot to a healthy pool cell. The tenancy-owned cell assignment workflow handles this atomically with candidates from cloud-iac and health exclusions from observability:

```
tenancy cell-migration-plan \
  --evict-cell <TARGET_CELL_ID> \
  --scope read_replica \
  --dry-run   # inspect output first
```

Once output verified:
```
tenancy cell-migration-apply \
  --evict-cell <TARGET_CELL_ID> \
  --scope read_replica \
  --confirm
```

Each migrated tenant emits `TenantCellBindingUpdated` to audit-chain.

### Step 3 — Migrate DR-only tenants (target: ≤20 min)

For tenants where the target cell is `dr_cell` but not `home_cell`:

```
tenancy cell-migration-apply \
  --evict-cell <TARGET_CELL_ID> \
  --scope dr_cell \
  --confirm
```

Verify each tenant's new `dr_cell` satisfies compliance pack cell-pinning rule (ADR-0251 §D-5):
```
presubmit (retired CLI gate validate) tenant-pack-cell-pinning --tenant-batch <BATCH_CSV>
```

All tenants must pass. Blocked tenants require `ops-compliance` review before proceeding.

### Step 4 — Migrate home-tenants (target: ≤60 min; varies with HOME_COUNT)

This is the highest-impact step. For each home-tenant, the migration promotes the DR cell to home and assigns a new DR cell.

**Pre-migration data-sync check:** Confirm the DR cell's replication lag is ≤5s for each tenant:
```
observability cell-replication-lag-check \
  --source-cell <TARGET_CELL_ID> \
  --scope home_tenants
```

If any tenant lag exceeds 5s, wait for sync to catch up before proceeding.

**Execute home-cell promotion via shuffle-sharding service** (per ADR-0248 §D-7 + ADR-0241 DR pairing):

```
tenancy cell-migration-apply \
  --evict-cell <TARGET_CELL_ID> \
  --scope home_cell \
  --traffic-drain-seconds 30 \
  --confirm
```

The `--traffic-drain-seconds 30` flag installs a Cilium network policy to drain in-flight requests from the target cell before the binding swap, targeting ≤30s of per-tenant request queueing. The workflow engine (per ADR-0248 §D-6 async-coordination exemption) buffers in-flight durable workflow executions.

For each batch of ≤50 tenants, the migration step emits:
- `TenantHomeCellMigrationStarted`
- `TenantHomeCellMigrationCompleted` (or `TenantHomeCellMigrationFailed`)

### Step 5 — Drain remaining workloads (target: ≤30 min)

After all tenant bindings are migrated, drain active Kubernetes workloads from the cell:

```
kubectl cordon <NODE_GROUP> --all-namespaces
kubectl drain <NODE_GROUP> --ignore-daemonsets --delete-emptydir-data --grace-period=60
```

The workflow-engine scheduler stops placing new workflow executions on this cell. Running workflow instances drain per ADR-0247 §D-7 "per-instance version pinning" semantics — they continue on the cell until completed or until `emergency-cancel` signal is sent.

For forced evacuations (<4h), send graceful-cancel signal to long-running workflows:
```
workflow-cli signal --cell <TARGET_CELL_ID> --signal drain --timeout 300s
```

### Step 6 — Seal etcd snapshot (target: ≤10 min)

Per ADR-0248 §D-2 bootstrap-cell self-retirement procedure (adapted for Tier 3):

```
etcdctl snapshot save /tmp/cell-<TARGET_CELL_ID>-<TIMESTAMP>.db
vault kv put secret/cell-snapshots/<TARGET_CELL_ID>-<TIMESTAMP> \
  snapshot=@/tmp/cell-<TARGET_CELL_ID>-<TIMESTAMP>.db
```

Seal under the `oyatie.security` key in OpenBao. Record `SNAPSHOT_REF`.

### Step 7 — Mark cell RETIRED and decommission (target: ≤10 min)

```
oya git exec -- psql -c "UPDATE cells SET status = 'RETIRED', retired_at = now(),
  etcd_snapshot_ref = '<SNAPSHOT_REF>'
  WHERE cell_id = '<TARGET_CELL_ID>';"
```

Emit:
```
audit-emit CellEvacuationComplete \
  --cell-id <TARGET_CELL_ID> \
  --etcd-snapshot-ref <SNAPSHOT_REF> \
  --home-tenants-migrated <HOME_COUNT> \
  --dr-tenants-migrated <DR_COUNT>
```

Decommission cloud resources via IaC:
```
cd microservices/cloud-iac/iac/helm/cell-tier-3/
helm uninstall cell-<TARGET_CELL_ID> --namespace cell-<TARGET_CELL_ID>
```

---

## §D Verification

1. **No tenants remain bound to the target cell:**
   ```
   psql -c "SELECT COUNT(*) FROM tenants
     WHERE home_cell = '<TARGET_CELL_ID>'
        OR dr_cell = '<TARGET_CELL_ID>'
        OR '<TARGET_CELL_ID>' = ANY(read_replica_cells);"
   ```
   Must return `0`.

2. **All migrated tenants pass cell-pinning rule** (ADR-0251 §D-5):
   ```
   presubmit (retired CLI gate validate) tenant-pack-cell-pinning --all-tenants
   ```

3. **Cedar fragment cache on receiving cells is current** (≤30s stale per ADR-0248 §D-9):
   ```
   presubmit (retired CLI gate validate) cell-isolation-tolerance --cell <DR_CELL_ID>
   ```

4. **Audit-chain completeness.** Verify each migrated tenant has a `TenantHomeCellMigrationCompleted` event with Merkle proof in the audit chain:
   ```
   audit-verify migration-completeness --cell <TARGET_CELL_ID> --expected-home-count <HOME_COUNT>
   ```

5. **SLO recovery.** Confirm affected tenants' SLO error budgets are recovering on `microservices/observability/dashboards/cellular-topology.md`.

---

## §E Rollback

If tenant migration fails mid-procedure and the target cell is still functional:

1. Mark cell back to `ACTIVE`:
   ```
   psql -c "UPDATE cells SET status = 'ACTIVE', drain_initiated_at = NULL WHERE cell_id = '<TARGET_CELL_ID>';"
   ```

2. Revert any completed binding changes for the affected tenant batch:
   ```
   tenancy cell-migration-revert --batch <BATCH_ID> --confirm
   ```

3. Emit `CellEvacuationRolledBack` to audit-chain.

4. Root-cause the migration failure before re-attempting. If migration failure is due to compliance pack cell-pinning mismatch, resolve via `ops-compliance` before retry.

---

## §F Post-Incident

1. File MFL row for any tenants that experienced >5s disruption during home-cell migration.
2. If evacuation was triggered by hardware failure, update the cell-capacity forecast model.
3. Schedule shuffle-sharding pool rebalance if the evacuation reduced the pool below the shuffle-sharding parameters' minimum pool size (ADR-0248 §D-7).
4. Post-mortem required within 72h for SEV-1 evacuations.
5. Update `docs/runbooks/cell-failover-intra-region.md` if new failure modes were observed.

---

## §G References

- ADR-0248 §D-4 (Tier 3 data plane cells)
- ADR-0248 §D-7 (Shuffle sharding)
- ADR-0248 §D-10 (Cell sizing + auto-spawn)
- ADR-0248 §D-12 (Planned shuffle-sharding migration)
- ADR-0241 (DR pairing)
- ADR-0009 (Cell architecture per-tenant per-region)
- ADR-0251 §D-5 (Tenant-pack cell pinning)
- ADR-0243 §D-6 (Cedar evaluation on data-plane cells)
- `docs/runbooks/cell-failover-intra-region.md`
- `docs/runbooks/compliance-pack-revocation.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
- [SLO-CATALOG.md](../SLO-CATALOG.md)
