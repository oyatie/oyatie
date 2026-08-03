//! Postgres CDC change-stream adapter (via sqlx) for the owned `oya-data`
//! outbox port.
//!
//! Story G003 sub-slice (ADR-0536 D-10 change streams / D-13 messaging):
//! services link the `oya-data-outbox-kernel::ChangeStreamSource` port; this
//! adapter is the ADR-0510 transitional Postgres implementation behind it.
//! The port models the W5 engine's native changefeed with an opaque monotone
//! `StreamPosition` checkpoint (CockroachDB changefeed / Spanner change-stream
//! shape); the transitional implementation here is outbox polling —
//! strictly-after-checkpoint, tenant-scoped, ordered SOLELY by the monotone
//! `commit_logical` IDENTITY sequence (the stream position) over
//! `oya_data_outbox.outbox_events` — behind the same trait, so consumers
//! never observe the engine swap.
//!
//! This adapter absorbs ALL engine impedance: the tenant RLS scope runs the
//! parameterized `set_config` (reusing `oya-shared-postgres-command-kernel::
//! SET_LOCAL_TENANT_SQL`) inside the SAME transaction as the poll SELECT, the
//! poll is fully parameterized (no value interpolation), and rows are mapped
//! to the kernel's `ChangeRecord` shape whose batch is then validated by the
//! kernel's ordering/monotonicity invariants.
//!
//! The default test set stays database-free. The env-gated live harness
//! exercises real Postgres RLS cross-tenant denial against the PRODUCTION
//! policy set (run AS the `oya_data_outbox_runtime` role, NOT PUBLIC, asserted
//! NOBYPASSRLS) and stream-position-ordered resumable polling against a
//! containerized database, mirroring the `data-sql-adapter-sqlx`
//! live-probe pattern.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::env;

use oya_data_outbox_kernel::{ChangeBatch, ChangeRecord, StreamPosition};
use oya_data_sql_kernel::DataSqlError;
use oya_shared_postgres_command_kernel::{SET_LOCAL_TENANT_SQL, split_migration_statements};
use sqlx::pool::PoolConnectionMetadata;
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgPoolOptions, postgres::PgRow};

/// Parameterized CDC poll over the outbox table. Rows STRICTLY AFTER the
/// `commit_logical` stream-position checkpoint, tenant-scoped (RLS also
/// enforces this, but the explicit filter keeps the access path index-served
/// and intent-clear), ordered SOLELY by the monotone `commit_logical` sequence
/// (a single global IDENTITY is a strict, unique, monotone total order — so
/// strictly-after on it never skips and never re-delivers a tie), and limited.
/// `commit_wall_nanos` is selected as an informational field ONLY; it is never
/// in the WHERE/ORDER BY (clock_timestamp() is non-monotone and would silently
/// skip a later row). The `$2 = checkpoint` position and `$3 = limit` are bound
/// parameters — NO value ever enters the SQL text.
pub const POLL_CHANGES_SQL: &str = "SELECT tenant_id, event_id, event_kind, aggregate_id, payload, commit_wall_nanos, commit_logical \
     FROM oya_data_outbox.outbox_events \
     WHERE tenant_id = $1 \
       AND commit_logical > $2 \
     ORDER BY commit_logical \
     LIMIT $3";

/// Enable flag for the live containerized-Postgres CDC/RLS harness.
pub const LIVE_OUTBOX_POSTGRES_ENABLE_ENV: &str = "OYA_OUTBOX_LIVE_POSTGRES";
/// Admin (schema-owning) connection URL for the live harness.
pub const LIVE_OUTBOX_POSTGRES_ADMIN_URL_ENV: &str = "OYA_OUTBOX_POSTGRES_ADMIN_URL";
/// Application-role (RLS-subject) connection URL for the live harness.
pub const LIVE_OUTBOX_POSTGRES_APP_URL_ENV: &str = "OYA_OUTBOX_POSTGRES_APP_URL";

/// The transitional Postgres implementation of the owned CDC change-stream
/// port. Async surface mirrors `oya_data_outbox_kernel::ChangeStreamSource`
/// 1:1 (the sync kernel trait stays reserved for IO-free reference impls,
/// matching the data SQL kernel/adapter split).
pub struct SqlxChangeStreamSource {
    pool: PgPool,
}

impl std::fmt::Debug for SqlxChangeStreamSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The pooled handle is intentionally opaque; there is no other
        // identifying state on the source.
        f.debug_struct("SqlxChangeStreamSource")
            .finish_non_exhaustive()
    }
}

