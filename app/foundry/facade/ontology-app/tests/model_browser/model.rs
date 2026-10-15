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
    document
        .entities
        .push(entity("ety_employee", "Employee", vec![tags, address]));
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

pub fn scale_document(baseline: ModelDocument) -> ModelDocument {
    use ModelCardinality::{ManyToMany as Many, OneToMany as One};
    let mut model = document(baseline);
    for (suffix, name) in [
        ("employment_contract", "Employment Contract"),
        ("leave_balance", "Leave Balance"),
        ("leave_policy", "Leave Policy"),
        ("leave_request", "Leave Request"),
        ("position", "Position"),
        ("team", "Team"),
    ] {
        model
            .entities
            .push(entity(&format!("ety_{suffix}"), name, vec![]));
    }
    for index in 0..43 {
        model.entities.push(entity(
            &format!("ety_context_{index:03}"),
            &format!("Context {index:03}"),
            vec![],
        ));
    }
    for (id, from, to, cardinality, label) in [
        (
            "employee_contract",
            "employee",
            "employment_contract",
            One,
            "Has contract",
        ),
        (
            "employee_balance",
            "employee",
            "leave_balance",
            One,
            "Has balances",
        ),
        (
            "employee_policy",
            "employee",
            "leave_policy",
            Many,
            "Covered by",
        ),
        (
            "employee_request",
            "employee",
            "leave_request",
            One,
            "Has request",
        ),
        ("employee_position", "employee", "position", Many, "Holds"),
        ("employee_team", "employee", "team", Many, "Belongs to"),
        ("employee_manages", "employee", "employee", One, "Manages"),
        (
            "employee_assignment",
            "employee",
            "department",
            Many,
            "Assigned to",
        ),
        (
            "department_employs",
            "department",
            "employee",
            One,
            "Employs",
        ),
        (
            "contract_governs",
            "employment_contract",
            "leave_policy",
            Many,
            "Governs",
        ),
    ] {
        model.links.push(ModelLink {
            id: format!("lty_{id}"),
            from_entity_type: format!("ety_{from}"),
            to_entity_type: format!("ety_{to}"),
            cardinality,
            allow_cross_tenant: false,
            revision: 1,
            display: Some(ModelDisplay {
                display_name: Some(label.into()),
                description: None,
                icon: None,
                color: None,
                plural_name: None,
            }),
        });
    }
    let unrelated: Vec<_> = std::iter::once("ety_record".to_owned())
        .chain((0..43).map(|index| format!("ety_context_{index:03}")))
        .collect();
    for index in 0..137 {
        model.links.push(ModelLink {
            id: format!("lty_context_{index:03}"),
            from_entity_type: unrelated[index % 44].clone(),
            to_entity_type: unrelated[(7 * index + 1) % 44].clone(),
            cardinality: Many,
            allow_cross_tenant: false,
            revision: 1,
            display: None,
        });
    }
    model
}

#[test]
fn scale_document_is_native_admitted() {
    let runtime = foundry_ontology_app::seed::registry_for(super::TENANT).unwrap();
    let baseline = ModelBaseline::capture(&runtime, super::TENANT).unwrap();
    let model = scale_document(baseline.document.clone());
    assert_eq!(model.entities[0], baseline.document.entities[0]);
    super::assert_scale_document(&model);
    let bytes = model.encode().unwrap();
    assert!(bytes.len() <= MAX_MODEL_BYTES);
    assert_eq!(ModelDocument::decode(&bytes).unwrap(), model);
    let caller = ModelCaller {
        tenant_id: super::TENANT.into(),
        owner: super::USER.into(),
    };
    let mut store = ReferenceModelStore::initialize(
        super::binding("controlled-scale-graph-model"),
        baseline.clone(),
    )
    .unwrap();
    let draft = store.create_draft(&caller, 1, model.clone()).unwrap();
    let prepared = store
        .prepare(&caller, draft.id, draft.version, draft.base_generation)
        .unwrap();
    assert_eq!(
        prepared
            .model_registry()
            .entity_types(super::TENANT)
            .count(),
        52
    );
    assert_eq!(
        prepared.model_registry().link_types(super::TENANT).count(),
        148
    );
    let schemas = schema_bytes(prepared.model_registry(), super::TENANT).unwrap();
    assert!(schemas.len() <= MAX_ONTOLOGY_SCHEMA_BYTES);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&schemas)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        52
    );
    let mut expected = model.clone();
    expected.entities.sort_by(|a, b| a.id.cmp(&b.id));
    expected.links.sort_by(|a, b| a.id.cmp(&b.id));
    assert_eq!(
        ModelBaseline::capture(prepared.model_registry(), super::TENANT)
            .unwrap()
            .document,
        expected
    );
    let receipt = store.publish(&caller, &prepared).unwrap();
    assert_eq!((receipt.prior_generation, receipt.generation), (1, 2));
    let history = store.history(&caller).unwrap();
    let restored = history.registry_with_runtime_baseline(&runtime).unwrap();
    assert_eq!(
        ModelBaseline::capture(&restored, super::TENANT)
            .unwrap()
            .document,
        expected
    );
    assert_eq!(schema_bytes(&restored, super::TENANT).unwrap(), schemas);
    let editable = super::editable_scale_draft(&mut store, &caller);
    let document_json = String::from_utf8(bytes).unwrap();
    let baseline_json = serde_json::to_string(&baseline).unwrap();
    let editable_json = serde_json::to_string(&editable).unwrap();
    println!("NATIVE_SCALE_DOCUMENT={document_json}");
    println!("NATIVE_SCALE_BASELINE={baseline_json}");
    println!("NATIVE_SCALE_EDITABLE_DRAFT={editable_json}");
    for change in 0..3 {
        let mut invalid = model.clone();
        let mut link = invalid.links[0].clone();
        link.id = "lty_refused".into();
        match change {
            0 => link.to_entity_type = "ety_absent".into(),
            1 => link.id = invalid.links[0].id.clone(),
            _ => link.allow_cross_tenant = true,
        }
        invalid.links.push(link);
        let before = store.head(&caller).unwrap();
        let draft = store
            .create_draft(&caller, before.generation, invalid)
            .unwrap();
        let error = store
            .prepare(&caller, draft.id, draft.version, draft.base_generation)
            .unwrap_err();
        match change {
            0 => assert!(matches!(error, ModelError::Native { path, cause }
                if path == "links.lty_refused" && format!("{cause:?}") == "UnknownEntityTypeEndpoint")),
            1 => assert_eq!(error, ModelError::MalformedDocument),
            _ => assert_eq!(
                error,
                ModelError::CrossTenantLink {
                    link_type: "lty_refused".into()
                }
            ),
        }
        assert_eq!(store.head(&caller).unwrap(), before);
    }
}
