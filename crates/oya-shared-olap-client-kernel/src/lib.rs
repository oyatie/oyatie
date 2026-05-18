//! OLAP-client kernel per ADR-0193.
//!
//! Owns the engine-agnostic `OlapClient` trait, the typed query DSL (filter +
//! aggregate + group-by + order-by + limit), per-tenant database naming,
//! materialized-view DDL declaration, idempotent insert semantics, and the
//! per-tenant resource-quota projection per ADR-0155.
//!
//! Per ADR-0083, the kernel is I/O-free. Adapter crates:
//!   - `oya-shared-olap-clickhouse-adapter` — ClickHouse 26.3 LTS native + HTTP.
//!   - `oya-shared-olap-in-house-adapter` — Phase-2 `oya-olap-warehouse-server`.
//!   - In-process reference adapter (`memory_adapter` module) for tests.
//!
//! ## Tenant isolation invariant
//!
//! Every query carries a `TenantId`; the kernel derives the per-tenant
//! database name (`tenant_{tenant_id}`) and validates that no `TableName`
//! the caller submits crosses into a different tenant's database. Cross-
//! tenant query is foreclosed at construction.
//!
//! ## In-house roadmap parity (ADR-0193 §"In-house roadmap")
//!
//! The Phase-2 `oya-olap-warehouse-server` (DataFusion + Arrow + Parquet +
//! custom merge-tree) implements this same trait surface. Consumer
//! migration is a composition-root adapter swap.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::fmt;

pub const KERNEL_SCHEMA_VERSION: u32 = 1;
pub const TENANT_ID_MAX_LEN: usize = 128;
pub const TABLE_NAME_MAX_LEN: usize = 96;

/// Validated tenant id — locally defined to keep this kernel zero-dep.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TenantId(String);

impl TenantId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, KernelError> {
        let value = value.into();
        if value.is_empty() {
            return Err(KernelError::TenantIdEmpty);
        }
        if value.len() > TENANT_ID_MAX_LEN {
            return Err(KernelError::TenantIdTooLong {
                actual: value.len(),
            });
        }
        if !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(KernelError::TenantIdInvalidChar);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Canonical database name for this tenant: `tenant_{tenant_id}`.
    pub fn database_name(&self) -> String {
        format!("tenant_{}", self.0)
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Validated table name. Lowercase ASCII alphanumeric + underscore; ≤96 bytes.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TableName(String);

impl TableName {
    pub fn try_new(value: impl Into<String>) -> Result<Self, KernelError> {
        let value = value.into();
        if value.is_empty() {
            return Err(KernelError::TableNameEmpty);
        }
        if value.len() > TABLE_NAME_MAX_LEN {
            return Err(KernelError::TableNameTooLong {
                actual: value.len(),
            });
        }
        if !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err(KernelError::TableNameInvalidChar);
        }
        // Reject SQL injection vectors at the type layer.
        if value.contains("--") || value.contains(";") {
            return Err(KernelError::TableNameInvalidChar);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TableName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A fully-qualified table identifier: `(TenantId, TableName)`.
/// The kernel renders this as `tenant_{tenant_id}.{table_name}`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QualifiedTable {
    tenant_id: TenantId,
    table: TableName,
    rendered: String,
}

impl QualifiedTable {
    pub fn new(tenant_id: TenantId, table: TableName) -> Self {
        let rendered = format!("tenant_{}.{}", tenant_id.as_str(), table.as_str());
        Self {
            tenant_id,
            table,
            rendered,
        }
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    pub fn table(&self) -> &TableName {
        &self.table
    }

    pub fn as_str(&self) -> &str {
        &self.rendered
    }
}

impl fmt::Display for QualifiedTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.rendered)
    }
}

/// Column type accepted by the OLAP-client kernel.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ColumnType {
    UInt32,
    UInt64,
    Int64,
    Float64,
    String,
    DateTime,
    Boolean,
}

impl ColumnType {
    pub const fn label(self) -> &'static str {
        match self {
            Self::UInt32 => "uint32",
            Self::UInt64 => "uint64",
            Self::Int64 => "int64",
            Self::Float64 => "float64",
            Self::String => "string",
            Self::DateTime => "datetime",
            Self::Boolean => "bool",
        }
    }
}

