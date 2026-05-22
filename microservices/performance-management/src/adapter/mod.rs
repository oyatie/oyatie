pub mod http {
    use crate::domain::{EmployeeId, FeedbackVisibility, RatingBand, ReviewCycleId, TenantId};
    use crate::error::ServiceResult;
    use crate::usecase::{
        OpenReviewCycleCommand, PerformanceManagementService, PerformancePorts,
        SealReviewEvidenceCommand, SubmitFeedbackCommand, UsecaseReceipt,
    };

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub enum HttpMethod {
        Get,
        Post,
        Put,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct RouteDescriptor {
        pub method: HttpMethod,
        pub path: &'static str,
        pub capability: &'static str,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct OpenReviewCycleHttpRequest {
        pub tenant_id: String,
        pub review_cycle_id: String,
        pub title: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct SubmitFeedbackHttpRequest {
        pub tenant_id: String,
        pub review_cycle_id: String,
        pub subject_employee_id: String,
        pub author_employee_id: String,
        pub narrative: String,
        pub rating_band: Option<RatingBand>,
    }

    pub struct PerformanceHttpHandler;

    impl PerformanceHttpHandler {
        pub fn routes() -> Vec<RouteDescriptor> {
            vec![
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/review-cycles",
                    capability: "performance.goal_cycle.open",
                },
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/review-cycles/{review_cycle_id}/feedback",
                    capability: "performance.feedback.submit",
                },
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/review-cycles/{review_cycle_id}/seal",
                    capability: "performance.evidence.seal",
                },
            ]
        }

        pub fn open_review_cycle(
            service: &mut PerformanceManagementService<impl PerformancePorts>,
            request: OpenReviewCycleHttpRequest,
        ) -> ServiceResult<UsecaseReceipt> {
            service.open_review_cycle(OpenReviewCycleCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                review_cycle_id: ReviewCycleId::parse(request.review_cycle_id)?,
                title: request.title,
            })
        }

        pub fn submit_feedback(
            service: &mut PerformanceManagementService<impl PerformancePorts>,
            request: SubmitFeedbackHttpRequest,
        ) -> ServiceResult<UsecaseReceipt> {
            service.submit_feedback(SubmitFeedbackCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                review_cycle_id: ReviewCycleId::parse(request.review_cycle_id)?,
                subject_employee_id: EmployeeId::parse(request.subject_employee_id)?,
                author_employee_id: EmployeeId::parse(request.author_employee_id)?,
                visibility: FeedbackVisibility::ManagerOnly,
                rating_band: request.rating_band,
                narrative: request.narrative,
            })
        }

        pub fn seal_review_cycle(
            service: &mut PerformanceManagementService<impl PerformancePorts>,
            tenant_id: String,
            review_cycle_id: String,
            sealed_by: String,
        ) -> ServiceResult<UsecaseReceipt> {
            service.seal_review_evidence(SealReviewEvidenceCommand {
                tenant_id: TenantId::parse(tenant_id)?,
                review_cycle_id: ReviewCycleId::parse(review_cycle_id)?,
                sealed_by: EmployeeId::parse(sealed_by)?,
            })
        }
    }
}

pub mod grpc {
    use crate::domain::{ReviewCycleId, TenantId};
    use crate::error::ServiceResult;
    use crate::usecase::{
        OpenReviewCycleCommand, PerformanceManagementService, PerformancePorts, UsecaseReceipt,
    };

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct ReviewCycleGrpcRequest {
        pub tenant_id: String,
        pub review_cycle_id: String,
        pub title: String,
        pub request_id: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct ReviewCycleGrpcResponse {
        pub tenant_id: String,
        pub review_cycle_id: String,
        pub status: String,
        pub audit_event: String,
    }

    pub struct PerformanceGrpcHandler;

    impl PerformanceGrpcHandler {
        pub fn open_review_cycle(
            service: &mut PerformanceManagementService<impl PerformancePorts>,
            request: ReviewCycleGrpcRequest,
        ) -> ServiceResult<ReviewCycleGrpcResponse> {
            let receipt = service.open_review_cycle(OpenReviewCycleCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                review_cycle_id: ReviewCycleId::parse(request.review_cycle_id)?,
                title: request.title,
            })?;
            Ok(Self::response_from_receipt(receipt))
        }

        fn response_from_receipt(receipt: UsecaseReceipt) -> ReviewCycleGrpcResponse {
            ReviewCycleGrpcResponse {
                tenant_id: receipt.tenant_id.as_str().to_owned(),
                review_cycle_id: receipt.review_cycle_id.as_str().to_owned(),
                status: format!("{:?}", receipt.status),
                audit_event: format!("{:?}", receipt.audit_event),
            }
        }
    }
}

