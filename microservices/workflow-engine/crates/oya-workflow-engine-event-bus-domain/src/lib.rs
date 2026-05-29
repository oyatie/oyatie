//! Workflow-engine event-bus domain foundation.
//!
//! This crate provides a source-level policy and residency binding layer above
//! the workflow event-bus kernel. It validates tenant, cell, principal,
//! authorization, policy bundle, residency, trace, audit, producer, consumer,
//! CloudEvents, AsyncAPI, partition, idempotency, causation, correlation, and
//! payload-reference metadata before kernel delegation; maps authorized publish
//! intents and consumer delivery checks into redaction-safe domain receipts; and
//! preserves source-only non-claims for later broker, durable outbox/inbox,
//! consumer group, and Oyatie Cloud tenant workload integration. It performs no
//! broker connection, topic creation, network I/O, serialization-framework work,
//! durable outbox or inbox writes, consumer group coordination, offset commits,
//! payload materialization, policy-engine execution, signing, filesystem access,
//! random/UUID generation, wall-clock reads, Kubernetes calls, cloud deployment,
//! or tenant workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_workflow_engine_event_bus_kernel::{
    WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF, WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION,
    WORKFLOW_EVENT_BUS_DEFAULT_CONTENT_TYPE, WORKFLOW_EVENT_BUS_KERNEL_SURFACE,
    WORKFLOW_EVENT_BUS_MAX_SUBSCRIPTION_BATCH_SIZE, WorkflowEventBusChannel,
    WorkflowEventBusCloudEvent, WorkflowEventBusContext, WorkflowEventBusDeliveryCandidate,
    WorkflowEventBusDeliveryDecision, WorkflowEventBusDeliveryStatus, WorkflowEventBusEventKind,
    WorkflowEventBusKernelError, WorkflowEventBusPublishPlan, WorkflowEventBusPublishRequest,
    WorkflowEventBusSubscription, evaluate_delivery, plan_publish,
};

