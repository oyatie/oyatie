//! Owned `oya-data` SQL interface: the only persistence API services may link
//! (ADR-0536 D-10; LANE-G03 / story G003).
//!
//! Ports model the OWNED destination stack — the W5 bespoke multi-Raft
//! leader-per-range engine with HLC commit timestamps behind the
//! TrueTime-shaped [`clock::ClockSource`] trait — per the founder
//! ports-for-owned-stack doctrine recorded in ADR-0536 Drivers. The
//! CockroachDB-class engine and the Postgres-RLS sqlx adapters are ADR-0510
//! transitional implementations that absorb ALL impedance behind this
//! contract. Review invariant: nothing here changes at W5 cutover.
//!
//! Precedent: Google Spanner / CockroachDB / TiKV (ranged consensus + HLC,
//! ADR-0536 D-10 5/5 convergence); AWS SaaS lens pooled isolation via
//! row-level security; ADR-0537 §bootstrap step 6 (separate single-Raft
//! bootstrap metastore breaks the persistence self-hosting recursion).
//!
//! Pure kernel: typed statements, sessions, and consistency levels with
//! surface-all validation. NO handlers, NO IO — concrete adapters bind these
//! shapes through sqlx/tokio later, mirroring
//! `oya-shared-postgres-command-kernel`'s executor seam.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod clock;

use serde::{Deserialize, Serialize};

use crate::clock::{ClockError, ClockSource, HlcTimestamp};

/// Errors the owned SQL port can surface. Closed set; adapters map their
/// engine failures into [`DataSqlError::Adapter`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataSqlError {
    MissingField {
        field: &'static str,
    },
    /// Tenant-scoped sessions are structurally forbidden against the
    /// bootstrap metastore: tenant data never lives there (ADR-0537 step 6).
    TenantScopeForbiddenInBootstrapMetastore,
    EmptyWriteBatch,
    /// Bounded staleness must name a positive staleness window.
    ZeroStalenessBound,
    /// A row's width does not match the column list (surface-all read shape).
    RowWidthMismatch {
        columns: usize,
        row_index: usize,
        row_width: usize,
    },
    Clock(ClockError),
    /// Adapter-side failure, carried opaquely so the port stays engine-free.
    Adapter(String),
}

impl From<ClockError> for DataSqlError {
    fn from(error: ClockError) -> Self {
        Self::Clock(error)
    }
}

impl core::fmt::Display for DataSqlError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingField { field } => write!(f, "missing or blank field: {field}"),
            Self::TenantScopeForbiddenInBootstrapMetastore => write!(
                f,
                "tenant-scoped sessions are forbidden against the bootstrap metastore"
            ),
            Self::EmptyWriteBatch => write!(f, "write batch carries no statements"),
            Self::ZeroStalenessBound => {
                write!(f, "bounded-staleness reads need a positive window")
            }
            Self::RowWidthMismatch {
                columns,
                row_index,
                row_width,
            } => write!(
                f,
                "row {row_index} has {row_width} values but the row set names {columns} columns"
            ),
            Self::Clock(error) => write!(f, "clock error: {error}"),
            Self::Adapter(detail) => write!(f, "adapter failure: {detail}"),
        }
    }
}

impl std::error::Error for DataSqlError {}

/// Which logical store a session targets. The bootstrap metastore is the
/// separate single-Raft store that holds engine bring-up metadata so the
/// data plane never self-hosts its own bootstrap (ADR-0537 step 6).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataStore {
    TenantData,
    BootstrapMetastore,
}

/// Who a session acts as. Pooled tenant isolation (AWS SaaS lens) makes the
/// tenant scope mandatory at session construction — the type-system one-way
/// door: a tenant-scoped session cannot name another tenant's rows, and
/// adapters enforce the same boundary with Postgres RLS / engine-native
/// row scoping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SessionScope {
    Tenant {
        tenant_id: String, // data_class: TENANT_SCOPED
        cell_id: String,   // data_class: INTERNAL_ONLY
    },
    ControlPlane {
        service: String, // data_class: INTERNAL_ONLY
    },
}

