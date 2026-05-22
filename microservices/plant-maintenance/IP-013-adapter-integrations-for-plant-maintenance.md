---
doc_class: ImplementationPlan
ip_id: IP-013
microservice: plant-maintenance
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0329, ADR-0330, ADR-0331]
journey_ref: j208-asset-lifecycle-and-maintenance
sap_submodule: Adapter layer over SAP Plant Connectivity (SAP PCo) + Asset Intelligence Network (AIN) + ECC/S4HANA RFC + S4HANA OData + SAP DMC + IBM Maximo REST + Infor EAM SOAP + Oracle Fusion REST + IFS Cloud REST
service_surface: substrate
persona: integration-engineer, maya-okafor (reliability), elena-volkov (data steward)
status: Accepted
date: 2026-05-20
owner_team: axis-plant-maintenance + axis-erp-parity + axis-integrations
planned_enforcement_ref: oya-governance-plant-maintenance-doc-suite
---

# IP-013: Adapter integrations for `plant-maintenance` — Postgres, Kafka, SAP/Maximo/Infor adapters

## A. Intent

Implements **layer-2 (adapters)** per ADR-0105 — the concrete implementations of the IP-001..006 domain port traits + IP-007..012 use-case dependencies. Adapters bridge to: **PostgreSQL 16** (repositories), **Apache Kafka 3.7** (outbox + ingestion), **gRPC clients** (inventory-management, identity, workplace-integration, production-planning, ontology), **OpenBao** (secrets), **Cedar 4.2 LTS** (policy evaluator), and **per-vendor ERP adapters** (SAP S/4HANA, IBM Maximo, Infor EAM, Oracle Fusion EAM, IFS Cloud, GE Digital APM).

Industry-precedent equivalents: SAP PCo (Plant Connectivity) — the vendor-neutral SCADA-to-ERP bridge; AWS IoT Greengrass; Azure IoT Edge; the Camel ESB integration-pattern catalogue. Hyperscaler analog: AWS Database Migration Service connector pattern (per-source, per-target adapter classes).

### A.1 Why adapters are non-trivial

1. **Outbox-as-table pattern.** Postgres `outbox_event` row + Debezium-shape change-data-capture into Kafka. Atomic with the domain write.
2. **Cedar adapter is hot path.** Every use-case calls Cedar; the adapter caches the policy bundle in-process (max age 60s) and uses zero-copy entity store.
3. **Per-vendor ERP adapters are dialect-translation.** SAP RFC dialect ≠ Maximo REST dialect ≠ Infor SOAP dialect. Each adapter is a separate crate; core stays vendor-neutral.
4. **Connection management.** PG pool sized per cell tier; Kafka producer batched at 16ms / 64KB; gRPC channel pool per upstream with keepalive 30s.
5. **Schema migrations.** All migrations are `sqlx-migrate`-style; idempotent; auditable via `oya-cloud-tenancy.migration_audit`.
6. **mTLS everywhere.** All inter-µservice gRPC uses SPIFFE workload identity per ADR-0295.

## B. Acceptance criteria

- **AC-1:** `PostgresEquipmentRepository`, `PostgresFlocRepository`, `PostgresMaintenancePlanRepository`, `PostgresWorkOrderRepository`, `PostgresReservationRepository`, `PostgresDispatchRepository`, `PostgresDowntimeRepository` implement their port traits.
- **AC-2:** `KafkaOutboxDispatcher` writes outbox row in tx; Debezium CDC drains to Kafka.
- **AC-3:** `CedarEvaluatorAdapter` caches bundle ≤ 60s; cold-start ≤ 200ms; warm eval ≤ 4ms p95.
- **AC-4:** `InventoryGrpcClient`, `IdentityGrpcClient`, `WorkplaceGrpcClient`, `ProductionPlanningGrpcClient`, `OntologyGrpcClient` use mTLS + SPIFFE; connection pool keepalive 30s.
- **AC-5:** Per-vendor ERP adapters in `crates/oya-plant-maintenance-erp-adapter-{sap-s4,ibm-maximo,infor-eam,oracle-fusion-eam,ifs-cloud,ge-digital-apm}-app`.
- **AC-6:** Schema migrations live in `microservices/plant-maintenance/migrations/` and pass `sqlx-migrate --dry-run`.
- **AC-7:** All gRPC clients honor `policy_bundle_version` propagation header.
- **AC-8:** Outbox lag metric `pm_outbox_lag_seconds` SLO ≤ 30s p95.
- **AC-9:** Cedar adapter exposes `cedar_eval_duration_ms` metric per action.
- **AC-10:** Adapter errors are typed; never raw `Box<dyn Error>` leaks.

