pub mod http {
    use crate::domain::{DatasetId, MaterializationId, TenantId};
    use crate::error::ServiceResult;
    use crate::usecase::{
        DataWarehousePorts, DataWarehouseService, RefreshMaterializationCommand,
        RegisterDatasetCommand, ShareDatasetCommand, UsecaseReceipt,
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
    pub struct RegisterDatasetHttpRequest {
        pub tenant_id: String,
        pub dataset_id: String,
        pub name: String,
    }

    pub struct DataWarehouseHttpHandler;

    impl DataWarehouseHttpHandler {
        pub fn routes() -> Vec<RouteDescriptor> {
            vec![
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/datasets",
                    capability: "warehouse.dataset.register",
                },
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/datasets/{dataset_id}/materializations/{materialization_id}/refresh",
                    capability: "warehouse.materialization.refresh",
                },
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/datasets/{dataset_id}/shares",
                    capability: "warehouse.dataset.share",
                },
            ]
        }

        pub fn register_dataset(
            service: &mut DataWarehouseService<impl DataWarehousePorts>,
            request: RegisterDatasetHttpRequest,
        ) -> ServiceResult<UsecaseReceipt> {
            service.register_dataset(RegisterDatasetCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                dataset_id: DatasetId::parse(request.dataset_id)?,
                name: request.name,
            })
        }

        pub fn refresh_materialization(
            service: &mut DataWarehouseService<impl DataWarehousePorts>,
            tenant_id: String,
            dataset_id: String,
            materialization_id: String,
        ) -> ServiceResult<UsecaseReceipt> {
            service.refresh_materialization(RefreshMaterializationCommand {
                tenant_id: TenantId::parse(tenant_id)?,
                dataset_id: DatasetId::parse(dataset_id)?,
                materialization_id: MaterializationId::parse(materialization_id)?,
            })
        }

        pub fn share_dataset(
            service: &mut DataWarehouseService<impl DataWarehousePorts>,
            tenant_id: String,
            dataset_id: String,
            consumer_tenant_id: String,
        ) -> ServiceResult<UsecaseReceipt> {
            service.share_dataset(ShareDatasetCommand {
                tenant_id: TenantId::parse(tenant_id)?,
                dataset_id: DatasetId::parse(dataset_id)?,
                consumer_tenant_id: TenantId::parse(consumer_tenant_id)?,
            })
        }
    }
}

pub mod grpc {
    use crate::domain::{DatasetId, TenantId};
    use crate::error::ServiceResult;
    use crate::usecase::{
        DataWarehousePorts, DataWarehouseService, RegisterDatasetCommand, UsecaseReceipt,
    };

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct WarehouseGrpcRequest {
        pub tenant_id: String,
        pub dataset_id: String,
        pub name: String,
        pub request_id: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct WarehouseGrpcResponse {
        pub tenant_id: String,
        pub dataset_id: String,
        pub status: String,
        pub audit_event: String,
    }

    pub struct DataWarehouseGrpcHandler;

    impl DataWarehouseGrpcHandler {
        pub fn register_dataset(
            service: &mut DataWarehouseService<impl DataWarehousePorts>,
            request: WarehouseGrpcRequest,
        ) -> ServiceResult<WarehouseGrpcResponse> {
            let receipt = service.register_dataset(RegisterDatasetCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                dataset_id: DatasetId::parse(request.dataset_id)?,
                name: request.name,
            })?;
            Ok(Self::response_from_receipt(receipt))
        }

        fn response_from_receipt(receipt: UsecaseReceipt) -> WarehouseGrpcResponse {
            WarehouseGrpcResponse {
                tenant_id: receipt.tenant_id.as_str().to_owned(),
                dataset_id: receipt.dataset_id.as_str().to_owned(),
                status: format!("{:?}", receipt.status),
                audit_event: format!("{:?}", receipt.audit_event),
            }
        }
    }
}

pub mod asyncapi {
    use crate::domain::{AuditEventKind, DatasetId, TenantId};
    use crate::error::ServiceResult;

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct MaterializationRefreshedEvent {
        pub tenant_id: TenantId,
        pub dataset_id: DatasetId,
        pub audit_event: AuditEventKind,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct FreshnessBreachedEvent {
        pub tenant_id: TenantId,
        pub dataset_id: DatasetId,
        pub lag_minutes: u32,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct DatasetSharedEvent {
        pub tenant_id: TenantId,
        pub dataset_id: DatasetId,
        pub consumer_tenant_id: TenantId,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct PublishedMessage {
        pub topic: String,
        pub payload_json: String,
    }

    pub struct DataWarehouseAsyncApiHandler;

    impl DataWarehouseAsyncApiHandler {
        pub fn materialization_refreshed(
            prefix: &str,
            event: MaterializationRefreshedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.materialization.refreshed"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn freshness_breached(
            prefix: &str,
            event: FreshnessBreachedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.freshness.breached"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn dataset_shared(
            prefix: &str,
            event: DatasetSharedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.dataset.shared"),
                payload_json: serde_json::to_string(&event)?,
            })
        }
    }
}

pub mod memory {
    use std::collections::BTreeMap;

    use crate::domain::{AuditEventKind, Capability, DatasetId, TenantId, WarehouseNamespace};
    use crate::error::{ServiceError, ServiceResult};
    use crate::usecase::{AuditPublisher, PolicyAuthorizer, WarehouseRepository};

    #[derive(Clone, Debug, Default)]
    pub struct InMemoryDataWarehousePorts {
        namespaces: BTreeMap<String, WarehouseNamespace>,
        audit_events: Vec<String>,
        denied_capabilities: Vec<Capability>,
    }

    impl InMemoryDataWarehousePorts {
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

        fn key(tenant_id: &TenantId, dataset_id: &DatasetId) -> String {
            format!("{}::{}", tenant_id.as_str(), dataset_id.as_str())
        }
    }

    impl WarehouseRepository for InMemoryDataWarehousePorts {
        fn put_namespace(
            &mut self,
            namespace: WarehouseNamespace,
        ) -> ServiceResult<WarehouseNamespace> {
            let key = Self::key(&namespace.tenant_id, &namespace.dataset_id);
            self.namespaces.insert(key, namespace.clone());
            Ok(namespace)
        }

        fn get_namespace(
            &self,
            tenant_id: &TenantId,
            dataset_id: &DatasetId,
        ) -> ServiceResult<Option<WarehouseNamespace>> {
            Ok(self
                .namespaces
                .get(&Self::key(tenant_id, dataset_id))
                .cloned())
        }
    }

    impl PolicyAuthorizer for InMemoryDataWarehousePorts {
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

    impl AuditPublisher for InMemoryDataWarehousePorts {
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
