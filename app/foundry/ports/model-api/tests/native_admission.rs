use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{EntityTypeId, OntologyEngine, OntologyEngineError};
use foundry_model_api::{conformance::*, *};

fn preparation(model: ModelDocument) -> Result<PreparedPublication, ModelError> {
    let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    let draft = store.create_draft(&caller(), 1, model).unwrap();
    store.prepare(&caller(), draft.id, draft.version, draft.base_generation)
}

fn new_entity() -> ModelEntity {
    let mut entity = baseline().document.entities.remove(0);
    entity.id = "ety_related".into();
    entity.display_name = "관련 <script>".into();
    entity.primary_key_property = Some("name".into());
    entity
}

#[test]
fn new_typed_scalar_keys_arrays_structs_and_same_tenant_links_use_native_laws() {
    for kind in [
        ModelScalar::String,
        ModelScalar::Integer,
        ModelScalar::Double,
        ModelScalar::Boolean,
        ModelScalar::Date,
        ModelScalar::Timestamp,
    ] {
        let mut model = additive_model();
        let mut entity = new_entity();
        entity.properties[0].value_type = Some(ModelValueType::Scalar { kind });
        let mut array = property("tags", false);
        array.tier = ModelTier::Vector;
        array.value_type = Some(ModelValueType::Array {
            element: Box::new(ModelValueType::Scalar {
                kind: ModelScalar::String,
            }),
        });
        let mut structure = property("address", false);
        structure.tier = ModelTier::Struct;
        structure.value_type = Some(ModelValueType::Struct {
            fields: vec![ModelStructField {
                name: "city".into(),
                required: false,
                value_type: ModelValueType::Scalar {
                    kind: ModelScalar::String,
                },
            }],
        });
        entity.properties.extend([array, structure]);
        model.entities.push(entity);
        model.links.push(ModelLink {
            id: "lty_related".into(),
            from_entity_type: "ety_record".into(),
            to_entity_type: "ety_related".into(),
            cardinality: ModelCardinality::OneToMany,
            allow_cross_tenant: false,
            revision: 1,
            display: None,
        });
        let prepared = preparation(model).unwrap();
        assert_eq!(
            prepared.model_registry().entity_types("ten_model").count(),
            2
        );
        assert_eq!(prepared.model_registry().link_types("ten_model").count(), 1);
    }
}

#[test]
fn model_evolution_refuses_first_key_adoption_and_composite_new_keys() {
    let mut model = additive_model();
    model.entities[0].primary_key_property = Some("name".into());
    assert!(matches!(
        preparation(model),
        Err(ModelError::UnsupportedKeyAdoption { .. })
    ));
    for kind in [
        ModelValueType::Array {
            element: Box::new(ModelValueType::Scalar {
                kind: ModelScalar::String,
            }),
        },
        ModelValueType::Struct {
            fields: vec![ModelStructField {
                name: "x".into(),
                required: true,
                value_type: ModelValueType::Scalar {
                    kind: ModelScalar::String,
                },
            }],
        },
    ] {
        let mut entity = new_entity();
        entity.properties[0].tier = match &kind {
            ModelValueType::Array { .. } => ModelTier::Vector,
            _ => ModelTier::Struct,
        };
        entity.properties[0].value_type = Some(kind);
        let mut model = additive_model();
        model.entities.push(entity);
        assert!(matches!(
            preparation(model),
            Err(ModelError::Native {
                cause: OntologyEngineError::PrimaryKeyPropertyNotScalar { .. },
                ..
            })
        ));
    }
}

#[test]
fn frozen_property_pillar_and_required_addition_laws_remain_native() {
    for change in 0..6 {
        let mut model = additive_model();
        let entity = &mut model.entities[0];
        match change {
            0 => entity.properties[0].required = false,
            1 => entity.properties[0].value_type = None,
            2 => {
                entity.properties.remove(0);
            }
            3 => entity.properties[1].required = true,
            4 => entity.pillar = Some(ModelPillar::Person),
            _ => entity.properties[0].tier = ModelTier::Geo,
        };
        assert!(preparation(model).is_err(), "change {change}");
    }
    let mut model = additive_model();
    model.entities.clear();
    assert!(matches!(
        preparation(model),
        Err(ModelError::UnsupportedRemoval { .. })
    ));
}

