# IP-097 — Milvus Cross-Region Replication

**Phase:** PHASE-02-FOUNDRY-DATA-SUBSTRATE
**Owner:** infra (axis-foundry + axis-residency)
**Authority ADRs:** ADR-0049 cross-region replication, ADR-0192 §"Cross-region replication policy", ADR-0010 regional packs, ADR-0009 cell architecture, ADR-0145 inter-microservice communication, ADR-0184 storage tier layering
**Depends on:** IP-091, IP-096
**Status:** Planned
**Phase trace:** PHASE-02 §"Residency-gated replication" (addendum lines 68-74).

## Scope

Implement the cross-region replication policy for Milvus collections, governed by per-tenant **ResidencyClass** (`StrictKr` / `StrictEu` / `KrWithUsFailover` / `Global`) per `oya-tenancy-kernel::ResidencyClass`. Milvus 2.6 supports per-collection replica placement via `db.replica.num` + node-affinity; this IP wires the residency engine to the per-tenant collection lifecycle.

The replication is **policy-driven**: the residency engine reads the tenant's `ResidencyClass` from the tenancy µservice and projects it into:

1. Per-collection replica placement (`replica.num` + per-replica cell affinity).
2. Cross-cell **backup** replication (which cells' backup buckets receive the per-tenant backup objects).
3. Network-policy enforcement (Cedar policies deny cross-region reads where forbidden).

## File targets

| Path | Action | Line range | Notes |
|---|---|---|---|
| `crates/oya-foundry-milvus-residency-engine/Cargo.toml` | create | 1-50 | tonic, sqlx, kernel deps |
| `crates/oya-foundry-milvus-residency-engine/src/main.rs` | create | 1-130 | composition root |
| `crates/oya-foundry-milvus-residency-engine/src/policy.rs` | create | 1-220 | ResidencyClass → placement plan |
| `crates/oya-foundry-milvus-residency-engine/src/placement_applier.rs` | create | 1-180 | applies `replica.num` + node-affinity via adapter |
| `crates/oya-foundry-milvus-residency-engine/src/backup_replicator.rs` | create | 1-160 | cross-cell backup object copy |
| `crates/oya-foundry-milvus-residency-engine/src/cedar_emitter.rs` | create | 1-100 | emits Cedar policy fragments per tenant |
| `crates/oya-foundry-milvus-residency-engine/src/audit_emit.rs` | create | 1-80 | audit-chain events |
| `crates/oya-foundry-milvus-residency-engine/tests/integration/strict_kr_isolation.rs` | create | 1-180 | StrictKR collection never visible from non-kr cells |
| `crates/oya-foundry-milvus-residency-engine/tests/integration/strict_eu_isolation.rs` | create | 1-180 | StrictEU equivalent |
| `crates/oya-foundry-milvus-residency-engine/tests/integration/kr_with_us_failover_path.rs` | create | 1-200 | KrWithUsFailover backup replicates to us-* |
| `crates/oya-foundry-milvus-residency-engine/tests/integration/global_residency.rs` | create | 1-160 | Global tenant replicates everywhere allowed |
| `crates/oya-foundry-milvus-residency-engine/tests/integration/residency_class_change.rs` | create | 1-180 | tenant class change → re-replicate |
| `microservices/foundry/iac/kustomize/overlays/pack-kr/network-policy-egress.yaml` | create | 1-60 | deny egress to non-kr cells |
| `microservices/foundry/iac/kustomize/overlays/pack-eu/network-policy-egress.yaml` | create | 1-60 | deny egress to non-eu cells |
| `microservices/foundry/runbooks/milvus-residency-incident.md` | create | 1-180 | residency violation triage |

## Residency policy matrix (per ADR-0049 + ADR-0010)

| ResidencyClass | Live replicas | Backup replication | Cross-region failover allowed |
|---|---|---|---|
| `StrictKr` | kr-* cells only | kr-* backup buckets only | No |
| `StrictEu` | eu-* cells only | eu-* backup buckets only | No |
| `KrWithUsFailover` | kr-* primary; us-* backup-only | kr-* + us-* (us as DR cold path) | Yes — DR only, audited |
| `Global` | per-cell capacity policy (typically primary cell + 2 cross-region replicas) | global backup distribution | Yes |

## Acceptance criteria

- **StrictKR**: tenant's collection live replicas are scheduled ONLY across kr-* cells; verified by node-affinity inspection + a synthetic cross-region read attempt is denied at the proxy.
- **StrictEU**: tenant's data NEVER leaves eu-* region; verified by network policy + Cedar deny + a synthetic cross-region read attempt.
- **KrWithUsFailover**: tenant's primary live replicas in kr-* cells; backup objects also present in us-* backup bucket (DR path per ADR-0049).
- **Global**: tenant's replicas distributed per the cell-capacity planner's recommendation; minimum 3 replicas across ≥ 2 regions.
- **Residency class change** (e.g., `Global` → `StrictKr`): replicas outside kr-* are drained within 24h; backup objects outside kr-* are quarantined + purged on the next backup cycle.
- **Audit chain** emits `tenant.residency.replication.{applied,changed,violation_detected}`.
- **Cedar policy fragment** generated per tenant; loaded by foundry-providers-rest middleware (denies cross-region reads).
- **Network policy** at the cluster edge enforces egress denial — defense in depth.
- **Residency violation detection** runs every 5min; any violation pages foundry-oncall + residency-oncall + emits high-severity audit event.

## Test plan

| Test | Verifies |
|---|---|
| `test_strict_kr_no_replica_outside_kr_cells` | replica node-affinity excludes non-kr |
| `test_strict_kr_cross_region_read_denied_at_proxy` | runtime denial |
| `test_strict_kr_cross_region_read_denied_by_cedar` | Cedar deny in middleware |
| `test_strict_kr_cross_region_read_denied_by_netpol` | network-policy deny |
| `test_strict_eu_equivalent` | full StrictEU isolation matrix |
| `test_kr_with_us_failover_backup_replicated` | us-* backup bucket has the backup object |
| `test_kr_with_us_failover_no_live_us_replica` | us-* has no live replica (backup-only) |
| `test_global_replicas_across_regions` | ≥ 2 regions, ≥ 3 replicas |
| `test_residency_class_change_drains_excess` | residency tightening drains excess replicas in 24h |
| `test_residency_change_purges_extraneous_backups` | tightening purges extra-region backups |
| `test_residency_violation_pages_oncall` | injected violation → alert fires |
| `test_audit_event_emitted_on_apply` | `tenant.residency.replication.applied` audited |
| `test_cedar_fragment_loaded_by_router` | foundry-providers-rest enforces fragment |

## Evidence emission

- **Audit chain (ADR-0145):** `tenant.residency.replication.{applied,changed,violation_detected,purged}`, `tenant.residency.cedar.fragment.loaded` to `oya.foundry.audit.milvus.residency`.
- **Metrics:** `foundry_milvus_residency_replicas_by_region{tenant_id,region}`, `foundry_milvus_residency_violations_total{class}`, `foundry_milvus_residency_drain_pending_seconds{tenant_id}`.
- **Evidence pack:** `evidence/residency/milvus-residency-attestation-<tenant_id>-<date>.json` — per-tenant attestation of replica geography + Cedar fragment hash.
- **DR-drill evidence:** quarterly residency drill simulates a forced failover for KrWithUsFailover tenants; report at `evidence/dr-drills/milvus-residency-<date>.json`.

## Rollback procedure

1. **Mis-applied residency policy.** Residency engine maintains a `last_applied_state` table in Postgres; rollback re-applies the previous state. Drained replicas may need to be re-created; this is non-destructive but takes minutes.
2. **Network-policy lockout.** If the egress policy mistakenly denies a legitimate path, ops can hot-edit the NetworkPolicy via `kubectl edit` — change is audited but does not require a Helm cycle.
3. **Cedar fragment regression.** Each fragment is versioned; rollback re-loads the prior version via the foundry-providers-rest middleware (zero-downtime).
4. **Residency-violation false positive.** Investigate within 30min; if confirmed false positive, suppress the alert (audited) + open a fix PR.

## Residency violation handling (high-severity)

A residency violation = actual cross-region read or replica placement that contradicts the tenant's `ResidencyClass`. This is **CONTRACTUAL** (per ADR-0049) and may have regulatory consequences (GDPR / Korean Data Localization).

1. Immediate page → foundry-oncall + residency-oncall + compliance-oncall.
2. Quarantine the affected collection (suspend reads via Cedar deny-all for the tenant).
3. Forensic snapshot — full state capture for audit.
4. Notify the tenant within 24h per the breach-notification clause.
5. Post-incident: root-cause + ADR amendment if policy fragment needs strengthening.

## Blocking deps

- IP-091, IP-096 (backups are part of the replication policy surface).
- Tenancy µservice exposes `ResidencyClass` per tenant (cross-µservice contract).
- Multi-region cell deployment (per the capacity µservice).
- ExternalSecret operator + per-region KMS keys.

## Exit criteria

All four residency classes pass their respective test rows in 3 consecutive CI runs; first quarterly residency drill complete with no findings; residency-attestation evidence pack accepted by legal review.

## Out of scope

- Cross-region replication for non-Milvus state (ClickHouse residency lives in IP-021/IP-024; Postgres residency lives in the runtime µservice).
- DSAR / takedown flow (lives in the tenancy + audit µservices).
- Tenant-initiated residency-class change UX (ops portal IP).

## Cedar policy fragment (per-tenant, illustrative)

```cedar
// Generated per-tenant by oya-foundry-milvus-residency-engine
// ResidencyClass = StrictKr
forbid (
    principal,
    action == Action::"milvus.read",
    resource is MilvusCollection
) when {
    resource.collection_qname like "tenant_ten_acme__*"
    && context.caller_cell != "kr-*"
};
forbid (
    principal,
    action == Action::"milvus.write",
    resource is MilvusCollection
) when {
    resource.collection_qname like "tenant_ten_acme__*"
    && context.caller_cell != "kr-*"
};
```

Cedar fragments are loaded by `oya-foundry-providers-router-rest` middleware at request time (per-tenant fragment lookup by `tenant_id`). The fragment's `caller_cell` context attribute is derived from the SPIFFE SVID's cell claim.

## Observability mapping

| Signal | Metric | Alert |
|---|---|---|
| Replicas by region | `foundry_milvus_residency_replicas_by_region{tenant_id,region}` | — |
| Violation count | `foundry_milvus_residency_violations_total{class}` | `MilvusResidencyViolation` (any > 0 → page high-sev) |
| Drain pending | `foundry_milvus_residency_drain_pending_seconds{tenant_id}` | `MilvusResidencyDrainStalled` (> 24h) |

## Capacity sizing

| Resource | Residency engine (per replica) | Replicas |
|---|---|---|
| CPU request | 0.5 | 2 (HA; one leader, one warm) |
| CPU limit | 1 | — |
| Memory request | 1Gi | — |
| Memory limit | 2Gi | — |

## References

- ADR-0049 — cross-region replication.
- ADR-0192 §"Cross-region replication policy".
- ADR-0010 — regional packs.
- ADR-0009 — cell architecture.
- ADR-0145 — communication reform.
- Kernel: `oya-tenancy-kernel::ResidencyClass`.
- Runbook: `microservices/foundry/runbooks/milvus-residency-incident.md`.
