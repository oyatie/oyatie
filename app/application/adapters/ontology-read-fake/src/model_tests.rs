use super::*;
use foundry_model_api::{ModelBaseline, ModelBinding, ModelDocument};
fn source(
    read: bool,
    author: bool,
    store: &ReferenceModelStore,
    owner: &str,
) -> FixtureOntologyModelSource {
    FixtureOntologyModelSource::new(
        store.clone(),
        ModelCaller {
            tenant_id: "ten_acme".into(),
            owner: owner.into(),
        },
        read,
        ModelCapabilities {
            author,
            publish: false,
        },
    )
}
#[tokio::test]
async fn native_fake_save_reopen_versions_owner_and_read_author_are_separate() {
    let document = ModelDocument {
        entities: vec![],
        links: vec![],
    };
    let store = ReferenceModelStore::initialize(
        ModelBinding {
            tenant_id: "ten_acme".into(),
            store_id: "fixture".into(),
            baseline_id: "initial".into(),
        },
        ModelBaseline {
            document: document.clone(),
            prior_entity_revisions: vec![],
        },
    )
    .unwrap();
    let author = source(true, true, &store, "usr_author");
    let reader = source(true, false, &store, "usr_author");
    let request = CreateModelDraft {
        expected_generation: ModelNumber::new(1).unwrap(),
        document,
    };
    assert_eq!(
        reader.create_draft("ten_acme", &request).await,
        Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden))
    );
    assert_eq!(
        source(false, false, &store, "usr_operator")
            .workspace("ten_acme")
            .await,
        Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden))
    );
    let draft = author.create_draft("ten_acme", &request).await.unwrap();
    let save = SaveModelDraft {
        expected_version: draft.version,
        expected_base: draft.base_generation,
        document: draft.document.clone(),
    };
    let saved = author
        .save_draft("ten_acme", draft.id, &save)
        .await
        .unwrap();
    assert_eq!(saved.version.get(), 2);
    let publisher = FixtureOntologyModelSource::new(
        store.clone(),
        ModelCaller {
            tenant_id: "ten_acme".into(),
            owner: "usr_author".into(),
        },
        true,
        ModelCapabilities {
            author: false,
            publish: true,
        },
    );
    assert!(
        publisher
            .validate_draft(
                "ten_acme",
                saved.id,
                &PrepareModelDraft {
                    expected_version: saved.version,
                    expected_base: saved.base_generation
                }
            )
            .await
            .is_ok()
    );
    assert_eq!(
        publisher.save_draft("ten_acme", saved.id, &save).await,
        Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden))
    );
    assert_eq!(
        publisher.create_draft("ten_acme", &request).await,
        Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden))
    );
    assert_eq!(reader.draft("ten_acme", draft.id).await.unwrap(), saved);
    assert_eq!(
        author.save_draft("ten_acme", draft.id, &save).await,
        Err(ModelSourceError::Refused(ModelRefusalCode::DraftConflict))
    );
    assert!(
        source(true, true, &store, "usr_other")
            .owned_drafts("ten_acme", None, 1)
            .await
            .unwrap()
            .drafts
            .is_empty()
    );
    assert_eq!(
        source(true, true, &store, "usr_other")
            .draft("ten_acme", draft.id)
            .await,
        Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden))
    );
    assert_eq!(
        author.workspace("ten_foreign").await,
        Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden))
    );
    assert_eq!(
        author.workspace("ten_acme").await.unwrap().generation.get(),
        1
    );
}

#[tokio::test]
async fn native_fake_validation_refuses_foreign_owner_without_masking_schema_diagnostics() {
    let document = ModelDocument {
        entities: vec![],
        links: vec![],
    };
    let store = ReferenceModelStore::initialize(
        ModelBinding {
            tenant_id: "ten_acme".into(),
            store_id: "fixture".into(),
            baseline_id: "initial".into(),
        },
        ModelBaseline {
            document: document.clone(),
            prior_entity_revisions: vec![],
        },
    )
    .unwrap();
    let author = source(true, true, &store, "usr_author");
    let draft = author
        .create_draft(
            "ten_acme",
            &CreateModelDraft {
                expected_generation: ModelNumber::new(1).unwrap(),
                document,
            },
        )
        .await
        .unwrap();
    let request = PrepareModelDraft {
        expected_version: draft.version,
        expected_base: draft.base_generation,
    };
    for capabilities in [
        ModelCapabilities {
            author: true,
            publish: false,
        },
        ModelCapabilities {
            author: false,
            publish: true,
        },
    ] {
        let foreign = FixtureOntologyModelSource::new(
            store.clone(),
            ModelCaller {
                tenant_id: "ten_acme".into(),
                owner: "usr_other".into(),
            },
            true,
            capabilities,
        );
        assert_eq!(
            foreign.validate_draft("ten_acme", draft.id, &request).await,
            Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden))
        );
    }
    let validation = author
        .validate_draft("ten_acme", draft.id, &request)
        .await
        .unwrap();
    assert!(!validation.valid);
    assert_eq!(validation.diagnostics[0].code, "metadata_incompatible");
    assert_eq!(
        author
            .validate_draft("ten_acme", ModelNumber::new(2).unwrap(), &request)
            .await,
        Err(ModelSourceError::Refused(ModelRefusalCode::DraftNotFound))
    );
    assert_eq!(
        author
            .validate_draft(
                "ten_acme",
                draft.id,
                &PrepareModelDraft {
                    expected_version: ModelNumber::new(2).unwrap(),
                    ..request
                }
            )
            .await,
        Err(ModelSourceError::Refused(ModelRefusalCode::DraftConflict))
    );
}
