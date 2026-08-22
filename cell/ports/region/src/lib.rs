//! Cloud Region API boundary for region and availability-zone listing.
//!
//! This crate owns authenticated request normalization and public projection for
//! the immutable Cloud region/AZ taxonomy before returning API records.

use cell_region::{
    AzState, CloudAz, CloudCell, CloudCellState, CloudRegion, CloudRegionCatalog, CloudRegionError,
    RegionCode, RegionState, TenantDensityClass,
};
use network_residency::ResidencyClass;

pub const CLOUD_REGION_LIST_SURFACE: &str = "cloud.region.list";
pub const CLOUD_AZ_LIST_SURFACE: &str = "cloud.az.list";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudRegionListApiStatus {
    Ok,
    BadRequest,
    Forbidden,
    NotFound,
}

impl CloudRegionListApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::NotFound => 404,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudRegionApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    PrincipalIdEmpty,
    PathRegionCodeEmpty,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    RegionInvalidRequest,
    RegionForbidden,
    RegionNotFound,
}

impl CloudRegionApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_REGION_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_REGION_TENANT_HEADER_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_REGION_PRINCIPAL_ID_EMPTY",
            Self::PathRegionCodeEmpty => "CLOUD_REGION_PATH_REGION_CODE_EMPTY",
            Self::TenantMismatch => "CLOUD_REGION_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "CLOUD_REGION_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "CLOUD_REGION_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "CLOUD_REGION_AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::AuthorizationDenied => "CLOUD_REGION_AUTHORIZATION_DENIED",
            Self::RegionInvalidRequest => "CLOUD_REGION_INVALID_REQUEST",
            Self::RegionForbidden => "CLOUD_REGION_FORBIDDEN",
            Self::RegionNotFound => "CLOUD_REGION_NOT_FOUND",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionApiBoundaryContext {
    pub request_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionListApiRequest {
    pub boundary: CloudRegionApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudRegionApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: CloudRegionApiAuthorization, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudAzListApiRequest {
    pub path_region_code: String,                   // data_class: PUBLIC
    pub boundary: CloudRegionApiBoundaryContext,    // data_class: INTERNAL_ONLY
    pub principal: CloudRegionApiPrincipal,         // data_class: INTERNAL_ONLY
    pub authorization: CloudRegionApiAuthorization, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionListSuccessResponse {
    pub data: Vec<CloudRegionRecord>,     // data_class: PUBLIC
    pub metadata: CloudRegionApiMetadata, // data_class: INTERNAL_ONLY
}

impl CloudRegionListSuccessResponse {
    pub fn ok(data: Vec<CloudRegionRecord>, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudRegionApiMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudAzListSuccessResponse {
    pub data: Vec<CloudAzRecord>,         // data_class: PUBLIC
    pub metadata: CloudRegionApiMetadata, // data_class: INTERNAL_ONLY
}

impl CloudAzListSuccessResponse {
    pub fn ok(data: Vec<CloudAzRecord>, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudRegionApiMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionApiMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionRecord {
    pub code: String,                                        // data_class: PUBLIC
    pub display_name: String,                                // data_class: PUBLIC
    pub regulatory_packs: Vec<CloudRegionRegulatoryPackRef>, // data_class: PUBLIC
    pub azs: Vec<CloudRegionAzRef>,                          // data_class: PUBLIC
    pub state: String,                                       // data_class: PUBLIC
    pub provider_facing: bool,                               // data_class: PUBLIC
    pub residency_strictness: String,                        // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,                       // data_class: PUBLIC
    pub schema_version: u32,                                 // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionRegulatoryPackRef {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionAzRef {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionPowerZoneRef {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionCellRef {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudAzRecord {
    pub code: String,                              // data_class: PUBLIC
    pub region_code: String,                       // data_class: PUBLIC
    pub power_zones: Vec<CloudRegionPowerZoneRef>, // data_class: PUBLIC
    pub cells: Vec<CloudRegionCellRef>,            // data_class: PUBLIC
    pub cell_isolation_evidence: Vec<CloudCellIsolationEvidenceRecord>, // data_class: PUBLIC
    pub state: String,                             // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,             // data_class: PUBLIC
    pub schema_version: u32,                       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellIsolationEvidenceRecord {
    pub cell_id: String,                // data_class: PUBLIC
    pub region_code: String,            // data_class: PUBLIC
    pub az_code: String,                // data_class: PUBLIC
    pub state: String,                  // data_class: PUBLIC
    pub tenant_density: String,         // data_class: PUBLIC
    pub allowed_residency: Vec<String>, // data_class: PUBLIC
    pub evidence_ref: String,           // data_class: PUBLIC
    pub schema_version: u32,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionApiErrorResponse {
    pub error: CloudRegionApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionApiErrorBody {
    pub code: String,                            // data_class: INTERNAL_ONLY
    pub message: String,                         // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,       // data_class: INTERNAL_ONLY
    pub request_id: String,                      // data_class: INTERNAL_ONLY
    pub details: Vec<CloudRegionApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudRegionApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyPrincipalId,
    EmptyPathRegionCode,
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
    },
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String,
        principal_tenant_id: String,
    },
    AuthorizationPrincipalMismatch {
        authorization_principal_id: String,
        principal_id: String,
    },
    AuthorizationDenied {
        surface: String,
    },
    Region(CloudRegionError),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudRegionApiStatusKind {
    BadRequest,
    Forbidden,
    NotFound,
}

pub fn validate_cloud_region_list_request(
    request: &CloudRegionListApiRequest,
) -> Result<(), CloudRegionApiError> {
    validate_boundary(&request.boundary)?;
    validate_tenant_binding(&request.boundary, &request.principal)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_REGION_LIST_SURFACE,
    )
}

pub fn validate_cloud_az_list_request(
    request: &CloudAzListApiRequest,
) -> Result<RegionCode, CloudRegionApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_region_code(&request.path_region_code)?;
    validate_tenant_binding(&request.boundary, &request.principal)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_AZ_LIST_SURFACE,
    )?;
    RegionCode::new(request.path_region_code.clone()).map_err(CloudRegionApiError::Region)
}

pub fn list_cloud_regions_from_api(
    catalog: &CloudRegionCatalog,
    request: CloudRegionListApiRequest,
) -> Result<CloudRegionListSuccessResponse, CloudRegionApiError> {
    validate_cloud_region_list_request(&request)?;
    let request_id = request.boundary.request_id;
    let data = catalog
        .regions()
        .filter(|region| region.provider_facing.value)
        .map(region_record)
        .collect();
    Ok(CloudRegionListSuccessResponse::ok(data, request_id))
}

pub fn list_cloud_azs_from_api(
    catalog: &CloudRegionCatalog,
    request: CloudAzListApiRequest,
) -> Result<CloudAzListSuccessResponse, CloudRegionApiError> {
    let region_code = validate_cloud_az_list_request(&request)?;
    let Some(region) = catalog.region(&region_code) else {
        return Err(CloudRegionApiError::Region(CloudRegionError::UnknownRegion));
    };
    if !region.provider_facing.value {
        return Err(CloudRegionApiError::Region(CloudRegionError::UnknownRegion));
    }
    let request_id = request.boundary.request_id;
    let data = catalog
        .azs_for_region(&region_code)
        .map(|az| az_record(az, catalog))
        .collect();
    Ok(CloudAzListSuccessResponse::ok(data, request_id))
}

fn validate_boundary(boundary: &CloudRegionApiBoundaryContext) -> Result<(), CloudRegionApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudRegionApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudRegionApiError::EmptyTenantHeader);
    }
    Ok(())
}

fn validate_path_region_code(path_region_code: &str) -> Result<(), CloudRegionApiError> {
    if path_region_code.trim().is_empty() {
        return Err(CloudRegionApiError::EmptyPathRegionCode);
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudRegionApiBoundaryContext,
    principal: &CloudRegionApiPrincipal,
) -> Result<(), CloudRegionApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudRegionApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id {
        return Err(CloudRegionApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudRegionApiPrincipal,
    authorization: &CloudRegionApiAuthorization,
    surface: &str,
) -> Result<(), CloudRegionApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudRegionApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudRegionApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudRegionApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudRegionApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn region_record(region: &CloudRegion) -> CloudRegionRecord {
    CloudRegionRecord {
        code: region.code.value.value.clone(),
        display_name: region.display_name.value.clone(),
        regulatory_packs: region
            .regulatory_packs
            .value
            .iter()
            .map(|value| CloudRegionRegulatoryPackRef {
                value: value.clone(),
            })
            .collect(),
        azs: region
            .azs
            .value
            .iter()
            .map(|az| CloudRegionAzRef {
                value: az.value.clone(),
            })
            .collect(),
        state: region_state_label(region.state.value).to_string(),
        provider_facing: region.provider_facing.value,
        residency_strictness: residency_class_label(&region.residency_strictness.value).to_string(),
        created_at_epoch_seconds: region.created_at_epoch_seconds.value,
        schema_version: region.schema_version.value,
    }
}

fn az_record(az: &CloudAz, catalog: &CloudRegionCatalog) -> CloudAzRecord {
    let cell_isolation_evidence: Vec<_> = catalog
        .cells_for_region(&az.region_code.value)
        .filter(|cell| cell.az_code.value == az.code.value)
        .map(cell_isolation_evidence_record)
        .collect();
    CloudAzRecord {
        code: az.code.value.value.clone(),
        region_code: az.region_code.value.value.clone(),
        power_zones: az
            .power_zones
            .value
            .iter()
            .map(|value| CloudRegionPowerZoneRef {
                value: value.clone(),
            })
            .collect(),
        cells: az
            .cells
            .value
            .iter()
            .map(|cell| CloudRegionCellRef {
                value: cell.value.clone(),
            })
            .collect(),
        cell_isolation_evidence,
        state: az_state_label(az.state.value).to_string(),
        created_at_epoch_seconds: az.created_at_epoch_seconds.value,
        schema_version: az.schema_version.value,
    }
}

fn cell_isolation_evidence_record(cell: &CloudCell) -> CloudCellIsolationEvidenceRecord {
    CloudCellIsolationEvidenceRecord {
        cell_id: cell.id.value.value.clone(),
        region_code: cell.region_code.value.value.clone(),
        az_code: cell.az_code.value.value.clone(),
        state: cloud_cell_state_label(cell.state.value).to_string(),
        tenant_density: tenant_density_label(cell.tenant_density.value).to_string(),
        allowed_residency: cell
            .allowed_residency
            .value
            .iter()
            .map(|residency_class| residency_class_label(residency_class).to_string())
            .collect(),
        evidence_ref: format!(
            "cell-isolation://{}/{}/{}",
            cell.region_code.value.value, cell.az_code.value.value, cell.id.value.value
        ),
        schema_version: cell.schema_version.value,
    }
}

fn region_state_label(state: RegionState) -> &'static str {
    match state {
        RegionState::Planned => "planned",
        RegionState::Preview => "preview",
        RegionState::Ga => "ga",
        RegionState::Retiring => "retiring",
    }
}

fn az_state_label(state: AzState) -> &'static str {
    match state {
        AzState::Planned => "planned",
        AzState::Active => "active",
        AzState::DrOnly => "dr_only",
        AzState::Retiring => "retiring",
    }
}

fn cloud_cell_state_label(state: CloudCellState) -> &'static str {
    match state {
        CloudCellState::Planned => "planned",
        CloudCellState::Active => "active",
        CloudCellState::DrOnly => "dr_only",
        CloudCellState::Draining => "draining",
        CloudCellState::Retired => "retired",
    }
}

fn tenant_density_label(density: TenantDensityClass) -> &'static str {
    match density {
        TenantDensityClass::Shared => "shared",
        TenantDensityClass::Dedicated => "dedicated",
        TenantDensityClass::Sovereign => "sovereign",
        TenantDensityClass::AirGapped => "air_gapped",
        TenantDensityClass::FoundryRuntime => "foundry_runtime",
    }
}

fn residency_class_label(residency_class: &ResidencyClass) -> &'static str {
    match residency_class {
        ResidencyClass::StrictHomeRegion => "strict_home_region",
        ResidencyClass::HomeWithRecoveryFailover => "home_with_recovery_failover",
        ResidencyClass::Global => "global",
        ResidencyClass::PerPack(_) => "per_pack",
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
        CloudRegionError::InvalidRegulatoryPack => {
            "regulatory pack id must use the pack- prefix"
        }
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
