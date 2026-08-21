//! Postgres/Citus migration bundle for messenger-message-stream.
//!
//! This crate is intentionally adapter-shaped but runtime-free: it publishes the
//! first tenant/cell/shard-safe SQL migration bundle plus deterministic checks
//! that can run in unit tests before a later sqlx/runtime adapter lands.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use shared_postgres_command_kernel::{
    SqlCommand, SqlCommandError, SqlParam, SqlWriteBatch, TenantSqlContext, required_field,
    text_array_values,
};

pub const MIGRATION_0001: &str = include_str!("../migrations/0001_message_stream.sql");
pub const SERVICE_ID: &str = "messenger-message-stream";
pub const DISTRIBUTION_COLUMN: &str = "tenant_id";
pub const INSERT_MESSAGE_SQL: &str = r#"
INSERT INTO messenger_message_stream.messages (
  tenant_id,
  home_cell,
  shard_key,
  jurisdiction_code,
  channel_id,
  message_id,
  author_ref,
  envelope_ref,
  retention_policy_id,
  legal_hold_ids,
  policy_decision_ref,
  idempotency_key,
  audit_correlation_id
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
)
ON CONFLICT (tenant_id, channel_id, message_id) DO NOTHING
"#;
pub const INSERT_RECEIPT_SQL: &str = r#"
INSERT INTO messenger_message_stream.message_receipts (
  tenant_id,
  home_cell,
  shard_key,
  jurisdiction_code,
  idempotency_key,
  channel_id,
  message_id,
  receipt_kind,
  audit_correlation_id,
  policy_decision_ref
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
)
ON CONFLICT (tenant_id, idempotency_key) DO NOTHING
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistenceTable {
    pub table_name: &'static str,
    pub distribution_column: &'static str,
    pub primary_key: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrationBundle {
    pub service_id: &'static str,
    pub migration_name: &'static str,
    pub sql: &'static str,
    pub tables: &'static [PersistenceTable],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MigrationValidationError {
    MissingCitusExtension,
    MissingRequiredColumn {
        column: &'static str,
    },
    MissingTenantRlsSetting,
    MissingTable {
        table: &'static str,
    },
    MissingDistributedTable {
        table: &'static str,
    },
    MissingRowLevelSecurity {
        table: &'static str,
    },
    MissingForceRowLevelSecurity {
        table: &'static str,
    },
    MissingPrimaryKey {
        table: &'static str,
        primary_key: &'static str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistMessageRecord {
    pub tenant: TenantSqlContext,
    pub channel_id: String,
    pub message_id: String,
    pub author_ref: String,
    pub envelope_ref: String,
    pub retention_policy_id: String,
    pub legal_hold_ids: Vec<String>,
    pub policy_decision_ref: String,
    pub idempotency_key: String,
    pub audit_correlation_id: String,
}

pub const TABLES: &[PersistenceTable] = &[
    PersistenceTable {
        table_name: "messenger_message_stream.messages",
        distribution_column: DISTRIBUTION_COLUMN,
        primary_key: "tenant_id, channel_id, message_id",
    },
    PersistenceTable {
        table_name: "messenger_message_stream.message_receipts",
        distribution_column: DISTRIBUTION_COLUMN,
        primary_key: "tenant_id, idempotency_key",
    },
    PersistenceTable {
        table_name: "messenger_message_stream.protocol_outbox_events",
        distribution_column: DISTRIBUTION_COLUMN,
        primary_key: "tenant_id, event_id",
    },
];

pub const MIGRATION_BUNDLE: MigrationBundle = MigrationBundle {
    service_id: SERVICE_ID,
    migration_name: "0001_message_stream.sql",
    sql: MIGRATION_0001,
    tables: TABLES,
};

pub fn validate_migration_bundle(bundle: &MigrationBundle) -> Result<(), MigrationValidationError> {
    validate_sql(bundle.sql, bundle.tables)
}

pub fn validate_sql(
    sql: &str,
    tables: &[PersistenceTable],
) -> Result<(), MigrationValidationError> {
    let normalized = sql.to_ascii_lowercase();
    if !normalized.contains("create extension if not exists citus") {
        return Err(MigrationValidationError::MissingCitusExtension);
    }
    for column in [
        "tenant_id",
        "home_cell",
        "shard_key",
        "jurisdiction_code",
        "audit_event_class",
    ] {
        let column_declaration = format!("{column} text not null");
        if !normalized.contains(&column_declaration) {
            return Err(MigrationValidationError::MissingRequiredColumn { column });
        }
    }
    if !normalized.contains("current_setting('oyatie.tenant_id', true)") {
        return Err(MigrationValidationError::MissingTenantRlsSetting);
    }
    for table in tables {
        validate_table(&normalized, table)?;
    }
    Ok(())
}

pub fn build_message_write_batch(
    record: &PersistMessageRecord,
) -> Result<SqlWriteBatch, SqlCommandError> {
    let message = SqlCommand::new(
        "insert_messenger_message",
        INSERT_MESSAGE_SQL,
        message_params(record)?,
    )?;
    let receipt = SqlCommand::new(
        "insert_messenger_message_receipt",
        INSERT_RECEIPT_SQL,
        receipt_params(record)?,
    )?;
    SqlWriteBatch::new(&record.tenant, vec![message, receipt])
}

fn message_params(record: &PersistMessageRecord) -> Result<Vec<SqlParam>, SqlCommandError> {
    let mut params = record.tenant.routing_params()?;
    params.push(SqlParam::text(required_field(
        &record.channel_id,
        "channel_id",
    )?));
    params.push(SqlParam::text(required_field(
        &record.message_id,
        "message_id",
    )?));
    params.push(SqlParam::text(required_field(
        &record.author_ref,
        "author_ref",
    )?));
    params.push(SqlParam::text(required_field(
        &record.envelope_ref,
        "envelope_ref",
    )?));
    params.push(SqlParam::text(required_field(
        &record.retention_policy_id,
        "retention_policy_id",
    )?));
    params.push(SqlParam::text_array(text_array_values(
        "legal_hold_ids",
        &record.legal_hold_ids,
    )?));
    params.push(SqlParam::text(required_field(
        &record.policy_decision_ref,
        "policy_decision_ref",
    )?));
    params.push(SqlParam::text(required_field(
        &record.idempotency_key,
        "idempotency_key",
    )?));
    params.push(SqlParam::text(required_field(
        &record.audit_correlation_id,
        "audit_correlation_id",
    )?));
    Ok(params)
}

fn receipt_params(record: &PersistMessageRecord) -> Result<Vec<SqlParam>, SqlCommandError> {
    let mut params = record.tenant.routing_params()?;
    params.push(SqlParam::text(required_field(
        &record.idempotency_key,
        "idempotency_key",
    )?));
    params.push(SqlParam::text(required_field(
        &record.channel_id,
        "channel_id",
    )?));
    params.push(SqlParam::text(required_field(
        &record.message_id,
        "message_id",
    )?));
    params.push(SqlParam::text("created"));
    params.push(SqlParam::text(required_field(
        &record.audit_correlation_id,
        "audit_correlation_id",
    )?));
    params.push(SqlParam::text(required_field(
        &record.policy_decision_ref,
        "policy_decision_ref",
    )?));
    Ok(params)
}

fn validate_table(sql: &str, table: &PersistenceTable) -> Result<(), MigrationValidationError> {
    let create_table = format!("create table if not exists {}", table.table_name);
    if !sql.contains(&create_table) {
        return Err(MigrationValidationError::MissingTable {
            table: table.table_name,
        });
    }
    let distributed_table = format!(
        "create_distributed_table('{}', '{}'",
        table.table_name, table.distribution_column
    );
    if !sql.contains(&distributed_table) {
        return Err(MigrationValidationError::MissingDistributedTable {
            table: table.table_name,
        });
    }
    let enable_rls = format!("alter table {} enable row level security", table.table_name);
    if !sql.contains(&enable_rls) {
        return Err(MigrationValidationError::MissingRowLevelSecurity {
            table: table.table_name,
        });
    }
    let force_rls = format!("alter table {} force row level security", table.table_name);
    if !sql.contains(&force_rls) {
        return Err(MigrationValidationError::MissingForceRowLevelSecurity {
            table: table.table_name,
        });
    }
    let primary_key = format!("primary key ({})", table.primary_key);
    if !sql.contains(&primary_key) {
        return Err(MigrationValidationError::MissingPrimaryKey {
            table: table.table_name,
            primary_key: table.primary_key,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_bundle_is_tenant_cell_shard_safe() {
        validate_migration_bundle(&MIGRATION_BUNDLE).unwrap();
    }

    #[test]
    fn every_table_is_distributed_by_tenant() {
        for table in TABLES {
            assert_eq!(table.distribution_column, DISTRIBUTION_COLUMN);
        }
    }

    #[test]
    fn validation_rejects_missing_rls_tenant_setting() {
        let broken =
            MIGRATION_0001.replace("current_setting('oyatie.tenant_id', true)", "current_user");
        assert_eq!(
            validate_sql(&broken, TABLES),
            Err(MigrationValidationError::MissingTenantRlsSetting)
        );
    }

    #[test]
    fn message_write_batch_sets_tenant_scope_before_parameterized_inserts() {
        let batch = build_message_write_batch(&record()).unwrap();

        assert_eq!(batch.tenant_scope.name, "set_local_oyatie_tenant");
        assert_eq!(
            batch.tenant_scope.params,
            vec![SqlParam::text("tenant:alpha")]
        );
        assert_eq!(batch.statements.len(), 2);
        assert!(
            batch.statements[0]
                .sql
                .contains("INSERT INTO messenger_message_stream.messages")
        );
        assert!(
            batch.statements[1]
                .sql
                .contains("INSERT INTO messenger_message_stream.message_receipts")
        );
    }

    #[test]
    fn message_write_batch_keeps_values_out_of_sql_text() {
        let batch = build_message_write_batch(&record()).unwrap();

        for command in &batch.statements {
            assert!(!command.sql.contains("tenant:alpha"));
            assert!(!command.sql.contains("msg-1"));
            assert!(!command.sql.contains("idem-1"));
        }
    }

    #[test]
    fn message_write_batch_rejects_missing_required_field() {
        let mut record = record();
        record.policy_decision_ref = " ".into();

        assert_eq!(
            build_message_write_batch(&record),
            Err(SqlCommandError::MissingField {
                field: "policy_decision_ref"
            })
        );
    }

    fn record() -> PersistMessageRecord {
        PersistMessageRecord {
            tenant: TenantSqlContext::new("tenant:alpha", "cell-us-1", "tenant:alpha#0", "US")
                .unwrap(),
            channel_id: "channel-1".into(),
            message_id: "msg-1".into(),
            author_ref: "principal:user-1".into(),
            envelope_ref: "kms://cell-us-1/message/msg-1".into(),
            retention_policy_id: "retention-7y".into(),
            legal_hold_ids: vec!["hold-1".into()],
            policy_decision_ref: "policy-decision-1".into(),
            idempotency_key: "idem-1".into(),
            audit_correlation_id: "audit-corr-1".into(),
        }
    }
}
