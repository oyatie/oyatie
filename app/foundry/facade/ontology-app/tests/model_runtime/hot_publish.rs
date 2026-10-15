use crate::support::*;
use foundry_model_api::{control::*, *};
use reqwest::{Method, StatusCode};

#[tokio::test]
async fn real_http_publication_keeps_replayed_objects_and_serves_new_revision_without_restart() {
    let fixture = Fixture::new();
    fixture.initialize();
    let iam = Iam::start_with_roles(&[
        "foundry-model-author",
        "foundry-model-publisher",
        "foundry-operator",
    ])
    .await;
    let process = fixture.start(&iam.origin).await;
    let write = |key: &str, object: &str| {
        process.client.post(format!("{}/v1/actions", process.origin)).bearer_auth("runtime-operator").json(&serde_json::json!({"object_ref":object,"action_type":"aty_record_write","idempotency_key":key,"occurred_at_epoch_seconds":1,"properties":{"name":"Durable native record","note":"retained value"}}))
    };
    assert_eq!(
        write("before-publish", "ent_before")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    let mut document = Fixture::baseline().document;
    document.entities[0].revision = 2;
    document.entities[0]
        .properties
        .push(conformance::property("new_optional", false));
    let saved = ModelDraftWire::decode(
        &bytes(
            process
                .request(Method::POST, "model/drafts")
                .header("content-type", "application/json")
                .body(
                    CreateModelDraft {
                        expected_generation: ModelNumber::new(1).unwrap(),
                        document,
                    }
                    .encode()
                    .unwrap(),
                )
                .send()
                .await
                .unwrap(),
            StatusCode::CREATED,
        )
        .await,
    )
    .unwrap();
    let published = ModelPublished::decode(
        &bytes(
            process
                .request(
                    Method::POST,
                    &format!("model/drafts/{}/publish", saved.id.get()),
                )
                .header("content-type", "application/json")
                .body(
                    PrepareModelDraft {
                        expected_version: saved.version,
                        expected_base: saved.base_generation,
                    }
                    .encode()
                    .unwrap(),
                )
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(published.lifecycle.phase, ModelLifecyclePhase::Current);
    assert_eq!(published.lifecycle.observed_generation.unwrap().get(), 2);
    assert!(published.lifecycle.ready);
    assert_eq!(
        write("after-publish", "ent_after")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    for (id, revision) in [("ent_before", 1), ("ent_after", 2)] {
        let value: serde_json::Value = serde_json::from_slice(
            &bytes(
                process
                    .request(
                        Method::GET,
                        &format!("objects/{id}?type=ety_record&revision=1"),
                    )
                    .send()
                    .await
                    .unwrap(),
                StatusCode::OK,
            )
            .await,
        )
        .unwrap();
        assert_eq!(value["written_revision"], revision);
        assert_eq!(
            value["properties"]["name"]["value"],
            "Durable native record"
        );
        assert!(value["properties"].get("new_optional").is_none());
    }
    assert_eq!(
        process
            .client
            .get(format!("{}/readyz", process.origin))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    // The live process owns all real native files; the API did not release its fence.
    assert!(
        foundry_model_sqlite_draft::SqliteModelStore::open_existing(
            &fixture.model_path(),
            Fixture::binding(),
            Fixture::baseline()
        )
        .is_err()
    );
    drop(process);
    let restarted = fixture.start(&iam.origin).await;
    let lifecycle = ModelLifecycle::decode(
        &bytes(
            restarted
                .request(Method::GET, "model/lifecycle")
                .send()
                .await
                .unwrap(),
            StatusCode::OK,
        )
        .await,
    )
    .unwrap();
    assert_eq!(lifecycle.observed_generation.unwrap().get(), 2);
    assert!(lifecycle.ready);
}
