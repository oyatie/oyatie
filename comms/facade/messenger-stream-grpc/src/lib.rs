//! Framework-free gRPC write-plan boundary for messenger message-stream.
//!
//! Tonic/prost server bindings stay in runtime composition. This crate owns the
//! proto RPC constants and a typed handler that returns the same app write plan
//! a future gRPC server must execute.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use comms_messenger_stream_api::{AuthorizedMessengerContext, SendMessageRequest};
use comms_messenger_stream_app::{MessengerAppError, MessengerWritePlan, plan_send_message};
use oya_shared_postgres_command_kernel::TenantSqlContext;
use oya_shared_protocol_transport_kernel::GrpcUnaryPlan;

pub const MESSENGER_PROTO_PACKAGE: &str = "oya.messenger.v1";
pub const MESSENGER_GRPC_SERVICE: &str = "MessageStream";
pub const POST_MESSAGE_RPC: &str = "PostMessage";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrpcStatusCode {
    Ok,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrpcResponse<T> {
    pub status: GrpcStatusCode, // data_class: INTERNAL_ONLY
    pub rpc: GrpcUnaryPlan,     // data_class: INTERNAL_ONLY
    pub body: T,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessengerGrpcError {
    App(MessengerAppError),
    RpcMismatch {
        expected_package: &'static str,
        expected_service: &'static str,
        expected_rpc: &'static str,
        actual_package: &'static str,
        actual_service: &'static str,
        actual_rpc: &'static str,
    },
}

pub fn post_message_write_plan(
    tenant: TenantSqlContext,
    context: AuthorizedMessengerContext,
    request: SendMessageRequest,
) -> Result<GrpcResponse<MessengerWritePlan>, MessengerGrpcError> {
    let plan = plan_send_message(tenant, context, request).map_err(MessengerGrpcError::App)?;
    let rpc = require_rpc(&plan.transport.grpc_unary)?;
    Ok(GrpcResponse {
        status: GrpcStatusCode::Ok,
        rpc,
        body: plan,
    })
}

fn require_rpc(rpc: &GrpcUnaryPlan) -> Result<GrpcUnaryPlan, MessengerGrpcError> {
    if rpc.package == MESSENGER_PROTO_PACKAGE
        && rpc.service == MESSENGER_GRPC_SERVICE
        && rpc.rpc == POST_MESSAGE_RPC
    {
        Ok(rpc.clone())
    } else {
        Err(MessengerGrpcError::RpcMismatch {
            expected_package: MESSENGER_PROTO_PACKAGE,
            expected_service: MESSENGER_GRPC_SERVICE,
            expected_rpc: POST_MESSAGE_RPC,
            actual_package: rpc.package,
            actual_service: rpc.service,
            actual_rpc: rpc.rpc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comms_messenger_stream_api::{MessengerApiContext, MessengerApiEnvelope};

    fn tenant() -> TenantSqlContext {
        TenantSqlContext::new("tenant:t", "cell-a", "tenant:t#cell-a", "US").unwrap()
    }

    fn context() -> AuthorizedMessengerContext {
        AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    fn request() -> SendMessageRequest {
        SendMessageRequest {
            message_id: "message:m".into(),
            channel_id: "channel:c".into(),
            author_ref: "user:u".into(),
            envelope: MessengerApiEnvelope::TenantDek {
                dek_ref: "dek:d".into(),
                four_eyes: true,
            },
            retention_policy_id: "retain".into(),
            legal_hold_ids: vec![],
        }
    }

    #[test]
    fn grpc_post_message_returns_app_plan_and_transport_descriptor() {
        let response = post_message_write_plan(tenant(), context(), request()).unwrap();

        assert_eq!(response.status, GrpcStatusCode::Ok);
        assert_eq!(response.rpc.rpc, POST_MESSAGE_RPC);
        assert_eq!(
            response.rpc.fully_qualified_method,
            "/oya.messenger.v1.MessageStream/PostMessage"
        );
        assert_eq!(response.body.receipt.event_type, "messenger.message.sent");
        assert_eq!(
            response.body.transport.broker_publish.channel_address,
            "workflow-events/messenger.message.posted"
        );
    }
}
