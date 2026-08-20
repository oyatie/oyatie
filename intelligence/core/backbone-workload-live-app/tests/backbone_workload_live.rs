#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::{collections::BTreeMap, env, time::Duration};

use comms_mail_mailbox_api::{
    AuthorizedMailContext, DmarcApiAction, DmarcApiPolicy, DmarcCheckRequest, MailApiContext,
    MailApiEnvelope, SubmitMessageRequest, message_sent_event_envelope,
};
use comms_mail_mailbox_postgres::{PersistMailMessageRecord, build_mail_message_write_batch};
use comms_mail_mailbox_usecase::submit_message;
use comms_messenger_stream_api::{
    AuthorizedMessengerContext, MessengerApiContext, MessengerApiEnvelope, SendMessageRequest,
    message_posted_event_envelope,
};
use comms_messenger_stream_postgres::{PersistMessageRecord, build_message_write_batch};
use comms_messenger_stream_usecase::send_message;
use community_post_store_adapter_postgres::{
    PersistCommunityModerationRecord, PersistCommunityPostRecord, PersistCommunityVoteRecord,
    build_moderation_write_batch, build_post_write_batch as build_community_post_write_batch,
    build_vote_write_batch,
};
use community_post_store_api::{
    AuthorizedCommunityContext, CastVoteRequest, CommunityApiMode, CreatePostRequest,
    ModeratePostRequest, ModerationVerb, VoteDirection, moderation_actioned_event_envelope,
    post_created_event_envelope, vote_cast_event_envelope,
};
use community_post_store_domain::{CommunityPost, VoteLedger};
use community_post_store_usecase::{cast_vote, create_post, moderate_post};
use community_social_post_composition_adapter_postgres::{
    PersistSocialPostRecord, build_post_write_batch as build_social_post_write_batch,
};
use community_social_post_composition_api::{
    AuthorizedSocialContext, ComposePostRequest, SocialApiArtifactKind, SocialApiContext,
    post_published_event_envelope,
};
use community_social_post_composition_usecase::{compose_post, plan_story_purge};
use oya_shared_postgres_command_adapter_sqlx::{
    SqlxPostgresBatchExecutor, SqlxPostgresCommandError, SqlxPostgresConnectionConfig,
};
use oya_shared_postgres_command_kernel::{
    PostgresPoolConfig, SET_LOCAL_TENANT_SQL, SqlExecutionPlan, SqlWriteBatch, TenantSqlContext,
};
use oya_shared_transactional_outbox_adapter_sqlx::{
    OutboxClaimRequest, SqlxOutboxDrainConfig, SqlxOutboxDrainError, SqlxTransactionalOutboxDrain,
};
use oya_shared_transactional_outbox_kernel::{BackboneOutboxTable, append_outbox_to_batch};
use sqlx::{Executor, PgPool, Postgres, Transaction, postgres::PgPoolOptions};

const WORKLOAD_LIVE_ENABLE_ENV: &str = "OYA_BACKBONE_LIVE_WORKLOAD_POSTGRES";
const WORKLOAD_POSTGRES_DATABASE_URL_ENV: &str = "OYA_BACKBONE_WORKLOAD_POSTGRES_URL";
const WORKLOAD_POSTGRES_APP_DATABASE_URL_ENV: &str = "OYA_BACKBONE_WORKLOAD_POSTGRES_APP_URL";
const WORKLOAD_POSTGRES_REQUIRE_TLS_ENV: &str = "OYA_BACKBONE_WORKLOAD_POSTGRES_REQUIRE_TLS";
const WORKLOAD_HARNESS_APPLICATION_NAME: &str = "oyatie-live-workload-harness";
const TENANT_A: &str = "tenant:live-a";
const TENANT_B: &str = "tenant:live-b";
const WORKLOAD_SCHEMAS: &[&str] = &[
    "messenger_message_stream",
    "mail_mailbox_store",
    "social_post_composition",
    "community_post_store",
];

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkloadLiveConfig {
    database_url: String,
    app_database_url: String,
    require_tls: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum WorkloadLiveError {
    Disabled {
        enable_env: &'static str,
    },
    MissingDatabaseUrl {
        database_url_env: &'static str,
    },
    MissingAppDatabaseUrl {
        app_database_url_env: &'static str,
    },
    InvalidBooleanEnv {
        env_name: &'static str,
        value: String,
    },
    Config(SqlxPostgresCommandError),
    Outbox(String),
    Sqlx(String),
    CitusExtensionUnavailable,
    AppRoleBypassesRls {
        role_name: String,
        rolsuper: bool,
        rolbypassrls: bool,
    },
    AppRoleMatchesSetupRole {
        role_name: String,
    },
    RlsIsolationFailed {
        tenant_scope_ref: String,
        table_name: &'static str,
        expected_rows: i64,
        visible_rows: i64,
    },
    OutboxClaimCountMismatch {
        tenant_scope_ref: String,
        table_name: &'static str,
        expected_events: usize,
        claimed_events: usize,
    },
}

impl From<SqlxPostgresCommandError> for WorkloadLiveError {
    fn from(error: SqlxPostgresCommandError) -> Self {
        Self::Config(error)
    }
}

impl From<sqlx::Error> for WorkloadLiveError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error.to_string())
    }
}

