//! Live tonic transport adapter seam for generated backbone write RPCs.
//!
//! This crate is the first socket-capable wrapper over the source-controlled
//! proto contracts and generated write-plan adapters. It binds tonic server
//! traits to existing in-process write-plan construction and exposes generated
//! clients over `tonic::transport::Channel`. It intentionally performs no SQL
//! execution, broker publish, gateway deployment, TLS/mTLS provisioning, or
//! production service supervision.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::future::Future;
use std::sync::{Arc, Mutex};

use comms_mail_mailbox_api::DmarcApiAction;
use community_post_store_domain::{CommunityPost, VoteLedger};
use shared_backbone_grpc_generated_adapter::{
    GeneratedBackboneGrpcAdapterError, community, community_apply_action_generated_write_plan,
    community_cast_vote_generated_write_plan, community_create_post_generated_write_plan, mail,
    mail_send_message_generated_write_plan, messenger, messenger_post_message_generated_write_plan,
    social, social_publish_post_generated_write_plan,
};
use shared_postgres_command_kernel::TenantSqlContext;
use tonic::transport::{Channel, Endpoint, Server, server::TcpIncoming};
use tonic::{Request, Response, Status};

pub type TransportResult<T> = Result<T, tonic::transport::Error>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackboneGrpcTenantConfig {
    pub messenger: TenantSqlContext, // data_class: INTERNAL_ONLY
    pub mail: TenantSqlContext,      // data_class: INTERNAL_ONLY
    pub social: TenantSqlContext,    // data_class: INTERNAL_ONLY
    pub community: TenantSqlContext, // data_class: INTERNAL_ONLY
}

#[derive(Clone)]
pub struct BackboneGrpcServiceSet {
    pub messenger: MessengerWritePlanTonicService,
    pub mail: MailWritePlanTonicService,
    pub social: SocialWritePlanTonicService,
    pub community: CommunityWritePlanTonicService,
}

impl BackboneGrpcServiceSet {
    pub fn from_tenants(config: BackboneGrpcTenantConfig) -> Self {
        Self {
            messenger: MessengerWritePlanTonicService::new(config.messenger),
            mail: MailWritePlanTonicService::new(config.mail),
            social: SocialWritePlanTonicService::new(config.social),
            community: CommunityWritePlanTonicService::new(config.community),
        }
    }
}

#[derive(Clone)]
pub struct MessengerWritePlanTonicService {
    tenant: TenantSqlContext,
}

impl MessengerWritePlanTonicService {
    pub fn new(tenant: TenantSqlContext) -> Self {
        Self { tenant }
    }
}

#[tonic::async_trait]
impl messenger::v1::message_stream_server::MessageStream for MessengerWritePlanTonicService {
    async fn post_message(
        &self,
        request: Request<messenger::v1::PostMessageRequest>,
    ) -> Result<Response<messenger::v1::PostMessageResponse>, Status> {
        let response =
            messenger_post_message_generated_write_plan(self.tenant.clone(), request.into_inner())
                .map_err(adapter_status)?;
        let receipt = response.body.receipt;
        Ok(Response::new(messenger::v1::PostMessageResponse {
            message_id: receipt.message_id,
            event_type: receipt.event_type.to_string(),
            audit_correlation_id: receipt.audit_correlation_id,
            idempotency_key: receipt.idempotency_key,
            policy_decision_ref: receipt.policy_decision_ref,
        }))
    }
}

#[derive(Clone)]
pub struct MailWritePlanTonicService {
    tenant: TenantSqlContext,
}

impl MailWritePlanTonicService {
    pub fn new(tenant: TenantSqlContext) -> Self {
        Self { tenant }
    }
}

#[tonic::async_trait]
impl mail::v1::mail_server::Mail for MailWritePlanTonicService {
    async fn send_message(
        &self,
        request: Request<mail::v1::SendMessageRequest>,
    ) -> Result<Response<mail::v1::SendMessageResponse>, Status> {
        let response =
            mail_send_message_generated_write_plan(self.tenant.clone(), request.into_inner())
                .map_err(adapter_status)?;
        let receipt = response.body.receipt;
        Ok(Response::new(mail::v1::SendMessageResponse {
            message_id: receipt.message_id,
            event_type: receipt.event_type.to_string(),
            audit_correlation_id: receipt.audit_correlation_id,
            idempotency_key: receipt.idempotency_key,
            policy_decision_ref: receipt.policy_decision_ref,
            dmarc_action: dmarc_action(receipt.dmarc_action),
        }))
    }
}

