use crate::domain::TenantId;
use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AsyncApiChannel {
    pub channel: &'static str,
    pub direction: ChannelDirection,
    pub message: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChannelDirection {
    Publish,
    Subscribe,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AsyncApiMessage {
    pub tenant_id: String,
    pub message_type: String,
    pub payload_json: serde_json::Value,
}

pub struct AsyncApiHandler;

impl AsyncApiHandler {
    pub fn channels() -> Vec<AsyncApiChannel> {
        vec![
            AsyncApiChannel {
                channel: "quality-management.inspection-plan.events.v1",
                direction: ChannelDirection::Publish,
                message: "InspectionPlanChanged",
            },
            AsyncApiChannel {
                channel: "quality-management.inspection-lot.events.v1",
                direction: ChannelDirection::Publish,
                message: "InspectionLotChanged",
            },
            AsyncApiChannel {
                channel: "quality-management.certificate-of-analysis.events.v1",
                direction: ChannelDirection::Publish,
                message: "CertificateOfAnalysisChanged",
            },
            AsyncApiChannel {
                channel: "quality-management.quality-notification.events.v1",
                direction: ChannelDirection::Publish,
                message: "QualityNotificationChanged",
            },
            AsyncApiChannel {
                channel: "quality-management.quality-hold.events.v1",
                direction: ChannelDirection::Publish,
                message: "QualityHoldChanged",
            },
            AsyncApiChannel {
                channel: "quality-management.audit-evidence.events.v1",
                direction: ChannelDirection::Publish,
                message: "AuditEvidenceChanged",
            },
        ]
    }

    pub fn handle(message: AsyncApiMessage) -> Result<()> {
        TenantId::new(message.tenant_id)?;
        let expected = expected_event(&message.message_type)?;
        require_payload_value(
            &message.payload_json,
            "audit_event_class",
            expected.audit_event_class,
        )?;
        require_payload_value(
            &message.payload_json,
            "bounded_context",
            expected.bounded_context,
        )?;
        if message
            .payload_json
            .get("runtime_audit_chain_emitted")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return Err(ServiceError::validation(
                "runtime_audit_chain_emitted",
                "AsyncAPI fixture must not claim runtime audit-chain emission",
            ));
        }
        Ok(())
    }
}

struct ExpectedEvent {
    audit_event_class: &'static str,
    bounded_context: &'static str,
}

fn expected_event(message_type: &str) -> Result<ExpectedEvent> {
    match message_type {
        "InspectionPlanChanged" => Ok(ExpectedEvent {
            audit_event_class: "EVT-QUALITY_MANAGEMENT-INSPECTION_PLAN-CHANGED",
            bounded_context: "inspection-plan",
        }),
        "InspectionLotChanged" => Ok(ExpectedEvent {
            audit_event_class: "EVT-QUALITY_MANAGEMENT-INSPECTION_LOT-CHANGED",
            bounded_context: "inspection-lot",
        }),
        "CertificateOfAnalysisChanged" => Ok(ExpectedEvent {
            audit_event_class: "EVT-QUALITY_MANAGEMENT-CERTIFICATE_OF_ANALYSIS-CHANGED",
            bounded_context: "certificate-of-analysis",
        }),
        "QualityNotificationChanged" => Ok(ExpectedEvent {
            audit_event_class: "EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-CHANGED",
            bounded_context: "quality-notification",
        }),
        "QualityHoldChanged" => Ok(ExpectedEvent {
            audit_event_class: "EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-CHANGED",
            bounded_context: "quality-hold",
        }),
        "AuditEvidenceChanged" => Ok(ExpectedEvent {
            audit_event_class: "EVT-QUALITY_MANAGEMENT-AUDIT_EVIDENCE-CHANGED",
            bounded_context: "audit-evidence",
        }),
        _ => Err(ServiceError::validation(
            "message_type",
            "unknown quality-management AsyncAPI message fixture",
        )),
    }
}

fn require_payload_value(payload: &Value, field: &'static str, expected: &str) -> Result<()> {
    let actual = payload.get(field).and_then(Value::as_str).ok_or_else(|| {
        ServiceError::validation(field, format!("{field} is required for AsyncAPI fixture"))
    })?;
    if actual == expected {
        Ok(())
    } else {
        Err(ServiceError::validation(
            field,
            format!("expected {expected}, got {actual}"),
        ))
    }
}

pub fn validate_channels(channels: &[AsyncApiChannel]) -> Result<()> {
    const EXPECTED_MESSAGES: [&str; 6] = [
        "InspectionPlanChanged",
        "InspectionLotChanged",
        "CertificateOfAnalysisChanged",
        "QualityNotificationChanged",
        "QualityHoldChanged",
        "AuditEvidenceChanged",
    ];

    if channels.len() < EXPECTED_MESSAGES.len() {
        return Err(ServiceError::validation(
            "asyncapi_channels",
            "scaffold must include all six Quality Management event channels",
        ));
    }

    for expected_message in EXPECTED_MESSAGES {
        let present = channels.iter().any(|channel| {
            channel.direction == ChannelDirection::Publish
                && channel.message == expected_message
                && channel.channel.ends_with(".events.v1")
        });
        if !present {
            return Err(ServiceError::validation(
                "asyncapi_channels",
                format!("missing AsyncAPI event fixture for {expected_message}"),
            ));
        }
    }
    Ok(())
}
