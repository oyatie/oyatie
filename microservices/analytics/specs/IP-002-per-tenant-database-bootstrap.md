# IP-002 — Per-Tenant Database Bootstrap

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** backend (council-analytics + council-tenancy)
**Authority ADRs:** ADR-0193, ADR-0155 quotas, ADR-0145 inter-µservice communication, ADR-AN-004-query-budget-tier
**Depends on:** IP-001
**Status:** Planned

## Scope

Stand up the controller that listens for `tenant.onboarded`, `tenant.offboarded`, and `tenant.tier_changed` events from the tenancy µservice (per ADR-0145 Invariant 3 ontology projections) and reconciles ClickHouse state:

- **On `tenant.onboarded`** — create `tenant_${tenant_id}` database; create per-tenant ClickHouse users `tenant_${tenant_id}_reader` + `tenant_${tenant_id}_writer`; apply per-tier QUOTA per ADR-AN-004; render per-tenant MV instances from the templates in `iac/clickhouse/mv-templates/`.
- **On `tenant.tier_changed`** — re-apply QUOTA per the new tier (idempotent).
- **On `tenant.offboarded`** — drop the `tenant_${tenant_id}` database (cascade); emit `oya.analytics.tenant_database.proof_of_erasure.v1` per ADR-0038.

The controller is `oya-analytics-tenant-bootstrap-app` per the BNF v4.1 catalog. Distinct from `oya-analytics-app` because its lifecycle is independent (controller-pattern per Kubernetes operator doctrine).

## Deliverables

1. Controller crate `crates/oya-analytics-tenant-bootstrap-app/` (Rust binary).
2. Cedar policy fragment `microservices/analytics/policy/tenant-bootstrap.cedar` (already authored).
3. Idempotency contract — re-emitting `tenant.onboarded` for the same tenant is a no-op.
4. Audit-chain event per onboard / offboard / tier-change (per ADR-0003).
5. Integration test against an ephemeral ClickHouse instance (Testcontainers).
6. State cursor persisted in Postgres so the controller resumes after restart.
7. Leader-election with 2 replicas (1 active) for HA — deferred to phase 2; for phase 1, single replica with k8s restart.

## Acceptance criteria

- Subscribing to a `tenant.onboarded` event for `ten_acme` creates database `tenant_ten_acme` within 30s p99.
- Re-subscribing the same event is a no-op (idempotent).
- Subscribing to `tenant.offboarded` for `ten_acme` drops database `tenant_ten_acme` within 60s p99 and emits a `tenant.proof_of_erasure` event.
- A query attempted by `tenant_ten_acme_reader` against `tenant_ten_bryan.events` is denied by ClickHouse RBAC.
- The per-tenant QUOTA limits match ADR-AN-004's matrix for the tenant's tier.
- Controller restarts cleanly without re-reconciling already-up-to-date tenants (uses a state cursor).
- Tier change from Starter → Growth re-applies QUOTA within 30s.
- All per-tenant MV templates from `iac/clickhouse/mv-templates/` render at onboard.

## Implementation tasks

### T1 — Controller skeleton

File: `crates/oya-analytics-tenant-bootstrap-app/src/main.rs`

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = AppConfig::load()?;
    let _otel = init_tracing(&cfg.otel)?;

    let olap = ClickHouseOlapClient::connect(&cfg.clickhouse).await?;
    let pulsar = PulsarConsumer::connect(&cfg.pulsar).await?;
    let cursor = StateCursor::open_postgres(&cfg.postgres).await?;
    let audit_chain = AuditChainPublisher::connect(&cfg.audit_chain).await?;

    let reconciler = TenantBootstrapReconciler::new(olap, audit_chain);

    let mut sub = pulsar.subscribe(
        "persistent://public/default/oya.tenancy.tenant.events",
        SubscriptionType::Shared,
        "analytics-tenant-bootstrap",
    ).await?;

    while let Some(msg) = sub.next().await {
        let event: TenancyEvent = serde_json::from_slice(&msg.payload)?;
        // Skip already-processed events (state cursor).
        if cursor.is_processed(&event.id).await? { sub.ack(&msg).await?; continue; }
        reconciler.handle(&event).await?;
        cursor.mark_processed(&event.id).await?;
        sub.ack(&msg).await?;
    }
    Ok(())
}
```

### T2 — Reconciler

File: `crates/oya-analytics-tenant-bootstrap-app/src/reconcile.rs`

```rust
impl TenantBootstrapReconciler {
    pub async fn handle(&self, event: &TenancyEvent) -> Result<()> {
        match event.kind.as_str() {
            "oya.tenancy.tenant.onboarded.v1" => self.handle_onboard(event).await,
            "oya.tenancy.tenant.offboarded.v1" => self.handle_offboard(event).await,
            "oya.tenancy.tenant.tier_changed.v1" => self.handle_tier_changed(event).await,
            other => {
                tracing::warn!(?other, "ignored event kind");
                Ok(())
            }
        }
    }