## C. Verification

```bash
cargo test -p oya-plant-maintenance-adapter-postgres -- equipment_repo_round_trip
cargo test -p oya-plant-maintenance-adapter-postgres -- floc_dag_validation
cargo test -p oya-plant-maintenance-adapter-postgres -- wo_state_machine_persistence
cargo test -p oya-plant-maintenance-adapter-postgres -- reservation_partial_issue_persists
cargo test -p oya-plant-maintenance-adapter-postgres -- dispatch_state_persistence
cargo test -p oya-plant-maintenance-adapter-postgres -- downtime_close_persistence
cargo test -p oya-plant-maintenance-adapter-kafka -- outbox_drain_at_least_once
cargo test -p oya-plant-maintenance-adapter-cedar -- bundle_cache_60s_freshness
cargo test -p oya-plant-maintenance-adapter-cedar -- cold_start_eval_under_200ms
cargo test -p oya-plant-maintenance-adapter-grpc -- inventory_client_mtls_spiffe
cargo test -p oya-plant-maintenance-adapter-grpc -- identity_client_propagates_bundle_version
cargo test -p oya-plant-maintenance-erp-adapter-sap-s4 -- equi_iflot_export_roundtrip
cargo test -p oya-plant-maintenance-erp-adapter-ibm-maximo -- workorder_table_export_roundtrip
cargo test -p oya-plant-maintenance-erp-adapter-infor-eam -- r5events_export
cargo test -p oya-plant-maintenance-erp-adapter-oracle-fusion-eam -- wie_work_orders_vl_export
cargo test -p oya-plant-maintenance-adapter-postgres -- migrations_apply_clean
```

## D. Detailed mechanics

### D-1. Postgres adapter — `EquipmentRepository`

```rust
pub struct PostgresEquipmentRepository {
    pool: PgPool,
    metrics: AdapterMetrics,
}

#[async_trait]
impl EquipmentRepository for PostgresEquipmentRepository {
    async fn save(&self, tx: &RepoTx, eq: &Equipment) -> Result<(), RepoError> {
        let timer = self.metrics.start("equipment_save");
        sqlx::query!(
            r#"
            INSERT INTO plant_maintenance.equipment
              (tenant_id, equipment_id, floc_id, equipment_class, serial_no,
               manufacturer, model_no, construction_year, installation_date,
               abc_indicator, cost_center, state, residency_pack, data_class,
               hlc, schema_version, decision_id)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            ON CONFLICT (tenant_id, equipment_id)
            DO UPDATE SET floc_id=$3, state=$12, hlc=$15, schema_version=$16, decision_id=$17
            WHERE plant_maintenance.equipment.hlc < EXCLUDED.hlc
            "#,
            eq.tenant_id.as_str(), eq.equipment_id.as_str(),
            eq.floc_id.as_str(), eq.equipment_class.as_str(),
            eq.serial_no.as_deref().map(|s| s.as_str()),
            eq.manufacturer.as_deref(), eq.model_no.as_deref(),
            eq.construction_year.map(|y| y as i32),
            eq.installation_date,
            eq.abc_indicator.map(|a| a.to_string()),
            eq.cost_center.as_deref().map(|c| c.as_str()),
            eq.state.to_string(),
            eq.residency_pack.as_str(),
            eq.data_class.to_string(),
            eq.hlc.as_str(),
            eq.schema_version as i32,
            eq.decision_id.as_uuid(),
        )
        .execute(tx.pg())
        .await
        .map_err(|e| { self.metrics.fail("equipment_save"); RepoError::Db(e.to_string()) })?;
        timer.observe();
        Ok(())
    }

    async fn load(&self, tenant: &TenantId, id: &EquipmentId) -> Result<Option<Equipment>, RepoError> {
        let row = sqlx::query_as!(
            EquipmentRow,
            "SELECT * FROM plant_maintenance.equipment WHERE tenant_id=$1 AND equipment_id=$2",
            tenant.as_str(), id.as_str(),
        ).fetch_optional(&self.pool).await.map_err(|e| RepoError::Db(e.to_string()))?;
        Ok(row.map(Into::into))
    }
    // ... rest of trait
}
```

