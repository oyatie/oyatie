#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod lake_engine;
pub mod usecase;

pub use adapter::{
    asyncapi::{
        DataWarehouseAsyncApiHandler, DatasetSharedEvent, FreshnessBreachedEvent,
        MaterializationRefreshedEvent,
    },
    grpc::{DataWarehouseGrpcHandler, WarehouseGrpcRequest, WarehouseGrpcResponse},
    http::{DataWarehouseHttpHandler, HttpMethod, RouteDescriptor},
};
pub use config::{RuntimeProfile, ServiceConfig};
pub use domain::{
    ArchitectureLayer, AuditEventKind, Capability, DataClass, DatasetId, DomainInvariant,
    FreshnessTier, LineageEdge, MaterializationId, QueryClass, QueryWorkload, TenantId,
    WarehouseNamespace, WarehousePolicy, WarehouseStatus,
};
pub use error::{ServiceError, ServiceResult};
pub use lake_engine::{
    ChangeDataFeedCursor, DeltaWriterCore, HudiWriterCore, IcebergWriterCore, LakeCommitReceipt,
    LakeProtocol, LakeTableRef,
};
pub use usecase::{
    DataWarehousePorts, DataWarehouseService, RefreshMaterialization,
    RefreshMaterializationCommand, RegisterDataset, RegisterDatasetCommand, ShareDataset,
    ShareDatasetCommand,
};

pub const MICROSERVICE: &str = "data-warehouse";
pub const BOUNDED_CONTEXT: &str = "tenant-olap";
pub const PRIMARY_CAPABILITY: &str = "tenant-olap-freshness";
pub const PRIMARY_ADR: &str = "ADR-0105";
pub const USECASE_RENAME_ADR: &str = "ADR-0106";
pub const OPENAPI_CONTRACT: &str = "data/data-warehouse/contracts/openapi-v1.yaml";
pub const GRPC_CONTRACT: &str = "data/data-warehouse/contracts/data-warehouse-v1.proto";
pub const ASYNCAPI_CONTRACT: &str = "data/data-warehouse/contracts/asyncapi-v1.yaml";

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

pub fn default_namespace() -> WarehouseNamespace {
    WarehouseNamespace::new(
        TenantId::new("tenant-demo"),
        DatasetId::new("dataset-demo"),
        "finance_mart".to_owned(),
        FreshnessTier::Hourly,
        WarehouseStatus::Draft,
    )
}

pub fn default_http_routes() -> Vec<RouteDescriptor> {
    DataWarehouseHttpHandler::routes()
}

pub fn default_domain_invariants() -> Vec<DomainInvariant> {
    vec![
        DomainInvariant::tenant_scoped("datasets remain tenant scoped"),
        DomainInvariant::policy_checked("dataset shares require explicit grant"),
        DomainInvariant::audit_emitted("materialization refresh emits lineage audit"),
        DomainInvariant::data_classified("query workloads declare data class"),
        DomainInvariant::region_bound("warehouse storage honors residency pack boundaries"),
        DomainInvariant::freshness_bounded("freshness tiers cannot silently widen"),
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
