use super::*;
use foundry_model_api::conformance::{additive_model, baseline};

#[tokio::test]
async fn exact_saved_base_survives_head_advance_without_writes_or_author_capability() {
    let mut store = ReferenceModelStore::initialize(
        ModelBinding {
            tenant_id: "ten_acme".into(),
            store_id: "base".into(),
            baseline_id: "initial".into(),
        },
        baseline(),
    )
    .unwrap();
    let caller = ModelCaller {
        tenant_id: "ten_acme".into(),
        owner: "usr_author".into(),
    };
    let retained = store.create_draft(&caller, 1, additive_model()).unwrap();
    let published = store.create_draft(&caller, 1, additive_model()).unwrap();
    let prepared = store.prepare(&caller, published.id, 1, 1).unwrap();
    store.publish(&caller, &prepared).unwrap();
    let before = store.history(&caller).unwrap();
    let reader = source(true, false, &store, "usr_author");
    let id = ModelNumber::new(retained.id).unwrap();
    let request = PrepareModelDraft {
        expected_version: ModelNumber::new(1).unwrap(),
        expected_base: ModelNumber::new(1).unwrap(),
    };
    let base = reader.draft_base("ten_acme", id, &request).await.unwrap();
    assert_eq!(base.document, baseline().document);
    assert_eq!(base.draft_id, id);
    assert_eq!(base.draft_version, request.expected_version);
    assert_eq!(base.base_generation, request.expected_base);
    assert_eq!(
        reader.workspace("ten_acme").await.unwrap().document,
        additive_model()
    );
    assert_eq!(store.history(&caller).unwrap(), before);
    assert_eq!(store.open_draft(&caller, retained.id).unwrap(), retained);
    for (version, generation, expected) in [
        (2, 1, ModelRefusalCode::DraftConflict),
        (1, 2, ModelRefusalCode::HeadConflict),
    ] {
        let wrong = PrepareModelDraft {
            expected_version: ModelNumber::new(version).unwrap(),
            expected_base: ModelNumber::new(generation).unwrap(),
        };
        assert_eq!(
            reader.draft_base("ten_acme", id, &wrong).await,
            Err(ModelSourceError::Refused(expected))
        );
        assert_eq!(
            source(true, false, &store, "usr_other")
                .draft_base("ten_acme", id, &wrong)
                .await,
            Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden))
        );
    }
    assert_eq!(
        reader.draft_base("ten_other", id, &request).await,
        Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden))
    );
    assert_eq!(
        source(false, false, &store, "usr_author")
            .draft_base("ten_acme", id, &request)
            .await,
        Err(ModelSourceError::Refused(ModelRefusalCode::Forbidden))
    );
    assert_eq!(
        reader
            .draft_base("ten_acme", ModelNumber::new(99).unwrap(), &request)
            .await,
        Err(ModelSourceError::Refused(ModelRefusalCode::DraftNotFound))
    );
}