pub mod asyncapi {
    use crate::domain::{AuditEventKind, ReviewCycleId, TenantId};
    use crate::error::ServiceResult;

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct ReviewCycleOpenedEvent {
        pub tenant_id: TenantId,
        pub review_cycle_id: ReviewCycleId,
        pub audit_event: AuditEventKind,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct FeedbackSubmittedEvent {
        pub tenant_id: TenantId,
        pub review_cycle_id: ReviewCycleId,
        pub evidence_count: u32,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct CalibrationCompletedEvent {
        pub tenant_id: TenantId,
        pub review_cycle_id: ReviewCycleId,
        pub distribution_locked: bool,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct PublishedMessage {
        pub topic: String,
        pub payload_json: String,
    }

    pub struct PerformanceAsyncApiHandler;

    impl PerformanceAsyncApiHandler {
        pub fn review_cycle_opened(
            prefix: &str,
            event: ReviewCycleOpenedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.review_cycle.opened"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn feedback_submitted(
            prefix: &str,
            event: FeedbackSubmittedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.feedback.submitted"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn calibration_completed(
            prefix: &str,
            event: CalibrationCompletedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.calibration.completed"),
                payload_json: serde_json::to_string(&event)?,
            })
        }
    }
}

pub mod memory {
    use std::collections::BTreeMap;

    use crate::domain::{AuditEventKind, Capability, ReviewCycle, ReviewCycleId, TenantId};
    use crate::error::{ServiceError, ServiceResult};
    use crate::usecase::{AuditPublisher, PolicyAuthorizer, ReviewCycleRepository};

    #[derive(Clone, Debug, Default)]
    pub struct InMemoryPerformancePorts {
        review_cycles: BTreeMap<String, ReviewCycle>,
        audit_events: Vec<String>,
        denied_capabilities: Vec<Capability>,
    }

    impl InMemoryPerformancePorts {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn deny(mut self, capability: Capability) -> Self {
            self.denied_capabilities.push(capability);
            self
        }

        pub fn audit_events(&self) -> &[String] {
            &self.audit_events
        }

        fn key(tenant_id: &TenantId, review_cycle_id: &ReviewCycleId) -> String {
            format!("{}::{}", tenant_id.as_str(), review_cycle_id.as_str())
        }
    }

    impl ReviewCycleRepository for InMemoryPerformancePorts {
        fn put_review_cycle(&mut self, cycle: ReviewCycle) -> ServiceResult<ReviewCycle> {
            let key = Self::key(&cycle.tenant_id, &cycle.review_cycle_id);
            self.review_cycles.insert(key, cycle.clone());
            Ok(cycle)
        }

        fn get_review_cycle(
            &self,
            tenant_id: &TenantId,
            review_cycle_id: &ReviewCycleId,
        ) -> ServiceResult<Option<ReviewCycle>> {
            Ok(self
                .review_cycles
                .get(&Self::key(tenant_id, review_cycle_id))
                .cloned())
        }
    }

    impl PolicyAuthorizer for InMemoryPerformancePorts {
        fn authorize(&self, _tenant_id: &TenantId, capability: Capability) -> ServiceResult<()> {
            if self.denied_capabilities.contains(&capability) {
                Err(ServiceError::policy_denied(
                    capability.action_slug(),
                    "capability denied by in-memory policy",
                ))
            } else {
                Ok(())
            }
        }
    }

    impl AuditPublisher for InMemoryPerformancePorts {
        fn publish_audit(
            &mut self,
            tenant_id: &TenantId,
            event_kind: AuditEventKind,
            subject: &str,
        ) -> ServiceResult<()> {
            self.audit_events
                .push(format!("{}::{event_kind:?}::{subject}", tenant_id.as_str()));
            Ok(())
        }
    }
}