### D-2. Kafka outbox dispatcher

```rust
pub struct KafkaOutboxDispatcher {
    pool: PgPool,
    kafka_producer: rdkafka::producer::FutureProducer,
    metrics: AdapterMetrics,
}

#[async_trait]
impl OutboxDispatcher for KafkaOutboxDispatcher {
    async fn append(&self, tx: &RepoTx, evt: &Event) -> Result<(), OutboxError> {
        sqlx::query!(
            r#"INSERT INTO plant_maintenance.outbox_event
                 (event_id, tenant_id, channel, payload, hlc, created_at)
               VALUES ($1, $2, $3, $4, $5, now())"#,
            evt.event_id.as_uuid(),
            evt.tenant_id.as_str(),
            evt.channel.as_str(),
            sqlx::types::Json(&evt.payload) as _,
            evt.hlc.as_str(),
        ).execute(tx.pg()).await?;
        Ok(())
    }
}

// Debezium drains plant_maintenance.outbox_event → Kafka topic per channel
```

### D-3. Cedar evaluator adapter — bundle cache

```rust
pub struct CedarEvaluatorAdapter {
    policy_loader: PolicyLoader,
    cache: ArcSwap<CachedBundle>,
    metrics: AdapterMetrics,
}

#[async_trait]
impl CedarEvaluator for CedarEvaluatorAdapter {
    async fn evaluate(&self, req: AuthzRequest) -> Result<Decision, CedarError> {
        let timer = self.metrics.start("cedar_eval");
        let snapshot = self.cache.load();
        if snapshot.is_stale(/*max_age*/ Duration::seconds(60)) {
            self.refresh_bundle_async();
        }
        let raw = snapshot.bundle.is_authorized(&req.into_cedar());
        timer.observe();
        Ok(Decision::from_cedar(raw))
    }
}
```

### D-4. gRPC client — inventory

```rust
pub struct InventoryGrpcClient {
    channel: tonic::transport::Channel,
    spiffe_id: SpiffeId,
}

impl InventoryGrpcClient {
    pub fn new(endpoint: Uri, spiffe_id: SpiffeId, tls: tonic::transport::ClientTlsConfig) -> Self {
        let channel = tonic::transport::Channel::builder(endpoint)
            .tls_config(tls).unwrap()
            .keep_alive_while_idle(true)
            .http2_keep_alive_interval(Duration::from_secs(30))
            .connect_lazy();
        Self { channel, spiffe_id }
    }
}

#[async_trait]
impl InventoryClient for InventoryGrpcClient {
    async fn atp_and_soft_reserve(&self, tenant: &TenantId, items: &[ReservationItem]) -> Result<SoftReserve, InvError> {
        let mut client = pb::inventory_service_client::InventoryServiceClient::new(self.channel.clone());
        let req = build_atp_request(tenant, items);
        let resp = client.atp_and_soft_reserve(req).await.map_err(InvError::Grpc)?;
        Ok(resp.into_inner().into())
    }
}
```

### D-5. Per-vendor ERP adapter — SAP S/4HANA

