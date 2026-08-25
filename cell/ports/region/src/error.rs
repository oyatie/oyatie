use cell_region::CloudRegionError;

use crate::model::{
    CloudRegionApiError, CloudRegionApiErrorBody, CloudRegionApiErrorCode,
    CloudRegionApiErrorDetail, CloudRegionApiErrorResponse, CloudRegionListApiStatus,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudRegionApiStatusKind {
    BadRequest,
    Forbidden,
    NotFound,
}

impl CloudRegionApiError {
    pub fn list_status(&self) -> CloudRegionListApiStatus {
        match self.status_kind() {
            CloudRegionApiStatusKind::BadRequest => CloudRegionListApiStatus::BadRequest,
            CloudRegionApiStatusKind::Forbidden => CloudRegionListApiStatus::Forbidden,
            CloudRegionApiStatusKind::NotFound => CloudRegionListApiStatus::NotFound,
        }
    }

    pub fn list_status_code(&self) -> u16 {
        self.list_status().code()
    }

    pub fn code(&self) -> CloudRegionApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudRegionApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudRegionApiErrorCode::TenantHeaderEmpty,
            Self::EmptyPrincipalId => CloudRegionApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathRegionCode => CloudRegionApiErrorCode::PathRegionCodeEmpty,
            Self::TenantMismatch { .. } => CloudRegionApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudRegionApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudRegionApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudRegionApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudRegionApiErrorCode::AuthorizationDenied,
            Self::Region(error) => match cloud_region_status_kind(error) {
                CloudRegionApiStatusKind::NotFound => CloudRegionApiErrorCode::RegionNotFound,
                CloudRegionApiStatusKind::BadRequest => {
                    CloudRegionApiErrorCode::RegionInvalidRequest
                }
                CloudRegionApiStatusKind::Forbidden => CloudRegionApiErrorCode::RegionForbidden,
            },
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudRegionApiErrorResponse {
        CloudRegionApiErrorResponse {
            error: CloudRegionApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudRegionApiStatusKind {
        match self {
            Self::TenantMismatch { .. }
            | Self::EmptyPrincipalId
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudRegionApiStatusKind::Forbidden,
            Self::Region(error) => cloud_region_status_kind(error),
            Self::EmptyRequestId | Self::EmptyTenantHeader | Self::EmptyPathRegionCode => {
                CloudRegionApiStatusKind::BadRequest
            }
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathRegionCode => "Path region code is required",
            Self::TenantMismatch { .. } => "Tenant header must match the authenticated principal",
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud Region surface"
            }
            Self::Region(error) => cloud_region_message(error),
        }
    }

    fn details(&self) -> Vec<CloudRegionApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathRegionCode => vec![detail("path.region_code", "must be non-empty")],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant and principal tenant must match",
            )],
            Self::EmptyAuthorizationDecisionId => vec![detail(
                "authorization.decision_id",
                "must be non-empty authorization evidence",
            )],
            Self::AuthorizationTenantMismatch { .. } => vec![detail(
                "authorization.tenant_id",
                "must match the authenticated principal tenant",
            )],
            Self::AuthorizationPrincipalMismatch { .. } => vec![detail(
                "authorization.principal_id",
                "must match the authenticated principal id",
            )],
            Self::AuthorizationDenied { .. } => vec![detail(
                "authorization.allowed_surfaces",
                "must include the requested Cloud Region surface",
            )],
            Self::Region(error) => vec![detail("cloud_region", cloud_region_issue(error))],
        }
    }
}

