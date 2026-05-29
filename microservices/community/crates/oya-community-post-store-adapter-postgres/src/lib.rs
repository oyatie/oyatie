//! Postgres/Citus migration bundle for community-post-store.
//!
//! This crate is intentionally adapter-shaped but runtime-free: it publishes the
//! first tenant/cell/shard-safe SQL migration bundle plus deterministic checks
//! that can run in unit tests before a later sqlx/runtime adapter lands.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_shared_postgres_command_kernel::{
    SqlCommand, SqlCommandError, SqlParam, SqlWriteBatch, TenantSqlContext, optional_field,
    required_field,
};

pub const MIGRATION_0001: &str = include_str!("../migrations/0001_post_store.sql");
pub const SERVICE_ID: &str = "community-post-store";
pub const DISTRIBUTION_COLUMN: &str = "tenant_id";
pub const INSERT_COMMUNITY_POST_SQL: &str = r#"
INSERT INTO community_post_store.posts (
  tenant_id,
  home_cell,
  shard_key,
  jurisdiction_code,
  space_id,
  thread_id,
  post_id,
  mode,
  routine_display_ref,
  audit_author_ref,
  disclosure_policy_ref,
  body_ref,
  retention_policy_id,
  policy_decision_ref,
  idempotency_key,
  audit_correlation_id
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
)
ON CONFLICT (tenant_id, post_id) DO NOTHING
"#;
pub const INSERT_COMMUNITY_VOTE_SQL: &str = r#"
INSERT INTO community_post_store.votes (
  tenant_id,
  home_cell,
  shard_key,
  jurisdiction_code,
  post_id,
  vote_id,
  voter_ref,
  direction,
  policy_decision_ref,
  audit_correlation_id
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
)
ON CONFLICT (tenant_id, post_id, vote_id) DO NOTHING
"#;
pub const INSERT_COMMUNITY_MODERATION_SQL: &str = r#"
INSERT INTO community_post_store.moderation_actions (
  tenant_id,
  home_cell,
  shard_key,
  jurisdiction_code,
  post_id,
  evidence_ref,
  policy_ref,
  verb,
  policy_decision_ref,
  audit_correlation_id
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
)
ON CONFLICT (tenant_id, post_id, evidence_ref) DO NOTHING
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
pub struct PersistCommunityPostRecord {
    pub tenant: TenantSqlContext,
    pub space_id: String,
    pub thread_id: String,
    pub post_id: String,
    pub mode: String,
    pub routine_display_ref: String,
    pub audit_author_ref: String,
    pub disclosure_policy_ref: Option<String>,
    pub body_ref: String,
    pub retention_policy_id: String,
    pub policy_decision_ref: String,
    pub idempotency_key: String,
    pub audit_correlation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistCommunityVoteRecord {
    pub tenant: TenantSqlContext,
    pub post_id: String,
    pub vote_id: String,
    pub voter_ref: String,
    pub direction: String,
    pub policy_decision_ref: String,
    pub audit_correlation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistCommunityModerationRecord {
    pub tenant: TenantSqlContext,
    pub post_id: String,
    pub evidence_ref: String,
    pub policy_ref: String,
    pub verb: String,
    pub policy_decision_ref: String,
    pub audit_correlation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunityPostVoteModerationBatches {
    pub post: SqlWriteBatch,
    pub vote: SqlWriteBatch,
    pub moderation: SqlWriteBatch,
}

pub const TABLES: &[PersistenceTable] = &[
    PersistenceTable {
        table_name: "community_post_store.posts",
        distribution_column: DISTRIBUTION_COLUMN,
        primary_key: "tenant_id, post_id",
    },
    PersistenceTable {
        table_name: "community_post_store.votes",
        distribution_column: DISTRIBUTION_COLUMN,
        primary_key: "tenant_id, post_id, vote_id",
    },
    PersistenceTable {
        table_name: "community_post_store.moderation_actions",
        distribution_column: DISTRIBUTION_COLUMN,
        primary_key: "tenant_id, post_id, evidence_ref",
    },
    PersistenceTable {
        table_name: "community_post_store.protocol_outbox_events",
        distribution_column: DISTRIBUTION_COLUMN,
        primary_key: "tenant_id, event_id",
    },
];

pub const MIGRATION_BUNDLE: MigrationBundle = MigrationBundle {
    service_id: SERVICE_ID,
    migration_name: "0001_post_store.sql",
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
    record: &PersistCommunityPostRecord,
) -> Result<SqlWriteBatch, SqlCommandError> {
    let command = SqlCommand::new(
        "insert_community_post",
        INSERT_COMMUNITY_POST_SQL,
        post_params(record)?,
    )?;
    SqlWriteBatch::new(&record.tenant, vec![command])
}

pub fn build_vote_write_batch(
    record: &PersistCommunityVoteRecord,
) -> Result<SqlWriteBatch, SqlCommandError> {
    let command = SqlCommand::new(
        "insert_community_vote",
        INSERT_COMMUNITY_VOTE_SQL,
        vote_params(record)?,
    )?;
    SqlWriteBatch::new(&record.tenant, vec![command])
}

pub fn build_moderation_write_batch(
    record: &PersistCommunityModerationRecord,
) -> Result<SqlWriteBatch, SqlCommandError> {
    let command = SqlCommand::new(
        "insert_community_moderation_action",
        INSERT_COMMUNITY_MODERATION_SQL,
        moderation_params(record)?,
    )?;
    SqlWriteBatch::new(&record.tenant, vec![command])
}

pub fn build_post_vote_moderation_write_batches(
    post: &PersistCommunityPostRecord,
    vote: &PersistCommunityVoteRecord,
    moderation: &PersistCommunityModerationRecord,
) -> Result<CommunityPostVoteModerationBatches, SqlCommandError> {
    Ok(CommunityPostVoteModerationBatches {
        post: build_post_write_batch(post)?,
        vote: build_vote_write_batch(vote)?,
        moderation: build_moderation_write_batch(moderation)?,
    })
}

fn post_params(record: &PersistCommunityPostRecord) -> Result<Vec<SqlParam>, SqlCommandError> {
    let mut params = record.tenant.routing_params()?;
    params.push(SqlParam::text(required_field(
        &record.space_id,
        "space_id",
    )?));
    params.push(SqlParam::text(required_field(
        &record.thread_id,
        "thread_id",
    )?));
    params.push(SqlParam::text(required_field(&record.post_id, "post_id")?));
    params.push(SqlParam::text(required_field(&record.mode, "mode")?));
    params.push(SqlParam::text(required_field(
        &record.routine_display_ref,
        "routine_display_ref",
    )?));
    params.push(SqlParam::text(required_field(
        &record.audit_author_ref,
        "audit_author_ref",
    )?));
    params.push(SqlParam::nullable_text(optional_field(
        &record.disclosure_policy_ref,
        "disclosure_policy_ref",
    )?));
    params.push(SqlParam::text(required_field(
        &record.body_ref,
        "body_ref",
    )?));
    params.push(SqlParam::text(required_field(
        &record.retention_policy_id,
        "retention_policy_id",
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

fn vote_params(record: &PersistCommunityVoteRecord) -> Result<Vec<SqlParam>, SqlCommandError> {
    let mut params = record.tenant.routing_params()?;
    params.push(SqlParam::text(required_field(&record.post_id, "post_id")?));
    params.push(SqlParam::text(required_field(&record.vote_id, "vote_id")?));
    params.push(SqlParam::text(required_field(
        &record.voter_ref,
        "voter_ref",
    )?));
    params.push(SqlParam::text(required_field(
        &record.direction,
        "direction",
    )?));
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

fn moderation_params(
    record: &PersistCommunityModerationRecord,
) -> Result<Vec<SqlParam>, SqlCommandError> {
    let mut params = record.tenant.routing_params()?;
    params.push(SqlParam::text(required_field(&record.post_id, "post_id")?));
    params.push(SqlParam::text(required_field(
        &record.evidence_ref,
        "evidence_ref",
    )?));
    params.push(SqlParam::text(required_field(
        &record.policy_ref,
        "policy_ref",
    )?));
    params.push(SqlParam::text(required_field(&record.verb, "verb")?));
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
    fn community_write_batches_set_tenant_scope_before_parameterized_inserts() {
        let batches =
            build_post_vote_moderation_write_batches(&post(), &vote(), &moderation()).unwrap();

        assert_eq!(batches.post.tenant_scope.name, "set_local_oyatie_tenant");
        assert_eq!(
            batches.post.tenant_scope.params,
            vec![SqlParam::text("tenant:alpha")]
        );
        assert_eq!(batches.post.statements.len(), 1);
        assert_eq!(batches.vote.statements.len(), 1);
        assert_eq!(batches.moderation.statements.len(), 1);
        assert!(
            batches.post.statements[0]
                .sql
                .contains("INSERT INTO community_post_store.posts")
        );
        assert!(
            batches.vote.statements[0]
                .sql
                .contains("INSERT INTO community_post_store.votes")
        );
        assert!(
            batches.moderation.statements[0]
                .sql
                .contains("INSERT INTO community_post_store.moderation_actions")
        );
    }

    #[test]
    fn community_write_batches_keep_values_out_of_sql_text() {
        let batches =
            build_post_vote_moderation_write_batches(&post(), &vote(), &moderation()).unwrap();

        for batch in [&batches.post, &batches.vote, &batches.moderation] {
            for command in &batch.statements {
                assert!(!command.sql.contains("tenant:alpha"));
                assert!(!command.sql.contains("post-1"));
                assert!(!command.sql.contains("policy-decision-1"));
            }
        }
    }

    #[test]
    fn community_post_write_batch_rejects_blank_optional_disclosure_policy() {
        let mut post = post();
        post.disclosure_policy_ref = Some(" ".into());

        assert_eq!(
            build_post_write_batch(&post),
            Err(SqlCommandError::MissingField {
                field: "disclosure_policy_ref"
            })
        );
    }

    #[test]
    fn community_vote_write_batch_rejects_missing_voter_ref() {
        let mut vote = vote();
        vote.voter_ref = " ".into();

        assert_eq!(
            build_vote_write_batch(&vote),
            Err(SqlCommandError::MissingField { field: "voter_ref" })
        );
    }

    #[test]
    fn community_moderation_write_batch_rejects_missing_evidence_ref() {
        let mut moderation = moderation();
        moderation.evidence_ref = " ".into();

        assert_eq!(
            build_moderation_write_batch(&moderation),
            Err(SqlCommandError::MissingField {
                field: "evidence_ref"
            })
        );
    }

    fn tenant() -> TenantSqlContext {
        TenantSqlContext::new("tenant:alpha", "cell-us-1", "tenant:alpha#0", "US").unwrap()
    }

    fn post() -> PersistCommunityPostRecord {
        PersistCommunityPostRecord {
            tenant: tenant(),
            space_id: "space-1".into(),
            thread_id: "thread-1".into(),
            post_id: "post-1".into(),
            mode: "teamblind".into(),
            routine_display_ref: "routine-display-1".into(),
            audit_author_ref: "principal:user-1".into(),
            disclosure_policy_ref: Some("disclosure-policy-1".into()),
            body_ref: "body-ref-1".into(),
            retention_policy_id: "retention-7y".into(),
            policy_decision_ref: "policy-decision-1".into(),
            idempotency_key: "idem-1".into(),
            audit_correlation_id: "audit-corr-1".into(),
        }
    }

    fn vote() -> PersistCommunityVoteRecord {
        PersistCommunityVoteRecord {
            tenant: tenant(),
            post_id: "post-1".into(),
            vote_id: "vote-1".into(),
            voter_ref: "principal:voter-1".into(),
            direction: "up".into(),
            policy_decision_ref: "policy-decision-1".into(),
            audit_correlation_id: "audit-corr-1".into(),
        }
    }

    fn moderation() -> PersistCommunityModerationRecord {
        PersistCommunityModerationRecord {
            tenant: tenant(),
            post_id: "post-1".into(),
            evidence_ref: "evidence-1".into(),
            policy_ref: "moderation-policy-1".into(),
            verb: "hide".into(),
            policy_decision_ref: "policy-decision-1".into(),
            audit_correlation_id: "audit-corr-1".into(),
        }
    }
}
