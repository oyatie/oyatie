//! Postgres/Citus migration bundle for social-post-composition.
//!
//! This crate is intentionally adapter-shaped but runtime-free: it publishes the
//! first tenant/cell/shard-safe SQL migration bundle plus deterministic checks
//! that can run in unit tests before a later sqlx/runtime adapter lands.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_shared_postgres_command_kernel::{
    SqlCommand, SqlCommandError, SqlParam, SqlWriteBatch, TenantSqlContext, optional_field,
    required_field, text_array_values,
};

pub const MIGRATION_0001: &str = include_str!("../migrations/0001_post_composition.sql");
pub const SERVICE_ID: &str = "social-post-composition";
pub const DISTRIBUTION_COLUMN: &str = "tenant_id";
pub const INSERT_SOCIAL_POST_SQL: &str = r#"
INSERT INTO social_post_composition.posts (
  tenant_id,
  home_cell,
  shard_key,
  jurisdiction_code,
  post_id,
  creator_ref,
  context_kind,
  artifact_kind,
  media_refs,
  workflow_consent_ref,
  policy_decision_ref,
  idempotency_key,
  audit_correlation_id,
  story_expires_at
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14::timestamptz
)
ON CONFLICT (tenant_id, post_id) DO NOTHING
"#;
pub const INSERT_STORY_PURGE_TARGET_SQL: &str = r#"
INSERT INTO social_post_composition.story_purge_targets (
  tenant_id,
  home_cell,
  shard_key,
  jurisdiction_code,
  post_id,
  purge_target,
  purge_after,
  policy_decision_ref,
  audit_correlation_id
) VALUES (
  $1, $2, $3, $4, $5, $6, $7::timestamptz, $8, $9
)
ON CONFLICT (tenant_id, post_id, purge_target) DO NOTHING
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
pub struct PersistSocialPostRecord {
    pub tenant: TenantSqlContext,
    pub post_id: String,
    pub creator_ref: String,
    pub context_kind: String,
    pub artifact_kind: String,
    pub media_refs: Vec<String>,
    pub workflow_consent_ref: Option<String>,
    pub policy_decision_ref: String,
    pub idempotency_key: String,
    pub audit_correlation_id: String,
    pub story_expires_at: Option<String>,
    pub story_purge_targets: Vec<String>,
}

pub const TABLES: &[PersistenceTable] = &[
    PersistenceTable {
        table_name: "social_post_composition.posts",
        distribution_column: DISTRIBUTION_COLUMN,
        primary_key: "tenant_id, post_id",
    },
    PersistenceTable {
        table_name: "social_post_composition.story_purge_targets",
        distribution_column: DISTRIBUTION_COLUMN,
        primary_key: "tenant_id, post_id, purge_target",
    },
    PersistenceTable {
        table_name: "social_post_composition.protocol_outbox_events",
        distribution_column: DISTRIBUTION_COLUMN,
        primary_key: "tenant_id, event_id",
    },
];