impl From<SqlxOutboxDrainError> for WorkloadLiveError {
    fn from(error: SqlxOutboxDrainError) -> Self {
        Self::Outbox(format!("{error:?}"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorkloadTable {
    qualified_name: &'static str,
    expected_rows_per_tenant: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OutboxExpectation {
    table: BackboneOutboxTable,
    expected_events: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkloadTenantPlans {
    sql_execution_plans: Vec<SqlExecutionPlan>,
    outbox_claims: Vec<OutboxExpectation>,
    expected_rows: BTreeMap<&'static str, i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommunityPostSqlPlan {
    post: CommunityPost,
    sql_execution: SqlExecutionPlan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkloadLiveReport {
    migrations_applied: usize,
    service_write_plans_executed: usize,
    outbox_batches_claimed: usize,
    citus_distribution_checked: bool,
    app_role_checked: bool,
    app_role_name: String,
    tenant_a_counts: BTreeMap<String, i64>,
    tenant_b_counts: BTreeMap<String, i64>,
    no_tenant_counts: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DatabaseRoleInfo {
    name: String,
    rolsuper: bool,
    rolbypassrls: bool,
}

impl WorkloadLiveConfig {
    fn from_env() -> Result<Self, WorkloadLiveError> {
        Self::from_env_map(|name| env::var(name).ok())
    }

    fn from_env_map(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, WorkloadLiveError> {
        let enabled = lookup(WORKLOAD_LIVE_ENABLE_ENV)
            .as_deref()
            .map(|value| parse_env_bool(WORKLOAD_LIVE_ENABLE_ENV, value))
            .transpose()?
            .unwrap_or(false);
        if !enabled {
            return Err(WorkloadLiveError::Disabled {
                enable_env: WORKLOAD_LIVE_ENABLE_ENV,
            });
        }
        let database_url = lookup(WORKLOAD_POSTGRES_DATABASE_URL_ENV).ok_or(
            WorkloadLiveError::MissingDatabaseUrl {
                database_url_env: WORKLOAD_POSTGRES_DATABASE_URL_ENV,
            },
        )?;
        let app_database_url = lookup(WORKLOAD_POSTGRES_APP_DATABASE_URL_ENV).ok_or(
            WorkloadLiveError::MissingAppDatabaseUrl {
                app_database_url_env: WORKLOAD_POSTGRES_APP_DATABASE_URL_ENV,
            },
        )?;
        let require_tls = lookup(WORKLOAD_POSTGRES_REQUIRE_TLS_ENV)
            .as_deref()
            .map(|value| parse_env_bool(WORKLOAD_POSTGRES_REQUIRE_TLS_ENV, value))
            .transpose()?
            .unwrap_or(true);
        let config = Self {
            database_url,
            app_database_url,
            require_tls,
        };
        config.connection_config()?;
        config.app_connection_config()?;
        Ok(config)
    }

    fn connection_config(&self) -> Result<SqlxPostgresConnectionConfig, SqlxPostgresCommandError> {
        SqlxPostgresConnectionConfig::new(
            self.database_url.clone(),
            workload_pool_config(self.require_tls)?,
        )
    }

    fn app_connection_config(
        &self,
    ) -> Result<SqlxPostgresConnectionConfig, SqlxPostgresCommandError> {
        SqlxPostgresConnectionConfig::new(
            self.app_database_url.clone(),
            workload_pool_config(self.require_tls)?,
        )
    }
}

impl WorkloadTenantPlans {
    fn expected_rows_for(&self, table_name: &'static str) -> i64 {
        *self.expected_rows.get(table_name).unwrap_or(&0)
    }
}

#[test]
fn workload_live_config_is_env_gated_and_requires_app_url() {
    assert_eq!(
        WorkloadLiveConfig::from_env_map(|_| None),
        Err(WorkloadLiveError::Disabled {
            enable_env: WORKLOAD_LIVE_ENABLE_ENV,
        })
    );

    let missing_app = WorkloadLiveConfig::from_env_map(|name| match name {
        WORKLOAD_LIVE_ENABLE_ENV => Some("true".to_string()),
        WORKLOAD_POSTGRES_DATABASE_URL_ENV => {
            Some("postgres://setup:secret@localhost/workloads?sslmode=require".to_string())
        }
        _ => None,
    });
    assert_eq!(
        missing_app,
        Err(WorkloadLiveError::MissingAppDatabaseUrl {
            app_database_url_env: WORKLOAD_POSTGRES_APP_DATABASE_URL_ENV,
        })
    );

    let config = WorkloadLiveConfig::from_env_map(|name| match name {
        WORKLOAD_LIVE_ENABLE_ENV => Some("true".to_string()),
        WORKLOAD_POSTGRES_DATABASE_URL_ENV => {
            Some("postgres://setup:secret@localhost/workloads?sslmode=require".to_string())
        }
        WORKLOAD_POSTGRES_APP_DATABASE_URL_ENV => {
            Some("postgres://app:secret@localhost/workloads?sslmode=require".to_string())
        }
        _ => None,
    })
    .unwrap();
    assert_eq!(
        config.connection_config().unwrap().database_url,
        "postgres://setup:secret@localhost/workloads?sslmode=require"
    );
    assert_eq!(
        config.app_connection_config().unwrap().database_url,
        "postgres://app:secret@localhost/workloads?sslmode=require"
    );
}

#[test]
fn workload_live_tables_cover_four_service_migration_bundles() {
    let names: Vec<&str> = workload_tables()
        .iter()
        .map(|table| table.qualified_name)
        .collect();

    assert_eq!(names.len(), 13);
    assert!(names.contains(&"messenger_message_stream.messages"));
    assert!(names.contains(&"mail_mailbox_store.mail_messages"));
    assert!(names.contains(&"social_post_composition.story_purge_targets"));
    assert!(names.contains(&"community_post_store.moderation_actions"));
    assert!(names.contains(&"community_post_store.protocol_outbox_events"));
}

#[test]
fn workload_live_app_plans_cover_business_rows_and_outbox_rows() {
    let plans = workload_app_plans_for_tenant(TENANT_A).unwrap();

    assert_eq!(plans.sql_execution_plans.len(), 6);
    assert_eq!(plans.outbox_claims.len(), 4);
    assert_eq!(
        plans.expected_rows_for("messenger_message_stream.messages"),
        1
    );
    assert_eq!(
        plans.expected_rows_for("social_post_composition.story_purge_targets"),
        0
    );
    assert_eq!(
        plans.expected_rows_for("community_post_store.protocol_outbox_events"),
        3
    );
}

#[tokio::test]
async fn backbone_workload_live_harness_runs_when_enabled_by_environment() {
    let config = match WorkloadLiveConfig::from_env() {
        Ok(config) => config,
        Err(WorkloadLiveError::Disabled { .. }) => return,
        Err(error) => panic!("invalid live workload harness environment: {error:?}"),
    };

    let report = run_backbone_workload_live_harness(&config)
        .await
        .expect("live workload harness should pass against disposable Citus/Postgres");

    assert_eq!(report.migrations_applied, 4);
    assert_eq!(report.service_write_plans_executed, 12);
    assert_eq!(report.outbox_batches_claimed, 8);
    assert!(report.citus_distribution_checked);
    assert!(report.app_role_checked);
    assert_eq!(
        report
            .tenant_a_counts
            .get("messenger_message_stream.messages"),
        Some(&1)
    );
    assert_eq!(
        report
            .tenant_b_counts
            .get("community_post_store.protocol_outbox_events"),
        Some(&3)
    );
    assert!(
        report
            .no_tenant_counts
            .values()
            .all(|visible_rows| *visible_rows == 0)
    );
}

async fn run_backbone_workload_live_harness(
    config: &WorkloadLiveConfig,
) -> Result<WorkloadLiveReport, WorkloadLiveError> {
    let setup_pool = connect_pool(&config.connection_config()?).await?;
    let app_pool = connect_pool(&config.app_connection_config()?).await?;
    let setup_role = current_database_role(&setup_pool).await?;
    let app_role = validate_app_role(&app_pool).await?;
    ensure_distinct_roles(&setup_role, &app_role)?;
    cleanup_workload_schemas(&setup_pool).await?;

    let body_result = async {
        apply_workload_migrations(&setup_pool).await?;
        grant_workload_privileges(&setup_pool, &app_role.name).await?;
        verify_workload_distribution(&setup_pool).await?;

        let app_executor = SqlxPostgresBatchExecutor::from_pool(app_pool.clone());
        let tenant_a = workload_app_plans_for_tenant(TENANT_A)?;
        let tenant_b = workload_app_plans_for_tenant(TENANT_B)?;
        execute_workload_plans(&app_executor, &tenant_a).await?;
        execute_workload_plans(&app_executor, &tenant_b).await?;

        let tenant_a_counts = visible_counts_for(&app_pool, Some(TENANT_A), &tenant_a).await?;
        let tenant_b_counts = visible_counts_for(&app_pool, Some(TENANT_B), &tenant_b).await?;
        let no_tenant_counts = visible_counts_for(&app_pool, None, &tenant_a).await?;
        ensure_expected_counts(TENANT_A, &tenant_a_counts, &tenant_a)?;
        ensure_expected_counts(TENANT_B, &tenant_b_counts, &tenant_b)?;
        ensure_no_tenant_counts(&no_tenant_counts)?;

        let outbox_batches_claimed = claim_outbox_batches(&app_pool, TENANT_A, &tenant_a).await?
            + claim_outbox_batches(&app_pool, TENANT_B, &tenant_b).await?;

        Ok(WorkloadLiveReport {
            migrations_applied: workload_migrations().len(),
            service_write_plans_executed: tenant_a.sql_execution_plans.len()
                + tenant_b.sql_execution_plans.len(),
            outbox_batches_claimed,
            citus_distribution_checked: true,
            app_role_checked: true,
            app_role_name: app_role.name,
            tenant_a_counts,
            tenant_b_counts,
            no_tenant_counts,
        })
    }
    .await;

    let cleanup_result = cleanup_workload_schemas(&setup_pool).await;
    match (body_result, cleanup_result) {
        (Ok(report), Ok(())) => Ok(report),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

fn workload_tables() -> &'static [WorkloadTable] {
    &[
        WorkloadTable {
            qualified_name: "messenger_message_stream.messages",
            expected_rows_per_tenant: 1,
        },
        WorkloadTable {
            qualified_name: "messenger_message_stream.message_receipts",
            expected_rows_per_tenant: 1,
        },
        WorkloadTable {
            qualified_name: "messenger_message_stream.protocol_outbox_events",
            expected_rows_per_tenant: 1,
        },
        WorkloadTable {
            qualified_name: "mail_mailbox_store.mail_messages",
            expected_rows_per_tenant: 1,
        },
        WorkloadTable {
            qualified_name: "mail_mailbox_store.mail_submission_receipts",
            expected_rows_per_tenant: 1,
        },
        WorkloadTable {
            qualified_name: "mail_mailbox_store.protocol_outbox_events",
            expected_rows_per_tenant: 1,
        },
        WorkloadTable {
            qualified_name: "social_post_composition.posts",
            expected_rows_per_tenant: 1,
        },
        WorkloadTable {
            qualified_name: "social_post_composition.story_purge_targets",
            expected_rows_per_tenant: 0,
        },
        WorkloadTable {
            qualified_name: "social_post_composition.protocol_outbox_events",
            expected_rows_per_tenant: 1,
        },
        WorkloadTable {
            qualified_name: "community_post_store.posts",
            expected_rows_per_tenant: 1,
        },
        WorkloadTable {
            qualified_name: "community_post_store.votes",
            expected_rows_per_tenant: 1,
        },
        WorkloadTable {
            qualified_name: "community_post_store.moderation_actions",
            expected_rows_per_tenant: 1,
        },
        WorkloadTable {
            qualified_name: "community_post_store.protocol_outbox_events",
            expected_rows_per_tenant: 3,
        },
    ]
}

fn workload_migrations() -> &'static [&'static str] {
    &[
        comms_messenger_stream_postgres::MIGRATION_0001,
        comms_mail_mailbox_postgres::MIGRATION_0001,
        community_social_post_composition_adapter_postgres::MIGRATION_0001,
        community_post_store_adapter_postgres::MIGRATION_0001,
    ]
}

fn workload_app_plans_for_tenant(
    tenant_scope_ref: &'static str,
) -> Result<WorkloadTenantPlans, WorkloadLiveError> {
    let mut sql_execution_plans = Vec::new();
    let suffix = tenant_scope_ref.replace(':', "-");
    let tenant = tenant_context(tenant_scope_ref)?;

    let messenger = plan_send_message_sql_execution(
        tenant.clone(),
        AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: tenant_scope_ref.to_string(),
            principal_ref: format!("user:{suffix}:messenger"),
            idempotency_key: format!("idem:{suffix}:messenger"),
            policy_decision_ref: format!("policy:{suffix}:messenger"),
            audit_correlation_id: format!("audit:{suffix}:messenger"),
        },
        SendMessageRequest {
            message_id: format!("message:{suffix}"),
            channel_id: "channel:live".to_string(),
            author_ref: format!("user:{suffix}:messenger"),
            envelope: MessengerApiEnvelope::TenantDek {
                dek_ref: format!("dek:{suffix}:messenger"),
                four_eyes: true,
            },
            retention_policy_id: "retain:fd001".to_string(),
            legal_hold_ids: vec![format!("hold:{suffix}")],
        },
    )?;
    sql_execution_plans.push(messenger);

    let mail = plan_submit_message_sql_execution(
        tenant.clone(),
        AuthorizedMailContext {
            context: MailApiContext::Work,
            scope_ref: tenant_scope_ref.to_string(),
            principal_ref: format!("user:{suffix}:mail"),
            idempotency_key: format!("idem:{suffix}:mail"),
            policy_decision_ref: format!("policy:{suffix}:mail"),
            audit_correlation_id: format!("audit:{suffix}:mail"),
        },
        SubmitMessageRequest {
            message_id: format!("mail-message:{suffix}"),
            mailbox_id: "mailbox:live".to_string(),
            subject_ref: format!("user:{suffix}:mail"),
            envelope: MailApiEnvelope::TenantDek {
                dek_ref: format!("dek:{suffix}:mail"),
            },
            retention_policy_id: "retain:fd001".to_string(),
            dmarc_check: Some(DmarcCheckRequest {
                domain_ref: "domain:fd001.example".to_string(),
                spf_aligned: false,
                dkim_aligned: false,
                policy: DmarcApiPolicy::Quarantine,
                evidence_ref: format!("evidence:{suffix}:dmarc"),
            }),
        },
    )?;
    sql_execution_plans.push(mail);

    let social = plan_publish_post_sql_execution(
        tenant.clone(),
        AuthorizedSocialContext {
            context: SocialApiContext::Work,
            scope_ref: tenant_scope_ref.to_string(),
            principal_ref: format!("user:{suffix}:social"),
            idempotency_key: format!("idem:{suffix}:social"),
            policy_decision_ref: format!("policy:{suffix}:social"),
            audit_correlation_id: format!("audit:{suffix}:social"),
        },
        ComposePostRequest {
            post_id: format!("social-post:{suffix}"),
            creator_ref: format!("user:{suffix}:social"),
            kind: SocialApiArtifactKind::FeedPost,
            media_refs: vec![format!("media:{suffix}")],
            story_expires_at: None,
            collab_owner_refs: Vec::new(),
            collab_consent_refs: Vec::new(),
            workflow_consent_ref: Some(format!("workflow-consent:{suffix}:social")),
            ar_biometric_persisted: false,
        },
        None,
    )?;
    sql_execution_plans.push(social);

    let community_context = AuthorizedCommunityContext {
        tenant_scope_ref: tenant_scope_ref.to_string(),
        principal_ref: format!("user:{suffix}:community"),
        idempotency_key: format!("idem:{suffix}:community:create"),
        policy_decision_ref: format!("policy:{suffix}:community"),
        audit_correlation_id: format!("audit:{suffix}:community:create"),
    };
    let community_post = plan_create_post_sql_execution(
        tenant.clone(),
        community_context.clone(),
        "space:live",
        CreatePostRequest {
            post_id: format!("community-post:{suffix}"),
            thread_id: "thread:live".to_string(),
            mode: CommunityApiMode::Teamblind,
            routine_display_ref: format!("anon:{suffix}"),
            audit_author_ref: format!("user:{suffix}:community"),
            disclosure_policy_ref: Some("disclosure:fd001".to_string()),
            body_ref: format!("body:{suffix}"),
            retention_policy_id: "retain:fd001".to_string(),
        },
    )?;
    let mut vote_ledger = VoteLedger::new(&community_post.post);
    sql_execution_plans.push(community_post.sql_execution.clone());

    let mut vote_context = community_context.clone();
    vote_context.principal_ref = format!("user:{suffix}:voter");
    vote_context.idempotency_key = format!("idem:{suffix}:community:vote");
    vote_context.audit_correlation_id = format!("audit:{suffix}:community:vote");
    let community_vote = plan_cast_vote_sql_execution(
        tenant.clone(),
        vote_context,
        &community_post.post,
        &mut vote_ledger,
        CastVoteRequest {
            post_id: format!("community-post:{suffix}"),
            voter_ref: format!("user:{suffix}:voter"),
            direction: VoteDirection::Up,
        },
    )?;
    sql_execution_plans.push(community_vote);

    let mut moderation_context = community_context;
    moderation_context.idempotency_key = format!("idem:{suffix}:community:moderation");
    moderation_context.audit_correlation_id = format!("audit:{suffix}:community:moderation");
    let community_moderation = plan_moderation_action_sql_execution(
        tenant,
        moderation_context,
        &community_post.post,
        ModeratePostRequest {
            policy_ref: "policy:moderation".to_string(),
            evidence_ref: format!("evidence:{suffix}:moderation"),
            verb: ModerationVerb::Hide,
        },
    )?;
    sql_execution_plans.push(community_moderation);

    Ok(WorkloadTenantPlans {
        sql_execution_plans,
        outbox_claims: vec![
            OutboxExpectation {
                table: BackboneOutboxTable::MessengerMessageStream,
                expected_events: 1,
            },
            OutboxExpectation {
                table: BackboneOutboxTable::MailMailboxStore,
                expected_events: 1,
            },
            OutboxExpectation {
                table: BackboneOutboxTable::SocialPostComposition,
                expected_events: 1,
            },
            OutboxExpectation {
                table: BackboneOutboxTable::CommunityPostStore,
                expected_events: 3,
            },
        ],
        expected_rows: workload_tables()
            .iter()
            .map(|table| (table.qualified_name, table.expected_rows_per_tenant))
            .collect(),
    })
}

fn plan_send_message_sql_execution(
    tenant: TenantSqlContext,
    context: AuthorizedMessengerContext,
    request: SendMessageRequest,
) -> Result<SqlExecutionPlan, WorkloadLiveError> {
    require_scope_match(&tenant, &context.scope_ref)?;
    let outbox_tenant = tenant.clone();
    let envelope_ref = messenger_envelope_ref(&request.envelope);
    let persistence_record = PersistMessageRecord {
        tenant,
        channel_id: request.channel_id.clone(),
        message_id: request.message_id.clone(),
        author_ref: request.author_ref.clone(),
        envelope_ref,
        retention_policy_id: request.retention_policy_id.clone(),
        legal_hold_ids: request.legal_hold_ids.clone(),
        policy_decision_ref: context.policy_decision_ref.clone(),
        idempotency_key: context.idempotency_key.clone(),
        audit_correlation_id: context.audit_correlation_id.clone(),
    };
    let (_, receipt) = send_message(&context, request).map_err(plan_error)?;
    let persistence =
        build_message_write_batch(&persistence_record).map_err(SqlxPostgresCommandError::from)?;
    let protocol_event = message_posted_event_envelope(&context, &receipt).map_err(plan_error)?;
    let persistence = append_outbox_to_batch(
        BackboneOutboxTable::MessengerMessageStream,
        &outbox_tenant,
        persistence,
        &protocol_event,
    )
    .map_err(plan_error)?;
    execution_plan_for_microservice("messenger", persistence)
}

fn plan_submit_message_sql_execution(
    tenant: TenantSqlContext,
    context: AuthorizedMailContext,
    request: SubmitMessageRequest,
) -> Result<SqlExecutionPlan, WorkloadLiveError> {
    require_scope_match(&tenant, &context.scope_ref)?;
    let outbox_tenant = tenant.clone();
    let envelope_ref = mail_envelope_ref(&request.envelope);
    let mailbox_id = request.mailbox_id.clone();
    let subject_ref = request.subject_ref.clone();
    let retention_policy_id = request.retention_policy_id.clone();
    let receipt = submit_message(&context, request).map_err(plan_error)?;
    let persistence_record = PersistMailMessageRecord {
        tenant,
        mailbox_id,
        message_id: receipt.message_id.clone(),
        subject_ref,
        envelope_ref,
        retention_policy_id,
        dmarc_action: dmarc_action_name(receipt.dmarc_action).to_string(),
        policy_decision_ref: receipt.policy_decision_ref.clone(),
        idempotency_key: receipt.idempotency_key.clone(),
        audit_correlation_id: receipt.audit_correlation_id.clone(),
    };
    let persistence = build_mail_message_write_batch(&persistence_record)
        .map_err(SqlxPostgresCommandError::from)?;
    let protocol_event = message_sent_event_envelope(&context, &receipt).map_err(plan_error)?;
    let persistence = append_outbox_to_batch(
        BackboneOutboxTable::MailMailboxStore,
        &outbox_tenant,
        persistence,
        &protocol_event,
    )
    .map_err(plan_error)?;
    execution_plan_for_microservice("mail", persistence)
}

fn plan_publish_post_sql_execution(
    tenant: TenantSqlContext,
    context: AuthorizedSocialContext,
    request: ComposePostRequest,
    story_purge_now: Option<u64>,
) -> Result<SqlExecutionPlan, WorkloadLiveError> {
    require_scope_match(&tenant, &context.scope_ref)?;
    let outbox_tenant = tenant.clone();
    let record_template = SocialRecordTemplate::from(&context, &request, tenant);
    let (post, receipt) = compose_post(&context, request).map_err(plan_error)?;
    let story_purge_targets = match (record_template.story_expires_at_value, story_purge_now) {
        (Some(expires_at), Some(now)) if now >= expires_at => plan_story_purge(&post, now)
            .map_err(plan_error)?
            .purge_targets
            .into_iter()
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    };
    let persistence_record = record_template.into_record(story_purge_targets);
    let persistence = build_social_post_write_batch(&persistence_record)
        .map_err(SqlxPostgresCommandError::from)?;
    let protocol_event = post_published_event_envelope(&context, &receipt).map_err(plan_error)?;
    let persistence = append_outbox_to_batch(
        BackboneOutboxTable::SocialPostComposition,
        &outbox_tenant,
        persistence,
        &protocol_event,
    )
    .map_err(plan_error)?;
    execution_plan_for_microservice("social", persistence)
}

fn plan_create_post_sql_execution(
    tenant: TenantSqlContext,
    context: AuthorizedCommunityContext,
    space_id: impl Into<String>,
    request: CreatePostRequest,
) -> Result<CommunityPostSqlPlan, WorkloadLiveError> {
    require_scope_match(&tenant, &context.tenant_scope_ref)?;
    let space_id = space_id.into();
    let record_template = CommunityPostRecordTemplate::from(&tenant, &context, &space_id, &request);
    let outbox_tenant = record_template.tenant.clone();
    let (post, receipt) = create_post(&context, request).map_err(plan_error)?;
    let persistence_record = record_template.into_record(receipt.post_id.clone());
    let persistence = build_community_post_write_batch(&persistence_record)
        .map_err(SqlxPostgresCommandError::from)?;
    let protocol_event = post_created_event_envelope(&context, &receipt).map_err(plan_error)?;
    let persistence = append_community_outbox(&outbox_tenant, persistence, &protocol_event)?;
    let sql_execution = execution_plan_for_microservice("community", persistence)?;
    Ok(CommunityPostSqlPlan {
        post,
        sql_execution,
    })
}

fn plan_cast_vote_sql_execution(
    tenant: TenantSqlContext,
    context: AuthorizedCommunityContext,
    post: &CommunityPost,
    ledger: &mut VoteLedger,
    request: CastVoteRequest,
) -> Result<SqlExecutionPlan, WorkloadLiveError> {
    require_scope_match(&tenant, &context.tenant_scope_ref)?;
    let outbox_tenant = tenant.clone();
    let voter_ref = request.voter_ref.clone();
    let direction = vote_direction(request.direction).to_string();
    let receipt = cast_vote(&context, post, ledger, request).map_err(plan_error)?;
    let persistence_record = PersistCommunityVoteRecord {
        tenant,
        post_id: receipt.post_id.clone(),
        vote_id: receipt.vote_id.clone(),
        voter_ref,
        direction,
        policy_decision_ref: receipt.policy_decision_ref.clone(),
        audit_correlation_id: context.audit_correlation_id.clone(),
    };
    let persistence =
        build_vote_write_batch(&persistence_record).map_err(SqlxPostgresCommandError::from)?;
    let protocol_event = vote_cast_event_envelope(&context, &receipt).map_err(plan_error)?;
    let persistence = append_community_outbox(&outbox_tenant, persistence, &protocol_event)?;
    execution_plan_for_microservice("community", persistence)
}

fn plan_moderation_action_sql_execution(
    tenant: TenantSqlContext,
    context: AuthorizedCommunityContext,
    post: &CommunityPost,
    request: ModeratePostRequest,
) -> Result<SqlExecutionPlan, WorkloadLiveError> {
    require_scope_match(&tenant, &context.tenant_scope_ref)?;
    let outbox_tenant = tenant.clone();
    let policy_ref = request.policy_ref.clone();
    let evidence_ref = request.evidence_ref.clone();
    let verb = moderation_verb(request.verb).to_string();
    let receipt = moderate_post(&context, post, request).map_err(plan_error)?;
    let persistence_record = PersistCommunityModerationRecord {
        tenant,
        post_id: receipt.post_id.clone(),
        evidence_ref,
        policy_ref,
        verb,
        policy_decision_ref: receipt.policy_decision_ref.clone(),
        audit_correlation_id: context.audit_correlation_id.clone(),
    };
    let persistence = build_moderation_write_batch(&persistence_record)
        .map_err(SqlxPostgresCommandError::from)?;
    let protocol_event =
        moderation_actioned_event_envelope(&context, &receipt).map_err(plan_error)?;
    let persistence = append_community_outbox(&outbox_tenant, persistence, &protocol_event)?;
    execution_plan_for_microservice("community", persistence)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SocialRecordTemplate {
    tenant: TenantSqlContext,
    post_id: String,
    creator_ref: String,
    context_kind: String,
    artifact_kind: String,
    media_refs: Vec<String>,
    workflow_consent_ref: Option<String>,
    policy_decision_ref: String,
    idempotency_key: String,
    audit_correlation_id: String,
    story_expires_at: Option<String>,
    story_expires_at_value: Option<u64>,
}

impl SocialRecordTemplate {
    fn from(
        context: &AuthorizedSocialContext,
        request: &ComposePostRequest,
        tenant: TenantSqlContext,
    ) -> Self {
        Self {
            tenant,
            post_id: request.post_id.clone(),
            creator_ref: request.creator_ref.clone(),
            context_kind: social_context_kind(context.context).to_string(),
            artifact_kind: social_artifact_kind(request.kind).to_string(),
            media_refs: request.media_refs.clone(),
            workflow_consent_ref: request.workflow_consent_ref.clone(),
            policy_decision_ref: context.policy_decision_ref.clone(),
            idempotency_key: context.idempotency_key.clone(),
            audit_correlation_id: context.audit_correlation_id.clone(),
            story_expires_at: request.story_expires_at.map(|value| value.to_string()),
            story_expires_at_value: request.story_expires_at,
        }
    }

    fn into_record(self, story_purge_targets: Vec<String>) -> PersistSocialPostRecord {
        PersistSocialPostRecord {
            tenant: self.tenant,
            post_id: self.post_id,
            creator_ref: self.creator_ref,
            context_kind: self.context_kind,
            artifact_kind: self.artifact_kind,
            media_refs: self.media_refs,
            workflow_consent_ref: self.workflow_consent_ref,
            policy_decision_ref: self.policy_decision_ref,
            idempotency_key: self.idempotency_key,
            audit_correlation_id: self.audit_correlation_id,
            story_expires_at: self.story_expires_at,
            story_purge_targets,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommunityPostRecordTemplate {
    tenant: TenantSqlContext,
    space_id: String,
    thread_id: String,
    mode: String,
    routine_display_ref: String,
    audit_author_ref: String,
    disclosure_policy_ref: Option<String>,
    body_ref: String,
    retention_policy_id: String,
    policy_decision_ref: String,
    idempotency_key: String,
    audit_correlation_id: String,
}

impl CommunityPostRecordTemplate {
    fn from(
        tenant: &TenantSqlContext,
        context: &AuthorizedCommunityContext,
        space_id: &str,
        request: &CreatePostRequest,
    ) -> Self {
        Self {
            tenant: tenant.clone(),
            space_id: space_id.to_string(),
            thread_id: request.thread_id.clone(),
            mode: community_mode(request.mode).to_string(),
            routine_display_ref: request.routine_display_ref.clone(),
            audit_author_ref: request.audit_author_ref.clone(),
            disclosure_policy_ref: request.disclosure_policy_ref.clone(),
            body_ref: request.body_ref.clone(),
            retention_policy_id: request.retention_policy_id.clone(),
            policy_decision_ref: context.policy_decision_ref.clone(),
            idempotency_key: context.idempotency_key.clone(),
            audit_correlation_id: context.audit_correlation_id.clone(),
        }
    }

    fn into_record(self, post_id: String) -> PersistCommunityPostRecord {
        PersistCommunityPostRecord {
            tenant: self.tenant,
            space_id: self.space_id,
            thread_id: self.thread_id,
            post_id,
            mode: self.mode,
            routine_display_ref: self.routine_display_ref,
            audit_author_ref: self.audit_author_ref,
            disclosure_policy_ref: self.disclosure_policy_ref,
            body_ref: self.body_ref,
            retention_policy_id: self.retention_policy_id,
            policy_decision_ref: self.policy_decision_ref,
            idempotency_key: self.idempotency_key,
            audit_correlation_id: self.audit_correlation_id,
        }
    }
}

fn append_community_outbox(
    tenant: &TenantSqlContext,
    persistence: SqlWriteBatch,
    protocol_event: &oya_shared_protocol_parity_kernel::ProtocolEventEnvelope,
) -> Result<SqlWriteBatch, WorkloadLiveError> {
    append_outbox_to_batch(
        BackboneOutboxTable::CommunityPostStore,
        tenant,
        persistence,
        protocol_event,
    )
    .map_err(plan_error)
}

fn execution_plan_for_microservice(
    microservice: &str,
    persistence: SqlWriteBatch,
) -> Result<SqlExecutionPlan, WorkloadLiveError> {
    let pool = PostgresPoolConfig::for_microservice(microservice, 16)
        .map_err(SqlxPostgresCommandError::from)?;
    Ok(SqlExecutionPlan::from_batch(pool, persistence).map_err(SqlxPostgresCommandError::from)?)
}

fn require_scope_match(
    tenant: &TenantSqlContext,
    context_scope_ref: &str,
) -> Result<(), WorkloadLiveError> {
    if tenant.tenant_id == context_scope_ref {
        Ok(())
    } else {
        Err(WorkloadLiveError::Outbox(format!(
            "tenant scope mismatch: tenant_id={} context_scope_ref={}",
            tenant.tenant_id, context_scope_ref
        )))
    }
}

fn messenger_envelope_ref(envelope: &MessengerApiEnvelope) -> String {
    match envelope {
        MessengerApiEnvelope::PersonalE2e { envelope_ref } => envelope_ref.clone(),
        MessengerApiEnvelope::TenantDek { dek_ref, .. } => dek_ref.clone(),
        MessengerApiEnvelope::CrossOrg { local_dek_ref, .. } => local_dek_ref.clone(),
    }
}

fn mail_envelope_ref(envelope: &MailApiEnvelope) -> String {
    match envelope {
        MailApiEnvelope::PersonalClientOnly { envelope_ref } => envelope_ref.clone(),
        MailApiEnvelope::TenantDek { dek_ref } => dek_ref.clone(),
        MailApiEnvelope::Imported {
            source_hash,
            evidence_ref,
        } => format!("{source_hash}#{evidence_ref}"),
    }
}

fn dmarc_action_name(action: DmarcApiAction) -> &'static str {
    match action {
        DmarcApiAction::Accept => "accept",
        DmarcApiAction::Quarantine => "quarantine",
        DmarcApiAction::Reject => "reject",
    }
}

fn social_context_kind(context: SocialApiContext) -> &'static str {
    match context {
        SocialApiContext::Personal => "personal",
        SocialApiContext::Work => "work",
    }
}

fn social_artifact_kind(kind: SocialApiArtifactKind) -> &'static str {
    match kind {
        SocialApiArtifactKind::FeedPost => "feed_post",
        SocialApiArtifactKind::Story => "story",
        SocialApiArtifactKind::CollaborativePost => "collaborative_post",
    }
}

fn community_mode(mode: CommunityApiMode) -> &'static str {
    match mode {
        CommunityApiMode::Reddit => "reddit",
        CommunityApiMode::Teamblind => "teamblind",
        CommunityApiMode::Handshake => "handshake",
        CommunityApiMode::KnowledgeBase => "knowledge_base",
    }
}

fn vote_direction(direction: VoteDirection) -> &'static str {
    match direction {
        VoteDirection::Up => "up",
        VoteDirection::Down => "down",
        VoteDirection::Clear => "clear",
    }
}

fn moderation_verb(verb: ModerationVerb) -> &'static str {
    match verb {
        ModerationVerb::Allow => "allow",
        ModerationVerb::Hide => "hide",
        ModerationVerb::Remove => "remove",
    }
}

fn plan_error(error: impl std::fmt::Debug) -> WorkloadLiveError {
    WorkloadLiveError::Outbox(format!("{error:?}"))
}

fn tenant_context(tenant_scope_ref: &str) -> Result<TenantSqlContext, WorkloadLiveError> {
    Ok(TenantSqlContext::new(
        tenant_scope_ref,
        "cell-live",
        format!("{tenant_scope_ref}#cell-live"),
        "US",
    )
    .map_err(SqlxPostgresCommandError::Kernel)?)
}

fn workload_pool_config(require_tls: bool) -> Result<PostgresPoolConfig, SqlxPostgresCommandError> {
    PostgresPoolConfig::new(
        WORKLOAD_HARNESS_APPLICATION_NAME,
        4,
        1_000,
        5_000,
        require_tls,
    )
    .map_err(SqlxPostgresCommandError::Kernel)
}

fn parse_env_bool(env_name: &'static str, value: &str) -> Result<bool, WorkloadLiveError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(WorkloadLiveError::InvalidBooleanEnv {
            env_name,
            value: value.to_string(),
        }),
    }
}

async fn connect_pool(config: &SqlxPostgresConnectionConfig) -> Result<PgPool, WorkloadLiveError> {
    config.validate()?;
    Ok(PgPoolOptions::new()
        .max_connections(config.pool.max_connections)
        .acquire_timeout(Duration::from_millis(config.pool.acquire_timeout_ms))
        .connect(&config.database_url)
        .await?)
}

async fn apply_workload_migrations(pool: &PgPool) -> Result<(), WorkloadLiveError> {
    if !citus_extension_available(pool).await? {
        return Err(WorkloadLiveError::CitusExtensionUnavailable);
    }
    for migration in workload_migrations() {
        for statement in migration_statements(migration) {
            pool.execute(statement).await?;
        }
    }
    Ok(())
}

fn migration_statements(migration: &'static str) -> impl Iterator<Item = &'static str> {
    migration.split(';').filter_map(|statement| {
        let trimmed = statement.trim();
        if trimmed.is_empty()
            || trimmed.eq_ignore_ascii_case("BEGIN")
            || trimmed.eq_ignore_ascii_case("COMMIT")
        {
            None
        } else {
            Some(trimmed)
        }
    })
}

async fn citus_extension_available(pool: &PgPool) -> Result<bool, WorkloadLiveError> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM pg_available_extensions WHERE name = 'citus')",
    )
    .fetch_one(pool)
    .await?)
}

