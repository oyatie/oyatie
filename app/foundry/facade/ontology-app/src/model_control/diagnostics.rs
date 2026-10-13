use super::*;
use data_ontology_kernel::OntologyEngineError as Native;

pub(super) fn diagnostic(error: ModelError) -> Option<ModelDiagnostic> {
    let (code, path, message) = match error {
        ModelError::NoChanges => (
            "no_changes",
            "document".into(),
            "Make a model change before validation.",
        ),
        ModelError::MalformedDocument => (
            "malformed_model",
            "document".into(),
            "Use unique, valid type and relationship declarations.",
        ),
        ModelError::NewDefinitionRevision { path } => (
            "new_definition_revision",
            path,
            "A new definition starts at revision 1.",
        ),
        ModelError::UnsupportedRemoval { path } => (
            "unsupported_removal",
            path,
            "Retain the existing definition; removal is not supported.",
        ),
        ModelError::UnsupportedKeyAdoption { entity_type } => (
            "unsupported_key_adoption",
            format!("entities.{entity_type}.primary_key_property"),
            "An existing type without a key cannot adopt one in this publication profile.",
        ),
        ModelError::UnsupportedPublicationDataClass { path, .. } => (
            "unsupported_data_class",
            path,
            "This publication profile supports InternalOnly declarations.",
        ),
        ModelError::CrossTenantLink { link_type } => (
            "cross_tenant_link",
            format!("links.{link_type}"),
            "Use a relationship within the tenant.",
        ),
        ModelError::Native { path, cause } => return Some(native(path, cause)),
        _ => return None,
    };
    Some(ModelDiagnostic {
        code: code.into(),
        path,
        message: message.into(),
    })
}

fn native(mut path: String, cause: Native) -> ModelDiagnostic {
    let (code, message) = match cause {
        Native::InvalidTypeId => (
            "invalid_entity_type_id",
            "Use a valid entity type identifier.",
        ),
        Native::InvalidLinkTypeId => (
            "invalid_link_type_id",
            "Use a valid relationship identifier.",
        ),
        Native::EmptyDisplayName => {
            path.push_str(".display_name");
            ("empty_display_name", "Enter a display name.")
        }
        Native::EmptyProperties => ("empty_properties", "Declare at least one property."),
        Native::EmptyPropertyName => ("empty_property_name", "Enter a property name."),
        Native::DuplicatePropertyName { name } => {
            path.push_str(&format!(".properties.{name}"));
            ("duplicate_property_name", "Use a unique property name.")
        }
        Native::UnknownEntityTypeEndpoint => (
            "unknown_endpoint",
            "Choose an entity type declared in this tenant.",
        ),
        Native::CrossPillarLink => (
            "cross_pillar_link",
            "Relationship endpoints must share a pillar or be pillar-agnostic.",
        ),
        Native::NonMonotonicRevision => {
            path.push_str(".revision");
            (
                "revision_not_increased",
                "Increase the revision for a changed existing definition.",
            )
        }
        Native::IncompatibleSchemaEvolution => (
            "incompatible_evolution",
            "Retain existing properties unchanged and make new properties optional.",
        ),
        Native::PillarChangedOnEvolution => {
            path.push_str(".pillar");
            ("frozen_pillar", "Retain the existing pillar.")
        }
        Native::BlankDisplayField { field } => {
            path.push_str(&format!(".display.{field}"));
            (
                "blank_display_field",
                "Enter display text or remove the optional display field.",
            )
        }
        Native::FrozenFieldChangedOnEvolution { field } => {
            path.push_str(&format!(".{field}"));
            (
                "frozen_field_changed",
                "Retain this field's existing value.",
            )
        }
        Native::InvalidValueType { name, .. } => {
            path.push_str(&format!(".properties.{name}"));
            (
                "invalid_value_type",
                "Use a valid scalar, array or struct declaration.",
            )
        }
        Native::ValueTypeTierMismatch { name } => {
            path.push_str(&format!(".properties.{name}"));
            (
                "value_type_tier_mismatch",
                "Match the property tier to its value declaration.",
            )
        }
        Native::DesignatedPropertyNotDeclared { .. } => (
            "designated_property_not_declared",
            "Choose a declared property for the primary key or title.",
        ),
        Native::PrimaryKeyPropertyNotRequired { name } => {
            path.push_str(&format!(".properties.{name}"));
            (
                "primary_key_not_required",
                "Make the primary key property required.",
            )
        }
        Native::PrimaryKeyPropertyNotScalar { name } => {
            path.push_str(&format!(".properties.{name}"));
            (
                "primary_key_not_scalar",
                "Choose a scalar primary key property.",
            )
        }
        Native::PrimaryKeyChangedOnEvolution => {
            path.push_str(".primary_key_property");
            (
                "frozen_primary_key",
                "Retain the existing primary key designation.",
            )
        }
        _ => (
            "native_invalid",
            "Correct this declaration to satisfy native Ontology model constraints.",
        ),
    };
    ModelDiagnostic {
        code: code.into(),
        path,
        message: message.into(),
    }
}
