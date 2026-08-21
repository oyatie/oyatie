//! Compile-time generated tonic/prost bindings for backbone write-service protos.
//!
//! This adapter crate compiles the source-controlled proto3 contracts into Rust
//! message and service bindings during `build.rs`. It provides generated types
//! for future runtime adapters while intentionally avoiding live gRPC server
//! startup, client connections, broker I/O, or database calls.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_shared_backbone_proto_contracts_kernel::{
    BackboneProtoContractError, validate_all_backbone_proto_contracts,
};
use oya_shared_postgres_command_kernel::TenantSqlContext;

pub mod messenger {
    pub mod v1 {
        tonic::include_proto!("oya.messenger.v1");
    }
}

pub mod mail {
    pub mod v1 {
        tonic::include_proto!("oya.mail.v1");
    }
}

pub mod social {
    pub mod v1 {
        tonic::include_proto!("oya.social.v1");
    }
}

pub mod community {
    pub mod v1 {
        tonic::include_proto!("oya.community.v1");
    }
}

pub const GENERATED_BACKBONE_PROTO_PACKAGES: &[&str] = &[
    "oya.messenger.v1",
    "oya.mail.v1",
    "oya.social.v1",
    "oya.community.v1",
];

pub const GENERATED_BACKBONE_GRPC_METHODS: &[&str] = &[
    "/oya.messenger.v1.MessageStream/PostMessage",
    "/oya.mail.v1.Mail/SendMessage",
    "/oya.social.v1.PostComposition/PublishPost",
    "/oya.community.v1.PostStoreService/CreatePost",
    "/oya.community.v1.VotingEngineService/CastVote",
    "/oya.community.v1.ModerationQueueService/ApplyAction",
];