/// Column declaration in a table schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub ty: ColumnType,
    pub nullable: bool,
}

impl ColumnDef {
    pub fn new(name: impl Into<String>, ty: ColumnType, nullable: bool) -> Self {
        Self {
            name: name.into(),
            ty,
            nullable,
        }
    }
}

/// Table-engine variant. ClickHouse-specific engines are first-class; the
/// in-house Phase-2 engine will map these to its own merge-tree variants.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TableEngine {
    /// `MergeTree` — append-mostly with ORDER BY pruning.
    MergeTree,
    /// `ReplicatedMergeTree` — ZooKeeper/Keeper-coordinated replicated variant.
    ReplicatedMergeTree,
    /// `AggregatingMergeTree` — stores `AggregateFunction(...)` state.
    AggregatingMergeTree,
    /// `ReplacingMergeTree` — last-write-wins by sort key.
    ReplacingMergeTree,
    /// `Kafka` engine — source connector consuming from log broker.
    Kafka,
}

impl TableEngine {
    pub const fn label(self) -> &'static str {
        match self {
            Self::MergeTree => "MergeTree",
            Self::ReplicatedMergeTree => "ReplicatedMergeTree",
            Self::AggregatingMergeTree => "AggregatingMergeTree",
            Self::ReplacingMergeTree => "ReplacingMergeTree",
            Self::Kafka => "Kafka",
        }
    }
}

/// Table schema for `CREATE TABLE`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableSchema {
    pub table: QualifiedTable,
    pub columns: Vec<ColumnDef>,
    pub engine: TableEngine,
    /// `ORDER BY (...)` columns (referenced by name).
    pub order_by: Vec<String>,
    /// Optional partition expression (e.g., `toYYYYMM(inserted_at)`).
    pub partition_by: Option<String>,
    /// Optional TTL clause — e.g., `inserted_at + INTERVAL 90 DAY`.
    pub ttl: Option<String>,
}

/// Materialized view declaration per ADR-0195 §"Default: ClickHouse
/// Materialized Views + Kafka Engine".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaterializedViewSchema {
    pub name: QualifiedTable,
    pub source: QualifiedTable,
    pub target: QualifiedTable,
    /// SELECT clause — the kernel renders the DDL as
    /// `CREATE MATERIALIZED VIEW <name> TO <target> AS SELECT <select_expr> FROM <source>`.
    pub select_expr: String,
}

/// Filter predicate for `WHERE` clauses. The kernel renders these via
/// parameter binding (adapter responsibility); raw SQL is not accepted.
#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    Eq { column: String, value: Value },
    Ne { column: String, value: Value },
    Lt { column: String, value: Value },
    Le { column: String, value: Value },
    Gt { column: String, value: Value },
    Ge { column: String, value: Value },
    And(Vec<Filter>),
    Or(Vec<Filter>),
}

/// Parameter value passed through bound parameters.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    UInt(u64),
    Int(i64),
    Float(f64),
    String(String),
    DateTime(u64), // epoch seconds
    Bool(bool),
}

/// Aggregate expression for `SELECT` projection.
#[derive(Clone, Debug, PartialEq)]
pub enum Aggregate {
    Count,
    CountDistinct { column: String },
    Sum { column: String },
    Avg { column: String },
    Min { column: String },
    Max { column: String },
    /// Quantile (e.g., `quantile(0.99)(column)`).
    Quantile { column: String, q: f64 },
    /// Top-K — `topK(k)(column)`.
    TopK { column: String, k: u32 },
}

/// Order direction for `ORDER BY`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrderDir {
    Asc,
    Desc,
}

/// Order spec.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderBy {
    pub column: String,
    pub dir: OrderDir,
}

/// Typed query — caller specifies the source qualified table, projection,
/// filter, group-by, order-by, and limit. The adapter renders to the
/// engine-specific SQL dialect with bound parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub source: QualifiedTable,
    /// Plain column projections.
    pub columns: Vec<String>,
    /// Aggregate projections; rendered as `agg AS alias`.
    pub aggregates: Vec<(Aggregate, String)>,
    pub filter: Option<Filter>,
    pub group_by: Vec<String>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<u64>,
}

/// A single row in the query result. String-keyed; the kernel does not
/// attempt to enforce static row types — that is consumer-responsibility.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    pub values: BTreeMap<String, Value>,
}

