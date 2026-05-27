#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum DesiredTier {
    #[default]
    Hosted,
    Dedicated,
}

impl DesiredTier {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Hosted => "hosted",
            Self::Dedicated => "dedicated",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "hosted" | "hosted_kamaji" => Some(Self::Hosted),
            "dedicated" | "dedicated_talos_spoke" => Some(Self::Dedicated),
            _ => None,
        }
    }
}

impl fmt::Display for DesiredTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterResourceRequest {
    pub nodes: u32,   // data_class: TENANT_SCOPED
    pub vcpu: u32,    // data_class: TENANT_SCOPED
    pub ram_gib: u32, // data_class: TENANT_SCOPED
}

impl ClusterResourceRequest {
    #[must_use]
    pub const fn new(nodes: u32, vcpu: u32, ram_gib: u32) -> Self {
        Self {
            nodes,
            vcpu,
            ram_gib,
        }
    }

    #[must_use]
    pub const fn default_small() -> Self {
        Self {
            nodes: 3,
            vcpu: 8,
            ram_gib: 32,
        }
    }

    fn validate(&self) -> Result<(), LifecycleValidationError> {
        if self.nodes == 0 {
            return Err(LifecycleValidationError::ZeroResource("nodes"));
        }
        if self.vcpu == 0 {
            return Err(LifecycleValidationError::ZeroResource("vcpu"));
        }
        if self.ram_gib == 0 {
            return Err(LifecycleValidationError::ZeroResource("ram_gib"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LifecycleRequest {
    pub tenant_id: String,                 // data_class: TENANT_SCOPED
    pub cluster_name: String,              // data_class: TENANT_SCOPED
    pub desired_tier: DesiredTier,         // data_class: TENANT_SCOPED
    pub resources: ClusterResourceRequest, // data_class: TENANT_SCOPED
}

impl LifecycleRequest {
    pub fn new(
        tenant_id: impl Into<String>,
        cluster_name: impl Into<String>,
        desired_tier: DesiredTier,
        resources: ClusterResourceRequest,
    ) -> Result<Self, LifecycleValidationError> {
        let request = Self {
            tenant_id: tenant_id.into(),
            cluster_name: cluster_name.into(),
            desired_tier,
            resources,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), LifecycleValidationError> {
        if self.tenant_id.trim().is_empty() {
            return Err(LifecycleValidationError::EmptyTenantId);
        }
        if self.cluster_name.trim().is_empty() {
            return Err(LifecycleValidationError::EmptyClusterName);
        }
        self.resources.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleValidationError {
    EmptyTenantId,
    EmptyClusterName,
    ZeroResource(&'static str),
}

impl fmt::Display for LifecycleValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTenantId => f.write_str("tenant_id must not be empty"),
            Self::EmptyClusterName => f.write_str("cluster_name must not be empty"),
            Self::ZeroResource(field) => write!(f, "resource field {field} must be > 0"),
        }
    }
}

impl std::error::Error for LifecycleValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_validates_identity_and_resources() {
        assert!(
            LifecycleRequest::new(
                "ten_zero",
                "dogfood-a",
                DesiredTier::Hosted,
                ClusterResourceRequest::default_small()
            )
            .is_ok()
        );
        assert!(matches!(
            LifecycleRequest::new(
                "",
                "dogfood-a",
                DesiredTier::Hosted,
                ClusterResourceRequest::default_small()
            ),
            Err(LifecycleValidationError::EmptyTenantId)
        ));
        assert!(matches!(
            LifecycleRequest::new(
                "ten_zero",
                " ",
                DesiredTier::Hosted,
                ClusterResourceRequest::default_small()
            ),
            Err(LifecycleValidationError::EmptyClusterName)
        ));
        assert!(matches!(
            LifecycleRequest::new(
                "ten_zero",
                "dogfood-a",
                DesiredTier::Hosted,
                ClusterResourceRequest::new(0, 8, 32)
            ),
            Err(LifecycleValidationError::ZeroResource("nodes"))
        ));
    }

    #[test]
    fn tier_parse_is_fail_closed() {
        assert_eq!(DesiredTier::parse("hosted"), Some(DesiredTier::Hosted));
        assert_eq!(
            DesiredTier::parse("dedicated_talos_spoke"),
            Some(DesiredTier::Dedicated)
        );
        assert_eq!(DesiredTier::parse("unknown"), None);
    }
}