async fn verify_workload_distribution(pool: &PgPool) -> Result<(), WorkloadLiveError> {
    for table in workload_tables() {
        let distributed = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM pg_dist_partition WHERE logicalrelid = $1::regclass)",
        )
        .bind(table.qualified_name)
        .fetch_one(pool)
        .await?;
        if !distributed {
            return Err(WorkloadLiveError::CitusExtensionUnavailable);
        }
    }
    Ok(())
}

async fn cleanup_workload_schemas(pool: &PgPool) -> Result<(), WorkloadLiveError> {
    for schema in WORKLOAD_SCHEMAS {
        let sql = format!("DROP SCHEMA IF EXISTS {} CASCADE", quote_identifier(schema));
        sqlx::query(&sql).execute(pool).await?;
    }
    Ok(())
}

async fn grant_workload_privileges(
    pool: &PgPool,
    app_role_name: &str,
) -> Result<(), WorkloadLiveError> {
    let role = quote_identifier(app_role_name);
    for schema in WORKLOAD_SCHEMAS {
        let schema = quote_identifier(schema);
        for sql in [
            format!("GRANT USAGE ON SCHEMA {schema} TO {role}"),
            format!("GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA {schema} TO {role}"),
        ] {
            sqlx::query(&sql).execute(pool).await?;
        }
    }
    Ok(())
}