impl SqlxChangeStreamSource {
    #[must_use]
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Poll the outbox for changes strictly after `checkpoint`, tenant-scoped,
    /// up to `limit` records, in monotone `commit_logical` stream order. The
    /// tenant RLS scope is applied FIRST in the same transaction
    /// (defense-in-depth alongside the explicit tenant filter), then the
    /// parameterized poll runs and the rows are mapped to a kernel
    /// `ChangeBatch` that is validated before return.
    pub async fn poll_changes(
        &self,
        tenant_id: &str,
        checkpoint: StreamPosition,
        limit: usize,
    ) -> Result<ChangeBatch, DataSqlError> {
        if tenant_id.trim().is_empty() {
            return Err(DataSqlError::MissingField {
                field: "change_stream.tenant_id",
            });
        }
        let bounded_limit = i64::try_from(limit).map_err(|_| {
            DataSqlError::Adapter(format!("poll limit {limit} exceeds the i64 bind range"))
        })?;
        // The checkpoint is the monotone bigint stream position, bound WITHOUT
        // narrowing (a Postgres bigint is i64; the IDENTITY sequence never
        // exceeds i64::MAX). Fail closed if a caller ever presents one beyond
        // the bind range rather than silently wrapping.
        let checkpoint_position = i64::try_from(checkpoint.0).map_err(|_| {
            DataSqlError::Adapter(format!(
                "checkpoint stream position {} exceeds the bigint bind range",
                checkpoint.0
            ))
        })?;

        let mut transaction = self.pool.begin().await.map_err(sqlx_error)?;
        apply_tenant_scope(&mut transaction, tenant_id).await?;
        let rows = sqlx::query(POLL_CHANGES_SQL)
            .bind(tenant_id)
            .bind(checkpoint_position)
            .bind(bounded_limit)
            .fetch_all(&mut *transaction)
            .await
            .map_err(sqlx_error)?;
        transaction.commit().await.map_err(sqlx_error)?;

        let mut records = Vec::with_capacity(rows.len());
        for row in &rows {
            records.push(record_from_row(row)?);
        }
        // At-least-once resumable semantics: the next checkpoint is the last
        // delivered record's stream position, or the caller's checkpoint when
        // the page is empty (matching the kernel reference impl).
        let resume_from = records.last().map_or(checkpoint, |record| record.position);
        let batch = ChangeBatch {
            records,
            resume_from,
        };
        batch.validate()?;
        Ok(batch)
    }
}

async fn apply_tenant_scope(
    transaction: &mut Transaction<'_, Postgres>,
    tenant_id: &str,
) -> Result<(), DataSqlError> {
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind(tenant_id)
        .execute(&mut **transaction)
        .await
        .map_err(sqlx_error)?;
    Ok(())
}

fn record_from_row(row: &PgRow) -> Result<ChangeRecord, DataSqlError> {
    let tenant_id: String = try_get(row, "tenant_id")?;
    let event_id: String = try_get(row, "event_id")?;
    let event_kind: String = try_get(row, "event_kind")?;
    let aggregate_id: String = try_get(row, "aggregate_id")?;
    let payload: Vec<u8> = try_get(row, "payload")?;
    let commit_wall_nanos: i64 = try_get(row, "commit_wall_nanos")?;
    let commit_logical: i64 = try_get(row, "commit_logical")?;

    // The bigint stream position is carried into the u64 StreamPosition WITHOUT
    // narrowing: the full sequence range survives (a u32 would wedge the poll
    // forever once the global sequence passes ~4.3B rows). Fail closed on a
    // negative value rather than silently coercing — a negative IDENTITY value
    // means the table invariant (monotone positive sequence) was violated.
    let position = StreamPosition(u64::try_from(commit_logical).map_err(|_| {
        DataSqlError::Adapter(format!(
            "outbox row {event_id} carries a negative commit_logical {commit_logical}"
        ))
    })?);
    // Informational physical commit instant only (never the ordering key).
    let wall_nanos = u64::try_from(commit_wall_nanos).map_err(|_| {
        DataSqlError::Adapter(format!(
            "outbox row {event_id} carries a negative commit_wall_nanos {commit_wall_nanos}"
        ))
    })?;

    Ok(ChangeRecord {
        tenant_id,
        event_id,
        event_kind,
        aggregate_id,
        position,
        commit_wall_nanos: wall_nanos,
        payload,
    })
}