#[derive(Clone)]
pub struct SocialWritePlanTonicService {
    tenant: TenantSqlContext,
}

impl SocialWritePlanTonicService {
    pub fn new(tenant: TenantSqlContext) -> Self {
        Self { tenant }
    }
}

#[tonic::async_trait]
impl social::v1::post_composition_server::PostComposition for SocialWritePlanTonicService {
    async fn publish_post(
        &self,
        request: Request<social::v1::PublishPostRequest>,
    ) -> Result<Response<social::v1::PublishPostResponse>, Status> {
        let response =
            social_publish_post_generated_write_plan(self.tenant.clone(), request.into_inner())
                .map_err(adapter_status)?;
        let plan = response.body;
        Ok(Response::new(social::v1::PublishPostResponse {
            post_id: plan.receipt.post_id,
            event_type: plan.receipt.event_type.to_string(),
            audit_correlation_id: plan.receipt.audit_correlation_id,
            idempotency_key: plan.receipt.idempotency_key,
            policy_decision_ref: plan.receipt.policy_decision_ref,
            story_purge_targets: plan.story_purge_targets,
        }))
    }
}

#[derive(Clone)]
pub struct CommunityWritePlanTonicService {
    tenant: TenantSqlContext,
    state: Arc<Mutex<CommunityState>>,
}

#[derive(Default)]
struct CommunityState {
    post: Option<CommunityPost>, // data_class: INTERNAL_ONLY
    ledger: Option<VoteLedger>,  // data_class: INTERNAL_ONLY
}

impl CommunityWritePlanTonicService {
    pub fn new(tenant: TenantSqlContext) -> Self {
        Self {
            tenant,
            state: Arc::new(Mutex::new(CommunityState::default())),
        }
    }
}

#[tonic::async_trait]
impl community::v1::post_store_service_server::PostStoreService for CommunityWritePlanTonicService {
    async fn create_post(
        &self,
        request: Request<community::v1::CreatePostRequest>,
    ) -> Result<Response<community::v1::CreatePostResponse>, Status> {
        let response =
            community_create_post_generated_write_plan(self.tenant.clone(), request.into_inner())
                .map_err(adapter_status)?;
        let plan = response.body;
        let mut state = lock_community_state(&self.state)?;
        state.ledger = Some(VoteLedger::new(&plan.post));
        state.post = Some(plan.post.clone());
        Ok(Response::new(community::v1::CreatePostResponse {
            post_id: plan.receipt.post_id,
            event_type: plan.receipt.event_type.to_string(),
            audit_correlation_id: plan.receipt.audit_correlation_id,
            idempotency_key: plan.receipt.idempotency_key,
            policy_decision_ref: plan.receipt.policy_decision_ref,
        }))
    }
}

#[tonic::async_trait]
impl community::v1::voting_engine_service_server::VotingEngineService
    for CommunityWritePlanTonicService
{
    async fn cast_vote(
        &self,
        request: Request<community::v1::CastVoteRequest>,
    ) -> Result<Response<community::v1::CastVoteResponse>, Status> {
        let mut state = lock_community_state(&self.state)?;
        let post = state.post.clone().ok_or_else(|| {
            Status::failed_precondition("community post must be loaded before voting")
        })?;
        let ledger = state.ledger.get_or_insert_with(|| VoteLedger::new(&post));
        let response = community_cast_vote_generated_write_plan(
            self.tenant.clone(),
            &post,
            ledger,
            request.into_inner(),
        )
        .map_err(adapter_status)?;
        let receipt = response.body.receipt;
        Ok(Response::new(community::v1::CastVoteResponse {
            post_id: receipt.post_id,
            vote_id: receipt.vote_id,
            event_type: receipt.event_type.to_string(),
            audit_correlation_id: response.body.protocol_event.audit_correlation_id,
            idempotency_key: response
                .body
                .protocol_event
                .idempotency_key
                .unwrap_or_default(),
            policy_decision_ref: receipt.policy_decision_ref,
        }))
    }
}