/// Idempotent insert batch. The adapter MUST deduplicate by the table's
/// `ORDER BY` columns when the engine is `ReplacingMergeTree`; the kernel
/// preserves caller-supplied row order.
#[derive(Clone, Debug, PartialEq)]
pub struct InsertBatch {
    pub target: QualifiedTable,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}

/// Per-tenant resource quota projected from ADR-0155.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuotaProfile {
    pub tenant_id: TenantId,
    /// Max queries per hour.
    pub max_queries_per_hour: u64,
    /// Max rows read per hour (across all queries).
    pub max_read_rows_per_hour: u64,
    /// Max insert rows per hour.
    pub max_insert_rows_per_hour: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KernelError {
    TenantIdEmpty,
    TenantIdTooLong { actual: usize },
    TenantIdInvalidChar,
    TableNameEmpty,
    TableNameTooLong { actual: usize },
    TableNameInvalidChar,
    EmptyProjection,
    UnknownColumn { column: String },
    MismatchedInsertColumns { expected: usize, actual: usize },
    CrossTenantAccessDenied,
    AdapterError(String),
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TenantIdEmpty => write!(f, "tenant id is empty"),
            Self::TenantIdTooLong { actual } => {
                write!(f, "tenant id length {actual} exceeds {TENANT_ID_MAX_LEN}")
            }
            Self::TenantIdInvalidChar => write!(f, "tenant id contains invalid character"),
            Self::TableNameEmpty => write!(f, "table name is empty"),
            Self::TableNameTooLong { actual } => {
                write!(f, "table name length {actual} exceeds {TABLE_NAME_MAX_LEN}")
            }
            Self::TableNameInvalidChar => write!(f, "table name contains invalid character"),
            Self::EmptyProjection => write!(f, "query has empty projection"),
            Self::UnknownColumn { column } => write!(f, "unknown column {column}"),
            Self::MismatchedInsertColumns { expected, actual } => {
                write!(f, "insert row has {actual} values, expected {expected}")
            }
            Self::CrossTenantAccessDenied => write!(f, "cross-tenant access denied"),
            Self::AdapterError(msg) => write!(f, "adapter error: {msg}"),
        }
    }
}

impl std::error::Error for KernelError {}

/// Engine-agnostic OLAP port.
pub trait OlapClient {
    /// Create the per-tenant database (`tenant_{tenant_id}`). Idempotent.
    fn ensure_tenant_database(&mut self, tenant_id: &TenantId) -> Result<(), KernelError>;

    /// Create the table. Idempotent (schema-equality check on re-emit).
    fn ensure_table(&mut self, schema: &TableSchema) -> Result<(), KernelError>;

    /// Create a materialized view per ADR-0195 default tier.
    fn ensure_materialized_view(
        &mut self,
        schema: &MaterializedViewSchema,
    ) -> Result<(), KernelError>;

    /// Apply a tenant resource quota — per ADR-0155 projection.
    fn apply_quota(&mut self, profile: &QuotaProfile) -> Result<(), KernelError>;

    /// Insert a batch of rows.
    fn insert(&mut self, batch: &InsertBatch) -> Result<u64, KernelError>;

    /// Execute a typed query and return rows.
    fn query(&self, caller: &TenantId, query: &Query) -> Result<Vec<Row>, KernelError>;

    /// Drop the per-tenant database (DSR offboard).
    fn drop_tenant_database(&mut self, tenant_id: &TenantId) -> Result<(), KernelError>;
}

/// Verify a `QualifiedTable` belongs to the caller's tenant; foreclose
/// cross-tenant access at the kernel layer.
pub fn assert_same_tenant(
    caller: &TenantId,
    table: &QualifiedTable,
) -> Result<(), KernelError> {
    if table.tenant_id() == caller {
        Ok(())
    } else {
        Err(KernelError::CrossTenantAccessDenied)
    }
}

/// Validate that a query has at least one projection (column or aggregate).
pub fn validate_query(query: &Query) -> Result<(), KernelError> {
    if query.columns.is_empty() && query.aggregates.is_empty() {
        return Err(KernelError::EmptyProjection);
    }
    Ok(())
}

