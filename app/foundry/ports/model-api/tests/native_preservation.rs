use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionTypeDefinition, ActionTypeId, AutonomyTier, EntityTypeDefinition, EntityTypeId,
    EntityTypePropertyDefinition, LinkCardinality, LinkTypeDefinition, LinkTypeId, OntologyEngine,
    PropertyTier,
};
use foundry_model_api::{conformance::*, *};

#[test]
fn native_privacy_alias_identity_survives_capture_wire_and_registry_restore() {
    let classes = [
        DataClass::Public,
        DataClass::InternalOnly,
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
    ];
    let properties = classes
        .into_iter()
        .enumerate()
        .map(|(index, class)| {
            EntityTypePropertyDefinition::new(
                format!("field_{index}"),
                PropertyTier::Scalar,
                PrivacyDataClass::new(class).unwrap(),
                false,
            )
            .unwrap()
        })
        .collect();
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(
            EntityTypeDefinition::new(
                "ten_model",
                EntityTypeId::new("ety_classes").unwrap(),
                "Classes",
                properties,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let captured = ModelBaseline::capture(&engine, "ten_model").unwrap();
    let bytes = serde_json::to_vec(&captured).unwrap();
    let reopened: ModelBaseline = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(reopened, captured);
    assert_eq!(reopened.registry("ten_model").unwrap(), engine);
    for class in [
        "Secret",
        "Audit",
        "Children",
        " InternalOnly",
        "INTERNAL_ONLY",
    ] {
        assert!(
            serde_json::from_value::<ModelPrivacy>(serde_json::json!(class)).is_err(),
            "{class}"
        );
    }
}

#[test]
fn metadata_restore_preserves_admitted_prior_revision_ordering() {
    let mut trusted = baseline().registry("ten_model").unwrap();
    let record = EntityTypeId::new("ety_record").unwrap();
    for revision in [2, 3] {
        let mut next = trusted.entity_type("ten_model", &record).unwrap().clone();
        next.revision = revision;
        trusted.evolve_entity_type(next).unwrap();
    }
    let mut baseline = ModelBaseline::capture(&trusted, "ten_model").unwrap();
    assert_eq!(baseline.prior_entity_revisions.len(), 2);
    baseline.prior_entity_revisions.reverse();
    assert_eq!(baseline.registry("ten_model").unwrap(), trusted);
    let history = ModelHistory {
        binding: binding(),
        publications: vec![ModelPublication {
            generation: 1,
            document: baseline.document.clone(),
            receipt: None,
        }],
        baseline,
    };
    assert_eq!(history.registry().unwrap(), trusted);
}

#[test]
fn runtime_restore_preserves_trusted_actions_revisions_and_refuses_another_baseline() {
    let mut trusted = baseline().registry("ten_model").unwrap();
    let action_id = ActionTypeId::new("aty_write").unwrap();
    trusted
        .register_action_type(
            ActionTypeDefinition::new(
                "ten_model",
                action_id.clone(),
                EntityTypeId::new("ety_record").unwrap(),
                "ops-console",
                AutonomyTier::T1Assist,
                "record.written",
            )
            .unwrap(),
        )
        .unwrap();
    let baseline = ModelBaseline::capture(&trusted, "ten_model").unwrap();
    let mut store = ReferenceModelStore::initialize(binding(), baseline).unwrap();
    let draft = store.create_draft(&caller(), 1, additive_model()).unwrap();
    let prepared = store.prepare(&caller(), draft.id, 1, 1).unwrap();
    store.publish(&caller(), &prepared).unwrap();
    let history = store.history(&caller()).unwrap();
    assert!(
        history
            .registry()
            .unwrap()
            .action_type("ten_model", &action_id)
            .is_none()
    );
    let restored = history.registry_with_runtime_baseline(&trusted).unwrap();
    assert_eq!(
        restored.action_type("ten_model", &action_id),
        trusted.action_type("ten_model", &action_id)
    );
    let record = EntityTypeId::new("ety_record").unwrap();
    assert_eq!(
        restored.entity_type("ten_model", &record).unwrap().revision,
        2
    );
    assert_eq!(
        restored.entity_type_at_revision("ten_model", &record, 1),
        trusted.entity_type("ten_model", &record)
    );
    let mut another = trusted.clone();
    let mut changed = another.entity_type("ten_model", &record).unwrap().clone();
    changed.revision = 2;
    another.evolve_entity_type(changed).unwrap();
    assert_eq!(
        history.registry_with_runtime_baseline(&another),
        Err(ModelError::BindingMismatch)
    );
}

#[test]
fn runtime_baseline_cannot_carry_projected_link_instances() {
    let mut trusted = baseline().registry("ten_model").unwrap();
    let link = LinkTypeId::new("lty_related").unwrap();
    trusted
        .register_link_type(
            LinkTypeDefinition::new(
                "ten_model",
                link.clone(),
                EntityTypeId::new("ety_record").unwrap(),
                EntityTypeId::new("ety_record").unwrap(),
                LinkCardinality::ManyToMany,
                false,
            )
            .unwrap(),
        )
        .unwrap();
    let model = ModelBaseline::capture(&trusted, "ten_model").unwrap();
    let store = ReferenceModelStore::initialize(binding(), model).unwrap();
    let history = store.history(&caller()).unwrap();
    trusted
        .register_link_instance("ten_model", &link, "ent_from", "ent_to")
        .unwrap();
    assert_eq!(
        history.registry_with_runtime_baseline(&trusted),
        Err(ModelError::BindingMismatch)
    );
}

#[test]
fn legacy_string_key_declarations_are_preserved_across_all_native_tiers() {
    for tier in [
        ModelTier::Scalar,
        ModelTier::Vector,
        ModelTier::Timeseries,
        ModelTier::Geo,
        ModelTier::Ciphertext,
        ModelTier::Struct,
    ] {
        let mut model = baseline().document;
        let mut entity = model.entities[0].clone();
        entity.id = "ety_legacy".into();
        entity.primary_key_property = Some("name".into());
        entity.properties[0].value_type = None;
        entity.properties[0].tier = tier;
        model.entities.push(entity);
        let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
        let draft = store.create_draft(&caller(), 1, model).unwrap();
        let prepared = store
            .prepare(&caller(), draft.id, draft.version, draft.base_generation)
            .unwrap();
        let native = prepared
            .model_registry()
            .entity_type("ten_model", &EntityTypeId::new("ety_legacy").unwrap())
            .unwrap();
        assert_eq!(native.properties[0].value_type, None);
        assert_eq!(native.primary_key_property.as_deref(), Some("name"));
        assert_eq!(
            ModelDocument::capture(prepared.model_registry(), "ten_model")
                .unwrap()
                .entities
                .iter()
                .find(|row| row.id == "ety_legacy")
                .unwrap()
                .properties[0]
                .tier,
            tier
        );
    }
}

#[test]
fn shared_receipt_helper_refuses_a_preparation_for_an_identical_different_draft() {
    let mut store = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    let draft_a = store.create_draft(&caller(), 1, additive_model()).unwrap();
    let draft_b = store.create_draft(&caller(), 1, additive_model()).unwrap();
    let prepared_a = store.prepare(&caller(), draft_a.id, 1, 1).unwrap();
    let prepared_b = store.prepare(&caller(), draft_b.id, 1, 1).unwrap();
    let history = store.history(&caller()).unwrap();
    assert_eq!(draft_a.document, draft_b.document);
    assert_ne!(draft_a.id, draft_b.id);
    assert_eq!(
        history.publication_receipt(&caller(), &draft_b, &prepared_a),
        Err(ModelError::BindingMismatch)
    );
    let original = store.publish(&caller(), &prepared_b).unwrap();
    let closed_b = store.open_draft(&caller(), draft_b.id).unwrap();
    let history = store.history(&caller()).unwrap();
    assert_eq!(
        history.publication_receipt(&caller(), &closed_b, &prepared_a),
        Err(ModelError::BindingMismatch)
    );
    assert_eq!(
        history
            .publication_receipt(&caller(), &closed_b, &prepared_b)
            .unwrap(),
        original
    );
}
