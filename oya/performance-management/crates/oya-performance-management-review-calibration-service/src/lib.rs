#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod usecase;

pub use adapter::{
    asyncapi::{
        CalibrationCompletedEvent, FeedbackSubmittedEvent, PerformanceAsyncApiHandler,
        ReviewCycleOpenedEvent,
    },
    grpc::{PerformanceGrpcHandler, ReviewCycleGrpcRequest, ReviewCycleGrpcResponse},
    http::{HttpMethod, PerformanceHttpHandler, RouteDescriptor},
};
pub use config::{RuntimeProfile, ServiceConfig};
pub use domain::{
    ArchitectureLayer, AuditEventKind, CalibrationCohortId, CalibrationRun, Capability, DataClass,
    DomainInvariant, EmployeeId, FeedbackVisibility, GoalAlignmentSnapshot, PerformancePolicy,
    RatingBand, ReviewCycle, ReviewCycleId, ReviewCycleStatus, ReviewEvidence, TenantId,
};
pub use error::{ServiceError, ServiceResult};
pub use usecase::{
    OpenReviewCycle, OpenReviewCycleCommand, PerformanceManagementService, PerformancePorts,
    SealReviewEvidence, SealReviewEvidenceCommand, SubmitFeedback, SubmitFeedbackCommand,
};

pub const MICROSERVICE: &str = "performance-management";
pub const BOUNDED_CONTEXT: &str = "review-calibration";
pub const PRIMARY_CAPABILITY: &str = "review-calibration";
pub const PRIMARY_ADR: &str = "ADR-0105";
pub const USECASE_RENAME_ADR: &str = "ADR-0106";
pub const OPENAPI_CONTRACT: &str = "microservices/performance-management/contracts/openapi-v1.yaml";
pub const GRPC_CONTRACT: &str =
    "microservices/performance-management/contracts/performance-management-v1.proto";
pub const ASYNCAPI_CONTRACT: &str =
    "microservices/performance-management/contracts/asyncapi-v1.yaml";

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

pub fn default_review_cycle() -> ReviewCycle {
    ReviewCycle::new(
        TenantId::new("tenant-demo"),
        ReviewCycleId::new("review-cycle-demo"),
        "FY26 calibrated review".to_owned(),
        ReviewCycleStatus::Draft,
    )
}

pub fn default_http_routes() -> Vec<RouteDescriptor> {
    PerformanceHttpHandler::routes()
}

pub fn default_domain_invariants() -> Vec<DomainInvariant> {
    vec![
        DomainInvariant::tenant_scoped("review cycles remain tenant scoped"),
        DomainInvariant::policy_checked("rating changes require Cedar policy approval"),
        DomainInvariant::audit_emitted("calibration close emits a review evidence seal"),
        DomainInvariant::data_classified("manager notes stay confidential workforce data"),
        DomainInvariant::region_bound("labor overlays honor residency pack boundaries"),
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