async fn execute_workload_plans(
    executor: &SqlxPostgresBatchExecutor,
    plans: &WorkloadTenantPlans,
) -> Result<(), WorkloadLiveError> {
    for plan in &plans.sql_execution_plans {
        executor.execute_batch(plan).await?;
    }
    Ok(())
}

async fn visible_counts_for(
    pool: &PgPool,
    tenant_scope_ref: Option<&str>,
    plans: &WorkloadTenantPlans,
) -> Result<BTreeMap<String, i64>, WorkloadLiveError> {
    let mut counts = BTreeMap::new();
    for table in workload_tables() {
        let visible_rows =
            visible_count_for_table(pool, tenant_scope_ref, table.qualified_name).await?;
        counts.insert(table.qualified_name.to_string(), visible_rows);
    }
    for table_name in plans.expected_rows.keys() {
        counts.entry((*table_name).to_string()).or_insert(0);
    }
    Ok(counts)
}

async fn visible_count_for_table(
    pool: &PgPool,
    tenant_scope_ref: Option<&str>,
    table_name: &'static str,
) -> Result<i64, WorkloadLiveError> {
    let mut transaction = pool.begin().await?;
    if let Some(tenant_scope_ref) = tenant_scope_ref {
        set_tenant_scope_for_rls(&mut transaction, tenant_scope_ref).await?;
    }
    let sql = format!("SELECT count(*)::bigint FROM {table_name}");
    let visible_rows = sqlx::query_scalar::<_, i64>(&sql)
        .fetch_one(&mut *transaction)
        .await?;
    transaction.rollback().await?;
    Ok(visible_rows)
}

