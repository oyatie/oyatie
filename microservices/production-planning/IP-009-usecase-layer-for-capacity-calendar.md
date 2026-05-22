---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-suite
ip_id: IP-009
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-CRP (Capacity Requirements Planning) + factory calendar use-cases
tenant_class: substrate
persona: shop-floor-supervisor
---

# IP-009: Usecase layer for capacity-calendar

## A. Intent

Wires the pure `FactoryCalendar` + `WorkCenterCapacity` domain (IP-003) to ports for persistence, Cedar gating, AsyncAPI consumption (downtime overlays from `plant-maintenance` / `quality-management`), and projection caching. Equivalent to SAP `CR01-CR03` (work-center), `SCAL` (factory calendar admin), and the capacity-evaluation behind `CM01-CM33`.

### A.1 Why orchestration here is non-trivial

Downtime overlays arrive asynchronously from external µservices (PM maintenance windows, QM inspection holds). The usecase enforces:

1. Tenant pin on every consumed event.
2. HLC ordering — late-arriving overlays merge into the right position in the interval timeline.
3. Eventual consistency window: projection cache invalidates after every overlay change but reads served from cache during quiescent periods.

## B. Acceptance criteria

- **AC-1:** `PublishFactoryCalendarUseCase::execute` Cedar-gated, persists calendar, dispatches `factory-calendar-published.v1`.
- **AC-2:** `UpsertWorkCenterCapacityUseCase::execute` idempotent on `(tenant_id, work_center_id, version)`.
- **AC-3:** `IngestDowntimeOverlayUseCase::execute(event)` consumes external AsyncAPI events; rejects mismatched tenant.
- **AC-4:** `QueryAvailableCapacityUseCase::execute(window)` reads from cache; cache invalidated on overlay upsert.
- **AC-5:** Cache TTL ≤ 5min; freshness floor advertised in response header.
- **AC-6:** Conflict detection: overlapping overlays for same WC + same window flag `CapacityAnomaly::DowntimeOverlap`.
- **AC-7:** Cedar default-deny on all writes; reads gated by `production_planning::capacity::read`.
- **AC-8:** Audit emission per ADR-0263; HLC stamping per ADR-0297.

## C. Verification

```bash
cargo test -p oya-production-planning-capacity-usecase -- publish_happy_path
cargo test -p oya-production-planning-capacity-usecase -- upsert_idempotent
cargo test -p oya-production-planning-capacity-usecase -- ingest_overlay_cross_tenant_rejected
cargo test -p oya-production-planning-capacity-usecase -- query_cache_hit
cargo test -p oya-production-planning-capacity-usecase -- query_cache_invalidated_on_overlay
cargo test -p oya-production-planning-capacity-usecase -- overlay_conflict_anomaly
cargo test -p oya-production-planning-capacity-usecase -- cedar_deny_on_publish
cargo test -p oya-production-planning-capacity-usecase -- hlc_ordering_late_arrival
```

## D. Detailed mechanics

### D-1. Publish / upsert use-cases

```rust
pub struct PublishFactoryCalendarUseCase<R, C, O, A> { /* fields */ }
impl<R: CalendarRepository, C: CedarEvaluator, O: OutboxDispatcher, A: AuditEmitter>
    PublishFactoryCalendarUseCase<R, C, O, A>
{
    pub async fn execute(&self, input: PublishCalendarInput) -> Result<PublishOutput, UseCaseError> {
        let decision = self.cedar.evaluate(cedar_req("production_planning::factory_calendar::publish", &input)).await?;
        if !decision.is_permit() { return Err(UseCaseError::PermissionDenied { reason: decision.reasons() }); }
        let cal = FactoryCalendar::new(input.tenant_id, input.plant_code, input.base_pattern,
                                       input.exception_days, input.timezone, input.effective_from)?;
        let tx = self.repo.begin_tx().await?;
        self.repo.save_calendar(&tx, &cal).await?;
        self.outbox.append(&tx, &calendar_published_event(&cal, &decision)).await?;
        self.audit.emit(&tx, AuditEntry::from(&decision, &cal)).await?;
        tx.commit().await?;
        Ok(PublishOutput { calendar_id: cal.calendar_id(), etag: cal.etag(), decision_id: decision.decision_id })
    }
}
```

### D-2. Overlay ingestion (AsyncAPI consumer)

```rust
pub struct IngestDowntimeOverlayUseCase<R, C, O, A> { /* fields */ }
impl<R: CapacityRepository, C: CedarEvaluator, O: OutboxDispatcher, A: AuditEmitter>
    IngestDowntimeOverlayUseCase<R, C, O, A>
{
    pub async fn handle(&self, ev: DowntimeOverlayEvent) -> Result<(), UseCaseError> {
        // Tenant pin defence-in-depth
        if ev.tenant_id != ev.work_center.tenant_id { return Err(UseCaseError::CrossTenant); }
        let decision = self.cedar.evaluate(cedar_req_overlay(&ev)).await?;
        if !decision.is_permit() { return Err(UseCaseError::PermissionDenied { reason: decision.reasons() }); }
        let mut wc = self.repo.load_work_center(&ev.tenant_id, &ev.work_center.work_center_id).await?
            .ok_or(UseCaseError::NotFound)?;
        wc.add_overlay(DowntimeOverlay::from(&ev))?;
        let tx = self.repo.begin_tx().await?;
        self.repo.save_work_center(&tx, &wc).await?;
        self.repo.invalidate_projection_cache(&tx, &wc.work_center_id).await?;
        self.outbox.append(&tx, &overlay_ingested_event(&wc, &decision)).await?;
        self.audit.emit(&tx, AuditEntry::from(&decision, &wc)).await?;
        tx.commit().await?;
        Ok(())
    }
}
```