```rust
pub struct SapS4Hana4Adapter { client: SapOdataClient }

impl SapS4Hana4Adapter {
    pub async fn export_equipment(&self, plant: &PlantCode) -> Result<Vec<Equipment>, ErpError> {
        let raw = self.client.get(&format!("/sap/opu/odata/sap/I_MaintenanceEquipment?$filter=Plant eq '{}'", plant)).await?;
        raw.into_iter().map(|r| Equipment::try_from(r).map_err(ErpError::Translate)).collect()
    }
    pub async fn export_workorder(&self, plant: &PlantCode) -> Result<Vec<WorkOrder>, ErpError> {
        let raw = self.client.get(&format!("/sap/opu/odata/sap/I_MaintenanceOrder?$filter=Plant eq '{}'", plant)).await?;
        raw.into_iter().map(|r| WorkOrder::try_from(r).map_err(ErpError::Translate)).collect()
    }
}
```

### D-6. Schema migrations

```
microservices/plant-maintenance/migrations/
  20260520_001_create_functional_location.sql
  20260520_002_create_equipment.sql
  20260520_003_create_equipment_characteristic.sql
  20260520_004_create_equipment_state_audit.sql
  20260520_005_create_maintenance_plan.sql
  20260520_006_create_maintenance_plan_counter.sql
  20260520_007_create_maintenance_strategy_package.sql
  20260520_008_create_plan_completion_audit.sql
  20260520_009_create_work_order.sql
  20260520_010_create_work_order_operation.sql
  20260520_011_create_work_order_component.sql
  20260520_012_create_work_order_confirm.sql
  20260520_013_create_work_order_state_audit.sql
  20260520_014_create_reservation.sql
  20260520_015_create_reservation_item.sql
  20260520_016_create_reservation_audit.sql
  20260520_017_create_dispatch.sql
  20260520_018_create_dispatch_required_skill.sql
  20260520_019_create_dispatch_required_cert.sql
  20260520_020_create_dispatch_audit.sql
  20260520_021_create_downtime_window.sql
  20260520_022_create_downtime_shift_split.sql
  20260520_023_create_oee_rollup_cache.sql
  20260520_024_create_outbox_event.sql
```

### D-7. Per-vendor adapter catalog

| Vendor | Source surface | Adapter crate | Idempotency strategy |
|---|---|---|---|
| SAP S/4HANA | OData I_Maintenance* + RFC BAPI_PM_* | `oya-plant-maintenance-erp-adapter-sap-s4-app` | EQUNR, AUFNR are stable; (tenant, equnr) idempotent |
| IBM Maximo | REST `mxasset, mxapiwo, mxapipm` | `oya-plant-maintenance-erp-adapter-ibm-maximo-app` | ASSETNUM, WONUM stable; (tenant, wonum) idempotent |
| Infor EAM | SOAP `MP*, PM*, WO*` | `oya-plant-maintenance-erp-adapter-infor-eam-app` | OBJECT_CODE, EVENT_CODE stable |
| Oracle Fusion EAM | REST `assets, workOrders, maintenancePrograms` | `oya-plant-maintenance-erp-adapter-oracle-fusion-eam-app` | AssetNumber, WorkOrderNumber stable |
| IFS Cloud | REST `Object, ActiveWorkOrder, PMAction` | `oya-plant-maintenance-erp-adapter-ifs-cloud-app` | OBJID stable |
| GE Digital APM | REST `MI_EQUIPMENT*, MI_WORK_ORDER*` | `oya-plant-maintenance-erp-adapter-ge-digital-apm-app` | (tenant, family_id, entity_key) idempotent |

### D-8. SLO targets (adapter overhead)

