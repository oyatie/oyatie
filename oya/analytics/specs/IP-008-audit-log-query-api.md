# IP-008 — Audit-Log Query API

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** backend (council-analytics + axis-audit-chain)
**Authority ADRs:** ADR-0003 audit chain, ADR-0193, ADR-0150 cursor pagination, ADR-0156 PII registry, ADR-0007 Cedar
**Depends on:** IP-005
**Status:** Planned

## Scope

Tenant-scoped audit-log search. ClickHouse-backed; sub-second p99 for the typical 30-day window. Filterable by axis, event type, principal, and date range. The audit-log query API is itself recursive — querying the audit log emits a single meta-event per minute per principal (not per query, to avoid amplification).

Backed by the per-tenant `audit_events` ReplacingMergeTree table (per `iac/clickhouse/mv-templates/audit-log-table.sql`).

## Deliverables

1. `GET /v1/audit-log/search` REST endpoint per OpenAPI 3.2.0 contract.
2. gRPC `SearchAuditLog` RPC per proto3 contract.
3. GraphQL `auditLog` query per SDL.
4. Per-axis index — ClickHouse `MergeTree` `ORDER BY (tenant_id, axis, emitted_at)` ensures axis-scoped queries prune efficiently.
5. Cedar policy `microservices/analytics/policy/audit-log-pii.cedar` (already authored — referenced here) filters PII columns.
6. Recursive observation defense — querying the audit log emits a meta-event.
7. Tier-aware SLO routing: hot-tier queries (≤90d) target p99 < 800ms; cold-tier (>90d) target p95 < 2s.
8. Integration test.

## Acceptance criteria

- `GET /v1/audit-log/search?axis=auth&from=2026-04-18T00:00:00Z&to=2026-05-18T00:00:00Z` returns matching events for the calling tenant.
- p99 ≤ 800ms for 30-day window (sized for tenant_test_tenant at 10M events/month).
- 7-year retention queryable; older data served from cold tier with p95 ≤ 2s.
- Cross-tenant query forbidden (403 + Cedar audit).
- PII columns filtered when caller lacks DUBO grant.
- Recursive audit event emitted once per principal per minute (rate-limited to avoid amplification).
- Cursor pagination opaque + HMAC-signed per ADR-0150.

## Implementation tasks

### T1 — Handler

File: `crates/oya-analytics-api/src/rest/audit_log.rs`

```rust
pub async fn search_audit_log(
    State(app): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AuditLogSearchParams>,
) -> Result<Json<AuditLogPage>, ApiError> {
    let principal = extract_principal(&headers)?;
    let request_id = extract_request_id(&headers);

    // Cedar authorize.
    app.cedar.check_action(
        &principal,
        "QueryAuditLog",
        Resource::Tenant(principal.tenant_id.clone()),
        cedar_ctx_audit_log(&principal, &params),
    )?;

    // Determine tier for SLO routing.
    let tier = if params.from > Utc::now() - Duration::days(90) {
        "hot"
    } else {
        "cold"
    };

    // Decide projection columns based on PII grants.
    let columns = projection_for_principal(&principal, &params);

    // Execute query.
    let page = app.use_cases.audit_log_search
        .execute(&principal, &params, &columns, tier)
        .await?;

    // Emit recursive meta-event (rate-limited).
    app.meta_audit_emitter.tick(&principal).await;

    Ok(Json(page))
}
```

### T2 — Usecase

File: `crates/oya-analytics-usecase/src/audit_log_search.rs`

```rust
pub struct AuditLogSearchUseCase<C: OlapClient> {
    pub olap: C,
}

impl<C: OlapClient> AuditLogSearchUseCase<C> {
    pub async fn execute(
        &self,
        principal: &Principal,
        params: &AuditLogSearchParams,
        columns: &[Column],
        tier: &str,
    ) -> Result<AuditLogPage, UseCaseError> {
        let query = build_sql(&principal.tenant_id, params, columns, tier);
        let rows = self.olap.query::<AuditLogEntry>(&query).await?;
        Ok(rows_into_page(rows, params.page_size, &params.cursor))
    }
}

fn build_sql(tid: &str, params: &AuditLogSearchParams, columns: &[Column], _tier: &str) -> ParameterizedQuery {
    let mut q = format!(
        "SELECT {} FROM tenant_{tid}.audit_events WHERE emitted_at BETWEEN {{from:DateTime64}} AND {{to:DateTime64}}",
        columns.iter().map(|c| c.name()).collect::<Vec<_>>().join(", ")
    );
    if params.axis != Axis::All {
        q.push_str(" AND axis = {axis:String}");
    }
    if params.event_type.is_some() {
        q.push_str(" AND event_type = {event_type:String}");
    }
    if params.principal_id.is_some() {
        q.push_str(" AND principal_id = {principal_id:String}");
    }
    if let Some(cursor) = &params.cursor {
        q.push_str(" AND (emitted_at, event_id) > {cursor_after:Tuple(DateTime64, UUID)}");
    }
    q.push_str(" ORDER BY emitted_at, event_id");
    q.push_str(" LIMIT {page_size:UInt32}");
    ParameterizedQuery::new(q, params)
}
```