fn cloud_region_status_kind(error: &CloudRegionError) -> CloudRegionApiStatusKind {
    match error {
        CloudRegionError::UnknownRegion
        | CloudRegionError::UnknownAz
        | CloudRegionError::UnknownCell => CloudRegionApiStatusKind::NotFound,
        CloudRegionError::CellBindingRejected(_) => CloudRegionApiStatusKind::Forbidden,
        CloudRegionError::InvalidRegionCode
        | CloudRegionError::InvalidAzCode
        | CloudRegionError::InvalidCellId
        | CloudRegionError::InvalidDisplayName
        | CloudRegionError::InvalidRegulatoryPack
        | CloudRegionError::EmptyRegulatoryPackSet
        | CloudRegionError::DuplicateRegulatoryPack
        | CloudRegionError::InvalidPhysicalRef
        | CloudRegionError::InvalidPowerZone
        | CloudRegionError::EmptyPowerZoneSet
        | CloudRegionError::DuplicatePowerZone
        | CloudRegionError::InvalidHsmPartitionRef
        | CloudRegionError::InvalidTenantId
        | CloudRegionError::InvalidCapacity
        | CloudRegionError::UtilizationExceedsCapacity
        | CloudRegionError::RegionResidencyMismatch
        | CloudRegionError::EmptyAllowedResidencySet
        | CloudRegionError::DuplicateAllowedResidencyClass
        | CloudRegionError::CellResidencyNotAllowedInRegion
        | CloudRegionError::CellResidencyDenied
        | CloudRegionError::DuplicateRegion
        | CloudRegionError::DuplicateAz
        | CloudRegionError::DuplicateCell
        | CloudRegionError::AzRegionMismatch
        | CloudRegionError::CellRegionMismatch
        | CloudRegionError::CellAzMismatch
        | CloudRegionError::NoCompatibleCell
        | CloudRegionError::ResidencyReferenceRejected(_) => CloudRegionApiStatusKind::BadRequest,
    }
}

fn cloud_region_message(error: &CloudRegionError) -> &'static str {
    match cloud_region_status_kind(error) {
        CloudRegionApiStatusKind::BadRequest => "Cloud Region rejected the request shape",
        CloudRegionApiStatusKind::Forbidden => "Cloud Region policy denied the request",
        CloudRegionApiStatusKind::NotFound => "Cloud Region resource was not found",
    }
}

fn cloud_region_issue(error: &CloudRegionError) -> &'static str {
    match error {
        CloudRegionError::InvalidRegionCode => "region_code must be canonical lowercase ASCII",
        CloudRegionError::InvalidAzCode => "az code must be canonical lowercase ASCII",
        CloudRegionError::InvalidCellId => "cell id must be canonical and use the cell- prefix",
        CloudRegionError::InvalidDisplayName => "display_name must be non-empty",
        CloudRegionError::InvalidRegulatoryPack => "regulatory pack id must use the pack- prefix",
        CloudRegionError::EmptyRegulatoryPackSet => "region must name at least one regulatory pack",
        CloudRegionError::DuplicateRegulatoryPack => "regulatory pack ids must be unique",
        CloudRegionError::InvalidPhysicalRef => "physical_ref must be non-empty",
        CloudRegionError::InvalidPowerZone => "power zone ids must be non-empty",
        CloudRegionError::EmptyPowerZoneSet => "AZ must name at least one power zone",
        CloudRegionError::DuplicatePowerZone => "power zone ids must be unique",
        CloudRegionError::InvalidHsmPartitionRef => "hsm_partition_ref must bind region and cell",
        CloudRegionError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudRegionError::InvalidCapacity => "cell capacity must include compute, memory, and SSD",
        CloudRegionError::UtilizationExceedsCapacity => "cell utilization must not exceed capacity",
        CloudRegionError::RegionResidencyMismatch => "residency class must allow the home region",
        CloudRegionError::EmptyAllowedResidencySet => {
            "cell must allow at least one residency class"
        }
        CloudRegionError::DuplicateAllowedResidencyClass => "cell residency classes must be unique",
        CloudRegionError::CellResidencyNotAllowedInRegion => {
            "cell residency class must be allowed by the region"
        }
        CloudRegionError::CellResidencyDenied => "cell does not allow requested residency class",
        CloudRegionError::DuplicateRegion => "region code is already present",
        CloudRegionError::DuplicateAz => "AZ code is already present",
        CloudRegionError::DuplicateCell => "cell id is already present",
        CloudRegionError::UnknownRegion => "region must exist before listing AZs",
        CloudRegionError::UnknownAz => "AZ must exist before cell registration",
        CloudRegionError::UnknownCell => "cell must exist before tenant binding",
        CloudRegionError::AzRegionMismatch => "AZ code must sit under its region code",
        CloudRegionError::CellRegionMismatch => "cell region must match its registered region",
        CloudRegionError::CellAzMismatch => "cell id and AZ must share a namespace",
        CloudRegionError::NoCompatibleCell => "no active cell satisfies the route request",
        CloudRegionError::CellBindingRejected(_) => "platform cell binding rejected the request",
        CloudRegionError::ResidencyReferenceRejected(_) => {
            "platform residency reference rejected the request"
        }
    }
}

fn detail(field: &str, issue: &str) -> CloudRegionApiErrorDetail {
    CloudRegionApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