### D-3. Query use-case (cached projection)

```rust
pub struct QueryAvailableCapacityUseCase<R, C, Cache> { /* fields */ }
impl<R: CapacityRepository, C: CedarEvaluator, Cache: ProjectionCache> QueryAvailableCapacityUseCase<R, C, Cache> {
    pub async fn execute(&self, q: QueryInput) -> Result<QueryOutput, UseCaseError> {
        let decision = self.cedar.evaluate(cedar_req_read(&q)).await?;
        if !decision.is_permit() { return Err(UseCaseError::PermissionDenied { reason: decision.reasons() }); }
        let cache_key = ProjectionCacheKey { tenant: q.tenant_id.clone(), work_center: q.work_center_id.clone(), window: q.window.clone() };
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(QueryOutput { intervals: cached.intervals, freshness: cached.freshness, decision_id: decision.decision_id });
        }
        let calendar = self.repo.load_calendar(&q.tenant_id, &q.plant_code).await?.ok_or(UseCaseError::NotFound)?;
        let wc = self.repo.load_work_center(&q.tenant_id, &q.work_center_id).await?.ok_or(UseCaseError::NotFound)?;
        let intervals = wc.available_intervals(&calendar, q.window.clone())?;
        self.cache.set(&cache_key, &intervals).await?;
        Ok(QueryOutput { intervals, freshness: Hlc::now(), decision_id: decision.decision_id })
    }
}
```

### D-4. Port traits

```rust
#[async_trait]
pub trait CalendarRepository {
    async fn save_calendar(&self, tx: &RepoTx, cal: &FactoryCalendar) -> Result<(), RepoError>;
    async fn load_calendar(&self, tenant: &TenantId, plant: &PlantCode) -> Result<Option<FactoryCalendar>, RepoError>;
}

#[async_trait]
pub trait CapacityRepository {
    async fn save_work_center(&self, tx: &RepoTx, wc: &WorkCenterCapacity) -> Result<(), RepoError>;
    async fn load_work_center(&self, tenant: &TenantId, wc_id: &WorkCenterId) -> Result<Option<WorkCenterCapacity>, RepoError>;
    async fn invalidate_projection_cache(&self, tx: &RepoTx, wc_id: &WorkCenterId) -> Result<(), RepoError>;
}

#[async_trait]
pub trait ProjectionCache {
    async fn get(&self, key: &ProjectionCacheKey) -> Result<Option<CachedProjection>, CacheError>;
    async fn set(&self, key: &ProjectionCacheKey, intervals: &[CapacityInterval]) -> Result<(), CacheError>;
    async fn invalidate(&self, wc_id: &WorkCenterId) -> Result<(), CacheError>;
}
```

### D-5. Audit-event classes

`EVT-PRODUCTION_PLANNING-FACTORY_CALENDAR-PUBLISHED`, `EVT-PRODUCTION_PLANNING-WORK_CENTER-UPSERTED`, `EVT-PRODUCTION_PLANNING-DOWNTIME_OVERLAY-INGESTED`, all per ADR-0263.

### D-6. SLO contribution

- Publish: ≤ 25ms P95.
- Overlay ingest: ≤ 30ms P95.
- Query (cache hit): ≤ 2ms P95.
- Query (cache miss): ≤ 18ms P95.

### D-7. Cross-µservice handoffs

| Direction | Counterparty | Channel |
|---|---|---|
| inbound | `plant-maintenance` | AsyncAPI `plant-maintenance.downtime-window.v1` |
| inbound | `quality-management` | AsyncAPI `quality-management.work-center-hold.v1` |
| outbound | `mrp-run` worker | gRPC `GetAvailableCapacity` |
| outbound | `shop-floor-release` (IP-006) | gRPC `GetAvailableCapacity` |
| outbound | `costing` | ontology projection |

## E. Failure modes & recovery

- **AsyncAPI overlay event from wrong tenant**: rejected; security audit; consumer DLQ.
- **Cache stale read after overlay**: TTL ≤ 5min limits drift; freshness header lets caller decide.
- **Conflicting overlays**: anomaly emitted; ambient alert; runbook `runbooks/downtime-overlap.md`.

## F. Migration

Phase 1: domain (IP-003).
Phase 2: this usecase.
Phase 3 (IP-013): adapter + Postgres + Valkey cache.
Phase 4 (IP-014): REST/gRPC surface.

Rollback: feature flag `production_planning_capacity_usecase_v1` → false.

## G. References

- ADR-0105, ADR-0244, ADR-0252, ADR-0263, ADR-0294, ADR-0297, ADR-0316.
- SAP Help: PP-CRP (CR01-CR03, CM01-CM33), SCAL.
- Benchmarks: SAP PP-CRP | Oracle Fusion Manufacturing capacity | Siemens Opcenter APS | Dassault DELMIA Quintiq | PlanetTogether.

## H. Out-of-scope

- Adapter (IP-013), REST surface (IP-014), domain (IP-003), finite scheduling (IP-020).

— end IP-009 —
