use super::*;

pub fn check_owned_draft_discovery(store: &mut impl ModelStore) -> Result<(), ModelError> {
    let actor = ModelCaller {
        owner: "usr_author\0one".into(),
        ..caller()
    };
    let other = ModelCaller {
        owner: "usr_author\0two".into(),
        ..caller()
    };
    let mut owned = Vec::new();
    for index in 0..6 {
        let owner = if index % 2 == 0 { &actor } else { &other };
        let draft = store.create_draft(owner, 1, additive_model())?;
        if index % 2 == 0 {
            owned.push(draft);
        }
    }
    let closed = &owned[1];
    let prepared = store.prepare(&actor, closed.id, closed.version, closed.base_generation)?;
    let receipt = store.publish(&actor, &prepared)?;
    assert_eq!(store.head(&actor)?.generation, receipt.generation);
    let first = store.owned_drafts(
        &actor,
        &DraftPageRequest {
            limit: 2,
            after: None,
        },
    )?;
    assert_eq!(
        first
            .drafts
            .iter()
            .map(|draft| draft.id)
            .collect::<Vec<_>>(),
        [1, 3]
    );
    assert_eq!(first.drafts[0].published_generation, None);
    assert_eq!(
        first.drafts[1].published_generation,
        Some(receipt.generation)
    );
    assert_eq!(first.next, Some(3));
    let last = store.owned_drafts(
        &actor,
        &DraftPageRequest {
            limit: 2,
            after: first.next,
        },
    )?;
    assert_eq!(
        last.drafts.iter().map(|draft| draft.id).collect::<Vec<_>>(),
        [5]
    );
    assert_eq!(last.next, None);
    let gap = store.owned_drafts(
        &actor,
        &DraftPageRequest {
            limit: 1,
            after: Some(2),
        },
    )?;
    assert_eq!(gap.drafts[0].id, 3);
    assert_eq!(gap.next, Some(3));
    assert!(
        store
            .owned_drafts(
                &actor,
                &DraftPageRequest {
                    limit: MAX_MODEL_ITEMS,
                    after: Some(MAX_MODEL_VERSION)
                }
            )?
            .drafts
            .is_empty()
    );
    let absent = ModelCaller {
        owner: "usr_absent".into(),
        ..actor.clone()
    };
    assert_eq!(
        store.owned_drafts(
            &absent,
            &DraftPageRequest {
                limit: 1,
                after: None
            }
        )?,
        DraftPage {
            drafts: vec![],
            next: None
        }
    );
    let foreign = ModelCaller {
        tenant_id: "ten_other".into(),
        ..actor.clone()
    };
    assert_eq!(
        store.owned_drafts(
            &foreign,
            &DraftPageRequest {
                limit: 1,
                after: None
            }
        ),
        Err(ModelError::TenantMismatch)
    );
    for request in [
        DraftPageRequest {
            limit: 0,
            after: None,
        },
        DraftPageRequest {
            limit: MAX_MODEL_ITEMS + 1,
            after: None,
        },
        DraftPageRequest {
            limit: 1,
            after: Some(0),
        },
        DraftPageRequest {
            limit: 1,
            after: Some(MAX_MODEL_VERSION + 1),
        },
    ] {
        assert_eq!(
            store.owned_drafts(&actor, &request),
            Err(ModelError::MalformedDocument)
        );
    }
    Ok(())
}
