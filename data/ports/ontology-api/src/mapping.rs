//! Converters between the wire DTOs and the kernel's object plane, plus
//! fingerprints, labels, and kernel-error mappings.

use data_boundary_kernel::{PrivacyDataClass, parse_data_class_label};
use data_ontology_domain::{ObjectEntity, ObjectGraphError, ObjectProperty, PropertyTier};

use crate::codes::ObjectGraphEntityUpsertApiErrorCode;
use crate::contract::*;
use crate::error::ObjectGraphEntityUpsertApiError;

pub(crate) fn object_entity_from_request(
    request: &ObjectGraphEntityUpsertRequest,
) -> Result<ObjectEntity, ObjectGraphEntityUpsertApiError> {
    let properties = request
        .property_refs
        .iter()
        .map(object_property_from_ref)
        .collect::<Result<Vec<_>, _>>()?;
    ObjectEntity::new(
        request.tenant_id.clone(),
        request.entity_id.clone(),
        request.entity_type.clone(),
        properties,
    )
    .map_err(ObjectGraphEntityUpsertApiError::Kernel)
}

pub(crate) fn object_property_from_ref(
    property: &ObjectGraphEntityPropertyRef,
) -> Result<ObjectProperty, ObjectGraphEntityUpsertApiError> {
    Ok(ObjectProperty::new(
        property.name.clone(),
        property.value.clone(),
        parse_property_tier(&property.tier)?,
        parse_property_data_class(&property.data_class)?,
    ))
}

pub(crate) fn parse_property_tier(
    tier: &str,
) -> Result<PropertyTier, ObjectGraphEntityUpsertApiError> {
    match tier.trim() {
        "scalar" => Ok(PropertyTier::Scalar),
        "vector" => Ok(PropertyTier::Vector),
        "timeseries" => Ok(PropertyTier::Timeseries),
        "geo" => Ok(PropertyTier::Geo),
        "ciphertext" => Ok(PropertyTier::Ciphertext),
        "struct" => Ok(PropertyTier::Struct),
        _ => Err(ObjectGraphEntityUpsertApiError::InvalidPropertyTier {
            tier: tier.to_string(),
        }),
    }
}

pub(crate) fn parse_property_data_class(
    label: &str,
) -> Result<PrivacyDataClass, ObjectGraphEntityUpsertApiError> {
    let data_class = parse_data_class_label(label).ok_or_else(|| {
        ObjectGraphEntityUpsertApiError::InvalidPropertyDataClass {
            data_class: label.to_string(),
        }
    })?;
    PrivacyDataClass::try_from(data_class).map_err(|_| {
        ObjectGraphEntityUpsertApiError::InvalidPropertyDataClass {
            data_class: label.to_string(),
        }
    })
}

pub(crate) fn object_graph_entity_record(
    entity: &ObjectEntity,
) -> Result<ObjectGraphEntityRecord, ObjectGraphEntityUpsertApiError> {
    Ok(ObjectGraphEntityRecord {
        tenant_id: entity.tenant_id.clone(),
        entity_id: entity.id.clone(),
        entity_type: entity.entity_type.value.clone(),
        property_refs: entity
            .properties
            .values()
            .map(object_graph_property_ref)
            .collect::<Result<Vec<_>, _>>()?,
        schema_version: 1,
    })
}

pub(crate) fn object_graph_property_ref(
    property: &ObjectProperty,
) -> Result<ObjectGraphEntityPropertyRef, ObjectGraphEntityUpsertApiError> {
    let value = property.value.value.as_str().ok_or_else(|| {
        ObjectGraphEntityUpsertApiError::NonStringPropertyValue {
            name: property.name.clone(),
        }
    })?;
    Ok(ObjectGraphEntityPropertyRef {
        name: property.name.clone(),
        value: value.to_string(),
        tier: property_tier_label(property.tier).to_string(),
        data_class: property.value.data_class.label().to_string(),
    })
}

