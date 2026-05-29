use crate::domain::TenantId;
use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ServiceConfig {
    pub service_name: String,
    pub environment: String,
    pub tenant: TenantConfig,
    pub inbound: InboundConfig,
    pub dependencies: DependencyConfig,
    pub observability: ObservabilityConfig,
    pub feature_flags: FeatureFlags,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TenantConfig {
    pub tenant_id: String,
    pub home_cell: String,
    pub data_residency_pack: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InboundConfig {
    pub host: String,
    pub port: u16,
    pub rest_enabled: bool,
    pub grpc_enabled: bool,
    pub asyncapi_enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DependencyConfig {
    pub workflow_engine_url: String,
    pub ontology_url: String,
    pub policy_url: String,
    pub audit_url: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ObservabilityConfig {
    pub service_namespace: String,
    pub audit_topic: String,
    pub metrics_prefix: String,
    pub tracing_sample_ratio: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FeatureFlags {
    pub enable_async_projection: bool,
    pub enable_governance_holds: bool,
    pub enable_evidence_export: bool,
}

impl ServiceConfig {
    pub fn local_default(tenant_id: impl Into<String>, port: u16) -> Self {
        Self {
            service_name: "plant-maintenance".to_string(),
            environment: "local".to_string(),
            tenant: TenantConfig {
                tenant_id: tenant_id.into(),
                home_cell: "local-cell-1".to_string(),
                data_residency_pack: "local-dev".to_string(),
            },
            inbound: InboundConfig {
                host: "127.0.0.1".to_string(),
                port,
                rest_enabled: true,
                grpc_enabled: true,
                asyncapi_enabled: true,
            },
            dependencies: DependencyConfig {
                workflow_engine_url: "http://127.0.0.1:8081".to_string(),
                ontology_url: "http://127.0.0.1:8082".to_string(),
                policy_url: "http://127.0.0.1:8083".to_string(),
                audit_url: "http://127.0.0.1:8084".to_string(),
            },
            observability: ObservabilityConfig {
                service_namespace: "oyatie.plant_maintenance".to_string(),
                audit_topic: "audit.plant_maintenance".to_string(),
                metrics_prefix: "oya_plant_maintenance".to_string(),
                tracing_sample_ratio: "1.0".to_string(),
            },
            feature_flags: FeatureFlags {
                enable_async_projection: false,
                enable_governance_holds: true,
                enable_evidence_export: false,
            },
        }
    }

    pub fn from_toml_str(input: &str) -> Result<Self> {
        let config: Self = toml::from_str(input)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<Self> {
        let input = fs::read_to_string(path)?;
        Self::from_toml_str(&input)
    }

    pub fn validate(&self) -> Result<()> {
        TenantId::new(self.tenant.tenant_id.clone())?;
        if self.service_name != "plant-maintenance" {
            return Err(ServiceError::validation(
                "service_name",
                "service_name must remain plant-maintenance",
            ));
        }
        if self.inbound.port == 0 {
            return Err(ServiceError::validation("port", "port must be non-zero"));
        }
        if self.tenant.home_cell.trim().is_empty() {
            return Err(ServiceError::validation(
                "home_cell",
                "tenant home cell is required",
            ));
        }
        Ok(())
    }
}