    async fn handle_onboard(&self, event: &TenancyEvent) -> Result<()> {
        let tid = TenantId::new(&event.data["tenant_id"].as_str().unwrap());
        let tier = parse_tier(&event.data["tier"]);
        let residency = parse_residency_class(&event.data["residency_class"]);

        // Step 1: Create database (idempotent).
        self.olap.ensure_tenant_database(&tid).await?;

        // Step 2: Per-tenant users + grants.
        self.olap.exec_ddl_raw(&format!(
            "CREATE USER IF NOT EXISTS tenant_{tid}_reader ON CLUSTER analytics-clickhouse-1 \
             IDENTIFIED WITH ldap_server BY 'oyatie-ldap'", tid = tid.as_str())).await?;
        self.olap.exec_ddl_raw(&format!(
            "GRANT SELECT ON tenant_{tid}.* TO tenant_{tid}_reader", tid = tid.as_str())).await?;
        self.olap.exec_ddl_raw(&format!(
            "CREATE USER IF NOT EXISTS tenant_{tid}_writer ON CLUSTER analytics-clickhouse-1 \
             IDENTIFIED WITH ldap_server BY 'oyatie-ldap'", tid = tid.as_str())).await?;
        self.olap.exec_ddl_raw(&format!(
            "GRANT INSERT ON tenant_{tid}.* TO tenant_{tid}_writer", tid = tid.as_str())).await?;

        // Step 3: Apply per-tier QUOTA (per ADR-AN-004 matrix).
        let quota_sql = render_quota_template(&tier, &tid);
        self.olap.exec_ddl_raw(&quota_sql).await?;

        // Step 4: Render per-tenant MV templates.
        for template_path in [
            "iac/clickhouse/mv-templates/audit-log-table.sql",
            "iac/clickhouse/mv-templates/mv-hour-workflow-per-tenant.sql",
            "iac/clickhouse/mv-templates/mv-minute-error-burst-per-tenant.sql",
            "iac/clickhouse/mv-templates/mv-day-billing-per-resource.sql",
            "iac/clickhouse/mv-templates/mv-month-billing-per-resource.sql",
        ] {
            let template = read_template(template_path)?;
            let rendered = template.replace("${tid}", tid.as_str());
            self.olap.exec_ddl_raw(&rendered).await?;
        }

        // Step 5: Emit audit-chain.
        self.audit_chain.emit("oya.analytics.tenant_database.created.v1", json!({
            "tenant_id": tid.as_str(),
            "tier": format!("{tier:?}"),
            "residency_class": format!("{residency:?}"),
            "ts": Utc::now(),
        })).await?;
        Ok(())
    }

    async fn handle_offboard(&self, event: &TenancyEvent) -> Result<()> {
        let tid = TenantId::new(event.data["tenant_id"].as_str().unwrap());
        self.olap.exec_ddl_raw(&format!(
            "DROP DATABASE IF EXISTS tenant_{tid} ON CLUSTER analytics-clickhouse-1 SYNC",
            tid = tid.as_str()
        )).await?;
        self.olap.exec_ddl_raw(&format!(
            "DROP USER IF EXISTS tenant_{tid}_reader, tenant_{tid}_writer ON CLUSTER analytics-clickhouse-1",
            tid = tid.as_str()
        )).await?;
        self.olap.exec_ddl_raw(&format!(
            "DROP QUOTA IF EXISTS quota_tenant_{tid} ON CLUSTER analytics-clickhouse-1",
            tid = tid.as_str()
        )).await?;

        // Proof-of-erasure per ADR-0038.
        let erasure_event = json!({
            "tenant_id": tid.as_str(),
            "erasure_completed_at": Utc::now(),
            "operator": "oya-analytics-tenant-bootstrap-app",
        });
        let signed = self.cosign.sign_event(&erasure_event).await?;
        self.audit_chain.emit("oya.analytics.tenant_database.proof_of_erasure.v1", &signed).await?;
        Ok(())
    }