pub fn validate_generated_backbone_contracts() -> Result<(), BackboneProtoContractError> {
    validate_all_backbone_proto_contracts()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeneratedBackboneGrpcAdapterError {
    MissingMessage {
        request: &'static str,
        field: &'static str,
    },
    InvalidEnum {
        request: &'static str,
        field: &'static str,
        value: i32,
    },
    Messenger(comms_messenger_stream_grpc::MessengerGrpcError),
    Mail(comms_mail_mailbox_grpc::MailGrpcError),
    Social(community_social_post_composition_grpc::SocialGrpcError),
    Community(community_post_store_grpc::CommunityGrpcError),
}

pub fn messenger_post_message_generated_write_plan(
    tenant: TenantSqlContext,
    request: messenger::v1::PostMessageRequest,
) -> Result<
    comms_messenger_stream_grpc::GrpcResponse<comms_messenger_stream_app::MessengerWritePlan>,
    GeneratedBackboneGrpcAdapterError,
> {
    let context = comms_messenger_stream_api::AuthorizedMessengerContext {
        context: messenger_context(request.context)?,
        scope_ref: request.scope_ref,
        principal_ref: request.principal_ref,
        idempotency_key: request.idempotency_key,
        policy_decision_ref: request.policy_decision_ref,
        audit_correlation_id: request.audit_correlation_id,
    };
    let envelope = request
        .envelope
        .and_then(|envelope| envelope.envelope)
        .ok_or(GeneratedBackboneGrpcAdapterError::MissingMessage {
            request: "PostMessageRequest",
            field: "envelope",
        })?;
    let request = comms_messenger_stream_api::SendMessageRequest {
        message_id: request.message_id,
        channel_id: request.channel_id,
        author_ref: request.author_ref,
        envelope: match envelope {
            messenger::v1::messenger_envelope::Envelope::PersonalE2eEnvelopeRef(envelope_ref) => {
                comms_messenger_stream_api::MessengerApiEnvelope::PersonalE2e { envelope_ref }
            }
            messenger::v1::messenger_envelope::Envelope::TenantDek(tenant_dek) => {
                comms_messenger_stream_api::MessengerApiEnvelope::TenantDek {
                    dek_ref: tenant_dek.dek_ref,
                    four_eyes: tenant_dek.four_eyes,
                }
            }
            messenger::v1::messenger_envelope::Envelope::CrossOrg(cross_org) => {
                comms_messenger_stream_api::MessengerApiEnvelope::CrossOrg {
                    local_dek_ref: cross_org.local_dek_ref,
                    partner_scope_ref: cross_org.partner_scope_ref,
                    partner_dek_ref: cross_org.partner_dek_ref,
                    partner_ediscovery_allowed: cross_org.partner_ediscovery_allowed,
                }
            }
        },
        retention_policy_id: request.retention_policy_id,
        legal_hold_ids: request.legal_hold_ids,
    };

    comms_messenger_stream_grpc::post_message_write_plan(tenant, context, request)
        .map_err(GeneratedBackboneGrpcAdapterError::Messenger)
}

pub fn mail_send_message_generated_write_plan(
    tenant: TenantSqlContext,
    request: mail::v1::SendMessageRequest,
) -> Result<
    comms_mail_mailbox_grpc::GrpcResponse<comms_mail_mailbox_app::MailSubmissionPlan>,
    GeneratedBackboneGrpcAdapterError,
> {
    let context = comms_mail_mailbox_api::AuthorizedMailContext {
        context: mail_context(request.context)?,
        scope_ref: request.scope_ref,
        principal_ref: request.principal_ref,
        idempotency_key: request.idempotency_key,
        policy_decision_ref: request.policy_decision_ref,
        audit_correlation_id: request.audit_correlation_id,
    };
    let envelope = request
        .envelope
        .and_then(|envelope| envelope.envelope)
        .ok_or(GeneratedBackboneGrpcAdapterError::MissingMessage {
            request: "SendMessageRequest",
            field: "envelope",
        })?;
    let request = comms_mail_mailbox_api::SubmitMessageRequest {
        message_id: request.message_id,
        mailbox_id: request.mailbox_id,
        subject_ref: request.subject_ref,
        envelope: match envelope {
            mail::v1::mail_envelope::Envelope::PersonalClientOnlyEnvelopeRef(envelope_ref) => {
                comms_mail_mailbox_api::MailApiEnvelope::PersonalClientOnly { envelope_ref }
            }
            mail::v1::mail_envelope::Envelope::TenantDek(tenant_dek) => {
                comms_mail_mailbox_api::MailApiEnvelope::TenantDek {
                    dek_ref: tenant_dek.dek_ref,
                }
            }
            mail::v1::mail_envelope::Envelope::Imported(imported) => {
                comms_mail_mailbox_api::MailApiEnvelope::Imported {
                    source_hash: imported.source_hash,
                    evidence_ref: imported.evidence_ref,
                }
            }
        },
        retention_policy_id: request.retention_policy_id,
        dmarc_check: request.dmarc_check.map(mail_dmarc_check).transpose()?,
    };

    comms_mail_mailbox_grpc::send_message_write_plan(tenant, context, request)
        .map_err(GeneratedBackboneGrpcAdapterError::Mail)
}

pub fn social_publish_post_generated_write_plan(
    tenant: TenantSqlContext,
    request: social::v1::PublishPostRequest,
) -> Result<
    community_social_post_composition_grpc::GrpcResponse<
        community_social_app::SocialPublishPlan,
    >,
    GeneratedBackboneGrpcAdapterError,
> {
    let context = community_social_post_composition_api::AuthorizedSocialContext {
        context: social_context(request.context)?,
        scope_ref: request.scope_ref,
        principal_ref: request.principal_ref,
        idempotency_key: request.idempotency_key,
        policy_decision_ref: request.policy_decision_ref,
        audit_correlation_id: request.audit_correlation_id,
    };
    let story_purge_now = nonzero_u64(request.story_purge_now);
    let request = community_social_post_composition_api::ComposePostRequest {
        post_id: request.post_id,
        creator_ref: request.creator_ref,
        kind: social_artifact_kind(request.kind)?,
        media_refs: request.media_refs,
        story_expires_at: nonzero_u64(request.story_expires_at),
        collab_owner_refs: request.collab_owner_refs,
        collab_consent_refs: request.collab_consent_refs,
        workflow_consent_ref: non_empty_string(request.workflow_consent_ref),
        ar_biometric_persisted: request.ar_biometric_persisted,
    };

    community_social_post_composition_grpc::publish_post_write_plan(
        tenant,
        context,
        request,
        story_purge_now,
    )
    .map_err(GeneratedBackboneGrpcAdapterError::Social)
}

pub fn community_create_post_generated_write_plan(
    tenant: TenantSqlContext,
    request: community::v1::CreatePostRequest,
) -> Result<
    community_post_store_grpc::GrpcResponse<community_post_store_app::CommunityPostPlan>,
    GeneratedBackboneGrpcAdapterError,
> {
    let context = community_context(request.context, "CreatePostRequest")?;
    let request_body = community_post_store_api::CreatePostRequest {
        post_id: request.post_id,
        thread_id: request.thread_id,
        mode: community_mode(request.mode)?,
        routine_display_ref: request.routine_display_ref,
        audit_author_ref: request.audit_author_ref,
        disclosure_policy_ref: non_empty_string(request.disclosure_policy_ref),
        body_ref: request.body_ref,
        retention_policy_id: request.retention_policy_id,
    };

    community_post_store_grpc::create_post_write_plan(
        tenant,
        context,
        request.space_id,
        request_body,
    )
    .map_err(GeneratedBackboneGrpcAdapterError::Community)
}

pub fn community_cast_vote_generated_write_plan(
    tenant: TenantSqlContext,
    post: &community_post_store_domain::CommunityPost,
    ledger: &mut community_post_store_domain::VoteLedger,
    request: community::v1::CastVoteRequest,
) -> Result<
    community_post_store_grpc::GrpcResponse<community_post_store_app::CommunityVotePlan>,
    GeneratedBackboneGrpcAdapterError,
> {
    let context = community_context(request.context, "CastVoteRequest")?;
    let request = community_post_store_api::CastVoteRequest {
        post_id: request.post_id,
        voter_ref: request.voter_ref,
        direction: vote_direction(request.direction)?,
    };

    community_post_store_grpc::cast_vote_write_plan(tenant, context, post, ledger, request)
        .map_err(GeneratedBackboneGrpcAdapterError::Community)
}

pub fn community_apply_action_generated_write_plan(
    tenant: TenantSqlContext,
    post: &community_post_store_domain::CommunityPost,
    request: community::v1::ApplyActionRequest,
) -> Result<
    community_post_store_grpc::GrpcResponse<
        community_post_store_app::CommunityModerationPlan,
    >,
    GeneratedBackboneGrpcAdapterError,
> {
    let context = community_context(request.context, "ApplyActionRequest")?;
    let request = community_post_store_api::ModeratePostRequest {
        policy_ref: request.policy_ref,
        evidence_ref: request.evidence_ref,
        verb: moderation_verb(request.verb)?,
    };

    community_post_store_grpc::apply_moderation_action_write_plan(
        tenant, context, post, request,
    )
    .map_err(GeneratedBackboneGrpcAdapterError::Community)
}

fn messenger_context(
    value: i32,
) -> Result<comms_messenger_stream_api::MessengerApiContext, GeneratedBackboneGrpcAdapterError> {
    if value == messenger::v1::MessengerContextKind::Personal as i32 {
        Ok(comms_messenger_stream_api::MessengerApiContext::Personal)
    } else if value == messenger::v1::MessengerContextKind::Work as i32 {
        Ok(comms_messenger_stream_api::MessengerApiContext::Work)
    } else {
        Err(GeneratedBackboneGrpcAdapterError::InvalidEnum {
            request: "PostMessageRequest",
            field: "context",
            value,
        })
    }
}

fn mail_context(
    value: i32,
) -> Result<comms_mail_mailbox_api::MailApiContext, GeneratedBackboneGrpcAdapterError> {
    if value == mail::v1::MailContextKind::Personal as i32 {
        Ok(comms_mail_mailbox_api::MailApiContext::Personal)
    } else if value == mail::v1::MailContextKind::Work as i32 {
        Ok(comms_mail_mailbox_api::MailApiContext::Work)
    } else {
        Err(GeneratedBackboneGrpcAdapterError::InvalidEnum {
            request: "SendMessageRequest",
            field: "context",
            value,
        })
    }
}

fn social_context(
    value: i32,
) -> Result<
    community_social_post_composition_api::SocialApiContext,
    GeneratedBackboneGrpcAdapterError,
> {
    if value == social::v1::SocialContextKind::Personal as i32 {
        Ok(community_social_post_composition_api::SocialApiContext::Personal)
    } else if value == social::v1::SocialContextKind::Work as i32 {
        Ok(community_social_post_composition_api::SocialApiContext::Work)
    } else {
        Err(GeneratedBackboneGrpcAdapterError::InvalidEnum {
            request: "PublishPostRequest",
            field: "context",
            value,
        })
    }
}

fn social_artifact_kind(
    value: i32,
) -> Result<
    community_social_post_composition_api::SocialApiArtifactKind,
    GeneratedBackboneGrpcAdapterError,
> {
    if value == social::v1::SocialArtifactKind::FeedPost as i32 {
        Ok(community_social_post_composition_api::SocialApiArtifactKind::FeedPost)
    } else if value == social::v1::SocialArtifactKind::Story as i32 {
        Ok(community_social_post_composition_api::SocialApiArtifactKind::Story)
    } else if value == social::v1::SocialArtifactKind::CollaborativePost as i32 {
        Ok(community_social_post_composition_api::SocialApiArtifactKind::CollaborativePost)
    } else {
        Err(GeneratedBackboneGrpcAdapterError::InvalidEnum {
            request: "PublishPostRequest",
            field: "kind",
            value,
        })
    }
}

fn community_context(
    context: Option<community::v1::CommunityAuthContext>,
    request: &'static str,
) -> Result<
    community_post_store_api::AuthorizedCommunityContext,
    GeneratedBackboneGrpcAdapterError,
> {
    let context = context.ok_or(GeneratedBackboneGrpcAdapterError::MissingMessage {
        request,
        field: "context",
    })?;
    Ok(community_post_store_api::AuthorizedCommunityContext {
        tenant_scope_ref: context.tenant_scope_ref,
        principal_ref: context.principal_ref,
        idempotency_key: context.idempotency_key,
        policy_decision_ref: context.policy_decision_ref,
        audit_correlation_id: context.audit_correlation_id,
    })
}

fn community_mode(
    value: i32,
) -> Result<community_post_store_api::CommunityApiMode, GeneratedBackboneGrpcAdapterError> {
    if value == community::v1::CommunityMode::Reddit as i32 {
        Ok(community_post_store_api::CommunityApiMode::Reddit)
    } else if value == community::v1::CommunityMode::Teamblind as i32 {
        Ok(community_post_store_api::CommunityApiMode::Teamblind)
    } else if value == community::v1::CommunityMode::Handshake as i32 {
        Ok(community_post_store_api::CommunityApiMode::Handshake)
    } else if value == community::v1::CommunityMode::KnowledgeBase as i32 {
        Ok(community_post_store_api::CommunityApiMode::KnowledgeBase)
    } else {
        Err(GeneratedBackboneGrpcAdapterError::InvalidEnum {
            request: "CreatePostRequest",
            field: "mode",
            value,
        })
    }
}

fn vote_direction(
    value: i32,
) -> Result<community_post_store_api::VoteDirection, GeneratedBackboneGrpcAdapterError> {
    if value == community::v1::VoteDirection::Up as i32 {
        Ok(community_post_store_api::VoteDirection::Up)
    } else if value == community::v1::VoteDirection::Down as i32 {
        Ok(community_post_store_api::VoteDirection::Down)
    } else if value == community::v1::VoteDirection::Clear as i32 {
        Ok(community_post_store_api::VoteDirection::Clear)
    } else {
        Err(GeneratedBackboneGrpcAdapterError::InvalidEnum {
            request: "CastVoteRequest",
            field: "direction",
            value,
        })
    }
}

fn moderation_verb(
    value: i32,
) -> Result<community_post_store_api::ModerationVerb, GeneratedBackboneGrpcAdapterError> {
    if value == community::v1::ModerationVerb::Allow as i32 {
        Ok(community_post_store_api::ModerationVerb::Allow)
    } else if value == community::v1::ModerationVerb::Hide as i32 {
        Ok(community_post_store_api::ModerationVerb::Hide)
    } else if value == community::v1::ModerationVerb::Remove as i32 {
        Ok(community_post_store_api::ModerationVerb::Remove)
    } else {
        Err(GeneratedBackboneGrpcAdapterError::InvalidEnum {
            request: "ApplyActionRequest",
            field: "verb",
            value,
        })
    }
}

fn mail_dmarc_check(
    check: mail::v1::DmarcCheck,
) -> Result<comms_mail_mailbox_api::DmarcCheckRequest, GeneratedBackboneGrpcAdapterError> {
    Ok(comms_mail_mailbox_api::DmarcCheckRequest {
        domain_ref: check.domain_ref,
        spf_aligned: check.spf_aligned,
        dkim_aligned: check.dkim_aligned,
        policy: if check.policy == mail::v1::DmarcPolicy::None as i32 {
            comms_mail_mailbox_api::DmarcApiPolicy::None
        } else if check.policy == mail::v1::DmarcPolicy::Quarantine as i32 {
            comms_mail_mailbox_api::DmarcApiPolicy::Quarantine
        } else if check.policy == mail::v1::DmarcPolicy::Reject as i32 {
            comms_mail_mailbox_api::DmarcApiPolicy::Reject
        } else {
            return Err(GeneratedBackboneGrpcAdapterError::InvalidEnum {
                request: "DmarcCheck",
                field: "policy",
                value: check.policy,
            });
        },
        evidence_ref: check.evidence_ref,
    })
}

fn nonzero_u64(value: u64) -> Option<u64> {
    if value == 0 { None } else { Some(value) }
}

fn non_empty_string(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn generated_registry_preserves_all_backbone_packages_and_methods() {
        validate_generated_backbone_contracts().unwrap();

        assert_eq!(
            GENERATED_BACKBONE_PROTO_PACKAGES,
            &[
                "oya.messenger.v1",
                "oya.mail.v1",
                "oya.social.v1",
                "oya.community.v1",
            ]
        );
        assert_eq!(GENERATED_BACKBONE_GRPC_METHODS.len(), 6);
        assert!(
            GENERATED_BACKBONE_GRPC_METHODS
                .contains(&"/oya.community.v1.ModerationQueueService/ApplyAction")
        );
    }

    #[test]
    fn generated_messenger_request_round_trips_with_prost() {
        let request = messenger::v1::PostMessageRequest {
            context: messenger::v1::MessengerContextKind::Work as i32,
            scope_ref: "tenant:t".into(),
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
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
            legal_hold_ids: vec!["hold:h".into()],
        };

        let mut bytes = Vec::new();
        request.encode(&mut bytes).unwrap();
        let decoded = messenger::v1::PostMessageRequest::decode(bytes.as_slice()).unwrap();

        assert_eq!(decoded.message_id, "message:m");
        assert_eq!(decoded.tenant_scope_ref, "tenant:t");
        assert_eq!(decoded.legal_hold_ids, vec!["hold:h"]);
        assert!(matches!(
            decoded.envelope.and_then(|envelope| envelope.envelope),
            Some(messenger::v1::messenger_envelope::Envelope::TenantDek(_))
        ));
    }

    #[test]
    fn generated_mail_request_round_trips_with_dmarc_fields() {
        let request = mail::v1::SendMessageRequest {
            context: mail::v1::MailContextKind::Work as i32,
            scope_ref: "tenant:t".into(),
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
            message_id: "message:m".into(),
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
        };

        let encoded = request.encode_to_vec();
        let decoded = mail::v1::SendMessageRequest::decode(encoded.as_slice()).unwrap();

        assert_eq!(decoded.mailbox_id, "mailbox:b");
        assert_eq!(
            decoded.dmarc_check.unwrap().policy,
            mail::v1::DmarcPolicy::Reject as i32
        );
    }

    #[test]
    fn generated_social_request_round_trips_with_story_and_collab_fields() {
        let request = social::v1::PublishPostRequest {
            context: social::v1::SocialContextKind::Personal as i32,
            scope_ref: "person:p".into(),
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
            post_id: "post:p".into(),
            creator_ref: "user:u".into(),
            kind: social::v1::SocialArtifactKind::Story as i32,
            media_refs: vec!["media:m".into()],
            story_expires_at: 10,
            collab_owner_refs: vec!["user:o".into()],
            collab_consent_refs: vec!["consent:c".into()],
            workflow_consent_ref: "workflow:w".into(),
            ar_biometric_persisted: false,
            story_purge_now: 11,
        };

        let encoded = request.encode_to_vec();
        let decoded = social::v1::PublishPostRequest::decode(encoded.as_slice()).unwrap();

        assert_eq!(decoded.post_id, "post:p");
        assert_eq!(decoded.media_refs, vec!["media:m"]);
        assert_eq!(decoded.story_purge_now, 11);
        assert!(!decoded.ar_biometric_persisted);
    }

    #[test]
    fn generated_community_requests_round_trip_all_write_rpcs() {
        let context = community::v1::CommunityAuthContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        };
        let create = community::v1::CreatePostRequest {
            context: Some(context.clone()),
            space_id: "space:s".into(),
            post_id: "post:p".into(),
            thread_id: "thread:t".into(),
            mode: community::v1::CommunityMode::Teamblind as i32,
            routine_display_ref: "anon".into(),
            audit_author_ref: "user:u".into(),
            disclosure_policy_ref: "disclosure".into(),
            body_ref: "body:b".into(),
            retention_policy_id: "retain".into(),
        };
        let vote = community::v1::CastVoteRequest {
            context: Some(context.clone()),
            post_id: "post:p".into(),
            voter_ref: "user:voter".into(),
            direction: community::v1::VoteDirection::Up as i32,
        };
        let moderation = community::v1::ApplyActionRequest {
            context: Some(context),
            post_id: "post:p".into(),
            policy_ref: "policy:moderation".into(),
            evidence_ref: "evidence:e".into(),
            verb: community::v1::ModerationVerb::Hide as i32,
        };

        let decoded_create =
            community::v1::CreatePostRequest::decode(create.encode_to_vec().as_slice()).unwrap();
        let decoded_vote =
            community::v1::CastVoteRequest::decode(vote.encode_to_vec().as_slice()).unwrap();
        let decoded_moderation =
            community::v1::ApplyActionRequest::decode(moderation.encode_to_vec().as_slice())
                .unwrap();

        assert_eq!(decoded_create.space_id, "space:s");
        assert_eq!(
            decoded_vote.direction,
            community::v1::VoteDirection::Up as i32
        );
        assert_eq!(
            decoded_moderation.verb,
            community::v1::ModerationVerb::Hide as i32
        );
    }

    #[test]
    fn generated_messenger_mail_and_social_requests_reach_write_plan_boundaries() {
        let messenger = messenger_post_message_generated_write_plan(
            tenant("tenant:t"),
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
            },
        )
        .unwrap();

        let mail = mail_send_message_generated_write_plan(
            tenant("tenant:t"),
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
            },
        )
        .unwrap();

        let social = social_publish_post_generated_write_plan(
            tenant("person:p"),
            social::v1::PublishPostRequest {
                context: social::v1::SocialContextKind::Personal as i32,
                scope_ref: "person:p".into(),
                tenant_scope_ref: "tenant:t".into(),
                principal_ref: "user:u".into(),
                idempotency_key: "idem-social".into(),
                policy_decision_ref: "policy".into(),
                audit_correlation_id: "audit".into(),
                post_id: "post:p".into(),
                creator_ref: "user:u".into(),
                kind: social::v1::SocialArtifactKind::Story as i32,
                media_refs: vec!["media:m".into()],
                story_expires_at: 10,
                collab_owner_refs: vec![],
                collab_consent_refs: vec![],
                workflow_consent_ref: String::new(),
                ar_biometric_persisted: false,
                story_purge_now: 11,
            },
        )
        .unwrap();

        assert_eq!(
            messenger.rpc.fully_qualified_method,
            "/oya.messenger.v1.MessageStream/PostMessage"
        );
        assert_eq!(
            mail.rpc.fully_qualified_method,
            "/oya.mail.v1.Mail/SendMessage"
        );
        assert_eq!(
            social.rpc.fully_qualified_method,
            "/oya.social.v1.PostComposition/PublishPost"
        );
        assert_eq!(messenger.body.receipt.event_type, "messenger.message.sent");
        assert_eq!(mail.body.receipt.event_type, "mail.message.submitted");
        assert_eq!(social.body.receipt.event_type, "social.post.created");
    }

    #[test]
    fn generated_community_requests_reach_create_vote_and_moderation_write_plans() {
        let context = community_context_message("user:u", "idem-create");
        let created = community_create_post_generated_write_plan(
            tenant("tenant:t"),
            community::v1::CreatePostRequest {
                context: Some(context),
                space_id: "space:s".into(),
                post_id: "post:p".into(),
                thread_id: "thread:t".into(),
                mode: community::v1::CommunityMode::Teamblind as i32,
                routine_display_ref: "anon".into(),
                audit_author_ref: "user:u".into(),
                disclosure_policy_ref: "disclosure".into(),
                body_ref: "body:b".into(),
                retention_policy_id: "retain".into(),
            },
        )
        .unwrap();
        let mut ledger = community_post_store_domain::VoteLedger::new(&created.body.post);
        let vote = community_cast_vote_generated_write_plan(
            tenant("tenant:t"),
            &created.body.post,
            &mut ledger,
            community::v1::CastVoteRequest {
                context: Some(community_context_message("user:voter", "idem-vote")),
                post_id: "post:p".into(),
                voter_ref: "user:voter".into(),
                direction: community::v1::VoteDirection::Up as i32,
            },
        )
        .unwrap();
        let moderation = community_apply_action_generated_write_plan(
            tenant("tenant:t"),
            &created.body.post,
            community::v1::ApplyActionRequest {
                context: Some(community_context_message("user:u", "idem-moderation")),
                post_id: "post:p".into(),
                policy_ref: "policy:moderation".into(),
                evidence_ref: "evidence:e".into(),
                verb: community::v1::ModerationVerb::Hide as i32,
            },
        )
        .unwrap();

        assert_eq!(
            created.rpc.fully_qualified_method,
            "/oya.community.v1.PostStoreService/CreatePost"
        );
        assert_eq!(
            vote.rpc.fully_qualified_method,
            "/oya.community.v1.VotingEngineService/CastVote"
        );
        assert_eq!(
            moderation.rpc.fully_qualified_method,
            "/oya.community.v1.ModerationQueueService/ApplyAction"
        );
        assert_eq!(vote.body.receipt.event_type, "community.vote.cast");
        assert_eq!(
            moderation.body.receipt.event_type,
            "community.moderation.actioned"
        );
    }

    #[test]
    fn generated_adapter_rejects_invalid_enums_before_write_plan_execution() {
        let err = social_publish_post_generated_write_plan(
            tenant("person:p"),
            social::v1::PublishPostRequest {
                context: social::v1::SocialContextKind::Personal as i32,
                scope_ref: "person:p".into(),
                tenant_scope_ref: "tenant:t".into(),
                principal_ref: "user:u".into(),
                idempotency_key: "idem-social".into(),
                policy_decision_ref: "policy".into(),
                audit_correlation_id: "audit".into(),
                post_id: "post:p".into(),
                creator_ref: "user:u".into(),
                kind: social::v1::SocialArtifactKind::Unspecified as i32,
                media_refs: vec![],
                story_expires_at: 0,
                collab_owner_refs: vec![],
                collab_consent_refs: vec![],
                workflow_consent_ref: String::new(),
                ar_biometric_persisted: false,
                story_purge_now: 0,
            },
        )
        .unwrap_err();

        assert_eq!(
            err,
            GeneratedBackboneGrpcAdapterError::InvalidEnum {
                request: "PublishPostRequest",
                field: "kind",
                value: social::v1::SocialArtifactKind::Unspecified as i32,
            }
        );
    }

    #[test]
    fn generated_adapter_rejects_missing_oneof_envelope_before_app_boundary() {
        let err = messenger_post_message_generated_write_plan(
            tenant("tenant:t"),
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
                envelope: None,
                retention_policy_id: "retain".into(),
                legal_hold_ids: vec![],
            },
        )
        .unwrap_err();

        assert_eq!(
            err,
            GeneratedBackboneGrpcAdapterError::MissingMessage {
                request: "PostMessageRequest",
                field: "envelope",
            }
        );
    }

    fn tenant(scope_ref: &str) -> TenantSqlContext {
        TenantSqlContext::new(scope_ref, "cell-a", format!("{scope_ref}#cell-a"), "US").unwrap()
    }

    fn community_context_message(
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
}
