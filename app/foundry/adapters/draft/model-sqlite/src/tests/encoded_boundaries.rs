use super::*;

fn escaped_binding() -> ModelBinding {
    ModelBinding {
        tenant_id: format!("ten_{}", "\0".repeat(MAX_MODEL_CONTEXT_BYTES - 4)),
        store_id: "\0".repeat(MAX_MODEL_CONTEXT_BYTES),
        baseline_id: "\0".repeat(MAX_MODEL_CONTEXT_BYTES),
    }
}

fn escaped_caller(binding: &ModelBinding) -> ModelCaller {
    ModelCaller {
        tenant_id: binding.tenant_id.clone(),
        owner: "\0".repeat(MAX_MODEL_CONTEXT_BYTES),
    }
}

fn maximum_document() -> ModelDocument {
    let mut document = baseline().document;
    document.entities[0].revision = 2;
    let full_display = ModelDisplay {
        display_name: Some("x".repeat(MAX_MODEL_TEXT_BYTES)),
        description: Some("x".repeat(MAX_MODEL_TEXT_BYTES)),
        icon: Some("x".repeat(MAX_MODEL_TEXT_BYTES)),
        color: Some("x".repeat(MAX_MODEL_TEXT_BYTES)),
        plural_name: Some("x".repeat(MAX_MODEL_TEXT_BYTES)),
    };
    document.entities[0].display = Some(full_display.clone());
    document.entities[0].properties[0].display = Some(full_display.clone());
    for index in 0..51 {
        let mut field = property(&format!("optional_{index}"), false);
        field.display = Some(full_display.clone());
        document.entities[0].properties.push(field);
    }
    let mut excess = serde_json::to_vec(&document).unwrap().len() - MAX_MODEL_BYTES;
    for property in &mut document.entities[0].properties {
        let display = property.display.as_mut().unwrap();
        for field in [
            &mut display.display_name,
            &mut display.description,
            &mut display.icon,
            &mut display.color,
            &mut display.plural_name,
        ] {
            let value = field.as_mut().unwrap();
            let remove = excess.min(value.len() - 1);
            value.truncate(value.len() - remove);
            excess -= remove;
        }
    }
    assert_eq!(excess, 0);
    assert_eq!(document.encode().unwrap().len(), MAX_MODEL_BYTES);
    document
}

#[test]
fn maximum_escaped_binding_initializes_and_reopens_through_public_api() {
    let database = Database::new();
    let binding = escaped_binding();
    binding.validate().unwrap();
    for field in 0..3 {
        let mut invalid = binding.clone();
        match field {
            0 => invalid.tenant_id.push('\0'),
            1 => invalid.store_id.push('\0'),
            _ => invalid.baseline_id.push('\0'),
        }
        assert_eq!(invalid.validate(), Err(ModelError::InvalidContext));
    }
    assert_eq!(encode(&binding).unwrap().len(), 4635);
    let store =
        SqliteModelStore::initialize(&database.path(), binding.clone(), baseline()).unwrap();
    drop(store);
    let opened =
        SqliteModelStore::open_existing(&database.path(), binding.clone(), baseline()).unwrap();
    assert_eq!(
        opened.history(&escaped_caller(&binding)).unwrap().binding,
        binding
    );
    let mut invalid_actor = escaped_caller(&binding);
    invalid_actor.owner.push('\0');
    assert_eq!(
        opened.history(&invalid_actor),
        Err(ModelError::InvalidContext)
    );
}

#[test]
fn exact_document_limit_and_escaped_context_save_publish_and_reopen_without_corrupting_state() {
    let database = Database::new();
    let mut binding = escaped_binding();
    binding.store_id = "store_model".into();
    binding.baseline_id = "baseline_model".into();
    let actor = escaped_caller(&binding);
    let mut store =
        SqliteModelStore::initialize(&database.path(), binding.clone(), baseline()).unwrap();
    let draft = store.create_draft(&actor, 1, baseline().document).unwrap();
    let maximum = maximum_document();
    let saved = store
        .save_draft(&actor, draft.id, 1, 1, maximum.clone())
        .unwrap();
    let mut oversized_document = maximum.clone();
    oversized_document.entities[0].display_name.push('x');
    assert!(matches!(
        store.save_draft(&actor, draft.id, saved.version, 1, oversized_document),
        Err(ModelError::Limit { .. })
    ));
    assert_eq!(store.open_draft(&actor, draft.id).unwrap(), saved);
    let prepared = store
        .prepare(&actor, draft.id, saved.version, saved.base_generation)
        .unwrap();
    let original = store.publish(&actor, &prepared).unwrap();
    let closed = store.open_draft(&actor, draft.id).unwrap();
    assert!(encode(&closed).unwrap().len() > MAX_MODEL_BYTES + 4096);
    assert!(encode(&closed).unwrap().len() <= codec::row_bounds().unwrap().draft);
    drop(store);
    let opened = SqliteModelStore::open_existing(&database.path(), binding, baseline()).unwrap();
    assert_eq!(
        opened.open_draft(&actor, draft.id).unwrap().document,
        maximum
    );
    assert_eq!(
        opened
            .receipt(&actor, draft.id, saved.version, saved.base_generation)
            .unwrap(),
        Some(original)
    );
}

#[test]
fn derived_row_caps_still_refuse_oversized_corrupt_binding_draft_and_publication() {
    for kind in 0..3 {
        let database = Database::new();
        let mut store = database.initialize();
        let draft = store.create_draft(&caller(), 1, additive_model()).unwrap();
        let bounds = codec::row_bounds().unwrap();
        let (table, column, limit) = match kind {
            0 => ("ontology_model_binding", "binding", bounds.binding),
            1 => ("ontology_model_draft", "record", bounds.draft),
            _ => ("ontology_model_publication", "record", bounds.publication),
        };
        store
            .connection
            .execute(
                &format!("UPDATE {table} SET {column} = ?1"),
                params![vec![b' '; limit + 1]],
            )
            .unwrap();
        assert!(matches!(
            store.open_draft(&caller(), draft.id),
            Err(ModelError::CorruptState)
        ));
        drop(store);
        assert!(matches!(
            SqliteModelStore::open_existing(&database.path(), binding(), baseline()),
            Err(ModelError::CorruptState)
        ));
    }
}
