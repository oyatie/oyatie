use oya_social_domain::{
    ContextKind, MediaFlavor, OwnershipPillar, SocialError, SocialFeedIndex, SocialPost,
    SocialPostCreate,
};

fn personal_photo(post_id: &str, body: &str) -> SocialPostCreate {
    personal_photo_for_tenant("tenant:acme", post_id, body)
}

fn personal_photo_for_tenant(tenant_id: &str, post_id: &str, body: &str) -> SocialPostCreate {
    SocialPostCreate {
        post_id: post_id.to_owned(),
        tenant_id: tenant_id.to_owned(),
        creator_ref: "user:personal-alice".to_owned(),
        context_kind: ContextKind::Personal,
        ownership_pillar: OwnershipPillar::Personal,
        body: body.to_owned(),
        media_flavor: MediaFlavor::Photo,
        idempotency_key: format!("idem:{post_id}"),
        consent_token: None,
    }
}

fn professional_post(post_id: &str, body: &str) -> SocialPostCreate {
    professional_post_for_tenant("tenant:acme", post_id, body)
}

fn professional_post_for_tenant(tenant_id: &str, post_id: &str, body: &str) -> SocialPostCreate {
    SocialPostCreate {
        post_id: post_id.to_owned(),
        tenant_id: tenant_id.to_owned(),
        creator_ref: "user:work-alice".to_owned(),
        context_kind: ContextKind::Professional,
        ownership_pillar: OwnershipPillar::Professional,
        body: body.to_owned(),
        media_flavor: MediaFlavor::Text,
        idempotency_key: format!("idem:{post_id}"),
        consent_token: Some("workflow-consent:brand-feed".to_owned()),
    }
}

#[test]
fn personal_post_pillar_is_immutable_and_absent_from_professional_feed_search_and_ontology() {
    let personal =
        SocialPost::publish(personal_photo("post_personal", "family lake photo")).unwrap();
    assert_eq!(personal.context_kind(), ContextKind::Personal);
    assert_eq!(personal.ownership_pillar(), OwnershipPillar::Personal);

    let professional =
        SocialPost::publish(professional_post("post_work", "quarterly launch note")).unwrap();
    let mut index = SocialFeedIndex::default();
    index.publish(personal.clone()).unwrap();
    index.publish(professional).unwrap();

    let professional_feed = index.feed_for(
        "tenant:acme",
        ContextKind::Professional,
        OwnershipPillar::Professional,
    );
    assert_eq!(
        professional_feed
            .iter()
            .map(|post| post.post_id())
            .collect::<Vec<_>>(),
        vec!["post_work"]
    );
    assert!(
        professional_feed
            .iter()
            .all(|post| post.context_kind() == ContextKind::Professional)
    );

    let professional_search = index.search(
        "family",
        "tenant:acme",
        ContextKind::Professional,
        OwnershipPillar::Professional,
    );
    assert!(
        professional_search.is_empty(),
        "personal artifacts must not leak into professional search"
    );

    let professional_work_graph = index.ontology_projection_for(
        "tenant:acme",
        ContextKind::Professional,
        OwnershipPillar::Professional,
    );
    assert!(!professional_work_graph.contains(&"post_personal".to_owned()));
}

#[test]
fn feed_search_and_ontology_projection_are_tenant_scoped() {
    let acme_personal = SocialPost::publish(personal_photo_for_tenant(
        "tenant:acme",
        "post_acme_personal",
        "family lake photo",
    ))
    .unwrap();
    let globex_personal = SocialPost::publish(personal_photo_for_tenant(
        "tenant:globex",
        "post_globex_personal",
        "family launch party",
    ))
    .unwrap();
    let globex_work = SocialPost::publish(professional_post_for_tenant(
        "tenant:globex",
        "post_globex_work",
        "quarterly launch note",
    ))
    .unwrap();

    let mut index = SocialFeedIndex::default();
    index.publish(acme_personal).unwrap();
    index.publish(globex_personal).unwrap();
    index.publish(globex_work).unwrap();

    let acme_personal_feed = index.feed_for(
        "tenant:acme",
        ContextKind::Personal,
        OwnershipPillar::Personal,
    );
    assert_eq!(
        acme_personal_feed
            .iter()
            .map(|post| post.post_id())
            .collect::<Vec<_>>(),
        vec!["post_acme_personal"]
    );

    let acme_search = index.search(
        "launch",
        "tenant:acme",
        ContextKind::Personal,
        OwnershipPillar::Personal,
    );
    assert!(
        acme_search.is_empty(),
        "other tenants' personal artifacts must not leak into tenant-scoped search"
    );

    let acme_graph = index.ontology_projection_for(
        "tenant:acme",
        ContextKind::Personal,
        OwnershipPillar::Personal,
    );
    assert_eq!(acme_graph, vec!["post_acme_personal".to_owned()]);

    let replay = index.replay_post_contracts();
    assert!(replay.no_cross_context_feed_leak);
    assert!(replay.no_cross_context_search_leak);
    assert!(replay.no_cross_context_ontology_leak);
}

#[test]
fn duplicate_tenant_idempotency_keys_are_rejected_before_indexing() {
    let mut first = personal_photo("post_first", "first submit");
    first.idempotency_key = "idem:shared-submit".to_owned();
    let mut duplicate = personal_photo("post_second", "duplicate submit");
    duplicate.idempotency_key = "idem:shared-submit".to_owned();

    let mut index = SocialFeedIndex::default();
    index.publish(SocialPost::publish(first).unwrap()).unwrap();

    assert_eq!(
        index.publish(SocialPost::publish(duplicate).unwrap()),
        Err(SocialError::DuplicatePost)
    );
}

#[test]
fn context_pillar_mismatch_is_rejected_before_indexing() {
    let mut mismatched = personal_photo("post_bad", "bad mixed context");
    mismatched.ownership_pillar = OwnershipPillar::Professional;

    assert_eq!(
        SocialPost::publish(mismatched),
        Err(SocialError::ContextPillarMismatch)
    );
}

#[test]
fn post_contract_replay_emits_context_pillar_idempotency_and_consent_boundary() {
    let personal = SocialPost::publish(personal_photo("post_event", "eventful photo")).unwrap();
    let event = personal.to_post_published_event();

    assert_eq!(event.contract_family, "community.social.post.v1");
    assert_eq!(event.event_name, "SocialContentPublished");
    assert_eq!(event.tenant_id, "tenant:acme");
    assert_eq!(event.context_kind, ContextKind::Personal);
    assert_eq!(event.ownership_pillar, OwnershipPillar::Personal);
    assert_eq!(event.idempotency_key, "idem:post_event");
    assert!(
        event.consent_token.is_none(),
        "personal post replay must not fabricate workflow consent"
    );
}

#[test]
fn short_video_is_a_social_media_flavor_not_a_standalone_shorts_authority() {
    let mut create = personal_photo("post_short", "ninety second clip");
    create.media_flavor = MediaFlavor::ShortVideo;

    let post = SocialPost::publish(create).unwrap();
    assert_eq!(post.media_flavor(), MediaFlavor::ShortVideo);
    assert_eq!(post.service_authority(), "social");
    assert_eq!(post.retired_standalone_shorts_authority(), None);
}