pub const WORKFLOW_EVENT_BUS_DOMAIN_SURFACE: &str = "surface:workflow-event-bus-domain";
pub const WORKFLOW_EVENT_BUS_DOMAIN_NON_CLAIM_REF: &str =
    "workflow-event-bus-domain:no-broker-runtime-claim";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusDomainPolicyBinding {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub cell_id: String,                                // data_class: INTERNAL_ONLY
    pub principal_id: String,                           // data_class: INTERNAL_ONLY
    pub authorization_decision_id: String,              // data_class: INTERNAL_ONLY
    pub authorization_evidence_ref: String,             // data_class: INTERNAL_ONLY
    pub policy_bundle_ref: String,                      // data_class: INTERNAL_ONLY
    pub residency_ref: String,                          // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,                      // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,                        // data_class: INTERNAL_ONLY
    pub allowed_channels: Vec<WorkflowEventBusChannel>, // data_class: INTERNAL_ONLY
    pub allowed_event_types: Vec<String>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusDomainPublishIntent {
    pub producer_ref: String,                  // data_class: INTERNAL_ONLY
    pub event_kind: WorkflowEventBusEventKind, // data_class: PUBLIC
    pub event_id: String,                      // data_class: INTERNAL_ONLY
    pub source_ref: String,                    // data_class: INTERNAL_ONLY
    pub subject_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub time_ref: Option<String>,              // data_class: INTERNAL_ONLY
    pub dataschema_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub partition_key_ref: String,             // data_class: INTERNAL_ONLY
    pub idempotency_key: String,               // data_class: INTERNAL_ONLY
    pub causation_ref: String,                 // data_class: INTERNAL_ONLY
    pub correlation_ref: String,               // data_class: INTERNAL_ONLY
    pub payload_ref: String,                   // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusDomainSubscriptionIntent {
    pub consumer_ref: String,               // data_class: INTERNAL_ONLY
    pub channel: WorkflowEventBusChannel,   // data_class: PUBLIC
    pub allowed_event_types: Vec<String>,   // data_class: INTERNAL_ONLY
    pub replay_cursor_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub max_batch_size: u32,                // data_class: INTERNAL_ONLY
    pub authorization_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusDomainStatus {
    Accepted,
    Denied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusDomainPublishReceipt {
    pub status: WorkflowEventBusDomainStatus, // data_class: PUBLIC
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub event_type: String,                   // data_class: PUBLIC
    pub channel_address: Option<String>,      // data_class: PUBLIC
    pub delivery_key: Option<String>,         // data_class: INTERNAL_ONLY
    pub publish_plan: Option<WorkflowEventBusPublishPlan>, // data_class: INTERNAL_ONLY
    pub denial_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusDomainDeliveryReceipt {
    pub status: WorkflowEventBusDomainStatus, // data_class: PUBLIC
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub event_type: String,                   // data_class: PUBLIC
    pub channel_address: String,              // data_class: PUBLIC
    pub consumer_ref: String,                 // data_class: INTERNAL_ONLY
    pub offset_ref: String,                   // data_class: INTERNAL_ONLY
    pub kernel_decision: WorkflowEventBusDeliveryDecision, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowEventBusDomainError {
    InvalidPolicyBinding { evidence_ref: String },
    InvalidPublishIntent { evidence_ref: String },
    InvalidSubscriptionIntent { evidence_ref: String },
    AuthorizationDenied { evidence_ref: String },
    KernelRejected { evidence_ref: String },
    UnsafeMetadata { evidence_ref: String },
}

impl WorkflowEventBusDomainError {
    pub fn primary_evidence_ref(&self) -> &str {
        match self {
            Self::InvalidPolicyBinding { evidence_ref }
            | Self::InvalidPublishIntent { evidence_ref }
            | Self::InvalidSubscriptionIntent { evidence_ref }
            | Self::AuthorizationDenied { evidence_ref }
            | Self::KernelRejected { evidence_ref }
            | Self::UnsafeMetadata { evidence_ref } => evidence_ref,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusDomain;

impl WorkflowEventBusDomain {
    pub fn authorize_publish(
        binding: WorkflowEventBusDomainPolicyBinding,
        intent: WorkflowEventBusDomainPublishIntent,
    ) -> Result<WorkflowEventBusDomainPublishReceipt, WorkflowEventBusDomainError> {
        validate_policy_binding(&binding)?;
        validate_publish_intent(&intent)?;
        validate_authorized_event(&binding, intent.event_kind)?;

        let request = WorkflowEventBusPublishRequest {
            context: WorkflowEventBusContext {
                tenant_id: binding.tenant_id.clone(),
                cell_id: binding.cell_id.clone(),
                producer_ref: intent.producer_ref.clone(),
                trace_context_ref: binding.trace_context_ref.clone(),
                policy_decision_ref: binding.authorization_decision_id.clone(),
                residency_ref: binding.residency_ref.clone(),
                audit_chain_ref: binding.audit_chain_ref.clone(),
            },
            channel: intent.event_kind.channel(),
            event: WorkflowEventBusCloudEvent {
                specversion: WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION.to_owned(),
                id: intent.event_id.clone(),
                source: intent.source_ref.clone(),
                event_type: intent.event_kind.event_type().to_owned(),
                subject_ref: intent.subject_ref.clone(),
                time_ref: intent.time_ref.clone(),
                datacontenttype: WORKFLOW_EVENT_BUS_DEFAULT_CONTENT_TYPE.to_owned(),
                dataschema_ref: intent.dataschema_ref.clone(),
            },
            partition_key_ref: intent.partition_key_ref.clone(),
            idempotency_key: intent.idempotency_key.clone(),
            causation_ref: intent.causation_ref.clone(),
            correlation_ref: intent.correlation_ref.clone(),
            payload_ref: intent.payload_ref.clone(),
            evidence_refs: sorted_unique(
                [
                    intent.evidence_refs.clone(),
                    vec![
                        binding.authorization_evidence_ref.clone(),
                        binding.policy_bundle_ref.clone(),
                        binding.residency_ref.clone(),
                        WORKFLOW_EVENT_BUS_DOMAIN_SURFACE.to_owned(),
                    ],
                ]
                .concat(),
            ),
        };
        let plan =
            plan_publish(request).map_err(|error| WorkflowEventBusDomainError::KernelRejected {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            })?;
        Ok(WorkflowEventBusDomainPublishReceipt {
            status: WorkflowEventBusDomainStatus::Accepted,
            tenant_id: binding.tenant_id,
            cell_id: binding.cell_id,
            event_type: plan.event_type.clone(),
            channel_address: Some(plan.channel_address.clone()),
            delivery_key: Some(plan.delivery_key.clone()),
            evidence_refs: sorted_unique(
                [
                    plan.evidence_refs.clone(),
                    vec!["workflow-event-bus-domain:publish-authorized".to_owned()],
                ]
                .concat(),
            ),
            non_claim_refs: domain_non_claim_refs(plan.non_claim_refs.clone()),
            publish_plan: Some(plan),
            denial_ref: None,
        })
    }

    pub fn evaluate_authorized_delivery(
        binding: WorkflowEventBusDomainPolicyBinding,
        subscription_intent: WorkflowEventBusDomainSubscriptionIntent,
        candidate: WorkflowEventBusDeliveryCandidate,
    ) -> Result<WorkflowEventBusDomainDeliveryReceipt, WorkflowEventBusDomainError> {
        validate_policy_binding(&binding)?;
        validate_subscription_intent(&subscription_intent)?;
        if binding.tenant_id != candidate.tenant_id || binding.cell_id != candidate.cell_id {
            return Err(WorkflowEventBusDomainError::AuthorizationDenied {
                evidence_ref: "workflow-event-bus-domain:tenant-cell-scope-mismatch".to_owned(),
            });
        }
        if !binding
            .allowed_channels
            .contains(&subscription_intent.channel)
            || !subscription_intent
                .allowed_event_types
                .iter()
                .all(|event_type| binding.allowed_event_types.contains(event_type))
        {
            return Err(WorkflowEventBusDomainError::AuthorizationDenied {
                evidence_ref: "workflow-event-bus-domain:subscription-not-authorized".to_owned(),
            });
        }
        let subscription = WorkflowEventBusSubscription {
            tenant_id: binding.tenant_id.clone(),
            cell_id: binding.cell_id.clone(),
            consumer_ref: subscription_intent.consumer_ref.clone(),
            channel: subscription_intent.channel,
            allowed_event_types: subscription_intent.allowed_event_types.clone(),
            replay_cursor_ref: subscription_intent.replay_cursor_ref.clone(),
            max_batch_size: subscription_intent.max_batch_size,
            authorization_evidence_ref: subscription_intent.authorization_evidence_ref.clone(),
        };
        let decision = evaluate_delivery(subscription, candidate).map_err(|error| {
            WorkflowEventBusDomainError::KernelRejected {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            }
        })?;
        let status = match decision.status {
            WorkflowEventBusDeliveryStatus::Accepted => WorkflowEventBusDomainStatus::Accepted,
            WorkflowEventBusDeliveryStatus::Rejected => WorkflowEventBusDomainStatus::Denied,
        };
        Ok(WorkflowEventBusDomainDeliveryReceipt {
            status,
            tenant_id: binding.tenant_id,
            cell_id: binding.cell_id,
            event_type: decision.event_type.clone(),
            channel_address: decision.channel_address.clone(),
            consumer_ref: decision.consumer_ref.clone(),
            offset_ref: decision.offset_ref.clone(),
            evidence_refs: sorted_unique(
                [
                    decision.evidence_refs.clone(),
                    vec![
                        binding.authorization_evidence_ref,
                        binding.policy_bundle_ref,
                        binding.residency_ref,
                        "workflow-event-bus-domain:delivery-evaluated".to_owned(),
                    ],
                ]
                .concat(),
            ),
            non_claim_refs: domain_non_claim_refs(decision.non_claim_refs.clone()),
            kernel_decision: decision,
        })
    }
}

fn validate_policy_binding(
    binding: &WorkflowEventBusDomainPolicyBinding,
) -> Result<(), WorkflowEventBusDomainError> {
    if !is_safe_tenant(&binding.tenant_id) {
        return Err(WorkflowEventBusDomainError::InvalidPolicyBinding {
            evidence_ref: "workflow-event-bus-domain:tenant-invalid".to_owned(),
        });
    }
    for value in [
        &binding.cell_id,
        &binding.principal_id,
        &binding.authorization_decision_id,
        &binding.authorization_evidence_ref,
        &binding.policy_bundle_ref,
        &binding.residency_ref,
        &binding.trace_context_ref,
        &binding.audit_chain_ref,
    ] {
        if !is_safe_ref(value) {
            return Err(WorkflowEventBusDomainError::InvalidPolicyBinding {
                evidence_ref: "workflow-event-bus-domain:policy-ref-invalid".to_owned(),
            });
        }
    }
    if binding.allowed_channels.is_empty() || binding.allowed_event_types.is_empty() {
        return Err(WorkflowEventBusDomainError::InvalidPolicyBinding {
            evidence_ref: "workflow-event-bus-domain:policy-empty-allowlist".to_owned(),
        });
    }
    Ok(())
}

fn validate_publish_intent(
    intent: &WorkflowEventBusDomainPublishIntent,
) -> Result<(), WorkflowEventBusDomainError> {
    for value in [
        &intent.producer_ref,
        &intent.event_id,
        &intent.source_ref,
        &intent.partition_key_ref,
        &intent.idempotency_key,
        &intent.causation_ref,
        &intent.correlation_ref,
        &intent.payload_ref,
    ] {
        if !is_safe_ref(value) {
            return Err(WorkflowEventBusDomainError::InvalidPublishIntent {
                evidence_ref: "workflow-event-bus-domain:publish-ref-invalid".to_owned(),
            });
        }
    }
    if !is_safe_optional_ref(intent.subject_ref.as_deref())
        || !is_safe_optional_ref(intent.time_ref.as_deref())
        || !is_safe_optional_ref(intent.dataschema_ref.as_deref())
    {
        return Err(WorkflowEventBusDomainError::InvalidPublishIntent {
            evidence_ref: "workflow-event-bus-domain:publish-optional-ref-invalid".to_owned(),
        });
    }
    if intent.evidence_refs.iter().any(|value| !is_safe_ref(value)) {
        return Err(WorkflowEventBusDomainError::UnsafeMetadata {
            evidence_ref: "workflow-event-bus-domain:evidence-ref-invalid".to_owned(),
        });
    }
    Ok(())
}

fn validate_subscription_intent(
    intent: &WorkflowEventBusDomainSubscriptionIntent,
) -> Result<(), WorkflowEventBusDomainError> {
    if !is_safe_ref(&intent.consumer_ref)
        || !is_safe_optional_ref(intent.replay_cursor_ref.as_deref())
        || !is_safe_ref(&intent.authorization_evidence_ref)
    {
        return Err(WorkflowEventBusDomainError::InvalidSubscriptionIntent {
            evidence_ref: "workflow-event-bus-domain:subscription-ref-invalid".to_owned(),
        });
    }
    if intent.max_batch_size == 0
        || intent.max_batch_size > WORKFLOW_EVENT_BUS_MAX_SUBSCRIPTION_BATCH_SIZE
    {
        return Err(WorkflowEventBusDomainError::InvalidSubscriptionIntent {
            evidence_ref: "workflow-event-bus-domain:subscription-batch-invalid".to_owned(),
        });
    }
    if intent.allowed_event_types.is_empty()
        || intent
            .allowed_event_types
            .iter()
            .any(|event_type| !is_safe_metadata(event_type))
    {
        return Err(WorkflowEventBusDomainError::InvalidSubscriptionIntent {
            evidence_ref: "workflow-event-bus-domain:subscription-event-type-invalid".to_owned(),
        });
    }
    Ok(())
}

fn validate_authorized_event(
    binding: &WorkflowEventBusDomainPolicyBinding,
    event_kind: WorkflowEventBusEventKind,
) -> Result<(), WorkflowEventBusDomainError> {
    if !binding.allowed_channels.contains(&event_kind.channel())
        || !binding
            .allowed_event_types
            .iter()
            .any(|event_type| event_type == event_kind.event_type())
    {
        return Err(WorkflowEventBusDomainError::AuthorizationDenied {
            evidence_ref: "workflow-event-bus-domain:publish-not-authorized".to_owned(),
        });
    }
    Ok(())
}

fn domain_non_claim_refs(mut kernel_non_claims: Vec<String>) -> Vec<String> {
    kernel_non_claims.extend([
        WORKFLOW_EVENT_BUS_DOMAIN_NON_CLAIM_REF.to_owned(),
        "workflow-event-bus-domain:no-policy-engine-execution".to_owned(),
        "workflow-event-bus-domain:no-cloud-runtime-deployment".to_owned(),
        "workflow-event-bus-domain:no-hyperscaler-claim".to_owned(),
    ]);
    sorted_unique(kernel_non_claims)
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("ten_") && value == trimmed && is_safe_metadata(value)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
}

fn is_safe_optional_ref(value: Option<&str>) -> bool {
    value.is_none_or(is_safe_ref)
}

fn is_safe_metadata(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && !value.chars().any(char::is_whitespace)
        && !contains_raw_secret_material(value)
        && !contains_raw_content_material(value)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("private key")
        || lower.contains("-----begin")
        || lower.contains("secret=")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("raw output")
        || lower.contains("raw payload")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty() && !contains_raw_secret_material(value));
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding() -> WorkflowEventBusDomainPolicyBinding {
        WorkflowEventBusDomainPolicyBinding {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            principal_id: "principal:workflow-runtime".to_owned(),
            authorization_decision_id: "policy-decision:event-bus-allow".to_owned(),
            authorization_evidence_ref: "policy-evidence:event-bus-allow".to_owned(),
            policy_bundle_ref: "policy-bundle:event-bus-v1".to_owned(),
            residency_ref: "residency:us:data-plane".to_owned(),
            trace_context_ref: "trace:event-bus-domain:root".to_owned(),
            audit_chain_ref: "audit-chain:event-bus-domain".to_owned(),
            allowed_channels: vec![
                WorkflowEventBusChannel::WorkflowRuns,
                WorkflowEventBusChannel::WorkflowState,
                WorkflowEventBusChannel::TriggerEvents,
                WorkflowEventBusChannel::IntelligenceRequests,
                WorkflowEventBusChannel::OntologyProjections,
            ],
            allowed_event_types: vec![
                WorkflowEventBusEventKind::WorkflowRunStarted
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::TriggerEvaluated
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::IntelligenceDraftRequested
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::OntologyProjectionUpdated
                    .event_type()
                    .to_owned(),
            ],
        }
    }

    fn publish_intent(kind: WorkflowEventBusEventKind) -> WorkflowEventBusDomainPublishIntent {
        WorkflowEventBusDomainPublishIntent {
            producer_ref: "producer:workflow-engine:execution".to_owned(),
            event_kind: kind,
            event_id: "event:workflow-run-started:001".to_owned(),
            source_ref: "urn:oyatie:workflow-engine:execution".to_owned(),
            subject_ref: Some("subject:workflow-run:001".to_owned()),
            time_ref: Some("time:2026-05-25T00:00:00Z".to_owned()),
            dataschema_ref: Some("schema:workflow-event-run-started".to_owned()),
            partition_key_ref: "partition:tenant-workflow-run".to_owned(),
            idempotency_key: "idem:event-bus-domain:publish:001".to_owned(),
            causation_ref: "cause:execution-engine:start-run".to_owned(),
            correlation_ref: "corr:workflow-run:001".to_owned(),
            payload_ref: "body-ref:workflow-run-started".to_owned(),
            evidence_refs: vec!["evidence:event-bus-domain:publish".to_owned()],
        }
    }

    fn subscription_intent() -> WorkflowEventBusDomainSubscriptionIntent {
        WorkflowEventBusDomainSubscriptionIntent {
            consumer_ref: "consumer:workflow-state-machine".to_owned(),
            channel: WorkflowEventBusChannel::WorkflowState,
            allowed_event_types: vec![
                WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
            ],
            replay_cursor_ref: Some("cursor:event-bus-domain:state".to_owned()),
            max_batch_size: 100,
            authorization_evidence_ref: "authz:event-bus-domain:consume".to_owned(),
        }
    }

    fn candidate(kind: WorkflowEventBusEventKind) -> WorkflowEventBusDeliveryCandidate {
        WorkflowEventBusDeliveryCandidate {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            channel: kind.channel(),
            event_id: "event:workflow-state:001".to_owned(),
            event_type: kind.event_type().to_owned(),
            idempotency_key: "idem:event-bus-domain:delivery:001".to_owned(),
            payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
            offset_ref: "offset:partition-0:42".to_owned(),
            evidence_refs: vec!["evidence:event-bus-domain:delivery".to_owned()],
        }
    }

    #[test]
    fn domain_authorizes_publish_and_binds_policy_residency_audit_before_kernel_plan() {
        let receipt = WorkflowEventBusDomain::authorize_publish(
            binding(),
            publish_intent(WorkflowEventBusEventKind::WorkflowRunStarted),
        )
        .expect("authorized publish");
        assert_eq!(receipt.status, WorkflowEventBusDomainStatus::Accepted);
        assert_eq!(receipt.tenant_id, "ten_workflow_event_bus");
        assert_eq!(
            receipt.channel_address.as_deref(),
            Some("workflow.runs.events.v1")
        );
        assert!(
            receipt
                .delivery_key
                .as_deref()
                .unwrap()
                .contains("workflow.runs.events.v1")
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"workflow-event-bus-domain:publish-authorized".to_owned())
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"policy-bundle:event-bus-v1".to_owned())
        );
        assert!(
            receipt
                .non_claim_refs
                .contains(&"workflow-event-bus-domain:no-policy-engine-execution".to_owned())
        );
        assert!(receipt.publish_plan.is_some());
    }

    #[test]
    fn publish_denies_channel_or_event_not_allowed_before_kernel_delegation() {
        let error = WorkflowEventBusDomain::authorize_publish(
            binding(),
            publish_intent(WorkflowEventBusEventKind::WorkflowStepDispatched),
        )
        .expect_err("not authorized");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-domain:publish-not-authorized"
        );
    }

    #[test]
    fn raw_publish_metadata_is_rejected_without_echoing_secret_or_payload() {
        let intent = WorkflowEventBusDomainPublishIntent {
            payload_ref: "raw payload bearer sk-test customer message".to_owned(),
            ..publish_intent(WorkflowEventBusEventKind::WorkflowRunStarted)
        };
        let error = WorkflowEventBusDomain::authorize_publish(binding(), intent)
            .expect_err("raw payload denied");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-domain:publish-ref-invalid"
        );
        assert!(!format!("{error:?}").contains("sk-test"));
    }

    #[test]
    fn delivery_evaluation_accepts_allowed_subscription_and_preserves_kernel_decision() {
        let receipt = WorkflowEventBusDomain::evaluate_authorized_delivery(
            binding(),
            subscription_intent(),
            candidate(WorkflowEventBusEventKind::WorkflowStateTransitioned),
        )
        .expect("delivery receipt");
        assert_eq!(receipt.status, WorkflowEventBusDomainStatus::Accepted);
        assert_eq!(
            receipt.kernel_decision.status,
            WorkflowEventBusDeliveryStatus::Accepted
        );
        assert_eq!(receipt.channel_address, "workflow.state.events.v1");
        assert!(
            receipt
                .evidence_refs
                .contains(&"workflow-event-bus-domain:delivery-evaluated".to_owned())
        );
    }

    #[test]
    fn delivery_denies_policy_scope_drift_and_subscription_not_in_binding() {
        let drift = WorkflowEventBusDeliveryCandidate {
            tenant_id: "ten_other".to_owned(),
            ..candidate(WorkflowEventBusEventKind::WorkflowStateTransitioned)
        };
        let error = WorkflowEventBusDomain::evaluate_authorized_delivery(
            binding(),
            subscription_intent(),
            drift,
        )
        .expect_err("scope drift");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-domain:tenant-cell-scope-mismatch"
        );

        let subscription = WorkflowEventBusDomainSubscriptionIntent {
            allowed_event_types: vec![
                WorkflowEventBusEventKind::WorkflowStepDispatched
                    .event_type()
                    .to_owned(),
            ],
            ..subscription_intent()
        };
        let error = WorkflowEventBusDomain::evaluate_authorized_delivery(
            binding(),
            subscription,
            candidate(WorkflowEventBusEventKind::WorkflowStateTransitioned),
        )
        .expect_err("subscription not authorized");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-domain:subscription-not-authorized"
        );
    }

    #[test]
    fn rejected_kernel_delivery_is_mapped_to_domain_denial_without_runtime_claims() {
        let receipt = WorkflowEventBusDomain::evaluate_authorized_delivery(
            binding(),
            subscription_intent(),
            candidate(WorkflowEventBusEventKind::WorkflowRunStarted),
        )
        .expect("kernel rejected delivery receipt");
        assert_eq!(receipt.status, WorkflowEventBusDomainStatus::Denied);
        assert_eq!(
            receipt.kernel_decision.status,
            WorkflowEventBusDeliveryStatus::Rejected
        );
        assert!(
            receipt
                .non_claim_refs
                .contains(&"workflow-event-bus-domain:no-hyperscaler-claim".to_owned())
        );
    }
}
