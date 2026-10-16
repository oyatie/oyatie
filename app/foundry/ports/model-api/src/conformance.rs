//! Run unchanged against the reference fake and durable adapters.
use crate::*;

pub fn binding() -> ModelBinding {
    ModelBinding {
        tenant_id: "ten_model".into(),
        store_id: "store_model".into(),
        baseline_id: "baseline_model".into(),
    }
}
pub fn caller() -> ModelCaller {
    ModelCaller {
        tenant_id: "ten_model".into(),
        owner: "usr_author".into(),
    }
}
pub fn property(name: &str, required: bool) -> ModelProperty {
    ModelProperty {
        name: name.into(),
        tier: ModelTier::Scalar,
        data_class: ModelPrivacy::internal_only(),
        required,
        value_type: Some(ModelValueType::Scalar {
            kind: ModelScalar::String,
        }),
        display: None,
        default_value: None,
    }
}
pub fn baseline() -> ModelBaseline {
    ModelBaseline {
        document: ModelDocument {
            named_string_types: vec![],
            entities: vec![ModelEntity {
                id: "ety_record".into(),
                display_name: "Record".into(),
                properties: vec![property("name", true)],
                revision: 1,
                pillar: None,
                primary_key_property: None,
                title_property: Some("name".into()),
                display: None,
            }],
            links: vec![],
        },
        prior_entity_revisions: vec![],
    }
}
pub fn additive_model() -> ModelDocument {
    let mut model = baseline().document;
    model.entities[0].revision = 2;
    model.entities[0].properties.push(property("notes", false));
    model
}

pub fn check_owned_versioned_lifecycle(store: &mut impl ModelStore) -> Result<(), ModelError> {
    let actor = caller();
    let first = store.create_draft(&actor, 1, baseline().document)?;
    let mut incomplete = first.document.clone();
    incomplete.entities[0].display_name.clear();
    let saved = store.save_draft(&actor, first.id, 1, 1, incomplete)?;
    assert_eq!(saved.version, 2);
    assert_eq!(store.open_draft(&actor, first.id)?, saved);
    assert!(
        store
            .prepare(&actor, saved.id, saved.version, saved.base_generation)
            .is_err()
    );
    assert_eq!(store.history(&actor)?.head()?.generation, 1);
    assert_eq!(
        store.save_draft(&actor, first.id, 1, 1, additive_model()),
        Err(ModelError::DraftConflict { current_version: 2 })
    );
    assert!(matches!(
        store.prepare(&actor, first.id, 1, 1),
        Err(ModelError::DraftConflict { current_version: 2 })
    ));
    assert!(matches!(
        store.prepare(&actor, first.id, 2, 99),
        Err(ModelError::HeadConflict {
            current_generation: 1
        })
    ));
    let draft = store.save_draft(&actor, first.id, 2, 1, additive_model())?;
    let prepared = store.prepare(&actor, draft.id, draft.version, draft.base_generation)?;
    let receipt = store.publish(&actor, &prepared)?;
    assert_eq!(receipt.generation, 2);
    assert_eq!(receipt.draft_version, 3);
    assert_eq!(store.publish(&actor, &prepared)?, receipt);
    assert_eq!(
        store.receipt(&actor, first.id, 3, 1)?,
        Some(receipt.clone())
    );
    assert_eq!(
        store.open_draft(&actor, first.id)?.publication,
        Some(receipt.clone())
    );
    assert_eq!(
        store.save_draft(&actor, first.id, 3, 1, additive_model()),
        Err(ModelError::DraftClosed)
    );
    let foreign = ModelCaller {
        tenant_id: "ten_foreign".into(),
        ..actor.clone()
    };
    assert_eq!(store.history(&foreign), Err(ModelError::TenantMismatch));
    assert_eq!(
        store.open_draft(&foreign, first.id),
        Err(ModelError::TenantMismatch)
    );
    let stranger = ModelCaller {
        owner: "usr_other".into(),
        ..actor.clone()
    };
    assert_eq!(
        store.open_draft(&stranger, first.id),
        Err(ModelError::OwnerMismatch)
    );
    assert_eq!(
        store.publish(&stranger, &prepared),
        Err(ModelError::OwnerMismatch)
    );
    assert!(matches!(
        store.prepare(&stranger, first.id, 3, 1),
        Err(ModelError::OwnerMismatch)
    ));
    assert_eq!(
        store.save_draft(&stranger, first.id, 3, 1, additive_model()),
        Err(ModelError::OwnerMismatch)
    );
    let history = store.history(&actor)?;
    assert_eq!(history.publications.len(), 2);
    let registry = history.registry()?;
    let revisions: Vec<_> = registry
        .entity_type_revisions("ten_model")
        .map(|row| row.revision)
        .collect();
    assert_eq!(revisions, [1, 2]);
    Ok(())
}

pub fn check_two_handle_conflicts(
    left: &mut impl ModelStore,
    right: &mut impl ModelStore,
) -> Result<(), ModelError> {
    let actor = caller();
    let draft = left.create_draft(&actor, 1, additive_model())?;
    let second = right.create_draft(&actor, 1, additive_model())?;
    let stale = right.prepare(&actor, second.id, second.version, second.base_generation)?;
    let updated = right.save_draft(&actor, draft.id, 1, 1, additive_model())?;
    assert_eq!(
        left.save_draft(&actor, draft.id, 1, 1, additive_model()),
        Err(ModelError::DraftConflict { current_version: 2 })
    );
    let prepared = left.prepare(&actor, updated.id, updated.version, updated.base_generation)?;
    let first = left.publish(&actor, &prepared)?;
    assert_eq!(
        right.publish(&actor, &stale),
        Err(ModelError::HeadConflict {
            current_generation: 2
        })
    );
    assert_eq!(right.open_draft(&actor, second.id)?, second);
    let mut third_model = additive_model();
    third_model.entities[0].revision = 3;
    third_model.entities[0].display_name = "Records".into();
    let third = right.create_draft(&actor, 2, third_model)?;
    let third_prepared = right.prepare(&actor, third.id, third.version, third.base_generation)?;
    right.publish(&actor, &third_prepared)?;
    assert_eq!(right.publish(&actor, &prepared)?, first);
    assert_eq!(left.history(&actor)?.head()?.generation, 3);
    Ok(())
}

mod discovery;
pub use discovery::check_owned_draft_discovery;

/// A test vocabulary for qualifying reusable allowed strings; never tenant business input.
pub fn named_string_model() -> ModelDocument {
    let mut model = additive_model();
    model.named_string_types.push(ModelNamedStringType {
        id: "vty_test_status".into(),
        revision: 1,
        display_name: "Test status".into(),
        members: vec!["TestActive".into(), "TestLeave".into()],
    });
    for name in ["test_status", "other_test_status"] {
        let mut row = property(name, false);
        row.value_type = Some(ModelValueType::NamedString {
            type_id: "vty_test_status".into(),
            revision: 1,
        });
        model.entities[0].properties.push(row);
    }
    model
}