    async fn handle_tier_changed(&self, event: &TenancyEvent) -> Result<()> {
        let tid = TenantId::new(event.data["tenant_id"].as_str().unwrap());
        let new_tier = parse_tier(&event.data["new_tier"]);
        let quota_sql = render_quota_template(&new_tier, &tid);
        self.olap.exec_ddl_raw(&quota_sql).await?;
        self.audit_chain.emit("oya.analytics.tenant.quota_applied.v1", json!({
            "tenant_id": tid.as_str(),
            "old_tier": event.data["old_tier"], "new_tier": event.data["new_tier"],
        })).await?;
        Ok(())
    }
}
```

### T3 — Default quota profile (per tier — ADR-AN-004)

File: `crates/oya-analytics-tenant-bootstrap-app/src/quota.rs`

```rust
pub fn render_quota_template(tier: &Tier, tid: &TenantId) -> String {
    let (max_queries, max_read_rows, max_insert_rows, max_concurrent, max_exec) = match tier {
        Tier::Trial => (100, 10_000_000, 1_000_000, 4, 30),
        Tier::Starter => (1_000, 1_000_000_000, 100_000_000, 16, 60),
        Tier::Growth => (10_000, 10_000_000_000, 1_000_000_000, 32, 120),
        Tier::Enterprise => (100_000, 1_000_000_000_000, 100_000_000_000, 64, 300),
    };
    format!(
        "CREATE QUOTA IF NOT EXISTS quota_tenant_{tid} \
         ON CLUSTER analytics-clickhouse-1 \
         KEYED BY user_name \
         FOR INTERVAL 1 HOUR \
           MAX queries = {max_queries}, \
               read_rows = {max_read_rows}, \
               written_rows = {max_insert_rows} \
         TO tenant_{tid}_reader, tenant_{tid}_writer; \
         \
         ALTER USER tenant_{tid}_reader ON CLUSTER analytics-clickhouse-1 \
         SETTINGS max_concurrent_queries_for_user = {max_concurrent}, max_execution_time = {max_exec}; \
         \
         ALTER USER tenant_{tid}_writer ON CLUSTER analytics-clickhouse-1 \
         SETTINGS max_concurrent_queries_for_user = {max_concurrent}, max_execution_time = {max_exec};",
        tid = tid.as_str(),
    )
}
```

### T4 — Cedar policy

The Cedar policy is at `microservices/analytics/policy/tenant-bootstrap.cedar` (already authored). It permits the tenancy µservice to publish onboard/offboard events and permits the analytics bootstrap controller to publish proof-of-erasure events.

### T5 — Audit-chain emission

Per ADR-0003: every reconcile action emits an audit event. The events are:

- `oya.analytics.tenant_database.created.v1` — on successful onboard.
- `oya.analytics.tenant_database.dropped.v1` — on successful offboard (before proof-of-erasure).
- `oya.analytics.tenant_database.proof_of_erasure.v1` — cosign-signed, emitted after the DROP completes.
- `oya.analytics.tenant.quota_applied.v1` — on tier change.
- `oya.analytics.tenant_bootstrap.error.v1` — on reconcile failure (for observability).

### T6 — State cursor

File: `crates/oya-analytics-tenant-bootstrap-app/src/cursor.rs`

The cursor is a Postgres table (in the analytics namespace's tenancy-shadow Postgres):

```sql
CREATE TABLE IF NOT EXISTS analytics_tenant_bootstrap_cursor (
    event_id UUID PRIMARY KEY,
    processed_at TIMESTAMP NOT NULL DEFAULT now()
);
```

Restart sequence: controller queries `SELECT event_id FROM analytics_tenant_bootstrap_cursor WHERE processed_at > now() - INTERVAL '7 days'`, loads into a `HashSet`, then resumes from Pulsar at the configured initial position (per ADR-0145 — typically `earliest` to ensure no event lost).

### T7 — Integration test

File: `crates/oya-analytics-tenant-bootstrap-app/tests/integration.rs`

```rust
#[tokio::test]
async fn test_onboard_creates_database() {
    let app = setup_test_controller().await;
    publish_tenancy_event(&app, json!({
        "type": "oya.tenancy.tenant.onboarded.v1",
        "id": uuid::Uuid::new_v4(),
        "data": { "tenant_id": "ten_test_onboard", "tier": "Starter", "residency_class": "Global" }
    })).await;

    wait_until_database_exists(&app, "tenant_ten_test_onboard", Duration::from_secs(30)).await;
    assert_user_exists(&app, "tenant_ten_test_onboard_reader").await;
    assert_user_exists(&app, "tenant_ten_test_onboard_writer").await;
    assert_quota_exists(&app, "quota_tenant_ten_test_onboard").await;
    // Verify MV templates rendered.
    assert_table_exists(&app, "tenant_ten_test_onboard.audit_events").await;
    assert_table_exists(&app, "tenant_ten_test_onboard.workflow_hour").await;
}