#[test]
fn every_richer_privacy_variant_is_saved_without_downgrade_and_refused_for_publication() {
    for class in [
        DataClass::Public,
        DataClass::PiiIdentifying,
        DataClass::PiiSensitive,
        DataClass::Phi,
        DataClass::Pci,
        DataClass::PipaArticle23,
        DataClass::Financial,
        DataClass::Usage,
        DataClass::PiiQuasiIdentifier,
        DataClass::FinancialRegulatedCredit,
        DataClass::BehavioralTenantProduct,
        DataClass::BehavioralAds,
        DataClass::DeclaredPreference,
        DataClass::SearchQuery,
        DataClass::SensitivePipaArticle23,
    ] {
        let mut model = additive_model();
        model.entities[0].properties[1].data_class =
            ModelPrivacy::new(PrivacyDataClass::new(class).unwrap());
        let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
        let draft = store.create_draft(&caller(), 1, model).unwrap();
        assert_eq!(
            store
                .open_draft(&caller(), draft.id)
                .unwrap()
                .document
                .entities[0]
                .properties[1]
                .data_class
                .native()
                .data_class(),
            class
        );
        assert!(matches!(
            store.prepare(&caller(), draft.id, draft.version, draft.base_generation),
            Err(ModelError::UnsupportedPublicationDataClass { .. })
        ));
        assert_eq!(
            store.history(&caller()).unwrap().head().unwrap().generation,
            1
        );
    }
}

#[test]
fn native_baseline_capture_restores_every_accepted_revision_including_gaps() {
    let mut registry = baseline().registry("ten_model").unwrap();
    let id = EntityTypeId::new("ety_record").unwrap();
    let mut definition = registry.entity_type("ten_model", &id).unwrap().clone();
    definition.revision = u32::MAX;
    definition.display_name.value = "Last".into();
    registry.evolve_entity_type(definition).unwrap();
    let captured = ModelBaseline::capture(&registry, "ten_model").unwrap();
    assert_eq!(captured.prior_entity_revisions.len(), 1);
    let restored = captured.registry("ten_model").unwrap();
    assert_eq!(registry, restored);
    assert_eq!(
        restored
            .entity_type_revisions("ten_model")
            .map(|row| row.revision)
            .collect::<Vec<_>>(),
        [1, u32::MAX]
    );
    assert_eq!(restored.entity_type_at_revision("ten_model", &id, 2), None);
    let mut corrupt = captured;
    corrupt.prior_entity_revisions[0].id = "ety_absent".into();
    assert!(corrupt.registry("ten_model").is_err());
    assert_eq!(
        OntologyEngine::default()
            .entity_type_revisions("ten_model")
            .count(),
        0
    );
}

#[test]
fn cross_tenant_link_and_duplicate_invalid_struct_metadata_fail_without_native_installation() {
    let mut model = additive_model();
    model.links.push(ModelLink {
        id: "lty_cross".into(),
        from_entity_type: "ety_record".into(),
        to_entity_type: "ety_record".into(),
        cardinality: ModelCardinality::ManyToMany,
        allow_cross_tenant: true,
        revision: 1,
        display: None,
    });
    assert!(matches!(
        preparation(model),
        Err(ModelError::CrossTenantLink { .. })
    ));
    let mut model = additive_model();
    model.entities.push(model.entities[0].clone());
    assert_eq!(
        preparation(model).unwrap_err(),
        ModelError::MalformedDocument
    );
    let mut model = additive_model();
    let field = ModelStructField {
        name: "x".into(),
        required: false,
        value_type: ModelValueType::Scalar {
            kind: ModelScalar::String,
        },
    };
    model.entities[0].properties[1].tier = ModelTier::Struct;
    model.entities[0].properties[1].value_type = Some(ModelValueType::Struct {
        fields: vec![field.clone(), field],
    });
    assert!(matches!(preparation(model), Err(ModelError::Native { .. })));
}