fn ensure_expected_counts(
    tenant_scope_ref: &str,
    counts: &BTreeMap<String, i64>,
    plans: &WorkloadTenantPlans,
) -> Result<(), WorkloadLiveError> {
    for table in workload_tables() {
        let visible_rows = *counts.get(table.qualified_name).unwrap_or(&0);
        let expected_rows = plans.expected_rows_for(table.qualified_name);
        if visible_rows != expected_rows {
            return Err(WorkloadLiveError::RlsIsolationFailed {
                tenant_scope_ref: tenant_scope_ref.to_string(),
                table_name: table.qualified_name,
                expected_rows,
                visible_rows,
            });
        }
    }
    Ok(())
}

fn ensure_no_tenant_counts(counts: &BTreeMap<String, i64>) -> Result<(), WorkloadLiveError> {
    for table in workload_tables() {
        let visible_rows = *counts.get(table.qualified_name).unwrap_or(&0);
        if visible_rows != 0 {
            return Err(WorkloadLiveError::RlsIsolationFailed {
                tenant_scope_ref: "tenant:none".to_string(),
                table_name: table.qualified_name,
                expected_rows: 0,
                visible_rows,
            });
        }
    }
    Ok(())
}

async fn claim_outbox_batches(
    pool: &PgPool,
    tenant_scope_ref: &str,
    plans: &WorkloadTenantPlans,
) -> Result<usize, WorkloadLiveError> {
    let config =
        SqlxOutboxDrainConfig::new(format!("workload-live-harness:{tenant_scope_ref}"), 10)?;
    let drain = SqlxTransactionalOutboxDrain::from_pool(pool.clone(), config.clone());
    let mut claimed_batches = 0usize;
    for expectation in &plans.outbox_claims {
        let request = OutboxClaimRequest::new(
            expectation.table,
            tenant_scope_ref,
            expectation.expected_events as u16,
            &config,
        )?;
        let batch = drain.claim_pending_batch(request).await?;
        if batch.claimed_count != expectation.expected_events {
            return Err(WorkloadLiveError::OutboxClaimCountMismatch {
                tenant_scope_ref: tenant_scope_ref.to_string(),
                table_name: expectation.table.table_name(),
                expected_events: expectation.expected_events,
                claimed_events: batch.claimed_count,
            });
        }
        claimed_batches += 1;
    }
    Ok(claimed_batches)
}