### T3 — Recursive audit observation defense

File: `crates/oya-analytics-api/src/meta_audit.rs`

```rust
pub struct MetaAuditEmitter {
    last_emit: DashMap<String, Instant>,  // principal_id → last meta emission
    audit_chain: AuditChainPublisher,
}

impl MetaAuditEmitter {
    pub async fn tick(&self, principal: &Principal) {
        let now = Instant::now();
        let should_emit = match self.last_emit.get(&principal.id) {
            Some(last) => now.duration_since(*last) > Duration::from_secs(60),
            None => true,
        };
        if should_emit {
            self.last_emit.insert(principal.id.clone(), now);
            let _ = self.audit_chain.emit(
                "oya.analytics.audit_log.queried.v1",
                json!({ "principal": principal.id, "tenant_id": principal.tenant_id, "minute": now }),
            ).await;
        }
    }
}
```

Per minute per principal, not per query. This avoids amplification.

### T4 — PII column projection

File: `crates/oya-analytics-api/src/pii_projection.rs`

```rust
pub fn projection_for_principal(principal: &Principal, params: &AuditLogSearchParams) -> Vec<Column> {
    let mut cols = vec![
        Column::EventId, Column::EmittedAt, Column::TenantId, Column::Axis,
        Column::EventType, Column::EvidenceRef, Column::PayloadHash,
    ];
    if principal.dubo_grants.contains(&DataClass::PII) {
        cols.push(Column::PrincipalId);
    } else {
        cols.push(Column::PrincipalIdHashed);  // SHA-256 of principal_id; non-PII
    }
    cols
}
```

Cedar policy `audit-log-pii.cedar` corroborates at the gateway layer.

### T5 — Cursor signing

Per ADR-0150, cursors are HMAC-SHA256-signed. The cursor encodes `(last_emitted_at, last_event_id)` and a signature.

```rust
pub fn encode_cursor(after: &(DateTime<Utc>, Uuid), signing_key: &[u8]) -> String {
    let body = format!("{}|{}", after.0.timestamp_nanos(), after.1);
    let sig = hmac_sha256(signing_key, body.as_bytes());
    base64::encode(format!("{}|{}", body, hex::encode(sig)))
}

pub fn decode_cursor(cursor: &str, signing_key: &[u8]) -> Result<(DateTime<Utc>, Uuid), CursorError> {
    let decoded = base64::decode(cursor)?;
    let s = std::str::from_utf8(&decoded)?;
    let parts: Vec<&str> = s.rsplitn(2, '|').collect();
    let (body, sig) = (parts[1], parts[0]);
    let expected = hmac_sha256(signing_key, body.as_bytes());
    if !constant_time_eq::eq(&hex::decode(sig)?, &expected) {
        return Err(CursorError::Tampered);
    }
    let (ts, id) = body.split_once('|').ok_or(CursorError::Format)?;
    Ok((DateTime::from_timestamp_nanos(ts.parse()?), Uuid::parse_str(id)?))
}
```

### T6 — Tier routing

The OpenSLO source declares two SLOs:

- `audit-log-query-latency.openslo.yaml` (hot): p99 < 800ms.
- `audit-log-query-cold-latency.openslo.yaml` (cold): p95 < 2s.

The handler adds a `tier` label to the Prometheus histogram so the SLO sources can filter correctly:

```rust
let timer = histogram.with_label_values(&["analytics", "/v1/audit-log/search", tier]).start_timer();
// ... execute
timer.observe_duration();
```

### T7 — Integration test

File: `crates/oya-analytics-api/tests/audit_log_search.rs`

