//! Framework-free gRPC write-plan boundary for mail mailbox-store.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use comms_mail_mailbox_api::{AuthorizedMailContext, SubmitMessageRequest};
use comms_mail_mailbox_app::{MailAppError, MailSubmissionPlan, plan_submit_message};
use shared_postgres_command_kernel::TenantSqlContext;
use shared_protocol_transport_kernel::GrpcUnaryPlan;

pub const MAIL_PROTO_PACKAGE: &str = "oya.mail.v1";
pub const MAIL_GRPC_SERVICE: &str = "Mail";
pub const SEND_MESSAGE_RPC: &str = "SendMessage";

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
pub enum MailGrpcError {
    App(MailAppError),
    RpcMismatch {
        expected_package: &'static str,
        expected_service: &'static str,
        expected_rpc: &'static str,
        actual_package: &'static str,
        actual_service: &'static str,
        actual_rpc: &'static str,
    },
}

pub fn send_message_write_plan(
    tenant: TenantSqlContext,
    context: AuthorizedMailContext,
    request: SubmitMessageRequest,
) -> Result<GrpcResponse<MailSubmissionPlan>, MailGrpcError> {
    let plan = plan_submit_message(tenant, context, request).map_err(MailGrpcError::App)?;
    let rpc = require_rpc(&plan.transport.grpc_unary)?;
    Ok(GrpcResponse {
        status: GrpcStatusCode::Ok,
        rpc,
        body: plan,
    })
}

fn require_rpc(rpc: &GrpcUnaryPlan) -> Result<GrpcUnaryPlan, MailGrpcError> {
    if rpc.package == MAIL_PROTO_PACKAGE
        && rpc.service == MAIL_GRPC_SERVICE
        && rpc.rpc == SEND_MESSAGE_RPC
    {
        Ok(rpc.clone())
    } else {
        Err(MailGrpcError::RpcMismatch {
            expected_package: MAIL_PROTO_PACKAGE,
            expected_service: MAIL_GRPC_SERVICE,
            expected_rpc: SEND_MESSAGE_RPC,
            actual_package: rpc.package,
            actual_service: rpc.service,
            actual_rpc: rpc.rpc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comms_mail_mailbox_api::{
        DmarcApiPolicy, DmarcCheckRequest, MailApiContext, MailApiEnvelope,
    };

    fn tenant() -> TenantSqlContext {
        TenantSqlContext::new("tenant:t", "cell-a", "tenant:t#cell-a", "US").unwrap()
    }

    fn context() -> AuthorizedMailContext {
        AuthorizedMailContext {
            context: MailApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    fn request() -> SubmitMessageRequest {
        SubmitMessageRequest {
            message_id: "message:m".into(),
            mailbox_id: "mailbox:b".into(),
            subject_ref: "user:u".into(),
            envelope: MailApiEnvelope::TenantDek {
                dek_ref: "dek:d".into(),
            },
            retention_policy_id: "retain".into(),
            dmarc_check: Some(DmarcCheckRequest {
                domain_ref: "domain:d".into(),
                spf_aligned: true,
                dkim_aligned: true,
                policy: DmarcApiPolicy::Reject,
                evidence_ref: "evidence:e".into(),
            }),
        }
    }

    #[test]
    fn grpc_send_message_returns_mail_transport_plan() {
        let response = send_message_write_plan(tenant(), context(), request()).unwrap();

        assert_eq!(response.status, GrpcStatusCode::Ok);
        assert_eq!(response.rpc.rpc, SEND_MESSAGE_RPC);
        assert_eq!(response.body.receipt.event_type, "mail.message.submitted");
        assert_eq!(
            response.body.transport.broker_publish.channel_address,
            "workflow-events/mail.message.sent"
        );
    }
}