#[tokio::test]
async fn test_re_onboard_is_noop() {
    let app = setup_test_controller().await;
    let event_id = uuid::Uuid::new_v4();
    publish_tenancy_event_with_id(&app, event_id, json!({"type": "oya.tenancy.tenant.onboarded.v1", "data": {"tenant_id": "ten_re_onboard", "tier": "Starter", "residency_class": "Global"}})).await;
    publish_tenancy_event_with_id(&app, event_id, json!({"type": "oya.tenancy.tenant.onboarded.v1", "data": {"tenant_id": "ten_re_onboard", "tier": "Starter", "residency_class": "Global"}})).await;
    // Second is a no-op via state cursor.
    let count = audit_chain_event_count(&app, "oya.analytics.tenant_database.created.v1", "ten_re_onboard").await;
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_offboard_drops_database_and_emits_proof_of_erasure() {
    let app = setup_test_controller().await;
    publish_onboard(&app, "ten_offboard", "Starter").await;
    wait_until_database_exists(&app, "tenant_ten_offboard", Duration::from_secs(30)).await;

    publish_offboard(&app, "ten_offboard").await;
    wait_until_database_dropped(&app, "tenant_ten_offboard", Duration::from_secs(60)).await;

    let proofs = audit_chain_events(&app, "oya.analytics.tenant_database.proof_of_erasure.v1", "ten_offboard").await;
    assert_eq!(proofs.len(), 1);
    // Verify cosign signature on the proof event.
    assert!(cosign::verify_blob(&app.cosign_pubkey, &proofs[0].canonical_json(), &proofs[0].cosign_signature).is_ok());
}

#[tokio::test]
async fn test_tier_change_reapplies_quota() {
    let app = setup_test_controller().await;
    publish_onboard(&app, "ten_tier", "Trial").await;
    publish_tier_changed(&app, "ten_tier", "Trial", "Growth").await;
    tokio::time::sleep(Duration::from_secs(30)).await;
    let quota = query_quota(&app, "quota_tenant_ten_tier").await;
    assert_eq!(quota.max_queries_per_hour, 10_000);  // Growth tier per ADR-AN-004
}

#[tokio::test]
async fn test_cross_tenant_rbac_enforced() {
    let app = setup_test_controller().await;
    publish_onboard(&app, "ten_a", "Starter").await;
    publish_onboard(&app, "ten_b", "Starter").await;
    wait_until_database_exists(&app, "tenant_ten_a", Duration::from_secs(30)).await;
    wait_until_database_exists(&app, "tenant_ten_b", Duration::from_secs(30)).await;

    let err = clickhouse_query_as(&app, "tenant_ten_a_reader", "SELECT count() FROM tenant_ten_b.events_test").await;
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("ACCESS_DENIED") || err.unwrap_err().to_string().contains("not enough privileges"));
}
```

## Out of scope

- Per-tenant ingestion pipeline (IP-004).
- Per-tenant table schema (per-µservice owners declare their tenant schemas via IP-007 / IP-008 / IP-009).
- Cross-cell tenant residency routing (IP-010).

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| ClickHouse Keeper quorum loss → DDL fails | Controller retries with exponential backoff; pages after 5min | IP-001 PrometheusRule already alerts |
| Tenant onboarded twice with different tier | Controller compares observed vs desired tier; reapplies quota | Idempotent |
| Tenant offboarded then re-onboarded same id | Controller treats as fresh onboard; data is gone by design | Documented in runbook |
| Controller crash mid-onboard | State cursor lag detected on restart; controller re-reconciles | Cursor persisted to Postgres |
| Cosign key unavailable on offboard | proof-of-erasure emission fails | retry on next event or operator-triggered re-emission |

## SLO commitment (downstream IP-014)

- `tenant.onboarded` → DB created within 30s p99 (per `slos/tenant-bootstrap-latency.openslo.yaml`).
- Offboard → DB dropped + proof-of-erasure within 60s p99.
- Controller uptime: 99.9% per cell.

## Rollback

- Per ADR-0159, feature-flag-gated.
- Disabling controller: in-flight tenancy events accumulate in Pulsar with configurable retention; re-enable resumes reconciliation.

## Evidence emission

- Per onboard: `oya.analytics.tenant_database.created.v1`.
- Per offboard: `oya.analytics.tenant_database.dropped.v1` + `oya.analytics.tenant_database.proof_of_erasure.v1`.
- Per tier change: `oya.analytics.tenant.quota_applied.v1`.
- Per reconcile error: `oya.analytics.tenant_bootstrap.error.v1`.

## References

- ADR-0193 §"Multi-tenancy isolation".
- ADR-0155 — per-tenant resource quotas.
- ADR-0145 — inter-microservice communication reform.
- ADR-0003 — audit chain and evidence emission.
- ADR-0038 — trust framework and DSR cascade and proof of erasure.
- ADR-AN-004-query-budget-tier (quota matrix).
- `microservices/analytics/policy/tenant-bootstrap.cedar`.
- `microservices/analytics/iac/clickhouse/mv-templates/`.