/// Everything an adapter needs to open a session. Surface-all validation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionDescriptor {
    pub store: DataStore,         // data_class: INTERNAL_ONLY
    pub scope: SessionScope,      // data_class: INTERNAL_ONLY
    pub application_name: String, // data_class: INTERNAL_ONLY
}

impl SessionDescriptor {
    /// Tenant-scoped session against the tenant data store.
    pub fn tenant_data(
        tenant_id: impl Into<String>,
        cell_id: impl Into<String>,
        application_name: impl Into<String>,
    ) -> Result<Self, DataSqlError> {
        let descriptor = Self {
            store: DataStore::TenantData,
            scope: SessionScope::Tenant {
                tenant_id: tenant_id.into(),
                cell_id: cell_id.into(),
            },
            application_name: application_name.into(),
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    /// Control-plane session (tenant lifecycle, schema custody) against a
    /// chosen store.
    pub fn control_plane(
        store: DataStore,
        service: impl Into<String>,
        application_name: impl Into<String>,
    ) -> Result<Self, DataSqlError> {
        let descriptor = Self {
            store,
            scope: SessionScope::ControlPlane {
                service: service.into(),
            },
            application_name: application_name.into(),
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    pub fn validate(&self) -> Result<(), DataSqlError> {
        require_non_blank(&self.application_name, "session.application_name")?;
        match &self.scope {
            SessionScope::Tenant { tenant_id, cell_id } => {
                require_non_blank(tenant_id, "session.tenant_id")?;
                require_non_blank(cell_id, "session.cell_id")?;
                if self.store == DataStore::BootstrapMetastore {
                    return Err(DataSqlError::TenantScopeForbiddenInBootstrapMetastore);
                }
            }
            SessionScope::ControlPlane { service } => {
                require_non_blank(service, "session.service")?;
            }
        }
        Ok(())
    }
}

/// Engine-neutral parameter values. Parameterized statements are the only
/// statement shape the port admits — string interpolation has no entry point.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum SqlValue {
    Text(String),
    Int64(i64),
    Bool(bool),
    Bytes(Vec<u8>),
    TextArray(Vec<String>),
    Null,
}

/// One named, parameterized statement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Statement {
    pub name: String,          // data_class: INTERNAL_ONLY
    pub sql: String,           // data_class: INTERNAL_ONLY
    pub params: Vec<SqlValue>, // data_class: TENANT_SCOPED
}

impl Statement {
    pub fn new(
        name: impl Into<String>,
        sql: impl Into<String>,
        params: Vec<SqlValue>,
    ) -> Result<Self, DataSqlError> {
        let statement = Self {
            name: name.into(),
            sql: sql.into(),
            params,
        };
        statement.validate()?;
        Ok(statement)
    }

    pub fn validate(&self) -> Result<(), DataSqlError> {
        require_non_blank(&self.name, "statement.name")?;
        require_non_blank(&self.sql, "statement.sql")
    }
}

/// Read consistency, shaped for the HLC destination engine: strong reads,
/// bounded-staleness follower reads, and exact snapshot reads at an HLC
/// timestamp (CockroachDB `AS OF SYSTEM TIME` / Spanner stale-read shapes).
/// Adapters absorb what their transitional engine cannot serve (a Postgres
/// adapter upgrades everything to `Strong`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "level", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReadConsistency {
    Strong,
    BoundedStaleness { max_staleness_ms: u32 },
    SnapshotAt { timestamp: HlcTimestamp },
}

impl ReadConsistency {
    pub fn validate(&self) -> Result<(), DataSqlError> {
        match self {
            Self::BoundedStaleness {
                max_staleness_ms: 0,
            } => Err(DataSqlError::ZeroStalenessBound),
            _ => Ok(()),
        }
    }
}

/// A consistency-tagged read.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadQuery {
    pub statement: Statement,         // data_class: TENANT_SCOPED
    pub consistency: ReadConsistency, // data_class: INTERNAL_ONLY
}

impl ReadQuery {
    pub fn new(statement: Statement, consistency: ReadConsistency) -> Result<Self, DataSqlError> {
        statement.validate()?;
        consistency.validate()?;
        Ok(Self {
            statement,
            consistency,
        })
    }
}

