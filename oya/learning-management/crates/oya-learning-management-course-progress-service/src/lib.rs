#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod usecase;

pub use adapter::{
    asyncapi::{
        CourseCompletedEvent, EnrollmentOpenedEvent, LearningAsyncApiHandler, ProgressRecordedEvent,
    },
    grpc::{LearningGrpcHandler, LearningPathGrpcRequest, LearningPathGrpcResponse},
    http::{HttpMethod, LearningHttpHandler, RouteDescriptor},
};
pub use config::{RuntimeProfile, ServiceConfig};
pub use domain::{
    ArchitectureLayer, AssessmentBand, AuditEventKind, Capability, CourseEvidence, CourseId,
    DataClass, DomainInvariant, EnrollmentId, EnrollmentStatus, EvidenceVisibility, LearnerId,
    LearningPath, LearningPolicy, ProgressSnapshot, TenantId,
};
pub use error::{ServiceError, ServiceResult};
pub use usecase::{
    LearningManagementService, LearningPorts, OpenEnrollment, OpenEnrollmentCommand,
    RecordProgress, RecordProgressCommand, SealCourseCompletion, SealCourseCompletionCommand,
};

pub const MICROSERVICE: &str = "learning-management";
pub const BOUNDED_CONTEXT: &str = "course-progress";
pub const PRIMARY_CAPABILITY: &str = "learning-path-completion";
pub const PRIMARY_ADR: &str = "ADR-0105";
pub const USECASE_RENAME_ADR: &str = "ADR-0106";
pub const OPENAPI_CONTRACT: &str = "microservices/learning-management/contracts/openapi-v1.yaml";
pub const GRPC_CONTRACT: &str =
    "microservices/learning-management/contracts/learning-management-v1.proto";
pub const ASYNCAPI_CONTRACT: &str = "microservices/learning-management/contracts/asyncapi-v1.yaml";

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ServiceDescriptor {
    pub microservice: &'static str,
    pub bounded_context: &'static str,
    pub primary_capability: &'static str,
    pub architecture_layers: Vec<&'static str>,
    pub contract_paths: Vec<&'static str>,
}

impl ServiceDescriptor {
    pub fn layer_count(&self) -> usize {
        self.architecture_layers.len()
    }

    pub fn contract_count(&self) -> usize {
        self.contract_paths.len()
    }

    pub fn includes_layer(&self, layer: ArchitectureLayer) -> bool {
        self.architecture_layers.contains(&layer.slug())
    }
}

pub fn descriptor() -> ServiceDescriptor {
    ServiceDescriptor {
        microservice: MICROSERVICE,
        bounded_context: BOUNDED_CONTEXT,
        primary_capability: PRIMARY_CAPABILITY,
        architecture_layers: ArchitectureLayer::all()
            .iter()
            .map(ArchitectureLayer::slug)
            .collect(),
        contract_paths: vec![OPENAPI_CONTRACT, GRPC_CONTRACT, ASYNCAPI_CONTRACT],
    }
}

pub fn default_learning_path() -> LearningPath {
    LearningPath::new(
        TenantId::new("tenant-demo"),
        EnrollmentId::new("enrollment-demo"),
        CourseId::new("course-demo"),
        "Security foundations".to_owned(),
        EnrollmentStatus::Draft,
    )
}

pub fn default_http_routes() -> Vec<RouteDescriptor> {
    LearningHttpHandler::routes()
}

pub fn default_domain_invariants() -> Vec<DomainInvariant> {
    vec![
        DomainInvariant::tenant_scoped("enrollments remain tenant scoped"),
        DomainInvariant::policy_checked("course assignment checks learner entitlement"),
        DomainInvariant::audit_emitted("course completion emits certificate evidence"),
        DomainInvariant::data_classified("assessment attempts stay learner confidential"),
        DomainInvariant::region_bound("learning records honor residency pack boundaries"),
        DomainInvariant::progress_monotonic("progress percentages never move backward"),
    ]
}

pub fn validate_scaffold() -> ServiceResult<()> {
    let descriptor = descriptor();
    if descriptor.layer_count() != 12 {
        return Err(ServiceError::InvariantViolation {
            invariant: "adr_0105_layer_count",
            details: format!("expected 12 layers, found {}", descriptor.layer_count()),
        });
    }
    if descriptor.contract_count() != 3 {
        return Err(ServiceError::InvariantViolation {
            invariant: "contract_surface_count",
            details: format!(
                "expected 3 contracts, found {}",
                descriptor.contract_count()
            ),
        });
    }
    Ok(())
}
