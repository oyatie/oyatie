#![forbid(unsafe_code)]
//! Horizontally scalable Drive metadata, object, ACL, sharing, and launch-point API scaffold.
//!
//! This deployable owns stateless Drive API route wiring. Runtime HTTP adapters
//! will map these pure contracts to Axum/tonic/etc. only after dependency review.

use oya_office_drive_api::{DRIVE_API_VERSION, DriveRoute, drive_routes};
use oya_office_kernel::RequestContext;

/// Stable application identifier used by workspace and Buck2 scaffold verification.
pub const APP_NAME: &str = "oya-office-drive-api-app";

/// Product vertical slice owned by this deployable.
pub const VERTICAL_SLICE: &str = "drive";

/// Source-shaped deployable layer represented by this scaffold.
pub const DEPLOYABLE_LAYER: &str = "api";

/// Health state for the Drive API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveApiHealth {
    /// Startup dependencies/config are valid enough to start.
    Startup,
    /// Ready to accept tenant traffic.
    Ready,
    /// Process is alive; should not check downstream dependencies.
    Live,
}

/// Drive API autoscaling signal. These metrics avoid CPU-only hyperscaler anti-patterns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DriveApiScaleSignal {
    /// Metadata request pressure for list/get/update/share operations.
    ObjectMetadataRequestsPerSecond,
    /// Object authorization p99 latency pressure.
    AclAuthorizationP99Milliseconds,
    /// Storage egress pressure from download/preview flows.
    StorageEgressMebibytesPerSecond,
}

impl DriveApiScaleSignal {
    /// Returns the product metric name.
    #[must_use]
    pub const fn metric_name(self) -> &'static str {
        match self {
            Self::ObjectMetadataRequestsPerSecond => "oya_office_drive_metadata_requests_per_second",
            Self::AclAuthorizationP99Milliseconds => {
                "oya_office_drive_acl_authorization_p99_milliseconds"
            }
            Self::StorageEgressMebibytesPerSecond => {
                "oya_office_drive_storage_egress_mebibytes_per_second"
            }
        }
    }

    /// Returns the initial HPA target average value.
    #[must_use]
    pub const fn target_average_value(self) -> &'static str {
        match self {
            Self::ObjectMetadataRequestsPerSecond => "75",
            Self::AclAuthorizationP99Milliseconds => "75",
            Self::StorageEgressMebibytesPerSecond => "128",
        }
    }
}

/// Returns Drive API scale signals in the sequence expected by K8s contracts and dashboards.
#[must_use]
pub const fn drive_api_scale_signals() -> [DriveApiScaleSignal; 3] {
    [
        DriveApiScaleSignal::ObjectMetadataRequestsPerSecond,
        DriveApiScaleSignal::AclAuthorizationP99Milliseconds,
        DriveApiScaleSignal::StorageEgressMebibytesPerSecond,
    ]
}

/// Drive API horizontal scaling contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveApiScalingContract {
    min_replicas: u16,
    max_replicas: u16,
    signals: [DriveApiScaleSignal; 3],
}

impl DriveApiScalingContract {
    /// Creates the default Drive API scaling contract.
    #[must_use]
    pub const fn production_baseline() -> Self {
        Self {
            min_replicas: 3,
            max_replicas: 40,
            signals: drive_api_scale_signals(),
        }
    }

    /// Returns minimum replicas.
    #[must_use]
    pub const fn min_replicas(&self) -> u16 {
        self.min_replicas
    }

    /// Returns maximum replicas.
    #[must_use]
    pub const fn max_replicas(&self) -> u16 {
        self.max_replicas
    }

    /// Returns scaling signals.
    #[must_use]
    pub const fn signals(&self) -> &[DriveApiScaleSignal; 3] {
        &self.signals
    }
}

/// Stateless Drive API surface descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveApiService {
    api_version: &'static str,
    routes: Vec<DriveRoute>,
}

impl DriveApiService {
    /// Creates a service descriptor with stable routes.
    #[must_use]
    pub fn new() -> Self {
        Self {
            api_version: DRIVE_API_VERSION,
            routes: drive_routes().to_vec(),
        }
    }

    /// Returns API version.
    #[must_use]
    pub const fn api_version(&self) -> &'static str {
        self.api_version
    }

    /// Returns declared routes.
    #[must_use]
    pub fn routes(&self) -> &[DriveRoute] {
        self.routes.as_slice()
    }

    /// Returns health response text for K8s probes.
    #[must_use]
    pub const fn health(&self, health: DriveApiHealth) -> &'static str {
        match health {
            DriveApiHealth::Startup => "drive-api-startup-ok",
            DriveApiHealth::Ready => "drive-api-ready",
            DriveApiHealth::Live => "drive-api-live",
        }
    }

    /// Validates that an API request contains tenant/cell context before routing.
    #[must_use]
    pub fn accepts_context(&self, context: &RequestContext) -> bool {
        !context.tenant_id().as_str().is_empty()
            && !context.cell_id().as_str().is_empty()
            && !context.principal_id().as_str().is_empty()
    }
}

impl Default for DriveApiService {
    fn default() -> Self {
        Self::new()
    }
}

/// Starts the scaffolded application entrypoint.
pub fn run() {}

#[cfg(test)]
mod tests {
    use oya_office_drive_api::DriveRoute;
    use oya_office_kernel::{CellId, PrincipalId, RequestContext, RequestId, TenantId};

    use super::{APP_NAME, DEPLOYABLE_LAYER, DriveApiHealth, DriveApiService, VERTICAL_SLICE};

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!APP_NAME.is_empty());
        assert!(!DEPLOYABLE_LAYER.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn drive_api_declares_routes_and_probe_states() {
        let service = DriveApiService::new();
        assert!(service.routes().contains(&DriveRoute::LaunchObject));
        assert_eq!(service.health(DriveApiHealth::Live), "drive-api-live");
    }

    #[test]
    fn drive_api_requires_tenant_context() {
        let context = RequestContext::new(
            RequestId::new("req-1").expect("valid request id"),
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            PrincipalId::new("user-1").expect("valid principal id"),
            CellId::new("iad-1").expect("valid cell id"),
        );
        assert!(DriveApiService::new().accepts_context(&context));
    }

    #[test]
    fn drive_api_scaling_signals_match_hyperscaler_contract() {
        let signals = super::drive_api_scale_signals();
        assert!(
            signals.iter().any(
                |signal| signal.metric_name() == "oya_office_drive_metadata_requests_per_second"
            )
        );
        assert!(signals.iter().any(|signal| {
            signal.metric_name() == "oya_office_drive_acl_authorization_p99_milliseconds"
        }));
        assert!(signals.iter().any(|signal| {
            signal.metric_name() == "oya_office_drive_storage_egress_mebibytes_per_second"
        }));
        assert_eq!(
            super::DriveApiScalingContract::production_baseline().max_replicas(),
            40
        );
    }
}