/// An atomic write: every statement commits or none does.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WriteBatch {
    pub statements: Vec<Statement>, // data_class: TENANT_SCOPED
}

impl WriteBatch {
    pub fn new(statements: Vec<Statement>) -> Result<Self, DataSqlError> {
        if statements.is_empty() {
            return Err(DataSqlError::EmptyWriteBatch);
        }
        for statement in &statements {
            statement.validate()?;
        }
        Ok(Self { statements })
    }

    #[must_use]
    pub fn statement_names(&self) -> Vec<String> {
        self.statements
            .iter()
            .map(|statement| statement.name.clone())
            .collect()
    }
}

/// Proof of commit: the HLC commit timestamp is the W5 contract shape;
/// transitional adapters synthesize it from their session clock so callers
/// never observe an engine difference.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitReceipt {
    pub store: DataStore,               // data_class: INTERNAL_ONLY
    pub commit_timestamp: HlcTimestamp, // data_class: INTERNAL_ONLY
    pub statement_names: Vec<String>,   // data_class: INTERNAL_ONLY
}

/// Column-named result rows with surface-all width validation.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RowSet {
    pub columns: Vec<String>,     // data_class: INTERNAL_ONLY
    pub rows: Vec<Vec<SqlValue>>, // data_class: TENANT_SCOPED
}

impl RowSet {
    pub fn new(columns: Vec<String>, rows: Vec<Vec<SqlValue>>) -> Result<Self, DataSqlError> {
        let row_set = Self { columns, rows };
        row_set.validate()?;
        Ok(row_set)
    }

    pub fn validate(&self) -> Result<(), DataSqlError> {
        for (row_index, row) in self.rows.iter().enumerate() {
            if row.len() != self.columns.len() {
                return Err(DataSqlError::RowWidthMismatch {
                    columns: self.columns.len(),
                    row_index,
                    row_width: row.len(),
                });
            }
        }
        Ok(())
    }
}

/// A scoped session: the unit adapters implement. Object-safe so services
/// hold `Box<dyn DataSession>` without naming an engine.
pub trait DataSession: core::fmt::Debug {
    fn descriptor(&self) -> &SessionDescriptor;
    fn execute_write(&mut self, batch: &WriteBatch) -> Result<CommitReceipt, DataSqlError>;
    fn execute_read(&mut self, query: &ReadQuery) -> Result<RowSet, DataSqlError>;
}

/// The owned client port: the only way services obtain persistence sessions.
pub trait DataClient {
    fn open_session(
        &mut self,
        descriptor: &SessionDescriptor,
    ) -> Result<Box<dyn DataSession>, DataSqlError>;
}

/// In-crate reference implementation for contract tests (the
/// `RecordingSqlBatchExecutor` / resource-provider reference-provider
/// pattern): validates every shape, records executed batches, and stamps
/// commits from an injected [`ClockSource`]. NOT a store — reads return the
/// empty row set; real adapters live beside sqlx.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingDataClient<C: ClockSource + Clone> {
    clock: C,
}

impl<C: ClockSource + Clone> RecordingDataClient<C> {
    #[must_use]
    pub fn new(clock: C) -> Self {
        Self { clock }
    }
}

impl<C: ClockSource + Clone + core::fmt::Debug + 'static> DataClient for RecordingDataClient<C> {
    fn open_session(
        &mut self,
        descriptor: &SessionDescriptor,
    ) -> Result<Box<dyn DataSession>, DataSqlError> {
        descriptor.validate()?;
        Ok(Box::new(RecordingDataSession {
            descriptor: descriptor.clone(),
            clock: self.clock.clone(),
            committed_batches: Vec::new(),
            executed_reads: Vec::new(),
        }))
    }
}

/// Session companion to [`RecordingDataClient`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingDataSession<C: ClockSource> {
    descriptor: SessionDescriptor,
    clock: C,
    pub committed_batches: Vec<WriteBatch>, // data_class: INTERNAL_ONLY
    pub executed_reads: Vec<ReadQuery>,     // data_class: INTERNAL_ONLY
}

