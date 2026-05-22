pub mod http {
    use crate::domain::{CourseId, EnrollmentId, LearnerId, TenantId};
    use crate::error::ServiceResult;
    use crate::usecase::{
        LearningManagementService, LearningPorts, OpenEnrollmentCommand, RecordProgressCommand,
        SealCourseCompletionCommand, UsecaseReceipt,
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
    pub struct OpenEnrollmentHttpRequest {
        pub tenant_id: String,
        pub enrollment_id: String,
        pub course_id: String,
        pub title: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct RecordProgressHttpRequest {
        pub tenant_id: String,
        pub enrollment_id: String,
        pub learner_id: String,
        pub progress_percent: u8,
    }

    pub struct LearningHttpHandler;

    impl LearningHttpHandler {
        pub fn routes() -> Vec<RouteDescriptor> {
            vec![
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/enrollments",
                    capability: "learning.course.assign",
                },
                RouteDescriptor {
                    method: HttpMethod::Put,
                    path: "/v1/enrollments/{enrollment_id}/progress",
                    capability: "learning.progress.record",
                },
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/enrollments/{enrollment_id}/certificate",
                    capability: "learning.certificate.seal",
                },
            ]
        }

        pub fn open_enrollment(
            service: &mut LearningManagementService<impl LearningPorts>,
            request: OpenEnrollmentHttpRequest,
        ) -> ServiceResult<UsecaseReceipt> {
            service.open_enrollment(OpenEnrollmentCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                enrollment_id: EnrollmentId::parse(request.enrollment_id)?,
                course_id: CourseId::parse(request.course_id)?,
                title: request.title,
            })
        }

        pub fn record_progress(
            service: &mut LearningManagementService<impl LearningPorts>,
            request: RecordProgressHttpRequest,
        ) -> ServiceResult<UsecaseReceipt> {
            service.record_progress(RecordProgressCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                enrollment_id: EnrollmentId::parse(request.enrollment_id)?,
                learner_id: LearnerId::parse(request.learner_id)?,
                progress_percent: request.progress_percent,
            })
        }

        pub fn seal_course_completion(
            service: &mut LearningManagementService<impl LearningPorts>,
            tenant_id: String,
            enrollment_id: String,
            sealed_by: String,
        ) -> ServiceResult<UsecaseReceipt> {
            service.seal_course_completion(SealCourseCompletionCommand {
                tenant_id: TenantId::parse(tenant_id)?,
                enrollment_id: EnrollmentId::parse(enrollment_id)?,
                sealed_by: LearnerId::parse(sealed_by)?,
            })
        }
    }
}

pub mod grpc {
    use crate::domain::{CourseId, EnrollmentId, TenantId};
    use crate::error::ServiceResult;
    use crate::usecase::{
        LearningManagementService, LearningPorts, OpenEnrollmentCommand, UsecaseReceipt,
    };

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct LearningPathGrpcRequest {
        pub tenant_id: String,
        pub enrollment_id: String,
        pub course_id: String,
        pub title: String,
        pub request_id: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct LearningPathGrpcResponse {
        pub tenant_id: String,
        pub enrollment_id: String,
        pub status: String,
        pub audit_event: String,
    }

    pub struct LearningGrpcHandler;

    impl LearningGrpcHandler {
        pub fn open_enrollment(
            service: &mut LearningManagementService<impl LearningPorts>,
            request: LearningPathGrpcRequest,
        ) -> ServiceResult<LearningPathGrpcResponse> {
            let receipt = service.open_enrollment(OpenEnrollmentCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                enrollment_id: EnrollmentId::parse(request.enrollment_id)?,
                course_id: CourseId::parse(request.course_id)?,
                title: request.title,
            })?;
            Ok(Self::response_from_receipt(receipt))
        }

        fn response_from_receipt(receipt: UsecaseReceipt) -> LearningPathGrpcResponse {
            LearningPathGrpcResponse {
                tenant_id: receipt.tenant_id.as_str().to_owned(),
                enrollment_id: receipt.enrollment_id.as_str().to_owned(),
                status: format!("{:?}", receipt.status),
                audit_event: format!("{:?}", receipt.audit_event),
            }
        }
    }
}

pub mod asyncapi {
    use crate::domain::{AuditEventKind, EnrollmentId, TenantId};
    use crate::error::ServiceResult;

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct EnrollmentOpenedEvent {
        pub tenant_id: TenantId,
        pub enrollment_id: EnrollmentId,
        pub audit_event: AuditEventKind,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct ProgressRecordedEvent {
        pub tenant_id: TenantId,
        pub enrollment_id: EnrollmentId,
        pub progress_percent: u8,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct CourseCompletedEvent {
        pub tenant_id: TenantId,
        pub enrollment_id: EnrollmentId,
        pub certificate_ref: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct PublishedMessage {
        pub topic: String,
        pub payload_json: String,
    }

    pub struct LearningAsyncApiHandler;

    impl LearningAsyncApiHandler {
        pub fn enrollment_opened(
            prefix: &str,
            event: EnrollmentOpenedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.enrollment.opened"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn progress_recorded(
            prefix: &str,
            event: ProgressRecordedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.progress.recorded"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn course_completed(
            prefix: &str,
            event: CourseCompletedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.course.completed"),
                payload_json: serde_json::to_string(&event)?,
            })
        }
    }
}

pub mod memory {
    use std::collections::BTreeMap;

    use crate::domain::{AuditEventKind, Capability, EnrollmentId, LearningPath, TenantId};
    use crate::error::{ServiceError, ServiceResult};
    use crate::usecase::{AuditPublisher, LearningPathRepository, PolicyAuthorizer};

    #[derive(Clone, Debug, Default)]
    pub struct InMemoryLearningPorts {
        learning_paths: BTreeMap<String, LearningPath>,
        audit_events: Vec<String>,
        denied_capabilities: Vec<Capability>,
    }

    impl InMemoryLearningPorts {
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

        fn key(tenant_id: &TenantId, enrollment_id: &EnrollmentId) -> String {
            format!("{}::{}", tenant_id.as_str(), enrollment_id.as_str())
        }
    }

    impl LearningPathRepository for InMemoryLearningPorts {
        fn put_learning_path(&mut self, path: LearningPath) -> ServiceResult<LearningPath> {
            let key = Self::key(&path.tenant_id, &path.enrollment_id);
            self.learning_paths.insert(key, path.clone());
            Ok(path)
        }

        fn get_learning_path(
            &self,
            tenant_id: &TenantId,
            enrollment_id: &EnrollmentId,
        ) -> ServiceResult<Option<LearningPath>> {
            Ok(self
                .learning_paths
                .get(&Self::key(tenant_id, enrollment_id))
                .cloned())
        }
    }

    impl PolicyAuthorizer for InMemoryLearningPorts {
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

    impl AuditPublisher for InMemoryLearningPorts {
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
