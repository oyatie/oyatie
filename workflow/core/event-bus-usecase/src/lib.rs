//! Workflow-engine event-bus usecase foundation.
//!
//! The usecase composes request/idempotency/trace validation, deterministic
//! in-memory idempotency replay/conflict handling, metadata-only audit events,
//! and the policy/residency-bound event-bus domain for publish and delivery
//! decisions. It performs no concrete idempotency storage, broker connection,
//! topic creation, network I/O, serialization-framework work, durable
//! outbox/inbox writes, consumer group coordination, offset commits, payload
//! materialization, signing, filesystem access, random/UUID generation,
//! wall-clock reads, Kubernetes calls, cloud deployment, or tenant workload
//! scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use workflow_event_bus_domain::{
    WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF, WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION,
    WORKFLOW_EVENT_BUS_DEFAULT_CONTENT_TYPE, WORKFLOW_EVENT_BUS_DOMAIN_NON_CLAIM_REF,
    WORKFLOW_EVENT_BUS_DOMAIN_SURFACE, WORKFLOW_EVENT_BUS_KERNEL_SURFACE,
    WORKFLOW_EVENT_BUS_MAX_SUBSCRIPTION_BATCH_SIZE, WorkflowEventBusChannel,
    WorkflowEventBusCloudEvent, WorkflowEventBusContext, WorkflowEventBusDeliveryCandidate,
    WorkflowEventBusDeliveryDecision, WorkflowEventBusDeliveryStatus, WorkflowEventBusDomain,
    WorkflowEventBusDomainDeliveryReceipt, WorkflowEventBusDomainError,
    WorkflowEventBusDomainPolicyBinding, WorkflowEventBusDomainPublishIntent,
    WorkflowEventBusDomainPublishReceipt, WorkflowEventBusDomainStatus,
    WorkflowEventBusDomainSubscriptionIntent, WorkflowEventBusEventKind,
    WorkflowEventBusKernelError, WorkflowEventBusPublishPlan, WorkflowEventBusPublishRequest,
    WorkflowEventBusSubscription, evaluate_delivery, plan_publish,
};

