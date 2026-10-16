use foundry_model_api::*;

fn entity(id: &str, name: &str, extra: Vec<ModelProperty>) -> ModelEntity {
    let mut properties = vec![
        conformance::property("id", true),
        conformance::property("name", true),
    ];
    properties.extend(extra);
    ModelEntity {
        id: id.into(),
        display_name: name.into(),
        properties,
        revision: 1,
        pillar: None,
        primary_key_property: Some("id".into()),
        title_property: Some("name".into()),
        display: Some(ModelDisplay {
            display_name: Some(name.into()),
            description: Some(format!("{name} declaration for controlled authoring proof")),
            icon: None,
            color: None,
            plural_name: Some(format!("{name}s")),
        }),
    }
}

pub fn document(mut document: ModelDocument) -> ModelDocument {
    let tags = ModelProperty {
        name: "skills".into(),
        tier: ModelTier::Vector,
        data_class: ModelPrivacy::internal_only(),
        required: false,
        display: None,
        default_value: None,
        value_type: Some(ModelValueType::Array {
            element: Box::new(ModelValueType::Scalar {
                kind: ModelScalar::String,
            }),
        }),
    };
    let address = ModelProperty {
        name: "work_address".into(),
        tier: ModelTier::Struct,
        data_class: ModelPrivacy::internal_only(),
        required: false,
        display: None,
        default_value: None,
        value_type: Some(ModelValueType::Struct {
            fields: vec![
                ModelStructField {
                    name: "city".into(),
                    required: true,
                    value_type: ModelValueType::Scalar {
                        kind: ModelScalar::String,
                    },
                },
                ModelStructField {
                    name: "postal_code".into(),
                    required: false,
                    value_type: ModelValueType::Scalar {
                        kind: ModelScalar::String,
                    },
                },
            ],
        }),
    };
    document.named_string_types.push(ModelNamedStringType {
        id: "vty_employment_status".into(),
        revision: 1,
        display_name: "EmploymentStatus".into(),
        members: vec!["active".into(), "onLeave".into(), "ended".into()],
    });
    let mut status = conformance::property("employment_status", false);
    status.value_type = Some(ModelValueType::NamedString {
        type_id: "vty_employment_status".into(),
        revision: 1,
    });
    document.entities.push(entity(
        "ety_employee",
        "Employee",
        vec![tags, address, status],
    ));
    document
        .entities
        .push(entity("ety_department", "Department", vec![]));
    document.links.push(ModelLink {
        id: "lty_member_of".into(),
        from_entity_type: "ety_employee".into(),
        to_entity_type: "ety_department".into(),
        cardinality: ModelCardinality::ManyToMany,
        allow_cross_tenant: false,
        revision: 1,
        display: Some(ModelDisplay {
            display_name: Some("Member of".into()),
            description: None,
            icon: None,
            color: None,
            plural_name: None,
        }),
    });
    document
}