#[tonic::async_trait]
impl community::v1::moderation_queue_service_server::ModerationQueueService
    for CommunityWritePlanTonicService
{
    async fn apply_action(
        &self,
        request: Request<community::v1::ApplyActionRequest>,
    ) -> Result<Response<community::v1::ApplyActionResponse>, Status> {
        let state = lock_community_state(&self.state)?;
        let post = state.post.as_ref().ok_or_else(|| {
            Status::failed_precondition("community post must be loaded before moderation")
        })?;
        let response = community_apply_action_generated_write_plan(
            self.tenant.clone(),
            post,
            request.into_inner(),
        )
        .map_err(adapter_status)?;
        let receipt = response.body.receipt;
        Ok(Response::new(community::v1::ApplyActionResponse {
            post_id: receipt.post_id,
            event_type: receipt.event_type.to_string(),
            evidence_ref: receipt.evidence_ref,
            policy_decision_ref: receipt.policy_decision_ref,
        }))
    }
}

pub async fn serve_backbone_grpc_until_shutdown<F>(
    incoming: TcpIncoming,
    services: BackboneGrpcServiceSet,
    shutdown: F,
) -> TransportResult<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    Server::builder()
        .add_service(
            messenger::v1::message_stream_server::MessageStreamServer::new(services.messenger),
        )
        .add_service(mail::v1::mail_server::MailServer::new(services.mail))
        .add_service(
            social::v1::post_composition_server::PostCompositionServer::new(services.social),
        )
        .add_service(
            community::v1::post_store_service_server::PostStoreServiceServer::new(
                services.community.clone(),
            ),
        )
        .add_service(
            community::v1::voting_engine_service_server::VotingEngineServiceServer::new(
                services.community.clone(),
            ),
        )
        .add_service(
            community::v1::moderation_queue_service_server::ModerationQueueServiceServer::new(
                services.community,
            ),
        )
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await
}

pub async fn connect_channel(endpoint: Endpoint) -> TransportResult<Channel> {
    endpoint.connect().await
}

pub fn endpoint_from_shared(uri: impl Into<String>) -> TransportResult<Endpoint> {
    Endpoint::from_shared(uri.into())
}

pub fn messenger_client(
    channel: Channel,
) -> messenger::v1::message_stream_client::MessageStreamClient<Channel> {
    messenger::v1::message_stream_client::MessageStreamClient::new(channel)
}

pub fn mail_client(channel: Channel) -> mail::v1::mail_client::MailClient<Channel> {
    mail::v1::mail_client::MailClient::new(channel)
}

pub fn social_client(
    channel: Channel,
) -> social::v1::post_composition_client::PostCompositionClient<Channel> {
    social::v1::post_composition_client::PostCompositionClient::new(channel)
}

pub fn community_post_client(
    channel: Channel,
) -> community::v1::post_store_service_client::PostStoreServiceClient<Channel> {
    community::v1::post_store_service_client::PostStoreServiceClient::new(channel)
}

pub fn community_vote_client(
    channel: Channel,
) -> community::v1::voting_engine_service_client::VotingEngineServiceClient<Channel> {
    community::v1::voting_engine_service_client::VotingEngineServiceClient::new(channel)
}

pub fn community_moderation_client(
    channel: Channel,
) -> community::v1::moderation_queue_service_client::ModerationQueueServiceClient<Channel> {
    community::v1::moderation_queue_service_client::ModerationQueueServiceClient::new(channel)
}

fn lock_community_state(
    state: &Mutex<CommunityState>,
) -> Result<std::sync::MutexGuard<'_, CommunityState>, Status> {
    state
        .lock()
        .map_err(|_| Status::internal("community adapter state lock poisoned"))
}