impl<C: ClockSource + core::fmt::Debug> DataSession for RecordingDataSession<C> {
    fn descriptor(&self) -> &SessionDescriptor {
        &self.descriptor
    }

    fn execute_write(&mut self, batch: &WriteBatch) -> Result<CommitReceipt, DataSqlError> {
        if batch.statements.is_empty() {
            return Err(DataSqlError::EmptyWriteBatch);
        }
        for statement in &batch.statements {
            statement.validate()?;
        }
        let bound = self.clock.now_bound()?;
        self.committed_batches.push(batch.clone());
        Ok(CommitReceipt {
            store: self.descriptor.store,
            commit_timestamp: bound.earliest,
            statement_names: batch.statement_names(),
        })
    }

    fn execute_read(&mut self, query: &ReadQuery) -> Result<RowSet, DataSqlError> {
        query.statement.validate()?;
        query.consistency.validate()?;
        self.executed_reads.push(query.clone());
        Ok(RowSet::default())
    }
}

fn require_non_blank(value: &str, field: &'static str) -> Result<(), DataSqlError> {
    if value.trim().is_empty() {
        Err(DataSqlError::MissingField { field })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::{ClockBound, FixedClockSource};

    fn tenant_descriptor() -> SessionDescriptor {
        SessionDescriptor::tenant_data("acme", "cell-001", "oyatie-tenancy").unwrap()
    }

    fn statement() -> Statement {
        Statement::new(
            "insert_tenant_row",
            "INSERT INTO tenants (tenant_id, display_name) VALUES ($1, $2)",
            vec![
                SqlValue::Text("acme".to_owned()),
                SqlValue::Text("Acme Corp".to_owned()),
            ],
        )
        .unwrap()
    }

    fn pinned_clock() -> FixedClockSource {
        FixedClockSource::new(
            ClockBound::new(HlcTimestamp::new(1_000, 0), HlcTimestamp::new(1_500, 0)).unwrap(),
        )
    }

    #[test]
    fn tenant_descriptor_requires_every_routing_field() {
        assert_eq!(
            SessionDescriptor::tenant_data("", "cell-001", "app").unwrap_err(),
            DataSqlError::MissingField {
                field: "session.tenant_id"
            }
        );
        assert_eq!(
            SessionDescriptor::tenant_data("acme", " ", "app").unwrap_err(),
            DataSqlError::MissingField {
                field: "session.cell_id"
            }
        );
        assert_eq!(
            SessionDescriptor::tenant_data("acme", "cell-001", "").unwrap_err(),
            DataSqlError::MissingField {
                field: "session.application_name"
            }
        );
        tenant_descriptor().validate().unwrap();
    }

    #[test]
    fn bootstrap_metastore_structurally_refuses_tenant_scope() {
        let descriptor = SessionDescriptor {
            store: DataStore::BootstrapMetastore,
            scope: SessionScope::Tenant {
                tenant_id: "acme".to_owned(),
                cell_id: "cell-001".to_owned(),
            },
            application_name: "oyatie-tenancy".to_owned(),
        };
        assert_eq!(
            descriptor.validate().unwrap_err(),
            DataSqlError::TenantScopeForbiddenInBootstrapMetastore
        );
        // Control-plane access to the bootstrap metastore stays legal.
        SessionDescriptor::control_plane(
            DataStore::BootstrapMetastore,
            "oya-data-operator",
            "oyatie-data-operator",
        )
        .unwrap();
    }

    #[test]
    fn descriptor_round_trips_and_rejects_unknown_fields() {
        let descriptor = tenant_descriptor();
        let json = serde_json::to_string(&descriptor).unwrap();
        assert_eq!(
            serde_json::from_str::<SessionDescriptor>(&json).unwrap(),
            descriptor
        );
        let mut value = serde_json::to_value(&descriptor).unwrap();
        value["surprise"] = serde_json::json!(1);
        assert!(serde_json::from_value::<SessionDescriptor>(value).is_err());
    }

    #[test]
    fn statement_refuses_blank_name_and_sql() {
        assert!(Statement::new(" ", "SELECT 1", vec![]).is_err());
        assert!(Statement::new("name", "", vec![]).is_err());
    }

    #[test]
    fn write_batch_refuses_empty_statement_set() {
        assert_eq!(
            WriteBatch::new(vec![]).unwrap_err(),
            DataSqlError::EmptyWriteBatch
        );
        assert_eq!(
            WriteBatch::new(vec![statement()])
                .unwrap()
                .statement_names(),
            vec!["insert_tenant_row"]
        );
    }

    #[test]
    fn bounded_staleness_needs_a_positive_window() {
        assert_eq!(
            ReadConsistency::BoundedStaleness {
                max_staleness_ms: 0
            }
            .validate()
            .unwrap_err(),
            DataSqlError::ZeroStalenessBound
        );
        ReadConsistency::Strong.validate().unwrap();
        ReadConsistency::SnapshotAt {
            timestamp: HlcTimestamp::new(9, 0),
        }
        .validate()
        .unwrap();
    }

    #[test]
    fn consistency_levels_round_trip_closed() {
        for consistency in [
            ReadConsistency::Strong,
            ReadConsistency::BoundedStaleness {
                max_staleness_ms: 250,
            },
            ReadConsistency::SnapshotAt {
                timestamp: HlcTimestamp::new(7, 3),
            },
        ] {
            let json = serde_json::to_string(&consistency).unwrap();
            assert_eq!(
                serde_json::from_str::<ReadConsistency>(&json).unwrap(),
                consistency
            );
        }
        assert!(serde_json::from_str::<ReadConsistency>(r#"{"level":"dirty_read"}"#).is_err());
    }

    #[test]
    fn row_set_surfaces_width_mismatches() {
        let err = RowSet::new(
            vec!["tenant_id".to_owned(), "display_name".to_owned()],
            vec![vec![SqlValue::Text("acme".to_owned())]],
        )
        .unwrap_err();
        assert_eq!(
            err,
            DataSqlError::RowWidthMismatch {
                columns: 2,
                row_index: 0,
                row_width: 1
            }
        );
    }

    #[test]
    fn recording_client_stamps_commits_from_the_clock_port() {
        let mut client = RecordingDataClient::new(pinned_clock());
        let mut session = client.open_session(&tenant_descriptor()).unwrap();
        let receipt = session
            .execute_write(&WriteBatch::new(vec![statement()]).unwrap())
            .unwrap();
        assert_eq!(receipt.commit_timestamp, HlcTimestamp::new(1_000, 0));
        assert_eq!(receipt.store, DataStore::TenantData);
        assert_eq!(receipt.statement_names, vec!["insert_tenant_row"]);
    }

    #[test]
    fn recording_client_refuses_invalid_descriptors_at_open() {
        let mut client = RecordingDataClient::new(pinned_clock());
        let invalid = SessionDescriptor {
            store: DataStore::BootstrapMetastore,
            scope: SessionScope::Tenant {
                tenant_id: "acme".to_owned(),
                cell_id: "cell-001".to_owned(),
            },
            application_name: "oyatie-tenancy".to_owned(),
        };
        assert_eq!(
            client.open_session(&invalid).unwrap_err(),
            DataSqlError::TenantScopeForbiddenInBootstrapMetastore
        );
    }

    #[test]
    fn recording_session_reads_validate_and_record() {
        let mut client = RecordingDataClient::new(pinned_clock());
        let mut session = client.open_session(&tenant_descriptor()).unwrap();
        let query = ReadQuery::new(
            Statement::new("read_tenant", "SELECT tenant_id FROM tenants", vec![]).unwrap(),
            ReadConsistency::BoundedStaleness {
                max_staleness_ms: 250,
            },
        )
        .unwrap();
        let rows = session.execute_read(&query).unwrap();
        assert_eq!(rows, RowSet::default());
    }

    #[test]
    fn errors_render_human_readable_diagnostics() {
        let rendered = DataSqlError::TenantScopeForbiddenInBootstrapMetastore.to_string();
        assert!(rendered.contains("bootstrap metastore"));
        let rendered =
            DataSqlError::from(ClockError::LogicalOverflow { wall_nanos: 5 }).to_string();
        assert!(rendered.contains("clock error"));
    }
}