```rust
#[tokio::test]
async fn test_audit_log_search_basic() {
    let app = setup_test_app().await;
    seed_audit_events(&app, "tenant_test", 1000).await;

    let principal = Principal::tenant("tenant_test");
    let params = AuditLogSearchParams::between("2026-04-01", "2026-05-01");
    let page = post_search(&app, &principal, params).await;
    assert!(!page.data.is_empty());
    assert!(page.data.iter().all(|e| e.tenant_id == "tenant_test"));
}

#[tokio::test]
async fn test_audit_log_cross_tenant_denied() {
    let app = setup_test_app().await;
    let principal = Principal::tenant("ten_acme");
    let mut params = AuditLogSearchParams::default();
    params.principal_id = Some("user_from_ten_bryan".into());
    // Even if filtered by principal_id, the query is scoped to ten_acme's DB by design;
    // cross-tenant attempts via URL-tampering would be caught by Cedar.
    let page = post_search(&app, &principal, params).await;
    assert!(page.data.iter().all(|e| e.tenant_id == "ten_acme"));
}

#[tokio::test]
async fn test_audit_log_pii_filtered_without_grant() {
    let app = setup_test_app().await;
    let principal = Principal::tenant("tenant_test");  // no PII grant
    let page = post_search(&app, &principal, default_params()).await;
    assert!(page.data.iter().all(|e| e.principal_id.starts_with("hash:")));
}

#[tokio::test]
async fn test_audit_log_recursive_meta_event_rate_limited() {
    let app = setup_test_app().await;
    let principal = Principal::tenant("tenant_test");
    for _ in 0..100 {
        post_search(&app, &principal, default_params()).await;
    }
    let meta_count = count_meta_events(&app, &principal.id).await;
    assert!(meta_count <= 5);  // <= 5 meta events for 100 queries in a short window
}

#[tokio::test]
async fn test_cursor_tamper_rejected() {
    let app = setup_test_app().await;
    let principal = Principal::tenant("tenant_test");
    let mut params = default_params();
    params.cursor = Some("tampered-cursor".into());
    let res = post_search_raw(&app, &principal, params).await;
    assert_eq!(res.status(), 400);
}
```

## Out of scope

- Full-text search over payload (different µservice — Meilisearch per ADR-0184 Tier 4).
- Cross-tenant search by InternalAdmin (separate path; deferred to phase 2 with explicit justification).

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| Cold-tier S3 slow → query >2s | SLO burn | runbook cold-tier-latency.md; customer notice |
| Cursor signing key rotated | new requests with old cursor fail | TTL on cursor (1h); SDK retries with no cursor |
| Cross-tenant attempt | Cedar forbid + audit | 403; account-team review |
| Recursive meta-event storm | rate-limit kicks in | 1 emission per principal per minute max |

## SLO commitment (downstream IP-014)

- Hot window p99 ≤ 800ms (per `slos/audit-log-query-latency.openslo.yaml`).
- Cold window p95 ≤ 2s (per `slos/audit-log-query-cold-latency.openslo.yaml`).

## Rollback

- Endpoint is purely additive; rollback = disable feature flag.
- Per-row data not mutated.

## Evidence emission

- Per query: one meta-audit event per principal per minute (`oya.analytics.audit_log.queried.v1`).
- Per Cedar denial: `oya.analytics.cedar.forbid.v1`.
- Per cold-tier hit: span attribute `oyatie.tier=cold` on the OTel span.

## References

- ADR-0003 audit chain.
- ADR-0193 §"Audit-log surface".
- ADR-0150 cursor pagination.
- ADR-0156 PII registry.
- ADR-0007 Cedar.
- `microservices/analytics/policy/audit-log-pii.cedar`.
- `microservices/analytics/contracts/openapi-v1.yaml` `/v1/audit-log/search`.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/analytics/specs/IP-008-audit-log-query-api.md:315` - - `microservices/analytics/contracts/openapi-v1.yaml` `/v1/audit-log/search`..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/analytics/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/analytics/specs/IP-008-audit-log-query-api.md:11` - Tenant-scoped audit-log search. ClickHouse-backed; sub-second p99 for the typical 30-day window. Filterable by axis, event type, principal, and date range. The audit-l...; `microservices/analytics/specs/IP-008-audit-log-query-api.md:23` - 7. Tier-aware SLO routing: hot-tier queries (≤90d) target p99 < 800ms; cold-tier (>90d) target p95 < 2s..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/analytics/specs/IP-008-audit-log-query-api.md:133` - last_emit: DashMap<String, Instant>,  // principal_id → last meta emission; `microservices/analytics/specs/IP-008-audit-log-query-api.md:289` - | Recursive meta-event storm | rate-limit kicks in | 1 emission per principal per minute max |.
