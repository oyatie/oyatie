// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use workspace_chat_api::{
    WORKSPACE_CHAT_MESSAGE_SEND_SURFACE, WORKSPACE_CHAT_OPENAPI_CONTRACT,
    WorkspaceChatApiAuthorization, WorkspaceChatApiError, WorkspaceChatApiPrincipal,
    WorkspaceChatAttachmentRequest, WorkspaceChatChannelSeed, WorkspaceChatMessageDirectory,
    WorkspaceChatMessageMetadata, WorkspaceChatMessageRecord, WorkspaceChatMessageSendApiRequest,
    WorkspaceChatMessageSendApiStatus, WorkspaceChatMessageSendIdempotencyLedger,
    WorkspaceChatMessageSendRequest, WorkspaceChatSendBoundaryContext,
    send_workspace_chat_message_from_api,
};

const CHANNEL_ID: &str = "chat_channel_001";
const MESSAGE_ID: &str = "chat_message_001";
const TENANT_ID: &str = "ten_workspace_alpha";
const SENDER: &str = "user:owner@example.com";

fn boundary(request_id: &str, idempotency_key: &str) -> WorkspaceChatSendBoundaryContext {
    WorkspaceChatSendBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal(principal_id: &str) -> WorkspaceChatApiPrincipal {
    WorkspaceChatApiPrincipal {
        tenant_id: TENANT_ID.to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization(principal_id: &str, surfaces: &[&str]) -> WorkspaceChatApiAuthorization {
    WorkspaceChatApiAuthorization {
        tenant_id: TENANT_ID.to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn seeded_directory() -> WorkspaceChatMessageDirectory {
    let mut directory = WorkspaceChatMessageDirectory::default();
    directory
        .insert_channel_seed(WorkspaceChatChannelSeed {
            channel_id: CHANNEL_ID.to_string(),
            tenant_id: TENANT_ID.to_string(),
            region: "region-home".to_string(),
            cell_id: "cell-workspace-kr-001".to_string(),
            kind: "channel".to_string(),
            name: Some("incident-room".to_string()),
            data_class: "PII_IDENTIFYING".to_string(),
            participants: vec![
                (
                    SENDER.to_string(),
                    Some("Owner User".to_string()),
                    "owner".to_string(),
                ),
                (
                    "user:member@example.com".to_string(),
                    Some("Member User".to_string()),
                    "member".to_string(),
                ),
                (
                    "bot:triage".to_string(),
                    Some("Triage Bot".to_string()),
                    "bot".to_string(),
                ),
            ],
            created_at_epoch_seconds: 1_700_000_000,
        })
        .expect("seed channel is valid");
    directory
}

fn send_body(message_id: &str) -> WorkspaceChatMessageSendRequest {
    WorkspaceChatMessageSendRequest {
        message_id: message_id.to_string(),
        channel_id: CHANNEL_ID.to_string(),
        tenant_id: TENANT_ID.to_string(),
        thread_id: None,
        parent_message_id: None,
        sender_ref: SENDER.to_string(),
        sender_kind: "human".to_string(),
        body: Some("Ship status?".to_string()),
        attachments: Vec::new(),
        data_class: "PII_IDENTIFYING".to_string(),
        created_at_epoch_seconds: 1_700_000_010,
    }
}

fn send_request(request_id: &str, idempotency_key: &str) -> WorkspaceChatMessageSendApiRequest {
    WorkspaceChatMessageSendApiRequest {
        path_channel_id: CHANNEL_ID.to_string(),
        path_message_id: MESSAGE_ID.to_string(),
        boundary: boundary(request_id, idempotency_key),
        principal: principal(SENDER),
        authorization: authorization(SENDER, &[WORKSPACE_CHAT_MESSAGE_SEND_SURFACE]),
        body: send_body(MESSAGE_ID),
    }
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(
        WORKSPACE_CHAT_MESSAGE_SEND_SURFACE,
        "workspace.chat.message.send"
    );
    assert_eq!(
        WORKSPACE_CHAT_OPENAPI_CONTRACT,
        "contracts/openapi/workspace/workspace-chat-v1.yaml"
    );
    assert_eq!(WorkspaceChatMessageSendApiStatus::Created.code(), 201);
    assert_eq!(WorkspaceChatMessageSendApiStatus::BadRequest.code(), 400);
    assert_eq!(WorkspaceChatMessageSendApiStatus::Forbidden.code(), 403);
    assert_eq!(WorkspaceChatMessageSendApiStatus::NotFound.code(), 404);
    assert_eq!(WorkspaceChatMessageSendApiStatus::Conflict.code(), 409);
    assert_eq!(
        WorkspaceChatMessageSendApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn send_chat_message_creates_once_and_replays_same_idempotent_result() {
    let mut directory = seeded_directory();
    let mut ledger = WorkspaceChatMessageSendIdempotencyLedger::default();
    let request = send_request("req-workspace-chat-send", "idem-workspace-chat-send");

    let first = send_workspace_chat_message_from_api(&mut directory, &mut ledger, request.clone())
        .expect("authorized Chat message send succeeds");
    let second = send_workspace_chat_message_from_api(&mut directory, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(directory.message_len(), 1);
    assert_eq!(first.metadata.request_id, "req-workspace-chat-send");
    assert_eq!(first.metadata.surface, WORKSPACE_CHAT_MESSAGE_SEND_SURFACE);
    assert_eq!(first.data.message_id, MESSAGE_ID);
    assert_eq!(first.data.channel_id, CHANNEL_ID);
    assert_eq!(first.data.sender_ref, SENDER);
    assert_eq!(first.data.sender_kind, "human");
    assert_eq!(first.data.body.as_deref(), Some("Ship status?"));
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn send_chat_message_rejects_path_body_tenant_and_sender_drift_before_mutation() {
    let mut directory = seeded_directory();
    let mut ledger = WorkspaceChatMessageSendIdempotencyLedger::default();
    let mut request = send_request("req-workspace-chat-drift", "idem-workspace-chat-drift");
    request.body.message_id = "chat_message_other".to_string();

    let error = send_workspace_chat_message_from_api(&mut directory, &mut ledger, request)
        .expect_err("path/body message id drift is rejected");
    assert_eq!(
        error,
        WorkspaceChatApiError::MessageIdMismatch {
            path_message_id: MESSAGE_ID.to_string(),
            body_message_id: "chat_message_other".to_string(),
        }
    );
    assert_eq!(error.message_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(directory.message_len(), 0);

    let mut tenant_drift = send_request("req-workspace-chat-tenant", "idem-workspace-chat-tenant");
    tenant_drift.boundary.tenant_id = "ten_other".to_string();
    let error = send_workspace_chat_message_from_api(&mut directory, &mut ledger, tenant_drift)
        .expect_err("tenant drift is rejected");
    assert_eq!(error.message_status_code(), 403);
    assert!(matches!(
        error,
        WorkspaceChatApiError::TenantMismatch { .. }
    ));
    assert!(ledger.is_empty());

    let mut sender_drift = send_request("req-workspace-chat-sender", "idem-workspace-chat-sender");
    sender_drift.body.sender_ref = "user:member@example.com".to_string();
    let error = send_workspace_chat_message_from_api(&mut directory, &mut ledger, sender_drift)
        .expect_err("principal cannot impersonate another channel participant");
    assert_eq!(error.message_status_code(), 403);
    assert!(matches!(
        error,
        WorkspaceChatApiError::SenderPermissionDenied { .. }
    ));
    assert!(ledger.is_empty());
}

#[test]
fn send_chat_message_rejects_authorization_missing_channel_and_reused_idempotency_key() {
    let mut directory = seeded_directory();
    let mut ledger = WorkspaceChatMessageSendIdempotencyLedger::default();
    let mut request = send_request("req-workspace-chat-authz", "idem-workspace-chat-authz");
    request.authorization.allowed_surfaces = vec!["workspace.drive.get".to_string()];

    let error = send_workspace_chat_message_from_api(&mut directory, &mut ledger, request)
        .expect_err("authorization decision does not allow Chat message send");
    assert_eq!(
        error,
        WorkspaceChatApiError::AuthorizationDenied {
            surface: WORKSPACE_CHAT_MESSAGE_SEND_SURFACE.to_string(),
        }
    );
    assert_eq!(error.message_status_code(), 403);
    assert!(ledger.is_empty());

    let mut missing = send_request("req-workspace-chat-missing", "idem-workspace-chat-missing");
    missing.path_channel_id = "chat_channel_missing".to_string();
    missing.body.channel_id = "chat_channel_missing".to_string();
    let error = send_workspace_chat_message_from_api(&mut directory, &mut ledger, missing)
        .expect_err("missing channel is not found");
    assert_eq!(error.message_status_code(), 404);
    assert!(matches!(
        error,
        WorkspaceChatApiError::ChannelNotFound { .. }
    ));
    assert!(ledger.is_empty());

    let request = send_request("req-workspace-chat-idem", "idem-workspace-chat-idem");
    send_workspace_chat_message_from_api(&mut directory, &mut ledger, request.clone())
        .expect("initial Chat message send succeeds");
    let mut drifted = request;
    drifted.body.body = Some("Different body".to_string());
    assert_eq!(
        send_workspace_chat_message_from_api(&mut directory, &mut ledger, drifted),
        Err(WorkspaceChatApiError::IdempotencyKeyReused {
            idempotency_key: "idem-workspace-chat-idem".to_string(),
        })
    );
    assert_eq!(directory.message_len(), 1);
}

#[test]
fn send_chat_message_maps_duplicate_invalid_attachment_kind_and_data_class() {
    let mut directory = seeded_directory();
    let mut ledger = WorkspaceChatMessageSendIdempotencyLedger::default();
    send_workspace_chat_message_from_api(
        &mut directory,
        &mut ledger,
        send_request("req-workspace-chat-dup-1", "idem-workspace-chat-dup-1"),
    )
    .expect("first Chat message send succeeds");

    let duplicate = send_workspace_chat_message_from_api(
        &mut directory,
        &mut ledger,
        send_request("req-workspace-chat-dup-2", "idem-workspace-chat-dup-2"),
    )
    .expect_err("same message through new idempotency key is conflict");
    assert_eq!(duplicate.message_status_code(), 409);
    assert!(matches!(
        duplicate,
        WorkspaceChatApiError::MessageAlreadyExists { .. }
    ));

    let mut invalid_kind = send_request("req-workspace-chat-kind", "idem-workspace-chat-kind");
    invalid_kind.body.sender_kind = "daemon".to_string();
    let error = send_workspace_chat_message_from_api(&mut directory, &mut ledger, invalid_kind)
        .expect_err("unknown sender kind is rejected before kernel");
    assert_eq!(error.message_status_code(), 400);
    assert!(matches!(
        error,
        WorkspaceChatApiError::InvalidSenderKind { .. }
    ));

    let mut invalid_class = send_request("req-workspace-chat-class", "idem-workspace-chat-class");
    invalid_class.body.data_class = "AUDIT".to_string();
    let error = send_workspace_chat_message_from_api(&mut directory, &mut ledger, invalid_class)
        .expect_err("operational data class is rejected for public privacy field");
    assert_eq!(error.message_status_code(), 400);
    assert!(matches!(
        error,
        WorkspaceChatApiError::InvalidDataClassLabel { .. }
    ));

    let mut invalid_attachment = send_request(
        "req-workspace-chat-attachment",
        "idem-workspace-chat-attachment",
    );
    invalid_attachment.body.attachments = vec![WorkspaceChatAttachmentRequest {
        attachment_id: "attachment-1".to_string(),
        storage_key: "tenant/chat/file.bin".to_string(),
        mime_type: "application/octet-stream".to_string(),
        byte_len: 0,
    }];
    let error =
        send_workspace_chat_message_from_api(&mut directory, &mut ledger, invalid_attachment)
            .expect_err("empty attachment is rejected before mutation");
    assert_eq!(error.message_status_code(), 400);
    assert!(matches!(error, WorkspaceChatApiError::Chat(_)));
}

#[test]
fn stable_error_response_shape_uses_request_id_and_field_details() {
    let error = WorkspaceChatApiError::InvalidDataClassLabel {
        data_class: "AUDIT".to_string(),
    };

    let response = error.error_response("req-workspace-chat-error");

    assert_eq!(response.error.code, "WORKSPACE_CHAT_DATA_CLASS_INVALID");
    assert_eq!(response.error.request_id, "req-workspace-chat-error");
    assert_eq!(response.error.details[0].field, "body.data_class");
    assert_eq!(response.error.retry_after_seconds, None);
}

#[test]
fn public_response_structs_keep_contract_names_stable() {
    let _metadata = WorkspaceChatMessageMetadata {
        request_id: "req-workspace-chat-structs".to_string(),
        surface: WORKSPACE_CHAT_MESSAGE_SEND_SURFACE.to_string(),
        openapi_contract: WORKSPACE_CHAT_OPENAPI_CONTRACT.to_string(),
    };
    let _record = WorkspaceChatMessageRecord {
        message_id: MESSAGE_ID.to_string(),
        channel_id: CHANNEL_ID.to_string(),
        tenant_id: TENANT_ID.to_string(),
        thread_id: None,
        parent_message_id: None,
        sender_ref: SENDER.to_string(),
        sender_kind: "human".to_string(),
        body: Some("Ship status?".to_string()),
        attachments: Vec::new(),
        data_class: "PII_IDENTIFYING".to_string(),
        created_at_epoch_seconds: 1_700_000_010,
        schema_version: 1,
    };
}