pub const MIGRATION_BUNDLE: MigrationBundle = MigrationBundle {
    service_id: SERVICE_ID,
    migration_name: "0001_post_composition.sql",
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

pub fn build_post_write_batch(
    record: &PersistSocialPostRecord,
) -> Result<SqlWriteBatch, SqlCommandError> {
    let mut statements = vec![SqlCommand::new(
        "insert_social_post",
        INSERT_SOCIAL_POST_SQL,
        post_params(record)?,
    )?];

    let purge_after = purge_after_for_targets(record)?;
    for target in &record.story_purge_targets {
        statements.push(SqlCommand::new(
            "insert_social_story_purge_target",
            INSERT_STORY_PURGE_TARGET_SQL,
            purge_target_params(record, target, &purge_after)?,
        )?);
    }

    SqlWriteBatch::new(&record.tenant, statements)
}

fn post_params(record: &PersistSocialPostRecord) -> Result<Vec<SqlParam>, SqlCommandError> {
    let mut params = record.tenant.routing_params()?;
    params.push(SqlParam::text(required_field(&record.post_id, "post_id")?));
    params.push(SqlParam::text(required_field(
        &record.creator_ref,
        "creator_ref",
    )?));
    params.push(SqlParam::text(required_field(
        &record.context_kind,
        "context_kind",
    )?));
    params.push(SqlParam::text(required_field(
        &record.artifact_kind,
        "artifact_kind",
    )?));
    params.push(SqlParam::text_array(text_array_values(
        "media_refs",
        &record.media_refs,
    )?));
    params.push(SqlParam::nullable_text(optional_field(
        &record.workflow_consent_ref,
        "workflow_consent_ref",
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
    params.push(SqlParam::nullable_text(optional_field(
        &record.story_expires_at,
        "story_expires_at",
    )?));
    Ok(params)
}

fn purge_after_for_targets(
    record: &PersistSocialPostRecord,
) -> Result<Option<String>, SqlCommandError> {
    if record.story_purge_targets.is_empty() {
        return optional_field(&record.story_expires_at, "story_expires_at");
    }
    match optional_field(&record.story_expires_at, "story_expires_at")? {
        Some(purge_after) => Ok(Some(purge_after)),
        None => Err(SqlCommandError::MissingField {
            field: "story_expires_at",
        }),
    }
}

fn purge_target_params(
    record: &PersistSocialPostRecord,
    purge_target: &str,
    purge_after: &Option<String>,
) -> Result<Vec<SqlParam>, SqlCommandError> {
    let Some(purge_after) = purge_after else {
        return Err(SqlCommandError::MissingField {
            field: "story_expires_at",
        });
    };
    let mut params = record.tenant.routing_params()?;
    params.push(SqlParam::text(required_field(&record.post_id, "post_id")?));
    params.push(SqlParam::text(required_field(
        purge_target,
        "story_purge_targets",
    )?));
    params.push(SqlParam::text(purge_after.clone()));
    params.push(SqlParam::text(required_field(
        &record.policy_decision_ref,
        "policy_decision_ref",
    )?));
    params.push(SqlParam::text(required_field(
        &record.audit_correlation_id,
        "audit_correlation_id",
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
    fn post_write_batch_sets_tenant_scope_and_story_purge_inserts() {
        let batch = build_post_write_batch(&record()).unwrap();

        assert_eq!(batch.tenant_scope.name, "set_local_oyatie_tenant");
        assert_eq!(
            batch.tenant_scope.params,
            vec![SqlParam::text("tenant:alpha")]
        );
        assert_eq!(batch.statements.len(), 3);
        assert!(
            batch.statements[0]
                .sql
                .contains("INSERT INTO social_post_composition.posts")
        );
        assert!(
            batch.statements[1]
                .sql
                .contains("INSERT INTO social_post_composition.story_purge_targets")
        );
    }

    #[test]
    fn post_write_batch_keeps_values_out_of_sql_text() {
        let batch = build_post_write_batch(&record()).unwrap();

        for command in &batch.statements {
            assert!(!command.sql.contains("tenant:alpha"));
            assert!(!command.sql.contains("post-1"));
            assert!(!command.sql.contains("idem-1"));
        }
    }

    #[test]
    fn post_write_batch_rejects_missing_story_expiry_for_purge_targets() {
        let mut record = record();
        record.story_expires_at = None;

        assert_eq!(
            build_post_write_batch(&record),
            Err(SqlCommandError::MissingField {
                field: "story_expires_at"
            })
        );
    }

    #[test]
    fn post_write_batch_rejects_blank_optional_workflow_consent() {
        let mut record = record();
        record.workflow_consent_ref = Some(" ".into());

        assert_eq!(
            build_post_write_batch(&record),
            Err(SqlCommandError::MissingField {
                field: "workflow_consent_ref"
            })
        );
    }

    fn record() -> PersistSocialPostRecord {
        PersistSocialPostRecord {
            tenant: TenantSqlContext::new("tenant:alpha", "cell-us-1", "tenant:alpha#0", "US")
                .unwrap(),
            post_id: "post-1".into(),
            creator_ref: "principal:user-1".into(),
            context_kind: "work".into(),
            artifact_kind: "story".into(),
            media_refs: vec!["media-1".into()],
            workflow_consent_ref: Some("workflow-consent-1".into()),
            policy_decision_ref: "policy-decision-1".into(),
            idempotency_key: "idem-1".into(),
            audit_correlation_id: "audit-corr-1".into(),
            story_expires_at: Some("2026-05-24T08:00:00Z".into()),
            story_purge_targets: vec!["cdn_object".into(), "search_index".into()],
        }
    }
}
