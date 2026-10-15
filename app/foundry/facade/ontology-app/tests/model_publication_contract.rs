#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "model_publication/authority.rs"]
mod authority;
mod facade_support;
#[path = "model_publication/freshness.rs"]
mod freshness;
#[path = "model_control/support.rs"]
mod model_support;
#[path = "model_publication/support.rs"]
mod native;
use model_support::*;
use native::*;

#[tokio::test]
async fn exact_owned_saved_model_installs_immediately_without_replacing_query_state() {
    let fixture = NativeFixture::new("hot-install");
    let state = fixture.state(&["foundry-model-author", "foundry-model-publisher"]);
    let before = lifecycle(&state).await;
    assert_eq!(before.phase, ModelLifecyclePhase::Current);
    assert_eq!(before.observed_generation, Some(number(1)));
    {
        let mut tenant = state.app.tenants[TENANT].lock().await;
        tenant.query_decisions.insert(
            ("usr_alice".into(), "earlier-query".into()),
            "earlier-decision".into(),
        );
    }
    let saved = create(&state.router).await;
    let installed = publish(&state, &saved).await;
    assert_eq!(installed.receipt.generation, number(2));
    assert_eq!(installed.lifecycle, lifecycle(&state).await);
    assert_eq!(installed.lifecycle.observed_generation, Some(number(2)));
    assert!(installed.lifecycle.ready);
    let tenant = state.app.tenants[TENANT].lock().await;
    assert_eq!(tenant.observed_model_generation(), Some(2));
    assert_eq!(
        tenant.query_decisions[&("usr_alice".into(), "earlier-query".into())],
        "earlier-decision"
    );
    assert_eq!(
        tenant
            .projection()
            .registry_input
            .entity_types(TENANT)
            .next()
            .unwrap()
            .revision,
        2
    );
    drop(tenant);
    let action = Request::builder().method("POST").uri("/v1/actions")
        .header("authorization", format!("Bearer {}", fixture.fixture.operator_token()))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"object_ref":"ent_hot","action_type":"aty_record_write","idempotency_key":"after-publish","occurred_at_epoch_seconds":1,"properties":{"name":"Native hot model","notes":"new property"}}"#)).unwrap();
    let response = send(&state.router, action).await;
    assert_eq!(response.status(), StatusCode::OK);
    let tenant = state.app.tenants[TENANT].lock().await;
    let object = tenant
        .projection_store()
        .get(TENANT, "ent_hot")
        .unwrap()
        .unwrap();
    assert_eq!(object.schema_revision, 2);
    assert_eq!(object.entity.properties.len(), 2);
}

#[tokio::test]
async fn an_original_receipt_retry_after_a_newer_publication_never_downgrades() {
    let fixture = NativeFixture::new("no-downgrade");
    let state = fixture.state(&["foundry-model-author", "foundry-model-publisher"]);
    let first = create(&state.router).await;
    let original = publish(&state, &first).await.receipt;
    let mut document = first.document.clone();
    document.entities[0].revision = 3;
    document.entities[0].display_name = "A newer model".into();
    let second = ModelDraftWire::decode(
        &body(
            send(
                &state.router,
                request(
                    "POST",
                    DRAFTS,
                    CreateModelDraft {
                        expected_generation: number(2),
                        document,
                    }
                    .encode()
                    .unwrap(),
                ),
            )
            .await,
            StatusCode::CREATED,
        )
        .await,
    )
    .unwrap();
    assert_eq!(publish(&state, &second).await.receipt.generation, number(3));
    let retried = publish(&state, &first).await;
    assert_eq!(retried.receipt, original);
    assert_eq!(retried.lifecycle.desired_generation, number(3));
    assert_eq!(retried.lifecycle.observed_generation, Some(number(3)));
    assert!(retried.lifecycle.ready);
}

#[tokio::test]
async fn metadata_only_composition_reports_unobserved_and_cannot_hot_publish() {
    let fixture = crate::facade_support::Fixture::new("metadata-not-serving");
    let state = instrument(
        fixture.state(),
        &["foundry-model-author", "foundry-model-publisher"],
        Some(origin()),
    );
    let bytes = body(
        send(&state.router, request("GET", LIFECYCLE, Body::empty())).await,
        StatusCode::OK,
    )
    .await;
    let lifecycle = ModelLifecycle::decode(&bytes).unwrap();
    assert_eq!(lifecycle.phase, ModelLifecyclePhase::Unobserved);
    assert_eq!(lifecycle.observed_generation, None);
    assert!(!lifecycle.ready);
    let saved = create(&state.router).await;
    refusal(
        send(&state.router, publication_request(&saved)).await,
        StatusCode::SERVICE_UNAVAILABLE,
        ModelRefusalCode::ModelUnavailable,
    )
    .await;
    assert_eq!(
        state
            .reference
            .head(&ModelCaller {
                tenant_id: TENANT.into(),
                owner: "usr_alice".into()
            })
            .unwrap()
            .generation,
        1
    );
}
