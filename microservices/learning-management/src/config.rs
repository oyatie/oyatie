use crate::error::{ServiceError, ServiceResult};

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum RuntimeProfile {
    Local,
    Dev,
    Staging,
    Production,
}

impl RuntimeProfile {
    pub fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "local" => Ok(Self::Local),
            "dev" => Ok(Self::Dev),
            "staging" => Ok(Self::Staging),
            "production" => Ok(Self::Production),
            other => Err(ServiceError::InvalidConfig {
                field: "runtime_profile",
                details: format!("unsupported profile {other}"),
            }),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Dev => "dev",
            Self::Staging => "staging",
            Self::Production => "production",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ServiceConfig {
    pub service_name: String,
    pub bind_addr: String,
    pub grpc_addr: String,
    pub asyncapi_topic_prefix: String,
    pub policy_namespace: String,
    pub audit_topic: String,
    pub runtime_profile: RuntimeProfile,
    pub tenant_header: String,
    pub max_progress_payload_bytes: usize,
    pub enable_certificate_shadow_write: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            service_name: "learning-management".to_owned(),
            bind_addr: "127.0.0.1:8081".to_owned(),
            grpc_addr: "127.0.0.1:9081".to_owned(),
            asyncapi_topic_prefix: "learning-management".to_owned(),
            policy_namespace: "learning_management".to_owned(),
            audit_topic: "audit.learning_management".to_owned(),
            runtime_profile: RuntimeProfile::Local,
            tenant_header: "x-oya-tenant-id".to_owned(),
            max_progress_payload_bytes: 32 * 1024,
            enable_certificate_shadow_write: false,
        }
    }
}

impl ServiceConfig {
    pub fn from_env() -> ServiceResult<Self> {
        let mut config = Self::default();
        if let Ok(value) = std::env::var("OYA_LEARNING_MANAGEMENT_BIND_ADDR") {
            config.bind_addr = value;
        }
        if let Ok(value) = std::env::var("OYA_LEARNING_MANAGEMENT_GRPC_ADDR") {
            config.grpc_addr = value;
        }
        if let Ok(value) = std::env::var("OYA_LEARNING_MANAGEMENT_TOPIC_PREFIX") {
            config.asyncapi_topic_prefix = value;
        }
        if let Ok(value) = std::env::var("OYA_LEARNING_MANAGEMENT_PROFILE") {
            config.runtime_profile = RuntimeProfile::parse(&value)?;
        }
        if let Ok(value) = std::env::var("OYA_LEARNING_MANAGEMENT_MAX_PROGRESS_BYTES") {
            config.max_progress_payload_bytes =
                value
                    .parse::<usize>()
                    .map_err(|parse_error| ServiceError::InvalidConfig {
                        field: "max_progress_payload_bytes",
                        details: parse_error.to_string(),
                    })?;
        }
        if let Ok(value) = std::env::var("OYA_LEARNING_MANAGEMENT_CERT_SHADOW_WRITE") {
            config.enable_certificate_shadow_write =
                matches!(value.as_str(), "1" | "true" | "TRUE");
        }
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> ServiceResult<()> {
        validate_non_empty("service_name", &self.service_name)?;
        validate_non_empty("bind_addr", &self.bind_addr)?;
        validate_non_empty("grpc_addr", &self.grpc_addr)?;
        validate_non_empty("asyncapi_topic_prefix", &self.asyncapi_topic_prefix)?;
        validate_non_empty("policy_namespace", &self.policy_namespace)?;
        validate_non_empty("audit_topic", &self.audit_topic)?;
        validate_non_empty("tenant_header", &self.tenant_header)?;
        if self.max_progress_payload_bytes < 2048 {
            return Err(ServiceError::InvalidConfig {
                field: "max_progress_payload_bytes",
                details: "must be at least 2048 bytes".to_owned(),
            });
        }
        Ok(())
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> ServiceResult<()> {
    if value.trim().is_empty() {
        Err(ServiceError::InvalidConfig {
            field,
            details: "value must not be empty".to_owned(),
        })
    } else {
        Ok(())
    }
}