| Adapter | p50 | p95 | p99 |
|---|---|---|---|
| Postgres save (equipment) | 2 ms | 5 ms | 12 ms |
| Postgres load (equipment) | 0.8 ms | 2 ms | 5 ms |
| Cedar eval (warm) | 1 ms | 4 ms | 9 ms |
| Cedar bundle refresh (cold) | 80 ms | 180 ms | 350 ms |
| Kafka outbox row insert | 1 ms | 3 ms | 7 ms |
| Kafka drain to topic | 50 ms (debezium lag) | 200 ms | 800 ms |
| gRPC inventory roundtrip (intra-cell) | 8 ms | 18 ms | 38 ms |
| gRPC inventory roundtrip (inter-cell) | 28 ms | 65 ms | 130 ms |

### D-9. Audit-event registry (adapter-level)

| Event class | Severity | Emitter |
|---|---|---|
| `EVT-PLANT_MAINTENANCE-ADAPTER-OUTBOX_LAG_HIGH` | warning | scheduler |
| `EVT-PLANT_MAINTENANCE-ADAPTER-CEDAR_BUNDLE_STALE` | warning | adapter |
| `EVT-PLANT_MAINTENANCE-ADAPTER-GRPC_CIRCUIT_OPEN` | warning | adapter |
| `EVT-PLANT_MAINTENANCE-ADAPTER-PG_TX_RETRY_EXCEEDED` | warning | adapter |
| `EVT-PLANT_MAINTENANCE-ADAPTER-MIGRATION_APPLIED` | informational | adapter |
| `EVT-PLANT_MAINTENANCE-ADAPTER-ERP_ADAPTER_TRANSLATION_FAIL` | warning | adapter |

### D-10. Failure modes & recovery

1. **`PgConnectionExhausted`** — pool starvation under load. Circuit-breaker opens; backpressure to use-cases; pool resized after metric review. Runbook `runbooks/pg-pool-exhausted.md`.
2. **`KafkaProducerBackpressure`** — Kafka cluster slow. Outbox table accumulates; Debezium drains; use-cases unaffected. Alert at outbox-table size > 100k rows. Runbook `runbooks/outbox-backlog.md`.
3. **`CedarBundleStale`** — bundle refresh task fails > 5 attempts. Adapter falls back to default-deny; alarm fires; bundle re-fetched on next cron. Runbook `runbooks/cedar-bundle-fetch-fail.md`.
4. **`GrpcCircuitOpen`** — downstream µservice unhealthy. Adapter returns `Transient`; use-case fails-fast; client retries with backoff. Runbook `runbooks/grpc-circuit-open.md`.
5. **`ErpAdapterTranslationFail`** — vendor source row doesn't map to canonical model (new SAP field, removed Maximo column). Row to DLQ; integration engineer reviews. Runbook `runbooks/erp-translation-fail.md`.
6. **`MigrationRollbackNeeded`** — migration applied to dev but found incorrect. Forward-only migration policy + dedicated rollback migration; never `ALTER TABLE DROP COLUMN` in same migration as add. Runbook `runbooks/migration-rollback.md`.

### D-11. Cross-µservice handoffs

The adapters serve every cross-µservice handoff listed in IPs 001-012. Adapter layer is the only place gRPC and SQL run.

## E. Failure-mode summary

See D-10.

## F. Migration / rollback

Schema migrations are forward-only; rollback is a forward migration. Vendor-adapter feature flags allow per-vendor disable.

## G. References

- ADR-0105, ADR-0244, ADR-0252, ADR-0263, ADR-0294, ADR-0295 (SPIFFE), ADR-0297, ADR-0314..0316.
- Postgres 16 documentation; Apache Kafka 3.7 producer docs.
- Cedar 4.2 LTS evaluator documentation.
- SAP S/4HANA OData I_Maintenance* catalog; IBM Maximo REST API; Infor EAM SOAP API; Oracle Fusion EAM REST; IFS Cloud OData; GE Digital APM REST.
- Hyperscaler reference patterns: AWS DMS connector pattern; Debezium outbox-pattern documentation.

## H. Out of scope

- REST / gRPC surface (IP-014), integration tests (IP-015), domain logic (IP-001..006).

— end IP-013 —
