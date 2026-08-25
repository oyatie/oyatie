use cell_region::{CloudRegionCatalog, CloudRegionError, RegionCode};

use crate::model::{
    CLOUD_AZ_LIST_SURFACE, CLOUD_REGION_LIST_SURFACE, CloudAzListApiRequest,
    CloudAzListSuccessResponse, CloudRegionApiAuthorization, CloudRegionApiBoundaryContext,
    CloudRegionApiError, CloudRegionApiPrincipal, CloudRegionListApiRequest,
    CloudRegionListSuccessResponse,
};
use crate::projection::{az_record, region_record};

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
    RegionCode::new(request.path_region_code.clone())
        .map_err(CloudRegionError::from)
        .map_err(CloudRegionApiError::Region)
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