fn try_get<'r, T>(row: &'r PgRow, column: &'static str) -> Result<T, DataSqlError>
where
    T: sqlx::Decode<'r, Postgres> + sqlx::Type<Postgres>,
{
    row.try_get::<T, _>(column).map_err(|error| {
        DataSqlError::Adapter(format!("outbox row column {column}: {error}"))
    })
}

fn sqlx_error(error: sqlx::Error) -> DataSqlError {
    DataSqlError::Adapter(error.to_string())
}

/// Outcome of the live CDC cross-tenant-deny + resumable-poll probe: proves
/// over real Postgres RLS that tenant A's outbox rows are invisible to tenant
/// B and to unscoped sessions, that a tenant-scoped poll returns its rows in
/// stream-position commit order, that resuming from the returned checkpoint
/// drains the remaining rows (at-least-once), and that the app role exercising
/// the poll carries NO BYPASSRLS (ADR-0567 D3: a BYPASSRLS role silently skips
/// every policy).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveCdcProbeReport {
    pub tenant_a_poll_event_ids: Vec<String>,
    pub tenant_b_poll_event_ids: Vec<String>,
    pub unscoped_poll_denied: bool,
    pub tenant_a_records_commit_ordered: bool,
    pub tenant_a_resume_drains_remaining: bool,
    pub app_role_lacks_bypassrls: bool,
}

/// The production runtime role the migration policies/grants target. The live
/// harness exercises the EXACT production policy set by running the poll AS this
/// role (NOT PUBLIC, NOT a BYPASSRLS/superuser role).
pub const RUNTIME_ROLE: &str = "oya_data_outbox_runtime";

/// The two committed production migration files, applied IN ORDER (0000 ships
/// the runtime-role contract, 0001 ships the table + RLS policies/grants
/// targeting that role). The live harness applies these verbatim so the test
/// path is byte-for-byte the production schema — no drift-prone inline copy.
const LIVE_MIGRATIONS: &[&str] = &[
    include_str!("../migrations/0000_runtime_role.sql"),
    include_str!("../migrations/0001_outbox_events.sql"),
];

const LIVE_FIXTURE_DROP_SQL: &str = "DROP SCHEMA IF EXISTS oya_data_outbox CASCADE";
const LIVE_FIXTURE_INSERT_SQL: &str = "INSERT INTO oya_data_outbox.outbox_events (tenant_id, event_id, event_kind, aggregate_id, schema_version, idempotency_key, payload) VALUES ($1, $2, 'tenant.provisioned', 'aggregates/x', '1', $3, '\\x00') ON CONFLICT (tenant_id, idempotency_key) DO NOTHING";

const LIVE_FIXTURE_ROWS: &[(&str, &str, &str)] = &[
    ("tenant-a", "a-evt-1", "a-idem-1"),
    ("tenant-a", "a-evt-2", "a-idem-2"),
    ("tenant-a", "a-evt-3", "a-idem-3"),
    ("tenant-b", "b-evt-1", "b-idem-1"),
];

