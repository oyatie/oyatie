use crate::domain::{
    AuditEventKind, Capability, DatasetId, MaterializationId, TenantId, WarehouseNamespace,
    WarehouseStatus,
};
use crate::error::{ServiceError, ServiceResult};

pub trait WarehouseRepository {
    fn put_namespace(&mut self, namespace: WarehouseNamespace)
    -> ServiceResult<WarehouseNamespace>;
    fn get_namespace(
        &self,
        tenant_id: &TenantId,
        dataset_id: &DatasetId,
    ) -> ServiceResult<Option<WarehouseNamespace>>;
}

pub trait PolicyAuthorizer {
    fn authorize(&self, tenant_id: &TenantId, capability: Capability) -> ServiceResult<()>;
}

pub trait AuditPublisher {
    fn publish_audit(
        &mut self,
        tenant_id: &TenantId,
        event_kind: AuditEventKind,
        subject: &str,
    ) -> ServiceResult<()>;
}

pub trait DataWarehousePorts: WarehouseRepository + PolicyAuthorizer + AuditPublisher {}

impl<T> DataWarehousePorts for T where T: WarehouseRepository + PolicyAuthorizer + AuditPublisher {}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RegisterDatasetCommand {
    pub tenant_id: TenantId,
    pub dataset_id: DatasetId,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RefreshMaterializationCommand {
    pub tenant_id: TenantId,
    pub dataset_id: DatasetId,
    pub materialization_id: MaterializationId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ShareDatasetCommand {
    pub tenant_id: TenantId,
    pub dataset_id: DatasetId,
    pub consumer_tenant_id: TenantId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UsecaseReceipt {
    pub tenant_id: TenantId,
    pub dataset_id: DatasetId,
    pub audit_event: AuditEventKind,
    pub status: WarehouseStatus,
}

pub struct RegisterDataset;

impl RegisterDataset {
    pub fn execute(
        ports: &mut impl DataWarehousePorts,
        command: RegisterDatasetCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::DatasetRegister)?;
        let namespace = WarehouseNamespace::new(
            command.tenant_id.clone(),
            command.dataset_id.clone(),
            command.name,
            crate::domain::FreshnessTier::Hourly,
            WarehouseStatus::Draft,
        )
        .register()?;
        namespace.validate()?;
        let namespace = ports.put_namespace(namespace)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::DatasetRegistered,
            command.dataset_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: namespace.tenant_id,
            dataset_id: namespace.dataset_id,
            audit_event: AuditEventKind::DatasetRegistered,
            status: namespace.status,
        })
    }
}

pub struct RefreshMaterialization;

impl RefreshMaterialization {
    pub fn execute(
        ports: &mut impl DataWarehousePorts,
        command: RefreshMaterializationCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::MaterializationRefresh)?;
        let namespace = ports
            .get_namespace(&command.tenant_id, &command.dataset_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "warehouse_repository",
            })?
            .refresh_materialization()?;
        let namespace = ports.put_namespace(namespace)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::MaterializationRefreshed,
            command.materialization_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: namespace.tenant_id,
            dataset_id: namespace.dataset_id,
            audit_event: AuditEventKind::MaterializationRefreshed,
            status: namespace.status,
        })
    }
}

pub struct ShareDataset;

impl ShareDataset {
    pub fn execute(
        ports: &mut impl DataWarehousePorts,
        command: ShareDatasetCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::DatasetShare)?;
        let namespace = ports
            .get_namespace(&command.tenant_id, &command.dataset_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "warehouse_repository",
            })?
            .share()?;
        let namespace = ports.put_namespace(namespace)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::DatasetShared,
            command.consumer_tenant_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: namespace.tenant_id,
            dataset_id: namespace.dataset_id,
            audit_event: AuditEventKind::DatasetShared,
            status: namespace.status,
        })
    }
}

pub struct DataWarehouseService<P> {
    ports: P,
}

impl<P> DataWarehouseService<P>
where
    P: DataWarehousePorts,
{
    pub fn new(ports: P) -> Self {
        Self { ports }
    }

    pub fn register_dataset(
        &mut self,
        command: RegisterDatasetCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        RegisterDataset::execute(&mut self.ports, command)
    }

    pub fn refresh_materialization(
        &mut self,
        command: RefreshMaterializationCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        RefreshMaterialization::execute(&mut self.ports, command)
    }

    pub fn share_dataset(&mut self, command: ShareDatasetCommand) -> ServiceResult<UsecaseReceipt> {
        ShareDataset::execute(&mut self.ports, command)
    }

    pub fn into_ports(self) -> P {
        self.ports
    }
}