pub(crate) fn object_graph_entity_mutation_event(
    request: &ObjectGraphEntityUpsertApiRequest,
    result: &str,
) -> ObjectGraphEntityMutationEvent {
    ObjectGraphEntityMutationEvent {
        event_id: format!("evt_og_{}", request.boundary.request_id),
        tenant_id: request.body.tenant_id.clone(),
        entity_id: request.body.entity_id.clone(),
        request_id: request.boundary.request_id.clone(),
        result: result.to_string(),
    }
}

pub(crate) fn property_tier_label(tier: PropertyTier) -> &'static str {
    match tier {
        PropertyTier::Scalar => "scalar",
        PropertyTier::Vector => "vector",
        PropertyTier::Timeseries => "timeseries",
        PropertyTier::Geo => "geo",
        PropertyTier::Ciphertext => "ciphertext",
        PropertyTier::Struct => "struct",
    }
}

pub(crate) fn idempotency_key_for(
    boundary: &ObjectGraphApiBoundaryContext,
    principal: &ObjectGraphApiPrincipal,
    surface: &str,
) -> ObjectGraphEntityUpsertIdempotencyLedgerKey {
    ObjectGraphEntityUpsertIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

pub(crate) fn object_graph_entity_upsert_fingerprint_for(
    request: &ObjectGraphEntityUpsertApiRequest,
) -> ObjectGraphEntityUpsertRequestFingerprint {
    let mut canonical = format!(
        "path_tenant_id={};path_entity_id={};tenant_id={};entity_id={};entity_type={};",
        request.path_tenant_id,
        request.path_entity_id,
        request.body.tenant_id,
        request.body.entity_id,
        request.body.entity_type
    );
    for property in &request.body.property_refs {
        canonical.push_str(&format!(
            "property[name={},value={},tier={},data_class={}];",
            property.name, property.value, property.tier, property.data_class
        ));
    }
    ObjectGraphEntityUpsertRequestFingerprint { canonical }
}

pub(crate) fn object_graph_kernel_error_code(
    error: &ObjectGraphError,
) -> ObjectGraphEntityUpsertApiErrorCode {
    match error {
        ObjectGraphError::InvalidEntityId => {
            ObjectGraphEntityUpsertApiErrorCode::KernelInvalidEntityId
        }
        ObjectGraphError::EmptyEntityType => {
            ObjectGraphEntityUpsertApiErrorCode::KernelEmptyEntityType
        }
        ObjectGraphError::MissingProperties => {
            ObjectGraphEntityUpsertApiErrorCode::KernelMissingProperties
        }
        ObjectGraphError::EmptyPropertyName => {
            ObjectGraphEntityUpsertApiErrorCode::KernelEmptyPropertyName
        }
        ObjectGraphError::InvalidDataClass => {
            ObjectGraphEntityUpsertApiErrorCode::KernelInvalidDataClass
        }
    }
}

pub(crate) fn object_graph_kernel_error_message(error: &ObjectGraphError) -> &'static str {
    match error {
        ObjectGraphError::InvalidEntityId => "Object Graph entity id must use the ent_ shape",
        ObjectGraphError::EmptyEntityType => "Object Graph entity type is required",
        ObjectGraphError::MissingProperties => "Object Graph entity requires at least one property",
        ObjectGraphError::EmptyPropertyName => "Object Graph property names must be non-empty",
        ObjectGraphError::InvalidDataClass => "Object Graph property data class is invalid",
    }
}

pub(crate) fn object_graph_kernel_issue(error: &ObjectGraphError) -> &'static str {
    match error {
        ObjectGraphError::InvalidEntityId => "entity_id must start with ent_",
        ObjectGraphError::EmptyEntityType => "entity_type must be non-empty",
        ObjectGraphError::MissingProperties => "properties must contain at least one property",
        ObjectGraphError::EmptyPropertyName => "property name must be non-empty",
        ObjectGraphError::InvalidDataClass => "property data_class must be a privacy class",
    }
}

pub(crate) fn detail(field: &str, issue: &str) -> ObjectGraphEntityUpsertApiErrorDetail {
    ObjectGraphEntityUpsertApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