fn adapter_status(error: GeneratedBackboneGrpcAdapterError) -> Status {
    match error {
        GeneratedBackboneGrpcAdapterError::MissingMessage { request, field } => {
            Status::invalid_argument(format!("{request}.{field} is required"))
        }
        GeneratedBackboneGrpcAdapterError::InvalidEnum {
            request,
            field,
            value,
        } => Status::invalid_argument(format!(
            "{request}.{field} contains unsupported enum value {value}"
        )),
        GeneratedBackboneGrpcAdapterError::Messenger(error) => {
            Status::failed_precondition(format!("messenger write-plan rejected request: {error:?}"))
        }
        GeneratedBackboneGrpcAdapterError::Mail(error) => {
            Status::failed_precondition(format!("mail write-plan rejected request: {error:?}"))
        }
        GeneratedBackboneGrpcAdapterError::Social(error) => {
            Status::failed_precondition(format!("social write-plan rejected request: {error:?}"))
        }
        GeneratedBackboneGrpcAdapterError::Community(error) => {
            Status::failed_precondition(format!("community write-plan rejected request: {error:?}"))
        }
    }
}

fn dmarc_action(action: DmarcApiAction) -> i32 {
    match action {
        DmarcApiAction::Accept => mail::v1::DmarcAction::Accept as i32,
        DmarcApiAction::Quarantine => mail::v1::DmarcAction::Quarantine as i32,
        DmarcApiAction::Reject => mail::v1::DmarcAction::Reject as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_backbone_grpc_generated_adapter::community::v1::voting_engine_service_server::VotingEngineService;
    use shared_backbone_grpc_generated_adapter::messenger::v1::message_stream_server::MessageStream;
    use tonic::Code;

    #[tokio::test]
    async fn loopback_clients_reach_all_backbone_write_services_over_tcp() {
        let incoming = TcpIncoming::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let addr = incoming.local_addr().unwrap();
        let services = BackboneGrpcServiceSet::from_tenants(tenant_config());
        let server = tokio::spawn(async move {
            serve_backbone_grpc_until_shutdown(incoming, services, std::future::pending()).await
        });
        let channel = connect_channel(endpoint_from_shared(format!("http://{addr}")).unwrap())
            .await
            .unwrap();

        let messenger = messenger_client(channel.clone())
            .post_message(messenger_request())
            .await
            .unwrap()
            .into_inner();
        let mail = mail_client(channel.clone())
            .send_message(mail_request())
            .await
            .unwrap()
            .into_inner();
        let social = social_client(channel.clone())
            .publish_post(social_request())
            .await
            .unwrap()
            .into_inner();
        let created = community_post_client(channel.clone())
            .create_post(community_create_request())
            .await
            .unwrap()
            .into_inner();
        let vote = community_vote_client(channel.clone())
            .cast_vote(community_vote_request())
            .await
            .unwrap()
            .into_inner();
        let moderation = community_moderation_client(channel)
            .apply_action(community_moderation_request())
            .await
            .unwrap()
            .into_inner();

        assert_eq!(messenger.event_type, "messenger.message.sent");
        assert_eq!(mail.dmarc_action, mail::v1::DmarcAction::Accept as i32);
        assert_eq!(
            social.story_purge_targets,
            vec!["cdn_object", "search_index", "ontology_node"]
        );
        assert_eq!(created.event_type, "community.post.created");
        assert_eq!(vote.vote_id, "idem-vote");
        assert_eq!(moderation.event_type, "community.moderation.actioned");

        server.abort();
    }

    #[tokio::test]
    async fn service_maps_generated_adapter_rejections_to_invalid_argument_status() {
        let service = MessengerWritePlanTonicService::new(tenant("tenant:t"));
        let err = service
            .post_message(Request::new(messenger::v1::PostMessageRequest {
                envelope: None,
                ..messenger_request()
            }))
            .await
            .unwrap_err();

        assert_eq!(err.code(), Code::InvalidArgument);
        assert!(err.message().contains("PostMessageRequest.envelope"));
    }

    #[tokio::test]
    async fn community_vote_requires_create_post_state_before_socket_handler_executes() {
        let service = CommunityWritePlanTonicService::new(tenant("tenant:t"));
        let err = service
            .cast_vote(Request::new(community_vote_request()))
            .await
            .unwrap_err();

        assert_eq!(err.code(), Code::FailedPrecondition);
        assert!(err.message().contains("post must be loaded"));
    }

    fn tenant_config() -> BackboneGrpcTenantConfig {
        BackboneGrpcTenantConfig {
            messenger: tenant("tenant:t"),
            mail: tenant("tenant:t"),
            social: tenant("person:p"),
            community: tenant("tenant:t"),
        }
    }

    fn tenant(scope_ref: &str) -> TenantSqlContext {
        TenantSqlContext::new(scope_ref, "cell-a", format!("{scope_ref}#cell-a"), "US").unwrap()
    }

    fn messenger_request() -> messenger::v1::PostMessageRequest {
        messenger::v1::PostMessageRequest {
            context: messenger::v1::MessengerContextKind::Work as i32,
            scope_ref: "tenant:t".into(),
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem-message".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
            message_id: "message:m".into(),
            channel_id: "channel:c".into(),
            author_ref: "user:u".into(),
            envelope: Some(messenger::v1::MessengerEnvelope {
                envelope: Some(messenger::v1::messenger_envelope::Envelope::TenantDek(
                    messenger::v1::TenantDekEnvelope {
                        dek_ref: "dek:d".into(),
                        four_eyes: true,
                    },
                )),
            }),
            retention_policy_id: "retain".into(),
            legal_hold_ids: vec![],
        }
    }

    fn mail_request() -> mail::v1::SendMessageRequest {
        mail::v1::SendMessageRequest {
            context: mail::v1::MailContextKind::Work as i32,
            scope_ref: "tenant:t".into(),
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem-mail".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
            message_id: "message:mail".into(),
            mailbox_id: "mailbox:b".into(),
            subject_ref: "user:u".into(),
            envelope: Some(mail::v1::MailEnvelope {
                envelope: Some(mail::v1::mail_envelope::Envelope::TenantDek(
                    mail::v1::MailTenantDekEnvelope {
                        dek_ref: "dek:d".into(),
                    },
                )),
            }),
            retention_policy_id: "retain".into(),
            dmarc_check: Some(mail::v1::DmarcCheck {
                domain_ref: "domain:d".into(),
                spf_aligned: true,
                dkim_aligned: true,
                policy: mail::v1::DmarcPolicy::Reject as i32,
                evidence_ref: "evidence:e".into(),
            }),
        }
    }

    fn social_request() -> social::v1::PublishPostRequest {
        social::v1::PublishPostRequest {
            context: social::v1::SocialContextKind::Personal as i32,
            scope_ref: "person:p".into(),
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem-social".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
            post_id: "post:social".into(),
            creator_ref: "user:u".into(),
            kind: social::v1::SocialArtifactKind::Story as i32,
            media_refs: vec!["media:m".into()],
            story_expires_at: 10,
            collab_owner_refs: vec![],
            collab_consent_refs: vec![],
            workflow_consent_ref: String::new(),
            ar_biometric_persisted: false,
            story_purge_now: 11,
        }
    }

    fn community_context(
        principal_ref: &str,
        idempotency_key: &str,
    ) -> community::v1::CommunityAuthContext {
        community::v1::CommunityAuthContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: principal_ref.into(),
            idempotency_key: idempotency_key.into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    fn community_create_request() -> community::v1::CreatePostRequest {
        community::v1::CreatePostRequest {
            context: Some(community_context("user:u", "idem-create")),
            space_id: "space:s".into(),
            post_id: "post:p".into(),
            thread_id: "thread:t".into(),
            mode: community::v1::CommunityMode::Teamblind as i32,
            routine_display_ref: "anon".into(),
            audit_author_ref: "user:u".into(),
            disclosure_policy_ref: "disclosure".into(),
            body_ref: "body:b".into(),
            retention_policy_id: "retain".into(),
        }
    }

    fn community_vote_request() -> community::v1::CastVoteRequest {
        community::v1::CastVoteRequest {
            context: Some(community_context("user:voter", "idem-vote")),
            post_id: "post:p".into(),
            voter_ref: "user:voter".into(),
            direction: community::v1::VoteDirection::Up as i32,
        }
    }

    fn community_moderation_request() -> community::v1::ApplyActionRequest {
        community::v1::ApplyActionRequest {
            context: Some(community_context("user:u", "idem-moderation")),
            post_id: "post:p".into(),
            policy_ref: "policy:moderation".into(),
            evidence_ref: "evidence:e".into(),
            verb: community::v1::ModerationVerb::Hide as i32,
        }
    }
}
