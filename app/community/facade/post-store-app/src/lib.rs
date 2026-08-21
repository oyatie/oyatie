//! App-layer write orchestration for community post-store write paths.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use community_post_store_api::CreatePostRequest;
use community_post_store_api::{
    AuthorizedCommunityContext, CastVoteRequest, CommunityApiMode, ModeratePostRequest,
    ModerationReceipt, ModerationVerb, PostReceipt, VoteDirection, VoteReceiptEnvelope,
    moderation_actioned_event_envelope, post_created_event_envelope, vote_cast_event_envelope,
};
use community_post_store_domain::{CommunityPost, VoteLedger};
use community_post_store_postgres::{
    PersistCommunityModerationRecord, PersistCommunityPostRecord, PersistCommunityVoteRecord,
    build_moderation_write_batch, build_post_write_batch, build_vote_write_batch,
};
use community_post_store_usecase::{CommunityUsecaseError, cast_vote, create_post, moderate_post};
use oya_shared_postgres_command_kernel::{
    PostgresPoolConfig, SqlCommandError, SqlExecutionPlan, SqlWriteBatch, TenantSqlContext,
};
use oya_shared_protocol_parity_kernel::{ProtocolEventEnvelope, ProtocolParityError};
use oya_shared_protocol_transport_kernel::{
    ProtocolTransportBudget, ProtocolTransportBundle, ProtocolTransportError,
    plan_transport_from_envelope,
};
use oya_shared_transactional_outbox_kernel::{
    BackboneOutboxTable, TransactionalOutboxError, append_outbox_to_batch,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunityAppError {
    ScopeMismatch {
        // data_class: INTERNAL_ONLY
        tenant_scope_ref: String, // data_class: INTERNAL_ONLY
        tenant_id: String,        // data_class: INTERNAL_ONLY
    },
    Usecase(CommunityUsecaseError),
    Sql(SqlCommandError),
    Protocol(ProtocolParityError),
    Transport(ProtocolTransportError),
    Outbox(TransactionalOutboxError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunityPostPlan {
    pub post: CommunityPost,                   // data_class: INTERNAL_ONLY
    pub receipt: PostReceipt,                  // data_class: INTERNAL_ONLY
    pub persistence: SqlWriteBatch,            // data_class: INTERNAL_ONLY
    pub sql_execution: SqlExecutionPlan,       // data_class: INTERNAL_ONLY
    pub protocol_event: ProtocolEventEnvelope, // data_class: INTERNAL_ONLY
    pub transport: ProtocolTransportBundle,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunityVotePlan {
    pub receipt: VoteReceiptEnvelope,    // data_class: INTERNAL_ONLY
    pub persistence: SqlWriteBatch,      // data_class: INTERNAL_ONLY
    pub sql_execution: SqlExecutionPlan, // data_class: INTERNAL_ONLY
    pub protocol_event: ProtocolEventEnvelope, // data_class: INTERNAL_ONLY
    pub transport: ProtocolTransportBundle, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunityModerationPlan {
    pub receipt: ModerationReceipt,            // data_class: INTERNAL_ONLY
    pub persistence: SqlWriteBatch,            // data_class: INTERNAL_ONLY
    pub sql_execution: SqlExecutionPlan,       // data_class: INTERNAL_ONLY
    pub protocol_event: ProtocolEventEnvelope, // data_class: INTERNAL_ONLY
    pub transport: ProtocolTransportBundle,    // data_class: INTERNAL_ONLY
}

pub fn plan_create_post(
    tenant: TenantSqlContext,
    context: AuthorizedCommunityContext,
    space_id: impl Into<String>,
    request: CreatePostRequest,
) -> Result<CommunityPostPlan, CommunityAppError> {
    require_scope_match(&tenant, &context.tenant_scope_ref)?;
    let space_id = space_id.into();
    let record_template = CommunityPostRecordTemplate::from(&tenant, &context, &space_id, &request);
    let outbox_tenant = record_template.tenant.clone();
    let (post, receipt) = create_post(&context, request).map_err(CommunityAppError::Usecase)?;
    let persistence_record = record_template.into_record(receipt.post_id.clone());
    let persistence =
        build_post_write_batch(&persistence_record).map_err(CommunityAppError::Sql)?;
    let protocol_event =
        post_created_event_envelope(&context, &receipt).map_err(CommunityAppError::Protocol)?;
    let persistence = append_community_outbox(&outbox_tenant, persistence, &protocol_event)?;
    let sql_execution = community_sql_execution(persistence.clone())?;
    let transport =
        plan_transport_from_envelope(&protocol_event, ProtocolTransportBudget::default())
            .map_err(CommunityAppError::Transport)?;

    Ok(CommunityPostPlan {
        post,
        receipt,
        persistence,
        sql_execution,
        protocol_event,
        transport,
    })
}

pub fn plan_cast_vote(
    tenant: TenantSqlContext,
    context: AuthorizedCommunityContext,
    post: &CommunityPost,
    ledger: &mut VoteLedger,
    request: CastVoteRequest,
) -> Result<CommunityVotePlan, CommunityAppError> {
    require_scope_match(&tenant, &context.tenant_scope_ref)?;
    let outbox_tenant = tenant.clone();
    let voter_ref = request.voter_ref.clone();
    let direction = vote_direction(request.direction).to_string();
    let receipt = cast_vote(&context, post, ledger, request).map_err(CommunityAppError::Usecase)?;
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
        build_vote_write_batch(&persistence_record).map_err(CommunityAppError::Sql)?;
    let protocol_event =
        vote_cast_event_envelope(&context, &receipt).map_err(CommunityAppError::Protocol)?;
    let persistence = append_community_outbox(&outbox_tenant, persistence, &protocol_event)?;
    let sql_execution = community_sql_execution(persistence.clone())?;
    let transport =
        plan_transport_from_envelope(&protocol_event, ProtocolTransportBudget::default())
            .map_err(CommunityAppError::Transport)?;

    Ok(CommunityVotePlan {
        receipt,
        persistence,
        sql_execution,
        protocol_event,
        transport,
    })
}

pub fn plan_moderation_action(
    tenant: TenantSqlContext,
    context: AuthorizedCommunityContext,
    post: &CommunityPost,
    request: ModeratePostRequest,
) -> Result<CommunityModerationPlan, CommunityAppError> {
    require_scope_match(&tenant, &context.tenant_scope_ref)?;
    let outbox_tenant = tenant.clone();
    let policy_ref = request.policy_ref.clone();
    let evidence_ref = request.evidence_ref.clone();
    let verb = moderation_verb(request.verb).to_string();
    let receipt = moderate_post(&context, post, request).map_err(CommunityAppError::Usecase)?;
    let persistence_record = PersistCommunityModerationRecord {
        tenant,
        post_id: receipt.post_id.clone(),
        evidence_ref,
        policy_ref,
        verb,
        policy_decision_ref: receipt.policy_decision_ref.clone(),
        audit_correlation_id: context.audit_correlation_id.clone(),
    };
    let persistence =
        build_moderation_write_batch(&persistence_record).map_err(CommunityAppError::Sql)?;
    let protocol_event = moderation_actioned_event_envelope(&context, &receipt)
        .map_err(CommunityAppError::Protocol)?;
    let persistence = append_community_outbox(&outbox_tenant, persistence, &protocol_event)?;
    let sql_execution = community_sql_execution(persistence.clone())?;
    let transport =
        plan_transport_from_envelope(&protocol_event, ProtocolTransportBudget::default())
            .map_err(CommunityAppError::Transport)?;

    Ok(CommunityModerationPlan {
        receipt,
        persistence,
        sql_execution,
        protocol_event,
        transport,
    })
}

fn community_sql_execution(
    persistence: SqlWriteBatch,
) -> Result<SqlExecutionPlan, CommunityAppError> {
    SqlExecutionPlan::from_batch(
        PostgresPoolConfig::for_microservice("community", 16).map_err(CommunityAppError::Sql)?,
        persistence,
    )
    .map_err(CommunityAppError::Sql)
}

fn append_community_outbox(
    tenant: &TenantSqlContext,
    persistence: SqlWriteBatch,
    protocol_event: &ProtocolEventEnvelope,
) -> Result<SqlWriteBatch, CommunityAppError> {
    append_outbox_to_batch(
        BackboneOutboxTable::CommunityPostStore,
        tenant,
        persistence,
        protocol_event,
    )
    .map_err(CommunityAppError::Outbox)
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

fn require_scope_match(
    tenant: &TenantSqlContext,
    tenant_scope_ref: &str,
) -> Result<(), CommunityAppError> {
    if tenant.tenant_id == tenant_scope_ref {
        Ok(())
    } else {
        Err(CommunityAppError::ScopeMismatch {
            tenant_scope_ref: tenant_scope_ref.to_string(),
            tenant_id: tenant.tenant_id.clone(),
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> TenantSqlContext {
        TenantSqlContext::new("tenant:t", "cell-a", "tenant:t#cell-a", "US").unwrap()
    }

    fn context() -> AuthorizedCommunityContext {
        AuthorizedCommunityContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    fn post_request() -> CreatePostRequest {
        CreatePostRequest {
            post_id: "post:p".into(),
            thread_id: "thread:t".into(),
            mode: CommunityApiMode::Teamblind,
            routine_display_ref: "anon".into(),
            audit_author_ref: "user:u".into(),
            disclosure_policy_ref: Some("disclosure".into()),
            body_ref: "body:b".into(),
            retention_policy_id: "retain".into(),
        }
    }

    #[test]
    fn create_post_plan_composes_receipt_persistence_and_protocol_event() {
        let plan = plan_create_post(tenant(), context(), "space:s", post_request()).unwrap();

        assert_eq!(plan.receipt.event_type, "community.post.created");
        assert_eq!(plan.persistence.statements.len(), 2);
        assert_eq!(plan.sql_execution.total_command_count, 3);
        assert_eq!(plan.sql_execution.pool.application_name, "oyatie-community");
        assert_eq!(plan.protocol_event.binding.proto_rpc, "CreatePost");
        assert_eq!(
            plan.transport.broker_publish.event_kind,
            "oya.community.post.created.v1"
        );
        assert_eq!(plan.transport.grpc_unary.rpc, "CreatePost");
        assert_eq!(
            plan.persistence.statements[1].name,
            "insert_transactional_outbox_event"
        );
        assert!(
            plan.persistence.statements[1]
                .sql
                .contains("community_post_store.protocol_outbox_events")
        );
        assert_eq!(plan.post.post_id.value, "post:p");
    }

    #[test]
    fn vote_and_moderation_plans_compose_persistence_and_protocol_events() {
        let created = plan_create_post(tenant(), context(), "space:s", post_request()).unwrap();
        let mut vote_context = context();
        vote_context.principal_ref = "user:voter".into();
        vote_context.idempotency_key = "vote:v".into();
        let mut ledger = VoteLedger::new(&created.post);

        let vote = plan_cast_vote(
            tenant(),
            vote_context,
            &created.post,
            &mut ledger,
            CastVoteRequest {
                post_id: "post:p".into(),
                voter_ref: "user:voter".into(),
                direction: VoteDirection::Up,
            },
        )
        .unwrap();
        assert_eq!(vote.receipt.event_type, "community.vote.cast");
        assert_eq!(vote.persistence.statements[0].name, "insert_community_vote");
        assert_eq!(vote.persistence.statements.len(), 2);
        assert_eq!(vote.sql_execution.total_command_count, 3);
        assert_eq!(vote.protocol_event.binding.proto_rpc, "CastVote");
        assert_eq!(
            vote.transport.broker_publish.channel_address,
            "community.vote.cast"
        );
        assert_eq!(vote.transport.grpc_unary.rpc, "CastVote");
        assert_eq!(
            vote.persistence.statements[1].name,
            "insert_transactional_outbox_event"
        );

        let moderation = plan_moderation_action(
            tenant(),
            context(),
            &created.post,
            ModeratePostRequest {
                policy_ref: "policy:moderation".into(),
                evidence_ref: "evidence:e".into(),
                verb: ModerationVerb::Hide,
            },
        )
        .unwrap();
        assert_eq!(
            moderation.receipt.event_type,
            "community.moderation.actioned"
        );
        assert_eq!(
            moderation.persistence.statements[0].name,
            "insert_community_moderation_action"
        );
        assert_eq!(moderation.persistence.statements.len(), 2);
        assert_eq!(moderation.sql_execution.total_command_count, 3);
        assert_eq!(moderation.protocol_event.binding.proto_rpc, "ApplyAction");
        assert_eq!(
            moderation.transport.broker_publish.message_name,
            "ModerationActioned"
        );
        assert_eq!(moderation.transport.grpc_unary.rpc, "ApplyAction");
        assert_eq!(
            moderation.persistence.statements[1].name,
            "insert_transactional_outbox_event"
        );
    }

    #[test]
    fn create_post_plan_rejects_scope_drift_before_building_sql() {
        let mut tenant = tenant();
        tenant.tenant_id = "tenant:other".into();

        assert_eq!(
            plan_create_post(tenant, context(), "space:s", post_request()),
            Err(CommunityAppError::ScopeMismatch {
                tenant_scope_ref: "tenant:t".into(),
                tenant_id: "tenant:other".into()
            })
        );
    }
}