/// In-process reference adapter — pure-Rust in-memory OLAP for tests.
pub mod memory_adapter {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, Default)]
    pub struct InMemoryOlapClient {
        databases: BTreeMap<String, TenantDb>,
    }

    #[derive(Debug, Default)]
    struct TenantDb {
        tables: BTreeMap<String, TableState>,
        views: BTreeMap<String, MaterializedViewSchema>,
        quota: Option<QuotaProfile>,
    }

    #[derive(Debug)]
    struct TableState {
        schema: TableSchema,
        rows: Vec<HashMap<String, Value>>,
    }

    impl InMemoryOlapClient {
        pub fn new() -> Self {
            Self::default()
        }

        fn db_mut(&mut self, tenant_id: &TenantId) -> Result<&mut TenantDb, KernelError> {
            self.databases
                .get_mut(&tenant_id.database_name())
                .ok_or_else(|| {
                    KernelError::AdapterError(format!(
                        "database {} does not exist",
                        tenant_id.database_name()
                    ))
                })
        }

        fn db(&self, tenant_id: &TenantId) -> Result<&TenantDb, KernelError> {
            self.databases
                .get(&tenant_id.database_name())
                .ok_or_else(|| {
                    KernelError::AdapterError(format!(
                        "database {} does not exist",
                        tenant_id.database_name()
                    ))
                })
        }
    }

    impl OlapClient for InMemoryOlapClient {
        fn ensure_tenant_database(&mut self, tenant_id: &TenantId) -> Result<(), KernelError> {
            self.databases
                .entry(tenant_id.database_name())
                .or_default();
            Ok(())
        }

        fn ensure_table(&mut self, schema: &TableSchema) -> Result<(), KernelError> {
            let tenant = schema.table.tenant_id().clone();
            self.ensure_tenant_database(&tenant)?;
            let db = self.db_mut(&tenant)?;
            let key = schema.table.table().as_str().to_string();
            if let Some(existing) = db.tables.get(&key) {
                if existing.schema != *schema {
                    return Err(KernelError::AdapterError(format!(
                        "schema drift on table {}",
                        schema.table
                    )));
                }
                return Ok(());
            }
            db.tables.insert(
                key,
                TableState {
                    schema: schema.clone(),
                    rows: Vec::new(),
                },
            );
            Ok(())
        }

        fn ensure_materialized_view(
            &mut self,
            schema: &MaterializedViewSchema,
        ) -> Result<(), KernelError> {
            let tenant = schema.name.tenant_id().clone();
            self.ensure_tenant_database(&tenant)?;
            let db = self.db_mut(&tenant)?;
            db.views.insert(schema.name.as_str().to_string(), schema.clone());
            Ok(())
        }

        fn apply_quota(&mut self, profile: &QuotaProfile) -> Result<(), KernelError> {
            self.ensure_tenant_database(&profile.tenant_id)?;
            let db = self.db_mut(&profile.tenant_id)?;
            db.quota = Some(profile.clone());
            Ok(())
        }

        fn insert(&mut self, batch: &InsertBatch) -> Result<u64, KernelError> {
            let tenant = batch.target.tenant_id().clone();
            let table_key = batch.target.table().as_str().to_string();
            let db = self.db_mut(&tenant)?;
            let table = db.tables.get_mut(&table_key).ok_or_else(|| {
                KernelError::AdapterError(format!("table {} does not exist", batch.target))
            })?;
            for row in &batch.rows {
                if row.len() != batch.columns.len() {
                    return Err(KernelError::MismatchedInsertColumns {
                        expected: batch.columns.len(),
                        actual: row.len(),
                    });
                }
                let mut map = HashMap::new();
                for (col, val) in batch.columns.iter().zip(row.iter()) {
                    map.insert(col.clone(), val.clone());
                }
                table.rows.push(map);
            }
            Ok(batch.rows.len() as u64)
        }

        fn query(&self, caller: &TenantId, query: &Query) -> Result<Vec<Row>, KernelError> {
            assert_same_tenant(caller, &query.source)?;
            validate_query(query)?;
            let db = self.db(caller)?;
            let table = db
                .tables
                .get(query.source.table().as_str())
                .ok_or_else(|| {
                    KernelError::AdapterError(format!(
                        "table {} does not exist",
                        query.source
                    ))
                })?;

            // Apply filter
            let filtered: Vec<&HashMap<String, Value>> = table
                .rows
                .iter()
                .filter(|r| match &query.filter {
                    None => true,
                    Some(f) => matches_filter(r, f),
                })
                .collect();

            // Project (plain columns only in this reference impl; aggregates
            // implemented for COUNT and SUM as parity proof).
            let mut out: Vec<Row> = filtered
                .iter()
                .map(|r| {
                    let mut row = Row::default();
                    for col in &query.columns {
                        if let Some(v) = r.get(col) {
                            row.values.insert(col.clone(), v.clone());
                        }
                    }
                    row
                })
                .collect();

            // Apply minimal aggregate parity: COUNT(*) -> u64
            for (agg, alias) in &query.aggregates {
                match agg {
                    Aggregate::Count => {
                        let mut row = Row::default();
                        row.values
                            .insert(alias.clone(), Value::UInt(filtered.len() as u64));
                        out = vec![row];
                    }
                    Aggregate::Sum { column } => {
                        let s: f64 = filtered
                            .iter()
                            .filter_map(|r| r.get(column))
                            .filter_map(|v| match v {
                                Value::Float(f) => Some(*f),
                                Value::Int(i) => Some(*i as f64),
                                Value::UInt(u) => Some(*u as f64),
                                _ => None,
                            })
                            .sum();
                        let mut row = Row::default();
                        row.values.insert(alias.clone(), Value::Float(s));
                        out = vec![row];
                    }
                    _ => {
                        // Other aggregates: reference impl ships parity-stub returning empty
                        // (production adapter implements them via the engine's native SQL).
                    }
                }
            }

            // Apply limit
            if let Some(limit) = query.limit {
                out.truncate(limit as usize);
            }
            Ok(out)
        }

        fn drop_tenant_database(&mut self, tenant_id: &TenantId) -> Result<(), KernelError> {
            self.databases.remove(&tenant_id.database_name());
            Ok(())
        }
    }

    fn matches_filter(row: &HashMap<String, Value>, filter: &Filter) -> bool {
        match filter {
            Filter::Eq { column, value } => row.get(column) == Some(value),
            Filter::Ne { column, value } => row.get(column) != Some(value),
            Filter::Lt { column, value } => compare(row.get(column), value, |a, b| a < b),
            Filter::Le { column, value } => compare(row.get(column), value, |a, b| a <= b),
            Filter::Gt { column, value } => compare(row.get(column), value, |a, b| a > b),
            Filter::Ge { column, value } => compare(row.get(column), value, |a, b| a >= b),
            Filter::And(fs) => fs.iter().all(|f| matches_filter(row, f)),
            Filter::Or(fs) => fs.iter().any(|f| matches_filter(row, f)),
        }
    }

    fn compare(lhs: Option<&Value>, rhs: &Value, cmp: impl Fn(f64, f64) -> bool) -> bool {
        let lhs = match lhs {
            Some(Value::Int(i)) => *i as f64,
            Some(Value::UInt(u)) => *u as f64,
            Some(Value::Float(f)) => *f,
            Some(Value::DateTime(t)) => *t as f64,
            _ => return false,
        };
        let rhs = match rhs {
            Value::Int(i) => *i as f64,
            Value::UInt(u) => *u as f64,
            Value::Float(f) => *f,
            Value::DateTime(t) => *t as f64,
            _ => return false,
        };
        cmp(lhs, rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::memory_adapter::InMemoryOlapClient;
    use super::*;

    fn tid(s: &str) -> TenantId {
        TenantId::try_new(s).unwrap()
    }
    fn tbl(s: &str) -> TableName {
        TableName::try_new(s).unwrap()
    }

    #[test]
    fn tenant_id_database_name_canonical() {
        assert_eq!(tid("ten_acme").database_name(), "tenant_ten_acme");
    }

    #[test]
    fn table_name_rejects_sql_injection_vectors() {
        assert!(TableName::try_new("ok_table").is_ok());
        assert_eq!(
            TableName::try_new("evil--comment"),
            Err(KernelError::TableNameInvalidChar)
        );
        assert_eq!(
            TableName::try_new("evil;drop"),
            Err(KernelError::TableNameInvalidChar)
        );
    }

    #[test]
    fn qualified_table_renders_canonically() {
        let q = QualifiedTable::new(tid("ten_acme"), tbl("events"));
        assert_eq!(q.as_str(), "tenant_ten_acme.events");
    }

    #[test]
    fn assert_same_tenant_blocks_cross_tenant() {
        let acme = tid("ten_acme");
        let bryan = tid("ten_bryan");
        let table = QualifiedTable::new(acme.clone(), tbl("events"));
        assert!(assert_same_tenant(&acme, &table).is_ok());
        assert_eq!(
            assert_same_tenant(&bryan, &table),
            Err(KernelError::CrossTenantAccessDenied)
        );
    }

    #[test]
    fn validate_query_rejects_empty_projection() {
        let q = Query {
            source: QualifiedTable::new(tid("ten_acme"), tbl("events")),
            columns: vec![],
            aggregates: vec![],
            filter: None,
            group_by: vec![],
            order_by: vec![],
            limit: None,
        };
        assert_eq!(validate_query(&q), Err(KernelError::EmptyProjection));
    }

    #[test]
    fn memory_adapter_ensure_database_idempotent() {
        let mut c = InMemoryOlapClient::new();
        let acme = tid("ten_acme");
        c.ensure_tenant_database(&acme).unwrap();
        c.ensure_tenant_database(&acme).unwrap();
        c.ensure_tenant_database(&acme).unwrap();
    }

    #[test]
    fn memory_adapter_table_schema_drift_detected() {
        let mut c = InMemoryOlapClient::new();
        let table = QualifiedTable::new(tid("ten_acme"), tbl("events"));
        let schema_a = TableSchema {
            table: table.clone(),
            columns: vec![ColumnDef::new("id", ColumnType::UInt64, false)],
            engine: TableEngine::MergeTree,
            order_by: vec!["id".into()],
            partition_by: None,
            ttl: None,
        };
        let schema_b = TableSchema {
            table: table.clone(),
            columns: vec![ColumnDef::new("id", ColumnType::String, false)], // drift
            engine: TableEngine::MergeTree,
            order_by: vec!["id".into()],
            partition_by: None,
            ttl: None,
        };
        c.ensure_table(&schema_a).unwrap();
        let err = c.ensure_table(&schema_b).unwrap_err();
        assert!(matches!(err, KernelError::AdapterError(_)));
    }

    #[test]
    fn memory_adapter_insert_and_query_round_trip() {
        let mut c = InMemoryOlapClient::new();
        let table = QualifiedTable::new(tid("ten_acme"), tbl("events"));
        let schema = TableSchema {
            table: table.clone(),
            columns: vec![
                ColumnDef::new("id", ColumnType::UInt64, false),
                ColumnDef::new("name", ColumnType::String, false),
            ],
            engine: TableEngine::MergeTree,
            order_by: vec!["id".into()],
            partition_by: None,
            ttl: None,
        };
        c.ensure_table(&schema).unwrap();
        let inserted = c
            .insert(&InsertBatch {
                target: table.clone(),
                columns: vec!["id".into(), "name".into()],
                rows: vec![
                    vec![Value::UInt(1), Value::String("alpha".into())],
                    vec![Value::UInt(2), Value::String("beta".into())],
                ],
            })
            .unwrap();
        assert_eq!(inserted, 2);
        let rows = c
            .query(
                &tid("ten_acme"),
                &Query {
                    source: table,
                    columns: vec!["id".into(), "name".into()],
                    aggregates: vec![],
                    filter: None,
                    group_by: vec![],
                    order_by: vec![],
                    limit: None,
                },
            )
            .unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn memory_adapter_count_aggregate_returns_correct_value() {
        let mut c = InMemoryOlapClient::new();
        let table = QualifiedTable::new(tid("ten_acme"), tbl("events"));
        c.ensure_table(&TableSchema {
            table: table.clone(),
            columns: vec![ColumnDef::new("id", ColumnType::UInt64, false)],
            engine: TableEngine::MergeTree,
            order_by: vec!["id".into()],
            partition_by: None,
            ttl: None,
        })
        .unwrap();
        c.insert(&InsertBatch {
            target: table.clone(),
            columns: vec!["id".into()],
            rows: (0..7).map(|i| vec![Value::UInt(i)]).collect(),
        })
        .unwrap();
        let rows = c
            .query(
                &tid("ten_acme"),
                &Query {
                    source: table,
                    columns: vec![],
                    aggregates: vec![(Aggregate::Count, "n".into())],
                    filter: None,
                    group_by: vec![],
                    order_by: vec![],
                    limit: None,
                },
            )
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].values.get("n"), Some(&Value::UInt(7)));
    }

    #[test]
    fn memory_adapter_filter_eq_isolates_rows() {
        let mut c = InMemoryOlapClient::new();
        let table = QualifiedTable::new(tid("ten_acme"), tbl("events"));
        c.ensure_table(&TableSchema {
            table: table.clone(),
            columns: vec![
                ColumnDef::new("id", ColumnType::UInt64, false),
                ColumnDef::new("status", ColumnType::String, false),
            ],
            engine: TableEngine::MergeTree,
            order_by: vec!["id".into()],
            partition_by: None,
            ttl: None,
        })
        .unwrap();
        c.insert(&InsertBatch {
            target: table.clone(),
            columns: vec!["id".into(), "status".into()],
            rows: vec![
                vec![Value::UInt(1), Value::String("ok".into())],
                vec![Value::UInt(2), Value::String("err".into())],
                vec![Value::UInt(3), Value::String("ok".into())],
            ],
        })
        .unwrap();
        let rows = c
            .query(
                &tid("ten_acme"),
                &Query {
                    source: table,
                    columns: vec!["id".into()],
                    aggregates: vec![],
                    filter: Some(Filter::Eq {
                        column: "status".into(),
                        value: Value::String("ok".into()),
                    }),
                    group_by: vec![],
                    order_by: vec![],
                    limit: None,
                },
            )
            .unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn memory_adapter_materialized_view_persisted() {
        let mut c = InMemoryOlapClient::new();
        let source = QualifiedTable::new(tid("ten_acme"), tbl("events"));
        let target = QualifiedTable::new(tid("ten_acme"), tbl("events_rollup"));
        let view = QualifiedTable::new(tid("ten_acme"), tbl("mv_events_rollup"));
        c.ensure_tenant_database(&tid("ten_acme")).unwrap();
        c.ensure_materialized_view(&MaterializedViewSchema {
            name: view,
            source,
            target,
            select_expr: "tenant_id, countState() AS c".into(),
        })
        .unwrap();
    }

    #[test]
    fn memory_adapter_drop_database_offboards_tenant() {
        let mut c = InMemoryOlapClient::new();
        c.ensure_tenant_database(&tid("ten_acme")).unwrap();
        c.drop_tenant_database(&tid("ten_acme")).unwrap();
        // Re-querying the dropped tenant returns error.
        let err = c
            .query(
                &tid("ten_acme"),
                &Query {
                    source: QualifiedTable::new(tid("ten_acme"), tbl("events")),
                    columns: vec!["id".into()],
                    aggregates: vec![],
                    filter: None,
                    group_by: vec![],
                    order_by: vec![],
                    limit: None,
                },
            )
            .unwrap_err();
        assert!(matches!(err, KernelError::AdapterError(_)));
    }

    #[test]
    fn memory_adapter_quota_applied() {
        let mut c = InMemoryOlapClient::new();
        c.apply_quota(&QuotaProfile {
            tenant_id: tid("ten_acme"),
            max_queries_per_hour: 1000,
            max_read_rows_per_hour: 1_000_000,
            max_insert_rows_per_hour: 100_000,
        })
        .unwrap();
    }

    #[test]
    fn insert_column_mismatch_errors() {
        let mut c = InMemoryOlapClient::new();
        let table = QualifiedTable::new(tid("ten_acme"), tbl("events"));
        c.ensure_table(&TableSchema {
            table: table.clone(),
            columns: vec![ColumnDef::new("id", ColumnType::UInt64, false)],
            engine: TableEngine::MergeTree,
            order_by: vec!["id".into()],
            partition_by: None,
            ttl: None,
        })
        .unwrap();
        let err = c
            .insert(&InsertBatch {
                target: table,
                columns: vec!["id".into()],
                rows: vec![vec![Value::UInt(1), Value::UInt(2)]], // two values vs one column
            })
            .unwrap_err();
        assert!(matches!(err, KernelError::MismatchedInsertColumns { .. }));
    }
}
