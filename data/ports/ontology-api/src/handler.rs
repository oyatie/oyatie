//! The upsert entry points and boundary/binding/authorization validators.

use data_ontology_domain::ObjectGraphError;

use crate::OBJECT_GRAPH_ENTITY_UPSERT_SURFACE;
use crate::contract::*;
use crate::error::ObjectGraphEntityUpsertApiError;
use crate::mapping::*;

pub fn validate_object_graph_entity_upsert_request(
    request: &ObjectGraphEntityUpsertApiRequest,
) -> Result<(), ObjectGraphEntityUpsertApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_body_binding(request)?;
    validate_principal_binding(&request.boundary, &request.principal)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        OBJECT_GRAPH_ENTITY_UPSERT_SURFACE,
    )?;
    for property in &request.body.property_refs {
        parse_property_tier(&property.tier)?;
        parse_property_data_class(&property.data_class)?;
    }
    Ok(())
}

pub fn upsert_object_graph_entity_from_api(
    directory: &mut ObjectGraphEntityDirectory,
    idempotency_ledger: &mut ObjectGraphEntityUpsertIdempotencyLedger,
    request: ObjectGraphEntityUpsertApiRequest,
) -> Result<ObjectGraphEntityUpsertSuccessResponse, ObjectGraphEntityUpsertApiError> {
    validate_object_graph_entity_upsert_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        OBJECT_GRAPH_ENTITY_UPSERT_SURFACE,
    );
    let fingerprint = object_graph_entity_upsert_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(ObjectGraphEntityUpsertApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let entity_key = ObjectGraphEntityKey {
        tenant_id: request.body.tenant_id.clone(),
        entity_id: request.body.entity_id.clone(),
    };
    let result = if directory.entities.contains_key(&entity_key) {
        "updated"
    } else {
        "created"
    };
    let entity = object_entity_from_request(&request.body)?;
    directory.entities.insert(entity_key, entity.clone());
    let event = object_graph_entity_mutation_event(&request, result);
    directory.events.push(event.clone());
    let response = ObjectGraphEntityUpsertSuccessResponse {
        data: object_graph_entity_record(&entity)?,
        metadata: ObjectGraphEntityUpsertMetadata {
            request_id: request.boundary.request_id.clone(),
            tenant_id: request.boundary.tenant_id.clone(),
            principal_id: request.principal.principal_id.clone(),
            result: result.to_string(),
            event_id: event.event_id,
        },
    };
    idempotency_ledger.entries.insert(
        key,
        ObjectGraphEntityUpsertIdempotencyLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn validate_boundary(
    boundary: &ObjectGraphApiBoundaryContext,
) -> Result<(), ObjectGraphEntityUpsertApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_body_binding(
    request: &ObjectGraphEntityUpsertApiRequest,
) -> Result<(), ObjectGraphEntityUpsertApiError> {
    if request.path_tenant_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyPathTenantId);
    }
    if request.path_entity_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyPathEntityId);
    }
    if request.path_tenant_id != request.body.tenant_id {
        return Err(ObjectGraphEntityUpsertApiError::TenantPathBodyMismatch {
            path_tenant_id: request.path_tenant_id.clone(),
            body_tenant_id: request.body.tenant_id.clone(),
        });
    }
    if request.path_entity_id != request.body.entity_id {
        return Err(ObjectGraphEntityUpsertApiError::EntityPathBodyMismatch {
            path_entity_id: request.path_entity_id.clone(),
            body_entity_id: request.body.entity_id.clone(),
        });
    }
    Ok(())
}

fn validate_principal_binding(
    boundary: &ObjectGraphApiBoundaryContext,
    principal: &ObjectGraphApiPrincipal,
) -> Result<(), ObjectGraphEntityUpsertApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyPrincipalId);
    }
    if principal.tenant_id != boundary.tenant_id {
        return Err(ObjectGraphEntityUpsertApiError::PrincipalTenantMismatch {
            principal_tenant_id: principal.tenant_id.clone(),
            boundary_tenant_id: boundary.tenant_id.clone(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &ObjectGraphApiPrincipal,
    authorization: &ObjectGraphApiAuthorization,
    surface: &str,
) -> Result<(), ObjectGraphEntityUpsertApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(
            ObjectGraphEntityUpsertApiError::AuthorizationTenantMismatch {
                authorization_tenant_id: authorization.tenant_id.clone(),
                principal_tenant_id: principal.tenant_id.clone(),
            },
        );
    }
    if authorization.principal_id != principal.principal_id {
        return Err(
            ObjectGraphEntityUpsertApiError::AuthorizationPrincipalMismatch {
                authorization_principal_id: authorization.principal_id.clone(),
                principal_id: principal.principal_id.clone(),
            },
        );
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed| allowed == surface)
    {
        return Err(ObjectGraphEntityUpsertApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}