async fn current_database_role(pool: &PgPool) -> Result<DatabaseRoleInfo, WorkloadLiveError> {
    let (name, rolsuper, rolbypassrls) = sqlx::query_as::<_, (String, bool, bool)>(
        "SELECT current_user::text, rolsuper, rolbypassrls FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(pool)
    .await?;
    Ok(DatabaseRoleInfo {
        name,
        rolsuper,
        rolbypassrls,
    })
}

async fn validate_app_role(pool: &PgPool) -> Result<DatabaseRoleInfo, WorkloadLiveError> {
    let role = current_database_role(pool).await?;
    if role.rolsuper || role.rolbypassrls {
        return Err(WorkloadLiveError::AppRoleBypassesRls {
            role_name: role.name,
            rolsuper: role.rolsuper,
            rolbypassrls: role.rolbypassrls,
        });
    }
    Ok(role)
}

fn ensure_distinct_roles(
    setup_role: &DatabaseRoleInfo,
    app_role: &DatabaseRoleInfo,
) -> Result<(), WorkloadLiveError> {
    if setup_role.name == app_role.name {
        return Err(WorkloadLiveError::AppRoleMatchesSetupRole {
            role_name: app_role.name.clone(),
        });
    }
    Ok(())
}

async fn set_tenant_scope_for_rls(
    transaction: &mut Transaction<'_, Postgres>,
    tenant_scope_ref: &str,
) -> Result<(), WorkloadLiveError> {
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind(tenant_scope_ref)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}