pub const WORKFLOW_EVENT_BUS_USECASE_SURFACE: &str = "surface:workflow-event-bus-usecase";
pub const WORKFLOW_EVENT_BUS_USECASE_CONTRACT_REF: &str =
    "contract:workflow-event-bus-usecase-preview";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusUsecasePublishCommand {
    pub request_id: String,                           // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                      // data_class: INTERNAL_ONLY
    pub trace_ref: String,                            // data_class: INTERNAL_ONLY
    pub binding: WorkflowEventBusDomainPolicyBinding, // data_class: INTERNAL_ONLY
    pub intent: WorkflowEventBusDomainPublishIntent,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusUsecaseDeliveryCommand {
    pub request_id: String,                           // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                      // data_class: INTERNAL_ONLY
    pub trace_ref: String,                            // data_class: INTERNAL_ONLY
    pub binding: WorkflowEventBusDomainPolicyBinding, // data_class: INTERNAL_ONLY
    pub subscription_intent: WorkflowEventBusDomainSubscriptionIntent, // data_class: INTERNAL_ONLY
    pub candidate: WorkflowEventBusDeliveryCandidate, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusUsecaseStatus {
    DeliveryAccepted,
    DeliveryDenied,
    DomainDenied,
    IdempotencyConflict,
    InvalidInput,
    Published,
}

impl WorkflowEventBusUsecaseStatus {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::DeliveryAccepted => "delivery-accepted",
            Self::DeliveryDenied => "delivery-denied",
            Self::DomainDenied => "domain-denied",
            Self::IdempotencyConflict => "idempotency-conflict",
            Self::InvalidInput => "invalid-input",
            Self::Published => "published",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusUsecaseAuditEventKind {
    DeliveryAccepted,
    DeliveryDenied,
    DeliveryRequested,
    DomainDenied,
    IdempotencyConflict,
    InvalidInput,
    PublishAccepted,
    PublishRequested,
}

impl WorkflowEventBusUsecaseAuditEventKind {
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::DeliveryAccepted => "delivery-accepted",
            Self::DeliveryDenied => "delivery-denied",
            Self::DeliveryRequested => "delivery-requested",
            Self::DomainDenied => "domain-denied",
            Self::IdempotencyConflict => "idempotency-conflict",
            Self::InvalidInput => "invalid-input",
            Self::PublishAccepted => "publish-accepted",
            Self::PublishRequested => "publish-requested",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusUsecaseAuditEvent {
    pub kind: WorkflowEventBusUsecaseAuditEventKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                           // data_class: INTERNAL_ONLY
    pub event_type: Option<String>,                  // data_class: PUBLIC
    pub channel_address: Option<String>,             // data_class: PUBLIC
    pub evidence_refs: Vec<String>,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusUsecasePublishReceipt {
    pub status: WorkflowEventBusUsecaseStatus, // data_class: PUBLIC
    pub domain_status: Option<WorkflowEventBusDomainStatus>, // data_class: PUBLIC
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub cell_id: String,                       // data_class: INTERNAL_ONLY
    pub event_type: String,                    // data_class: PUBLIC
    pub channel_address: Option<String>,       // data_class: PUBLIC
    pub delivery_key: Option<String>,          // data_class: INTERNAL_ONLY
    pub domain_receipt: Option<WorkflowEventBusDomainPublishReceipt>, // data_class: INTERNAL_ONLY
    pub denial_ref: Option<String>,            // data_class: INTERNAL_ONLY
    pub audit_events: Vec<WorkflowEventBusUsecaseAuditEvent>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusUsecaseDeliveryReceipt {
    pub status: WorkflowEventBusUsecaseStatus, // data_class: PUBLIC
    pub domain_status: Option<WorkflowEventBusDomainStatus>, // data_class: PUBLIC
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub cell_id: String,                       // data_class: INTERNAL_ONLY
    pub event_type: String,                    // data_class: PUBLIC
    pub channel_address: String,               // data_class: PUBLIC
    pub consumer_ref: String,                  // data_class: INTERNAL_ONLY
    pub offset_ref: String,                    // data_class: INTERNAL_ONLY
    pub domain_receipt: Option<WorkflowEventBusDomainDeliveryReceipt>, // data_class: INTERNAL_ONLY
    pub denial_ref: Option<String>,            // data_class: INTERNAL_ONLY
    pub audit_events: Vec<WorkflowEventBusUsecaseAuditEvent>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkflowEventBusUsecaseIntent {
    fingerprint: String,
}

#[derive(Default, Debug)]
pub struct WorkflowEventBusUsecase {
    publish_receipts_by_idempotency_key: BTreeMap<
        String,
        (
            WorkflowEventBusUsecaseIntent,
            WorkflowEventBusUsecasePublishReceipt,
        ),
    >,
    delivery_receipts_by_idempotency_key: BTreeMap<
        String,
        (
            WorkflowEventBusUsecaseIntent,
            WorkflowEventBusUsecaseDeliveryReceipt,
        ),
    >,
}

impl WorkflowEventBusUsecase {
    pub fn publish(
        &mut self,
        command: WorkflowEventBusUsecasePublishCommand,
    ) -> WorkflowEventBusUsecasePublishReceipt {
        if let Some(receipt) = invalid_publish_receipt(&command) {
            return receipt;
        }

        let intent = WorkflowEventBusUsecaseIntent {
            fingerprint: publish_fingerprint(&command),
        };
        if let Some((existing_intent, existing_receipt)) = self
            .publish_receipts_by_idempotency_key
            .get(&command.idempotency_key)
        {
            if existing_intent == &intent {
                return existing_receipt.clone();
            }
            return publish_idempotency_conflict_receipt(&command);
        }

        let requested = publish_requested_event(&command);
        let receipt = match WorkflowEventBusDomain::authorize_publish(
            command.binding.clone(),
            command.intent.clone(),
        ) {
            Ok(domain_receipt) => publish_receipt_from_domain(&command, requested, domain_receipt),
            Err(error) => publish_domain_denied_receipt(&command, requested, error),
        };
        self.publish_receipts_by_idempotency_key
            .insert(command.idempotency_key, (intent, receipt.clone()));
        receipt
    }

    pub fn evaluate_delivery(
        &mut self,
        command: WorkflowEventBusUsecaseDeliveryCommand,
    ) -> WorkflowEventBusUsecaseDeliveryReceipt {
        if let Some(receipt) = invalid_delivery_receipt(&command) {
            return receipt;
        }

        let intent = WorkflowEventBusUsecaseIntent {
            fingerprint: delivery_fingerprint(&command),
        };
        if let Some((existing_intent, existing_receipt)) = self
            .delivery_receipts_by_idempotency_key
            .get(&command.idempotency_key)
        {
            if existing_intent == &intent {
                return existing_receipt.clone();
            }
            return delivery_idempotency_conflict_receipt(&command);
        }

        let requested = delivery_requested_event(&command);
        let receipt = match WorkflowEventBusDomain::evaluate_authorized_delivery(
            command.binding.clone(),
            command.subscription_intent.clone(),
            command.candidate.clone(),
        ) {
            Ok(domain_receipt) => delivery_receipt_from_domain(&command, requested, domain_receipt),
            Err(error) => delivery_domain_denied_receipt(&command, requested, error),
        };
        self.delivery_receipts_by_idempotency_key
            .insert(command.idempotency_key, (intent, receipt.clone()));
        receipt
    }
}

fn invalid_publish_receipt(
    command: &WorkflowEventBusUsecasePublishCommand,
) -> Option<WorkflowEventBusUsecasePublishReceipt> {
    let mut refs = collect_invalid_publish_refs(command);
    if refs.is_empty() {
        return None;
    }
    refs.push("workflow-event-bus-usecase:invalid-publish-input".to_owned());
    let refs = sorted_unique(refs);
    let tenant_id = safe_tenant_or_redacted(&command.binding.tenant_id);
    let cell_id = safe_ref_or_redacted(&command.binding.cell_id, "redacted:cell");
    let event_type = command.intent.event_kind.event_type().to_owned();
    let channel_address = Some(command.intent.event_kind.channel().address().to_owned());
    Some(WorkflowEventBusUsecasePublishReceipt {
        status: WorkflowEventBusUsecaseStatus::InvalidInput,
        domain_status: None,
        tenant_id: tenant_id.clone(),
        cell_id,
        event_type: event_type.clone(),
        channel_address: channel_address.clone(),
        delivery_key: None,
        domain_receipt: None,
        denial_ref: Some("workflow-event-bus-usecase:invalid-input".to_owned()),
        audit_events: vec![audit_event(
            WorkflowEventBusUsecaseAuditEventKind::InvalidInput,
            &tenant_id,
            Some(event_type),
            channel_address,
            refs.clone(),
        )],
        evidence_refs: refs,
        non_claim_refs: usecase_non_claim_refs(Vec::new()),
    })
}

fn invalid_delivery_receipt(
    command: &WorkflowEventBusUsecaseDeliveryCommand,
) -> Option<WorkflowEventBusUsecaseDeliveryReceipt> {
    let mut refs = collect_invalid_delivery_refs(command);
    if refs.is_empty() {
        return None;
    }
    refs.push("workflow-event-bus-usecase:invalid-delivery-input".to_owned());
    let refs = sorted_unique(refs);
    let tenant_id = safe_tenant_or_redacted(&command.binding.tenant_id);
    let cell_id = safe_ref_or_redacted(&command.binding.cell_id, "redacted:cell");
    let event_type =
        safe_metadata_or_redacted(&command.candidate.event_type, "redacted:event-type");
    let channel_address = command.candidate.channel.address().to_owned();
    let consumer_ref = safe_ref_or_redacted(
        &command.subscription_intent.consumer_ref,
        "redacted:consumer",
    );
    let offset_ref = safe_ref_or_redacted(&command.candidate.offset_ref, "redacted:offset");
    Some(WorkflowEventBusUsecaseDeliveryReceipt {
        status: WorkflowEventBusUsecaseStatus::InvalidInput,
        domain_status: None,
        tenant_id: tenant_id.clone(),
        cell_id,
        event_type: event_type.clone(),
        channel_address: channel_address.clone(),
        consumer_ref,
        offset_ref,
        domain_receipt: None,
        denial_ref: Some("workflow-event-bus-usecase:invalid-input".to_owned()),
        audit_events: vec![audit_event(
            WorkflowEventBusUsecaseAuditEventKind::InvalidInput,
            &tenant_id,
            Some(event_type),
            Some(channel_address),
            refs.clone(),
        )],
        evidence_refs: refs,
        non_claim_refs: usecase_non_claim_refs(Vec::new()),
    })
}

fn publish_idempotency_conflict_receipt(
    command: &WorkflowEventBusUsecasePublishCommand,
) -> WorkflowEventBusUsecasePublishReceipt {
    let refs = sorted_unique(vec![
        command.request_id.clone(),
        command.idempotency_key.clone(),
        "workflow-event-bus-usecase:publish-idempotency-conflict".to_owned(),
    ]);
    let tenant_id = safe_tenant_or_redacted(&command.binding.tenant_id);
    let event_type = command.intent.event_kind.event_type().to_owned();
    let channel_address = Some(command.intent.event_kind.channel().address().to_owned());
    WorkflowEventBusUsecasePublishReceipt {
        status: WorkflowEventBusUsecaseStatus::IdempotencyConflict,
        domain_status: None,
        tenant_id: tenant_id.clone(),
        cell_id: safe_ref_or_redacted(&command.binding.cell_id, "redacted:cell"),
        event_type: event_type.clone(),
        channel_address: channel_address.clone(),
        delivery_key: None,
        domain_receipt: None,
        denial_ref: Some("workflow-event-bus-usecase:publish-idempotency-conflict".to_owned()),
        audit_events: vec![audit_event(
            WorkflowEventBusUsecaseAuditEventKind::IdempotencyConflict,
            &tenant_id,
            Some(event_type),
            channel_address,
            refs.clone(),
        )],
        evidence_refs: refs,
        non_claim_refs: usecase_non_claim_refs(Vec::new()),
    }
}

fn delivery_idempotency_conflict_receipt(
    command: &WorkflowEventBusUsecaseDeliveryCommand,
) -> WorkflowEventBusUsecaseDeliveryReceipt {
    let refs = sorted_unique(vec![
        command.request_id.clone(),
        command.idempotency_key.clone(),
        "workflow-event-bus-usecase:delivery-idempotency-conflict".to_owned(),
    ]);
    let tenant_id = safe_tenant_or_redacted(&command.binding.tenant_id);
    let event_type =
        safe_metadata_or_redacted(&command.candidate.event_type, "redacted:event-type");
    let channel_address = command.candidate.channel.address().to_owned();
    WorkflowEventBusUsecaseDeliveryReceipt {
        status: WorkflowEventBusUsecaseStatus::IdempotencyConflict,
        domain_status: None,
        tenant_id: tenant_id.clone(),
        cell_id: safe_ref_or_redacted(&command.binding.cell_id, "redacted:cell"),
        event_type: event_type.clone(),
        channel_address: channel_address.clone(),
        consumer_ref: safe_ref_or_redacted(
            &command.subscription_intent.consumer_ref,
            "redacted:consumer",
        ),
        offset_ref: safe_ref_or_redacted(&command.candidate.offset_ref, "redacted:offset"),
        domain_receipt: None,
        denial_ref: Some("workflow-event-bus-usecase:delivery-idempotency-conflict".to_owned()),
        audit_events: vec![audit_event(
            WorkflowEventBusUsecaseAuditEventKind::IdempotencyConflict,
            &tenant_id,
            Some(event_type),
            Some(channel_address),
            refs.clone(),
        )],
        evidence_refs: refs,
        non_claim_refs: usecase_non_claim_refs(Vec::new()),
    }
}

fn publish_receipt_from_domain(
    command: &WorkflowEventBusUsecasePublishCommand,
    requested: WorkflowEventBusUsecaseAuditEvent,
    domain_receipt: WorkflowEventBusDomainPublishReceipt,
) -> WorkflowEventBusUsecasePublishReceipt {
    let refs = sorted_unique(
        [
            domain_receipt.evidence_refs.clone(),
            vec![
                command.request_id.clone(),
                command.idempotency_key.clone(),
                command.trace_ref.clone(),
                WORKFLOW_EVENT_BUS_USECASE_SURFACE.to_owned(),
                WORKFLOW_EVENT_BUS_USECASE_CONTRACT_REF.to_owned(),
                "workflow-event-bus-usecase:publish-accepted".to_owned(),
            ],
        ]
        .concat(),
    );
    let accepted = audit_event(
        WorkflowEventBusUsecaseAuditEventKind::PublishAccepted,
        &domain_receipt.tenant_id,
        Some(domain_receipt.event_type.clone()),
        domain_receipt.channel_address.clone(),
        refs.clone(),
    );
    WorkflowEventBusUsecasePublishReceipt {
        status: WorkflowEventBusUsecaseStatus::Published,
        domain_status: Some(domain_receipt.status),
        tenant_id: domain_receipt.tenant_id.clone(),
        cell_id: domain_receipt.cell_id.clone(),
        event_type: domain_receipt.event_type.clone(),
        channel_address: domain_receipt.channel_address.clone(),
        delivery_key: domain_receipt.delivery_key.clone(),
        domain_receipt: Some(domain_receipt.clone()),
        denial_ref: None,
        audit_events: vec![requested, accepted],
        evidence_refs: refs,
        non_claim_refs: usecase_non_claim_refs(domain_receipt.non_claim_refs),
    }
}

fn publish_domain_denied_receipt(
    command: &WorkflowEventBusUsecasePublishCommand,
    requested: WorkflowEventBusUsecaseAuditEvent,
    error: WorkflowEventBusDomainError,
) -> WorkflowEventBusUsecasePublishReceipt {
    let denial_ref = error.primary_evidence_ref().to_owned();
    let refs = sorted_unique(vec![
        command.request_id.clone(),
        command.idempotency_key.clone(),
        command.trace_ref.clone(),
        denial_ref.clone(),
        WORKFLOW_EVENT_BUS_USECASE_SURFACE.to_owned(),
        "workflow-event-bus-usecase:domain-denied".to_owned(),
    ]);
    let tenant_id = safe_tenant_or_redacted(&command.binding.tenant_id);
    let event_type = command.intent.event_kind.event_type().to_owned();
    let channel_address = Some(command.intent.event_kind.channel().address().to_owned());
    let denied = audit_event(
        WorkflowEventBusUsecaseAuditEventKind::DomainDenied,
        &tenant_id,
        Some(event_type.clone()),
        channel_address.clone(),
        refs.clone(),
    );
    WorkflowEventBusUsecasePublishReceipt {
        status: WorkflowEventBusUsecaseStatus::DomainDenied,
        domain_status: None,
        tenant_id,
        cell_id: safe_ref_or_redacted(&command.binding.cell_id, "redacted:cell"),
        event_type,
        channel_address,
        delivery_key: None,
        domain_receipt: None,
        denial_ref: Some(denial_ref),
        audit_events: vec![requested, denied],
        evidence_refs: refs,
        non_claim_refs: usecase_non_claim_refs(Vec::new()),
    }
}

fn delivery_receipt_from_domain(
    command: &WorkflowEventBusUsecaseDeliveryCommand,
    requested: WorkflowEventBusUsecaseAuditEvent,
    domain_receipt: WorkflowEventBusDomainDeliveryReceipt,
) -> WorkflowEventBusUsecaseDeliveryReceipt {
    let status = match domain_receipt.status {
        WorkflowEventBusDomainStatus::Accepted => WorkflowEventBusUsecaseStatus::DeliveryAccepted,
        WorkflowEventBusDomainStatus::Denied => WorkflowEventBusUsecaseStatus::DeliveryDenied,
    };
    let audit_kind = match domain_receipt.status {
        WorkflowEventBusDomainStatus::Accepted => {
            WorkflowEventBusUsecaseAuditEventKind::DeliveryAccepted
        }
        WorkflowEventBusDomainStatus::Denied => {
            WorkflowEventBusUsecaseAuditEventKind::DeliveryDenied
        }
    };
    let status_ref = match domain_receipt.status {
        WorkflowEventBusDomainStatus::Accepted => "workflow-event-bus-usecase:delivery-accepted",
        WorkflowEventBusDomainStatus::Denied => "workflow-event-bus-usecase:delivery-denied",
    };
    let refs = sorted_unique(
        [
            domain_receipt.evidence_refs.clone(),
            vec![
                command.request_id.clone(),
                command.idempotency_key.clone(),
                command.trace_ref.clone(),
                WORKFLOW_EVENT_BUS_USECASE_SURFACE.to_owned(),
                WORKFLOW_EVENT_BUS_USECASE_CONTRACT_REF.to_owned(),
                status_ref.to_owned(),
            ],
        ]
        .concat(),
    );
    let evaluated = audit_event(
        audit_kind,
        &domain_receipt.tenant_id,
        Some(domain_receipt.event_type.clone()),
        Some(domain_receipt.channel_address.clone()),
        refs.clone(),
    );
    WorkflowEventBusUsecaseDeliveryReceipt {
        status,
        domain_status: Some(domain_receipt.status),
        tenant_id: domain_receipt.tenant_id.clone(),
        cell_id: domain_receipt.cell_id.clone(),
        event_type: domain_receipt.event_type.clone(),
        channel_address: domain_receipt.channel_address.clone(),
        consumer_ref: domain_receipt.consumer_ref.clone(),
        offset_ref: domain_receipt.offset_ref.clone(),
        domain_receipt: Some(domain_receipt.clone()),
        denial_ref: None,
        audit_events: vec![requested, evaluated],
        evidence_refs: refs,
        non_claim_refs: usecase_non_claim_refs(domain_receipt.non_claim_refs),
    }
}

fn delivery_domain_denied_receipt(
    command: &WorkflowEventBusUsecaseDeliveryCommand,
    requested: WorkflowEventBusUsecaseAuditEvent,
    error: WorkflowEventBusDomainError,
) -> WorkflowEventBusUsecaseDeliveryReceipt {
    let denial_ref = error.primary_evidence_ref().to_owned();
    let refs = sorted_unique(vec![
        command.request_id.clone(),
        command.idempotency_key.clone(),
        command.trace_ref.clone(),
        denial_ref.clone(),
        WORKFLOW_EVENT_BUS_USECASE_SURFACE.to_owned(),
        "workflow-event-bus-usecase:domain-denied".to_owned(),
    ]);
    let tenant_id = safe_tenant_or_redacted(&command.binding.tenant_id);
    let event_type =
        safe_metadata_or_redacted(&command.candidate.event_type, "redacted:event-type");
    let channel_address = command.candidate.channel.address().to_owned();
    let denied = audit_event(
        WorkflowEventBusUsecaseAuditEventKind::DomainDenied,
        &tenant_id,
        Some(event_type.clone()),
        Some(channel_address.clone()),
        refs.clone(),
    );
    WorkflowEventBusUsecaseDeliveryReceipt {
        status: WorkflowEventBusUsecaseStatus::DomainDenied,
        domain_status: None,
        tenant_id,
        cell_id: safe_ref_or_redacted(&command.binding.cell_id, "redacted:cell"),
        event_type,
        channel_address,
        consumer_ref: safe_ref_or_redacted(
            &command.subscription_intent.consumer_ref,
            "redacted:consumer",
        ),
        offset_ref: safe_ref_or_redacted(&command.candidate.offset_ref, "redacted:offset"),
        domain_receipt: None,
        denial_ref: Some(denial_ref),
        audit_events: vec![requested, denied],
        evidence_refs: refs,
        non_claim_refs: usecase_non_claim_refs(Vec::new()),
    }
}

fn publish_requested_event(
    command: &WorkflowEventBusUsecasePublishCommand,
) -> WorkflowEventBusUsecaseAuditEvent {
    audit_event(
        WorkflowEventBusUsecaseAuditEventKind::PublishRequested,
        &command.binding.tenant_id,
        Some(command.intent.event_kind.event_type().to_owned()),
        Some(command.intent.event_kind.channel().address().to_owned()),
        sorted_unique(vec![
            command.request_id.clone(),
            command.idempotency_key.clone(),
            command.trace_ref.clone(),
            command.binding.authorization_evidence_ref.clone(),
            command.intent.correlation_ref.clone(),
        ]),
    )
}

fn delivery_requested_event(
    command: &WorkflowEventBusUsecaseDeliveryCommand,
) -> WorkflowEventBusUsecaseAuditEvent {
    audit_event(
        WorkflowEventBusUsecaseAuditEventKind::DeliveryRequested,
        &command.binding.tenant_id,
        Some(command.candidate.event_type.clone()),
        Some(command.candidate.channel.address().to_owned()),
        sorted_unique(vec![
            command.request_id.clone(),
            command.idempotency_key.clone(),
            command.trace_ref.clone(),
            command
                .subscription_intent
                .authorization_evidence_ref
                .clone(),
            command.candidate.offset_ref.clone(),
        ]),
    )
}

fn audit_event(
    kind: WorkflowEventBusUsecaseAuditEventKind,
    tenant_id: &str,
    event_type: Option<String>,
    channel_address: Option<String>,
    evidence_refs: Vec<String>,
) -> WorkflowEventBusUsecaseAuditEvent {
    WorkflowEventBusUsecaseAuditEvent {
        kind,
        tenant_id: tenant_id.to_owned(),
        event_type,
        channel_address,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn collect_invalid_publish_refs(command: &WorkflowEventBusUsecasePublishCommand) -> Vec<String> {
    let mut refs = Vec::new();
    push_invalid_ref(
        &mut refs,
        "validation:request-id-invalid",
        &command.request_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:idempotency-key-invalid",
        &command.idempotency_key,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trace-ref-invalid",
        &command.trace_ref,
    );
    collect_invalid_binding_refs(&mut refs, &command.binding);
    collect_invalid_publish_intent_refs(&mut refs, &command.intent);
    refs
}

fn collect_invalid_delivery_refs(command: &WorkflowEventBusUsecaseDeliveryCommand) -> Vec<String> {
    let mut refs = Vec::new();
    push_invalid_ref(
        &mut refs,
        "validation:request-id-invalid",
        &command.request_id,
    );
    push_invalid_ref(
        &mut refs,
        "validation:idempotency-key-invalid",
        &command.idempotency_key,
    );
    push_invalid_ref(
        &mut refs,
        "validation:trace-ref-invalid",
        &command.trace_ref,
    );
    collect_invalid_binding_refs(&mut refs, &command.binding);
    collect_invalid_subscription_intent_refs(&mut refs, &command.subscription_intent);
    collect_invalid_candidate_refs(&mut refs, &command.candidate);
    refs
}

fn collect_invalid_binding_refs(
    refs: &mut Vec<String>,
    binding: &WorkflowEventBusDomainPolicyBinding,
) {
    push_invalid_tenant(
        refs,
        "validation:binding-tenant-invalid",
        &binding.tenant_id,
    );
    for (label, value) in [
        ("validation:binding-cell-invalid", &binding.cell_id),
        (
            "validation:binding-principal-invalid",
            &binding.principal_id,
        ),
        (
            "validation:binding-authorization-decision-invalid",
            &binding.authorization_decision_id,
        ),
        (
            "validation:binding-authorization-evidence-invalid",
            &binding.authorization_evidence_ref,
        ),
        (
            "validation:binding-policy-bundle-invalid",
            &binding.policy_bundle_ref,
        ),
        (
            "validation:binding-residency-invalid",
            &binding.residency_ref,
        ),
        (
            "validation:binding-trace-context-invalid",
            &binding.trace_context_ref,
        ),
        (
            "validation:binding-audit-chain-invalid",
            &binding.audit_chain_ref,
        ),
    ] {
        push_invalid_ref(refs, label, value);
    }
    if binding.allowed_channels.is_empty() {
        refs.push("validation:binding-allowed-channels-empty".to_owned());
    }
    if binding.allowed_event_types.is_empty()
        || binding
            .allowed_event_types
            .iter()
            .any(|event_type| !is_safe_metadata(event_type))
    {
        refs.push("validation:binding-allowed-event-types-invalid".to_owned());
    }
}

fn collect_invalid_publish_intent_refs(
    refs: &mut Vec<String>,
    intent: &WorkflowEventBusDomainPublishIntent,
) {
    for (label, value) in [
        ("validation:publish-producer-invalid", &intent.producer_ref),
        ("validation:publish-event-id-invalid", &intent.event_id),
        ("validation:publish-source-invalid", &intent.source_ref),
        (
            "validation:publish-partition-key-invalid",
            &intent.partition_key_ref,
        ),
        (
            "validation:publish-idempotency-key-invalid",
            &intent.idempotency_key,
        ),
        (
            "validation:publish-causation-invalid",
            &intent.causation_ref,
        ),
        (
            "validation:publish-correlation-invalid",
            &intent.correlation_ref,
        ),
        (
            "validation:publish-payload-ref-invalid",
            &intent.payload_ref,
        ),
    ] {
        push_invalid_ref(refs, label, value);
    }
    push_invalid_optional_ref(
        refs,
        "validation:publish-subject-invalid",
        intent.subject_ref.as_deref(),
    );
    push_invalid_optional_ref(
        refs,
        "validation:publish-time-invalid",
        intent.time_ref.as_deref(),
    );
    push_invalid_optional_ref(
        refs,
        "validation:publish-dataschema-invalid",
        intent.dataschema_ref.as_deref(),
    );
    push_invalid_ref_vec(
        refs,
        "validation:publish-evidence-refs-invalid",
        &intent.evidence_refs,
    );
}

fn collect_invalid_subscription_intent_refs(
    refs: &mut Vec<String>,
    intent: &WorkflowEventBusDomainSubscriptionIntent,
) {
    push_invalid_ref(
        refs,
        "validation:consumer-ref-invalid",
        &intent.consumer_ref,
    );
    push_invalid_optional_ref(
        refs,
        "validation:replay-cursor-invalid",
        intent.replay_cursor_ref.as_deref(),
    );
    push_invalid_ref(
        refs,
        "validation:subscription-authorization-evidence-invalid",
        &intent.authorization_evidence_ref,
    );
    if intent.max_batch_size == 0
        || intent.max_batch_size > WORKFLOW_EVENT_BUS_MAX_SUBSCRIPTION_BATCH_SIZE
    {
        refs.push("validation:subscription-batch-size-invalid".to_owned());
    }
    if intent.allowed_event_types.is_empty()
        || intent
            .allowed_event_types
            .iter()
            .any(|event_type| !is_safe_metadata(event_type))
    {
        refs.push("validation:subscription-event-types-invalid".to_owned());
    }
}

fn collect_invalid_candidate_refs(
    refs: &mut Vec<String>,
    candidate: &WorkflowEventBusDeliveryCandidate,
) {
    push_invalid_tenant(
        refs,
        "validation:candidate-tenant-invalid",
        &candidate.tenant_id,
    );
    for (label, value) in [
        ("validation:candidate-cell-invalid", &candidate.cell_id),
        ("validation:candidate-event-id-invalid", &candidate.event_id),
        (
            "validation:candidate-idempotency-key-invalid",
            &candidate.idempotency_key,
        ),
        (
            "validation:candidate-payload-ref-invalid",
            &candidate.payload_ref,
        ),
        (
            "validation:candidate-offset-ref-invalid",
            &candidate.offset_ref,
        ),
    ] {
        push_invalid_ref(refs, label, value);
    }
    if !is_safe_metadata(&candidate.event_type) {
        refs.push("validation:candidate-event-type-invalid".to_owned());
    }
    push_invalid_ref_vec(
        refs,
        "validation:candidate-evidence-refs-invalid",
        &candidate.evidence_refs,
    );
}

fn publish_fingerprint(command: &WorkflowEventBusUsecasePublishCommand) -> String {
    let mut parts = common_command_fingerprint(
        command.request_id.as_str(),
        command.idempotency_key.as_str(),
        command.trace_ref.as_str(),
    );
    push_binding_fingerprint(&mut parts, &command.binding);
    let intent = &command.intent;
    push_fingerprint_value(
        &mut parts,
        "publish.event_kind",
        intent.event_kind.event_type(),
    );
    push_fingerprint_value(
        &mut parts,
        "publish.channel",
        intent.event_kind.channel().address(),
    );
    push_fingerprint_value(&mut parts, "publish.producer_ref", &intent.producer_ref);
    push_fingerprint_value(&mut parts, "publish.event_id", &intent.event_id);
    push_fingerprint_value(&mut parts, "publish.source_ref", &intent.source_ref);
    push_fingerprint_optional_ref(
        &mut parts,
        "publish.subject_ref",
        intent.subject_ref.as_deref(),
    );
    push_fingerprint_optional_ref(&mut parts, "publish.time_ref", intent.time_ref.as_deref());
    push_fingerprint_optional_ref(
        &mut parts,
        "publish.dataschema_ref",
        intent.dataschema_ref.as_deref(),
    );
    push_fingerprint_value(
        &mut parts,
        "publish.partition_key_ref",
        &intent.partition_key_ref,
    );
    push_fingerprint_value(
        &mut parts,
        "publish.idempotency_key",
        &intent.idempotency_key,
    );
    push_fingerprint_value(&mut parts, "publish.causation_ref", &intent.causation_ref);
    push_fingerprint_value(
        &mut parts,
        "publish.correlation_ref",
        &intent.correlation_ref,
    );
    push_fingerprint_value(&mut parts, "publish.payload_ref", &intent.payload_ref);
    push_fingerprint_sorted_refs(&mut parts, "publish.evidence_refs", &intent.evidence_refs);
    parts.join("\n")
}

fn delivery_fingerprint(command: &WorkflowEventBusUsecaseDeliveryCommand) -> String {
    let mut parts = common_command_fingerprint(
        command.request_id.as_str(),
        command.idempotency_key.as_str(),
        command.trace_ref.as_str(),
    );
    push_binding_fingerprint(&mut parts, &command.binding);
    let subscription = &command.subscription_intent;
    push_fingerprint_value(
        &mut parts,
        "subscription.consumer_ref",
        &subscription.consumer_ref,
    );
    push_fingerprint_value(
        &mut parts,
        "subscription.channel",
        subscription.channel.address(),
    );
    push_fingerprint_sorted_refs(
        &mut parts,
        "subscription.allowed_event_types",
        &subscription.allowed_event_types,
    );
    push_fingerprint_optional_ref(
        &mut parts,
        "subscription.replay_cursor_ref",
        subscription.replay_cursor_ref.as_deref(),
    );
    push_fingerprint_u64(
        &mut parts,
        "subscription.max_batch_size",
        u64::from(subscription.max_batch_size),
    );
    push_fingerprint_value(
        &mut parts,
        "subscription.authorization_evidence_ref",
        &subscription.authorization_evidence_ref,
    );

    let candidate = &command.candidate;
    push_fingerprint_value(&mut parts, "candidate.tenant_id", &candidate.tenant_id);
    push_fingerprint_value(&mut parts, "candidate.cell_id", &candidate.cell_id);
    push_fingerprint_value(&mut parts, "candidate.channel", candidate.channel.address());
    push_fingerprint_value(&mut parts, "candidate.event_id", &candidate.event_id);
    push_fingerprint_value(&mut parts, "candidate.event_type", &candidate.event_type);
    push_fingerprint_value(
        &mut parts,
        "candidate.idempotency_key",
        &candidate.idempotency_key,
    );
    push_fingerprint_value(&mut parts, "candidate.payload_ref", &candidate.payload_ref);
    push_fingerprint_value(&mut parts, "candidate.offset_ref", &candidate.offset_ref);
    push_fingerprint_sorted_refs(
        &mut parts,
        "candidate.evidence_refs",
        &candidate.evidence_refs,
    );
    parts.join("\n")
}

fn common_command_fingerprint(
    request_id: &str,
    idempotency_key: &str,
    trace_ref: &str,
) -> Vec<String> {
    let mut parts = Vec::new();
    push_fingerprint_value(&mut parts, "usecase.request_id", request_id);
    push_fingerprint_value(&mut parts, "usecase.idempotency_key", idempotency_key);
    push_fingerprint_value(&mut parts, "usecase.trace_ref", trace_ref);
    parts
}

fn push_binding_fingerprint(
    parts: &mut Vec<String>,
    binding: &WorkflowEventBusDomainPolicyBinding,
) {
    push_fingerprint_value(parts, "binding.tenant_id", &binding.tenant_id);
    push_fingerprint_value(parts, "binding.cell_id", &binding.cell_id);
    push_fingerprint_value(parts, "binding.principal_id", &binding.principal_id);
    push_fingerprint_value(
        parts,
        "binding.authorization_decision_id",
        &binding.authorization_decision_id,
    );
    push_fingerprint_value(
        parts,
        "binding.authorization_evidence_ref",
        &binding.authorization_evidence_ref,
    );
    push_fingerprint_value(
        parts,
        "binding.policy_bundle_ref",
        &binding.policy_bundle_ref,
    );
    push_fingerprint_value(parts, "binding.residency_ref", &binding.residency_ref);
    push_fingerprint_value(
        parts,
        "binding.trace_context_ref",
        &binding.trace_context_ref,
    );
    push_fingerprint_value(parts, "binding.audit_chain_ref", &binding.audit_chain_ref);
    let mut channels: Vec<String> = binding
        .allowed_channels
        .iter()
        .map(|channel| channel.address().to_owned())
        .collect();
    channels.sort();
    push_fingerprint_sorted_refs(parts, "binding.allowed_channels", &channels);
    push_fingerprint_sorted_refs(
        parts,
        "binding.allowed_event_types",
        &binding.allowed_event_types,
    );
}

fn usecase_non_claim_refs(mut domain_non_claims: Vec<String>) -> Vec<String> {
    domain_non_claims.extend([
        "workflow-event-bus-usecase:no-concrete-idempotency-store".to_owned(),
        "workflow-event-bus-usecase:no-broker-runtime".to_owned(),
        "workflow-event-bus-usecase:no-durable-outbox".to_owned(),
        "workflow-event-bus-usecase:no-durable-inbox".to_owned(),
        "workflow-event-bus-usecase:no-consumer-group-runtime".to_owned(),
        "workflow-event-bus-usecase:no-offset-commit-runtime".to_owned(),
        "workflow-event-bus-usecase:no-cloud-deployment".to_owned(),
        "workflow-event-bus-usecase:no-tenant-workload-scheduling".to_owned(),
        "workflow-event-bus-usecase:no-hyperscaler-claim".to_owned(),
    ]);
    sorted_unique(domain_non_claims)
}

fn safe_tenant_or_redacted(value: &str) -> String {
    if is_safe_tenant(value) {
        value.to_owned()
    } else {
        "redacted:tenant".to_owned()
    }
}

fn safe_ref_or_redacted(value: &str, redacted: &str) -> String {
    if is_safe_ref(value) {
        value.to_owned()
    } else {
        redacted.to_owned()
    }
}

fn safe_metadata_or_redacted(value: &str, redacted: &str) -> String {
    if is_safe_metadata(value) {
        value.to_owned()
    } else {
        redacted.to_owned()
    }
}

fn push_invalid_tenant(refs: &mut Vec<String>, label: &str, value: &str) {
    if !is_safe_tenant(value) {
        refs.push(label.to_owned());
    }
}

fn push_invalid_ref(refs: &mut Vec<String>, label: &str, value: &str) {
    if !is_safe_ref(value) {
        refs.push(label.to_owned());
    }
}

fn push_invalid_optional_ref(refs: &mut Vec<String>, label: &str, value: Option<&str>) {
    if !is_safe_optional_ref(value) {
        refs.push(label.to_owned());
    }
}

fn push_invalid_ref_vec(refs: &mut Vec<String>, label: &str, values: &[String]) {
    if !values.iter().all(|value| is_safe_ref(value)) {
        refs.push(label.to_owned());
    }
}

fn push_fingerprint_value(parts: &mut Vec<String>, field: &str, value: &str) {
    parts.push(format!("{field}={}:{}", value.len(), value));
}

fn push_fingerprint_u64(parts: &mut Vec<String>, field: &str, value: u64) {
    push_fingerprint_value(parts, field, &value.to_string());
}

fn push_fingerprint_optional_ref(parts: &mut Vec<String>, field: &str, value: Option<&str>) {
    match value {
        Some(value) => {
            push_fingerprint_value(parts, &format!("{field}.present"), "true");
            push_fingerprint_value(parts, field, value);
        }
        None => push_fingerprint_value(parts, &format!("{field}.present"), "false"),
    }
}

fn push_fingerprint_sorted_refs(parts: &mut Vec<String>, field: &str, values: &[String]) {
    let mut sorted = values.to_vec();
    sorted.sort();
    push_fingerprint_u64(parts, &format!("{field}.count"), sorted.len() as u64);
    for (index, value) in sorted.iter().enumerate() {
        push_fingerprint_value(parts, &format!("{field}[{index}]"), value);
    }
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
    values.retain(|value| {
        !value.trim().is_empty()
            && !contains_raw_secret_material(value)
            && !contains_raw_content_material(value)
    });
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn usecase_status_and_audit_labels_are_unique() {
        let statuses = [
            WorkflowEventBusUsecaseStatus::DeliveryAccepted,
            WorkflowEventBusUsecaseStatus::DeliveryDenied,
            WorkflowEventBusUsecaseStatus::DomainDenied,
            WorkflowEventBusUsecaseStatus::IdempotencyConflict,
            WorkflowEventBusUsecaseStatus::InvalidInput,
            WorkflowEventBusUsecaseStatus::Published,
        ];
        let labels: Vec<&str> = statuses.iter().map(|status| status.as_wire()).collect();
        let unique: BTreeSet<&str> = labels.iter().copied().collect();
        assert_eq!(labels.len(), unique.len());

        let audit_kinds = [
            WorkflowEventBusUsecaseAuditEventKind::DeliveryAccepted,
            WorkflowEventBusUsecaseAuditEventKind::DeliveryDenied,
            WorkflowEventBusUsecaseAuditEventKind::DeliveryRequested,
            WorkflowEventBusUsecaseAuditEventKind::DomainDenied,
            WorkflowEventBusUsecaseAuditEventKind::IdempotencyConflict,
            WorkflowEventBusUsecaseAuditEventKind::InvalidInput,
            WorkflowEventBusUsecaseAuditEventKind::PublishAccepted,
            WorkflowEventBusUsecaseAuditEventKind::PublishRequested,
        ];
        let audit_labels: Vec<&str> = audit_kinds.iter().map(|kind| kind.as_wire()).collect();
        let audit_unique: BTreeSet<&str> = audit_labels.iter().copied().collect();
        assert_eq!(audit_labels.len(), audit_unique.len());
    }

    #[test]
    fn publish_authorizes_domain_and_replays_same_idempotency_key_without_broker_claims() {
        let mut usecase = WorkflowEventBusUsecase::default();
        let command = publish_command(WorkflowEventBusEventKind::WorkflowRunStarted);

        let first = usecase.publish(command.clone());
        let second = usecase.publish(command);

        assert_eq!(first, second);
        assert_eq!(first.status, WorkflowEventBusUsecaseStatus::Published);
        assert_eq!(
            first.domain_status,
            Some(WorkflowEventBusDomainStatus::Accepted)
        );
        assert_eq!(
            first.channel_address.as_deref(),
            Some("workflow.runs.events.v1")
        );
        assert!(
            first
                .delivery_key
                .as_deref()
                .unwrap()
                .contains("workflow.runs.events.v1")
        );
        assert!(first.domain_receipt.is_some());
        assert!(
            first
                .evidence_refs
                .contains(&"workflow-event-bus-usecase:publish-accepted".to_owned())
        );
        assert!(
            first
                .non_claim_refs
                .contains(&"workflow-event-bus-usecase:no-broker-runtime".to_owned())
        );
        assert!(
            first
                .non_claim_refs
                .contains(&"workflow-event-bus-usecase:no-hyperscaler-claim".to_owned())
        );
        assert_eq!(first.audit_events.len(), 2);
    }

    #[test]
    fn publish_conflict_detects_drift_before_replacing_original_receipt() {
        let mut usecase = WorkflowEventBusUsecase::default();
        let original = publish_command(WorkflowEventBusEventKind::WorkflowRunStarted);
        let first = usecase.publish(original.clone());
        let mut changed = original.clone();
        changed.intent.event_id = "event:workflow-run-started:drift".to_owned();

        let conflict = usecase.publish(changed);
        let replay = usecase.publish(original);

        assert_eq!(
            conflict.status,
            WorkflowEventBusUsecaseStatus::IdempotencyConflict
        );
        assert_eq!(
            conflict.denial_ref.as_deref(),
            Some("workflow-event-bus-usecase:publish-idempotency-conflict")
        );
        assert!(conflict.domain_receipt.is_none());
        assert_eq!(replay, first);
    }

    #[test]
    fn raw_publish_metadata_is_rejected_without_echoing_payload_or_secret() {
        let mut usecase = WorkflowEventBusUsecase::default();
        let mut command = publish_command(WorkflowEventBusEventKind::WorkflowRunStarted);
        command.intent.payload_ref = "raw payload bearer sk-test customer message".to_owned();

        let receipt = usecase.publish(command);
        let rendered = format!("{receipt:?}");

        assert_eq!(receipt.status, WorkflowEventBusUsecaseStatus::InvalidInput);
        assert!(receipt.domain_receipt.is_none());
        assert!(
            receipt
                .evidence_refs
                .contains(&"validation:publish-payload-ref-invalid".to_owned())
        );
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("raw payload"));
    }

    #[test]
    fn delivery_accepts_subscription_and_preserves_domain_receipt_with_idempotent_replay() {
        let mut usecase = WorkflowEventBusUsecase::default();
        let command = delivery_command(WorkflowEventBusEventKind::WorkflowStateTransitioned);

        let first = usecase.evaluate_delivery(command.clone());
        let second = usecase.evaluate_delivery(command);

        assert_eq!(first, second);
        assert_eq!(
            first.status,
            WorkflowEventBusUsecaseStatus::DeliveryAccepted
        );
        assert_eq!(
            first.domain_status,
            Some(WorkflowEventBusDomainStatus::Accepted)
        );
        assert_eq!(first.channel_address, "workflow.state.events.v1");
        assert_eq!(first.consumer_ref, "consumer:workflow-state-machine");
        assert!(first.domain_receipt.is_some());
        assert!(
            first
                .evidence_refs
                .contains(&"workflow-event-bus-usecase:delivery-accepted".to_owned())
        );
        assert!(
            first
                .non_claim_refs
                .contains(&"workflow-event-bus-usecase:no-consumer-group-runtime".to_owned())
        );
    }

    #[test]
    fn delivery_denied_by_kernel_maps_to_delivery_denied_without_offset_commit_claim() {
        let mut usecase = WorkflowEventBusUsecase::default();
        let command = delivery_command(WorkflowEventBusEventKind::WorkflowRunStarted);

        let receipt = usecase.evaluate_delivery(command);

        assert_eq!(
            receipt.status,
            WorkflowEventBusUsecaseStatus::DeliveryDenied
        );
        assert_eq!(
            receipt.domain_status,
            Some(WorkflowEventBusDomainStatus::Denied)
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"workflow-event-bus-kernel:channel-not-subscribed".to_owned())
        );
        assert!(
            receipt
                .non_claim_refs
                .contains(&"workflow-event-bus-usecase:no-offset-commit-runtime".to_owned())
        );
        assert!(receipt.denial_ref.is_none());
    }

    #[test]
    fn delivery_domain_authorization_denial_is_fail_closed_and_cached_for_replay() {
        let mut usecase = WorkflowEventBusUsecase::default();
        let mut command = delivery_command(WorkflowEventBusEventKind::WorkflowStateTransitioned);
        command.subscription_intent.allowed_event_types = vec![
            WorkflowEventBusEventKind::WorkflowStepDispatched
                .event_type()
                .to_owned(),
        ];

        let first = usecase.evaluate_delivery(command.clone());
        let second = usecase.evaluate_delivery(command);

        assert_eq!(first, second);
        assert_eq!(first.status, WorkflowEventBusUsecaseStatus::DomainDenied);
        assert_eq!(
            first.denial_ref.as_deref(),
            Some("workflow-event-bus-domain:subscription-not-authorized")
        );
        assert!(first.domain_receipt.is_none());
        assert!(
            first
                .evidence_refs
                .contains(&"workflow-event-bus-usecase:domain-denied".to_owned())
        );
    }

    fn binding() -> WorkflowEventBusDomainPolicyBinding {
        WorkflowEventBusDomainPolicyBinding {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            principal_id: "principal:workflow-runtime".to_owned(),
            authorization_decision_id: "policy-decision:event-bus-allow".to_owned(),
            authorization_evidence_ref: "policy-evidence:event-bus-allow".to_owned(),
            policy_bundle_ref: "policy-bundle:event-bus-v1".to_owned(),
            residency_ref: "residency:us:data-plane".to_owned(),
            trace_context_ref: "trace:event-bus-usecase:root".to_owned(),
            audit_chain_ref: "audit-chain:event-bus-usecase".to_owned(),
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

    fn publish_command(kind: WorkflowEventBusEventKind) -> WorkflowEventBusUsecasePublishCommand {
        WorkflowEventBusUsecasePublishCommand {
            request_id: "request:event-bus-usecase:publish:001".to_owned(),
            idempotency_key: "idem:event-bus-usecase:publish:001".to_owned(),
            trace_ref: "trace:event-bus-usecase:publish".to_owned(),
            binding: binding(),
            intent: WorkflowEventBusDomainPublishIntent {
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
                evidence_refs: vec!["evidence:event-bus-usecase:publish".to_owned()],
            },
        }
    }

    fn delivery_command(kind: WorkflowEventBusEventKind) -> WorkflowEventBusUsecaseDeliveryCommand {
        WorkflowEventBusUsecaseDeliveryCommand {
            request_id: "request:event-bus-usecase:delivery:001".to_owned(),
            idempotency_key: "idem:event-bus-usecase:delivery:001".to_owned(),
            trace_ref: "trace:event-bus-usecase:delivery".to_owned(),
            binding: binding(),
            subscription_intent: WorkflowEventBusDomainSubscriptionIntent {
                consumer_ref: "consumer:workflow-state-machine".to_owned(),
                channel: WorkflowEventBusChannel::WorkflowState,
                allowed_event_types: vec![
                    WorkflowEventBusEventKind::WorkflowStateTransitioned
                        .event_type()
                        .to_owned(),
                ],
                replay_cursor_ref: Some("cursor:event-bus-usecase:state".to_owned()),
                max_batch_size: 100,
                authorization_evidence_ref: "authz:event-bus-usecase:consume".to_owned(),
            },
            candidate: WorkflowEventBusDeliveryCandidate {
                tenant_id: "ten_workflow_event_bus".to_owned(),
                cell_id: "cell:us-east-1a".to_owned(),
                channel: kind.channel(),
                event_id: "event:workflow-state:001".to_owned(),
                event_type: kind.event_type().to_owned(),
                idempotency_key: "idem:event-bus-domain:delivery:001".to_owned(),
                payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
                offset_ref: "offset:partition-0:42".to_owned(),
                evidence_refs: vec!["evidence:event-bus-usecase:delivery".to_owned()],
            },
        }
    }
}