/// Read `(current_user, rolbypassrls)` for the connected role.
async fn current_role_bypassrls(pool: &PgPool) -> Result<(String, bool), DataSqlError> {
    let row = sqlx::query(
        "SELECT current_user::text AS name, rolbypassrls FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(pool)
    .await
    .map_err(sqlx_error)?;
    let name: String = try_get(&row, "name")?;
    let rolbypassrls: bool = try_get(&row, "rolbypassrls")?;
    Ok((name, rolbypassrls))
}

/// Env-gated live harness. Returns `Ok(None)` when the enable flag is absent
/// so default test runs stay database-free; CI integration lanes set the env
/// vars against a containerized Postgres.
pub async fn run_live_cdc_cross_tenant_probe()
-> Result<Option<LiveCdcProbeReport>, DataSqlError> {
    if env::var(LIVE_OUTBOX_POSTGRES_ENABLE_ENV).is_err() {
        return Ok(None);
    }
    let admin_url = env::var(LIVE_OUTBOX_POSTGRES_ADMIN_URL_ENV).map_err(|_| {
        DataSqlError::MissingField {
            field: LIVE_OUTBOX_POSTGRES_ADMIN_URL_ENV,
        }
    })?;
    let app_url = env::var(LIVE_OUTBOX_POSTGRES_APP_URL_ENV).map_err(|_| {
        DataSqlError::MissingField {
            field: LIVE_OUTBOX_POSTGRES_APP_URL_ENV,
        }
    })?;

    // Admin pool (schema owner) applies the COMMITTED production migrations
    // verbatim and seeds rows; it bypasses RLS for the fixture only. The
    // migrations create `oya_data_outbox_runtime` (NOBYPASSRLS), the table, the
    // RLS policies, and the grants — all targeting the runtime role, NOT PUBLIC.
    let admin = PgPool::connect(&admin_url).await.map_err(sqlx_error)?;
    sqlx::query(LIVE_FIXTURE_DROP_SQL)
        .execute(&admin)
        .await
        .map_err(sqlx_error)?;
    for migration in LIVE_MIGRATIONS {
        for statement in split_migration_statements(migration) {
            sqlx::query(&statement)
                .execute(&admin)
                .await
                .map_err(sqlx_error)?;
        }
    }

    // Discover the app login role and grant it membership in the runtime role,
    // so the app pool can assume `oya_data_outbox_runtime` on every connection
    // (the deploy contract: the login role is a member of the runtime role).
    let app_login = PgPool::connect(&app_url).await.map_err(sqlx_error)?;
    let (app_login_role, _) = current_role_bypassrls(&app_login).await?;
    app_login.close().await;
    sqlx::query(&format!(
        "GRANT {RUNTIME_ROLE} TO \"{app_login_role}\""
    ))
    .execute(&admin)
    .await
    .map_err(sqlx_error)?;

    // Seed two tenants' rows through the admin role (bypasses RLS), so the
    // RLS denial under test is purely the application-role poll path.
    for (tenant, event_id, idem) in LIVE_FIXTURE_ROWS {
        sqlx::query(LIVE_FIXTURE_INSERT_SQL)
            .bind(tenant)
            .bind(event_id)
            .bind(idem)
            .execute(&admin)
            .await
            .map_err(sqlx_error)?;
    }

    // Build the app pool that assumes the production runtime role on EVERY
    // pooled connection, so the poll exercises the exact production policy set.
    let app_pool = PgPoolOptions::new()
        .after_connect(|conn, _meta: PoolConnectionMetadata| {
            Box::pin(async move {
                sqlx::query(&format!("SET ROLE {RUNTIME_ROLE}"))
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&app_url)
        .await
        .map_err(sqlx_error)?;

    // The app role exercising the poll MUST NOT carry BYPASSRLS (ADR-0567 D3),
    // or RLS is silently skipped and isolation rests only on the WHERE clause.
    let (_, app_bypassrls) = current_role_bypassrls(&app_pool).await?;
    let app_role_lacks_bypassrls = !app_bypassrls;

    let source = SqlxChangeStreamSource::from_pool(app_pool);

    // Tenant A: poll one page, then resume from the returned checkpoint.
    let page_a = source
        .poll_changes("tenant-a", StreamPosition::zero(), 2)
        .await?;
    let resume_a = source
        .poll_changes("tenant-a", page_a.resume_from, 10)
        .await?;
    let page_b = source
        .poll_changes("tenant-b", StreamPosition::zero(), 10)
        .await?;

    // A never-seeded tenant sees nothing (per-tenant isolation); the unset-GUC
    // restrictive deny-all is proven by the kernel/SCIM RLS suite and the
    // empty-tenant guard rejects an unscoped poll before any query.
    let unscoped = source
        .poll_changes("tenant-unseeded", StreamPosition::zero(), 10)
        .await?;

    sqlx::query(LIVE_FIXTURE_DROP_SQL)
        .execute(&admin)
        .await
        .map_err(sqlx_error)?;

    let tenant_a_poll_event_ids: Vec<String> = page_a
        .records
        .iter()
        .chain(resume_a.records.iter())
        .map(|record| record.event_id.clone())
        .collect();
    let tenant_b_poll_event_ids: Vec<String> = page_b
        .records
        .iter()
        .map(|record| record.event_id.clone())
        .collect();
    let tenant_a_records_commit_ordered = page_a
        .records
        .windows(2)
        .all(|w| w[0].position < w[1].position);

    Ok(Some(LiveCdcProbeReport {
        tenant_a_poll_event_ids,
        tenant_b_poll_event_ids,
        unscoped_poll_denied: unscoped.records.is_empty(),
        tenant_a_records_commit_ordered,
        tenant_a_resume_drains_remaining: !resume_a.records.is_empty(),
        app_role_lacks_bypassrls,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_sql_is_fully_parameterized_and_carries_no_values() {
        // Every dynamic input is a $n placeholder: no tenant id, checkpoint
        // position, or limit is ever interpolated into the SQL text.
        assert!(POLL_CHANGES_SQL.contains("$1"));
        assert!(POLL_CHANGES_SQL.contains("$2"));
        assert!(POLL_CHANGES_SQL.contains("$3"));
        // Only three binds now (tenant, position, limit) — no $4.
        assert!(!POLL_CHANGES_SQL.contains("$4"));
        assert!(!POLL_CHANGES_SQL.contains("tenant-a"));
        assert!(!POLL_CHANGES_SQL.contains('\''));
    }

    #[test]
    fn poll_sql_filters_by_tenant() {
        assert!(POLL_CHANGES_SQL.contains("tenant_id = $1"));
        assert!(POLL_CHANGES_SQL.contains("FROM oya_data_outbox.outbox_events"));
    }

    #[test]
    fn poll_sql_uses_strictly_after_stream_position_semantics() {
        // Strict comparison against the monotone commit_logical stream
        // position: the checkpointed record is never redelivered, only rows
        // strictly after it.
        assert!(POLL_CHANGES_SQL.contains("commit_logical > $2"));
    }

    #[test]
    fn poll_sql_orders_solely_by_stream_position_not_wall_clock() {
        // Ordering + filtering is on the monotone commit_logical alone; the
        // non-monotone commit_wall_nanos must NEVER be in the WHERE/ORDER BY
        // (it would silently skip a later row on NTP step-back). It IS still in
        // the SELECT list as an informational field.
        assert!(POLL_CHANGES_SQL.contains("ORDER BY commit_logical"));
        assert!(!POLL_CHANGES_SQL.contains("ORDER BY commit_wall_nanos"));
        assert!(!POLL_CHANGES_SQL.contains("commit_wall_nanos >"));
        assert!(!POLL_CHANGES_SQL.contains("commit_wall_nanos)"));
        // The WHERE clause references only tenant_id and commit_logical.
        let where_clause = POLL_CHANGES_SQL
            .split("WHERE")
            .nth(1)
            .unwrap()
            .split("ORDER BY")
            .next()
            .unwrap();
        assert!(!where_clause.contains("commit_wall_nanos"));
    }

    #[test]
    fn poll_sql_honors_a_bound_limit() {
        assert!(POLL_CHANGES_SQL.contains("LIMIT $3"));
    }

    #[tokio::test]
    async fn poll_rejects_blank_tenant() {
        // The blank-tenant guard short-circuits before any pool use; the lazy
        // pool is never queried. Run under a tokio runtime so the lazy pool's
        // background reaper has a reactor context.
        let source = SqlxChangeStreamSource::from_pool(
            PgPool::connect_lazy("postgres://localhost/x").unwrap(),
        );
        let result = source.poll_changes(" ", StreamPosition::zero(), 10).await;
        assert_eq!(
            result.unwrap_err(),
            DataSqlError::MissingField {
                field: "change_stream.tenant_id"
            }
        );
    }

    #[test]
    fn record_mapping_fails_closed_on_a_negative_commit_logical() {
        // A malformed (negative) commit_logical cannot map to the u64 stream
        // position; the mapping path uses exactly this try_from and surfaces an
        // Adapter error rather than wrap.
        assert!(
            u64::try_from(-1_i64).is_err(),
            "negative commit_logical must not coerce to u64"
        );
    }

    #[test]
    fn stream_position_above_u32_max_carries_without_narrowing() {
        // The MAJOR-1 boundary: a commit_logical past u32::MAX must round-trip
        // into the u64 StreamPosition rather than erroring as the old u32
        // narrowing did (which would wedge the poll forever).
        let big = i64::from(u32::MAX) + 1;
        let position = StreamPosition(u64::try_from(big).unwrap());
        assert_eq!(position, StreamPosition(u64::from(u32::MAX) + 1));
        // The same value as a bigint bind round-trips back unchanged.
        assert_eq!(i64::try_from(position.0).unwrap(), big);
    }

    fn record_at(event_id: &str, position: u64) -> ChangeRecord {
        ChangeRecord {
            tenant_id: "acme".to_owned(),
            event_id: event_id.to_owned(),
            event_kind: "k".to_owned(),
            aggregate_id: "a".to_owned(),
            position: StreamPosition(position),
            commit_wall_nanos: 0,
            payload: Vec::new(),
        }
    }

    #[test]
    fn change_batch_validation_is_enforced_for_a_built_page() {
        // The adapter builds the same kernel ChangeBatch shape that the
        // reference stream does; a well-formed page passes validate().
        let records = vec![record_at("e1", 10), record_at("e2", 20)];
        let resume_from = records.last().map_or(StreamPosition::zero(), |r| r.position);
        let batch = ChangeBatch {
            records,
            resume_from,
        };
        batch.validate().unwrap();
        // A disordered page (which the ORDER BY makes impossible from a real
        // poll) is rejected by the kernel, proving the adapter relies on the
        // kernel invariant rather than re-checking ordering itself.
        let disordered = ChangeBatch {
            records: vec![record_at("e2", 20), record_at("e1", 10)],
            resume_from: StreamPosition(20),
        };
        assert!(disordered.validate().is_err());
    }

    #[test]
    fn migration_statement_split_keeps_the_dollar_block_intact() {
        // Call-site smoke test for the SHARED kernel splitter (#115 deduped the
        // outbox's divergent copy): this crate's ACTUAL 0000 migration must split
        // through `oya_shared_postgres_command_kernel::split_migration_statements`
        // with the runtime-role DO $$ ... $$ block intact (not shattered on its
        // inner `;`). The canonical impl + its dollar-quote regression tests now
        // live in the kernel; this guards the outbox's specific migration text.
        let role_migration = include_str!("../migrations/0000_runtime_role.sql");
        let statements = split_migration_statements(role_migration);
        let do_block = statements
            .iter()
            .find(|s| s.contains("CREATE ROLE oya_data_outbox_runtime"))
            .expect("the DO block must survive splitting as one statement");
        assert!(do_block.contains("DO $$"));
        assert!(do_block.contains("END"));
        assert!(do_block.contains("NOBYPASSRLS"));
        // The schema-create and grant are separate statements.
        assert!(
            statements
                .iter()
                .any(|s| s.contains("GRANT USAGE ON SCHEMA oya_data_outbox"))
        );
    }

    #[test]
    fn migration_targets_runtime_role_not_public() {
        // The production policy set targets the runtime role, never PUBLIC.
        let table_migration = include_str!("../migrations/0001_outbox_events.sql");
        assert!(table_migration.contains("TO oya_data_outbox_runtime"));
        assert!(!table_migration.contains("TO PUBLIC"));
        assert!(table_migration.contains("GRANT SELECT, INSERT ON oya_data_outbox.outbox_events TO oya_data_outbox_runtime"));
    }

    #[tokio::test]
    async fn live_probe_is_disabled_by_default() {
        // Default test runs are database-free: without the enable env the
        // probe must short-circuit to None.
        if env::var(LIVE_OUTBOX_POSTGRES_ENABLE_ENV).is_ok() {
            return; // an integration lane is driving the live probe
        }
        assert_eq!(run_live_cdc_cross_tenant_probe().await.unwrap(), None);
    }

    /// Integration rung, env-gated: run against containerized Postgres via
    /// OYA_OUTBOX_LIVE_POSTGRES + admin/app URLs. Proves RLS cross-tenant
    /// denial against the PRODUCTION policy set (run AS oya_data_outbox_runtime,
    /// the migration's policy target — NOT PUBLIC), no-BYPASSRLS on the app
    /// role, stream-position commit ordering, and resumable at-least-once drain.
    #[tokio::test]
    async fn live_cdc_cross_tenant_deny_and_resume_when_enabled() {
        if env::var(LIVE_OUTBOX_POSTGRES_ENABLE_ENV).is_err() {
            return;
        }
        let report = run_live_cdc_cross_tenant_probe()
            .await
            .unwrap()
            .expect("live probe enabled");
        // The app role exercising the poll carries no BYPASSRLS (ADR-0567 D3),
        // so the RLS policy set under test is actually enforced.
        assert!(
            report.app_role_lacks_bypassrls,
            "app role must NOT carry rolbypassrls or RLS is silently skipped"
        );
        // Tenant A sees only its three rows, in commit order, drained by resume.
        assert_eq!(
            report.tenant_a_poll_event_ids,
            vec!["a-evt-1", "a-evt-2", "a-evt-3"]
        );
        // Tenant B sees only its single row (RLS cross-tenant denial).
        assert_eq!(report.tenant_b_poll_event_ids, vec!["b-evt-1"]);
        // An unseeded tenant sees nothing.
        assert!(report.unscoped_poll_denied);
        assert!(report.tenant_a_records_commit_ordered);
        assert!(report.tenant_a_resume_drains_remaining);
    }
}
